#!/usr/bin/env python3
"""Compile a CUDA kernel to PTX for an architecture nvcc has dropped.

Why this exists rather than a call to nvcc
------------------------------------------
The machine this was written on has CUDA 13.3, whose nvcc refuses `sm_61`
outright::

    $ nvcc -arch=sm_61 probe.cu
    nvcc fatal : Unsupported gpu architecture 'sm_61'
    $ nvcc --list-gpu-arch
    compute_75 compute_80 ... compute_121

CUDA 13 dropped Maxwell, Pascal and Volta from offline compilation. The
*driver* still runs sm_61 (the GTX 1070 is present and healthy), so the missing
piece is only a compiler that will emit code for it. NVRTC 12.9, published as a
PyPI wheel needing no install and no administrator rights, does: it warns that
pre-75 architectures are deprecated and then compiles. Commit the PTX it
produces and the driver JIT-compiles it to sm_61 at module load.

Consequences, both good: the host crate needs no CUDA toolkit to build, only
the driver itself, and the kernel that was measured is in the repository as
readable text rather than being whatever the local toolchain happened to
produce that day. The reasoning is recorded in
docs/adr/0002-pascal-without-a-pascal-toolkit.md.

Reproducibility, and why it needs three things
----------------------------------------------
"The kernel that was measured is in the repository" is only true if a second
machine regenerating it gets the same bytes. Three things are required, and
only the first is obvious:

1. **A pinned compiler.** An unpinned wheel fetches whatever NVRTC is current,
   and a different NVRTC emits different PTX. LICENSES.md names 12.9.86 as the
   version whose licence was looked at, so anything else is also an unrecorded
   input. The version check below is fatal rather than a warning.
2. **No absolute paths.** `-lineinfo` makes NVVM emit a `.file` directive, and
   NVRTC resolves the program name against the process working directory. The
   first build of this script therefore embedded the absolute path of the
   kernel on the machine that ran it, which is machine-specific and defeats
   byte-for-byte comparison. The directive is rewritten to a path relative to
   the working directory.
3. **No line-ending rewrite.** This script writes LF. Mark `*.ptx` binary in
   .gitattributes in whatever repository commits it, or a Windows checkout with
   core.autocrlf=true will rewrite it to CRLF and the working tree will stop
   matching what was committed and measured.

The kernel this was written for is part of an implementation that is not
published in this repository; see SCOPE.md. The script takes the kernel and the
target architecture as arguments and is not specific to it.

Compile a kernel::

    python tools/build_ptx.py path/to/kernel.cu path/to/out.ptx --arch compute_61

Check a committed PTX still matches its source, without writing::

    python tools/build_ptx.py path/to/kernel.cu path/to/out.ptx --check
"""

from __future__ import annotations

import ctypes
import os
import pathlib
import re
import subprocess
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
VENDOR = ROOT / "tools" / ".nvrtc"          # gitignored

# Pinned, not floating. See "Reproducibility" above.
NVRTC_VERSION = (12, 9)
WHEEL = "nvidia-cuda-nvrtc-cu12==12.9.86"


def find_nvrtc() -> pathlib.Path:
    """Return the directory holding nvrtc64_120_0.dll / libnvrtc.so.12."""
    names = ("nvrtc64_120_0.dll", "libnvrtc.so.12")
    for name in names:
        hits = sorted(VENDOR.rglob(name)) if VENDOR.exists() else []
        if hits:
            return hits[0].parent

    VENDOR.mkdir(parents=True, exist_ok=True)
    print(f"fetching {WHEEL} into {VENDOR} (not committed) ...")
    subprocess.run(
        [sys.executable, "-m", "pip", "download", WHEEL, "--no-deps", "-d", str(VENDOR)],
        check=True,
    )
    for whl in VENDOR.glob("*.whl"):
        zipfile.ZipFile(whl).extractall(VENDOR)
    for name in names:
        hits = sorted(VENDOR.rglob(name))
        if hits:
            return hits[0].parent
    raise SystemExit(f"could not find {names} under {VENDOR}")


def normalise(ptx: str, kernel_rel: str) -> str:
    """Point the `.file` directive at `kernel_rel` instead of an absolute path.

    NVRTC resolves the program name against the process CWD, so without this
    the committed PTX carries an absolute path from whichever machine last ran
    the script. The assertion is deliberate: if NVRTC's output shape changes,
    this should fail loudly rather than silently stop normalising.
    """
    files = [l for l in ptx.splitlines() if l.lstrip().startswith(".file")]
    if len(files) != 1:
        raise SystemExit(
            f"expected exactly one .file directive, found {len(files)}: {files}"
        )
    return re.sub(
        r'(?m)^(\s*\.file\s+\d+\s+)".*"$',
        lambda mo: f'{mo.group(1)}"{kernel_rel}"',
        ptx,
    )


def compile_ptx(bindir: pathlib.Path, source: bytes, arch: str,
                kernel_name: str, kernel_rel: str) -> str:
    if hasattr(os, "add_dll_directory"):
        os.add_dll_directory(str(bindir))
    lib = None
    for name in ("nvrtc64_120_0.dll", "libnvrtc.so.12"):
        candidate = bindir / name
        if candidate.exists():
            lib = ctypes.CDLL(str(candidate))
            break
    if lib is None:
        raise SystemExit(f"no nvrtc library in {bindir}")

    major, minor = ctypes.c_int(), ctypes.c_int()
    lib.nvrtcVersion(ctypes.byref(major), ctypes.byref(minor))
    print(f"nvrtc {major.value}.{minor.value} -> {arch}")
    if (major.value, minor.value) != NVRTC_VERSION:
        want = "%d.%d" % NVRTC_VERSION
        raise SystemExit(
            f"nvrtc {major.value}.{minor.value} is not the pinned {want}. "
            f"CUDA 13 dropped Pascal entirely, and any other 12.x emits different "
            f"PTX than a file committed under the pinned version. Delete "
            f"tools/.nvrtc and re-run to fetch "
            f"{WHEEL}."
        )

    prog = ctypes.c_void_p()
    if lib.nvrtcCreateProgram(ctypes.byref(prog), source, kernel_name.encode(),
                              0, None, None) != 0:
        raise SystemExit("nvrtcCreateProgram failed")

    opts = (ctypes.c_char_p * 2)(f"--gpu-architecture={arch}".encode(), b"-lineinfo")
    rc = lib.nvrtcCompileProgram(prog, 2, opts)

    size = ctypes.c_size_t()
    lib.nvrtcGetProgramLogSize(prog, ctypes.byref(size))
    log = ctypes.create_string_buffer(size.value)
    lib.nvrtcGetProgramLog(prog, log)
    text = log.value.decode(errors="replace").strip()
    if text:
        print(text)
    if rc != 0:
        raise SystemExit(f"nvrtcCompileProgram failed rc={rc}")

    lib.nvrtcGetPTXSize(prog, ctypes.byref(size))
    ptx = ctypes.create_string_buffer(size.value)
    lib.nvrtcGetPTX(prog, ptx)
    return normalise(ptx.value.decode(), kernel_rel)


def main() -> None:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    check_only = "--check" in sys.argv[1:]
    arch = next((a.split("=", 1)[1] for a in sys.argv[1:]
                 if a.startswith("--arch=")), "compute_61")
    if len(args) != 2:
        raise SystemExit(
            "usage: build_ptx.py <kernel.cu> <out.ptx> [--arch=compute_61] [--check]"
        )

    kernel, out = pathlib.Path(args[0]), pathlib.Path(args[1])
    if not kernel.is_file():
        raise SystemExit(f"{kernel} is not a file")

    # The path the committed PTX names, whatever directory the build ran from.
    # Relative to CWD so two machines building the same tree agree; absolute
    # only if the kernel lies outside it, which is already not reproducible.
    try:
        kernel_rel = kernel.resolve().relative_to(pathlib.Path.cwd()).as_posix()
    except ValueError:
        kernel_rel = kernel.name

    ptx = compile_ptx(find_nvrtc(), kernel.read_bytes(), arch, kernel.name, kernel_rel)
    entries = [line.split()[-1].rstrip("(")
               for line in ptx.splitlines() if ".entry" in line]

    if check_only:
        if not out.exists():
            raise SystemExit(f"{out} does not exist")
        committed = out.read_text(encoding="utf-8")
        if committed != ptx:
            raise SystemExit(
                f"{out} does NOT match {kernel_rel}.\n"
                f"committed {len(committed)} bytes, recompiled {len(ptx)} bytes.\n"
                f"Rebuild it and re-measure: a stale PTX means the card ran code "
                f"that is not the code in this repository."
            )
        print(f"{out} matches {kernel_rel} ({len(ptx)} bytes), entries: {entries}")
        return

    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(ptx, encoding="utf-8", newline="\n")
    print(f"wrote {out} ({len(ptx)} bytes), entries: {entries}")


if __name__ == "__main__":
    main()
