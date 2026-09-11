# ADR 0002: Running the GPU arm on Pascal after the toolkit dropped Pascal

- Status: accepted
- Date: 2026-08-28
- Deciders: the maintainers of the PIR prototype
- Extends `docs/adr/0001-gpu-before-fpga-for-pir.md`
- **Ported from an internal repository and renumbered.** This was ADR 0005
  there, extending what is `docs/adr/0001-gpu-before-fpga-for-pir.md` here. Its
  own self-references are renumbered with it. `SCOPE.md` section 5 records what
  the port removed and what it kept.

## Context

ADR 0001 put the CPU baseline first and deferred the hardware question until
there was a multiplier to decide it with. It also said the GPU was worth
measuring before an FPGA was rented. The machine this work moved onto has a
**GeForce GTX 1070** already in it, so the GPU arm became a measurement that
costs nothing rather than a purchase that has to be justified first.

Three things then went wrong at once, and all three are the kind that get
worked around quietly and forgotten.

### The installed toolkit cannot compile for this GPU at all

```
$ nvcc --version
Cuda compilation tools, release 13.3, V13.3.73
$ nvcc -arch=sm_61 probe.cu
nvcc fatal   : Unsupported gpu architecture 'sm_61'
$ nvcc --list-gpu-arch
compute_75 compute_80 compute_86 compute_87 compute_88 compute_89
compute_90 compute_100 compute_110 compute_103 compute_120 compute_121
```

CUDA 13 removed offline compilation for Maxwell, Pascal and Volta. The GTX 1070
is Pascal, `sm_61`, below the floor of that list. This is not a misconfiguration
and there is no flag for it.

The *driver* is a separate matter and is fine:

```
NVIDIA-SMI 582.66   Driver Version: 582.66   CUDA Version: 13.0
  0  NVIDIA GeForce GTX 1070   WDDM   8192MiB
```

The card is present, healthy, and reports compute capability 6.1. The driver
still JIT-compiles PTX for it. What is missing is only a *compiler* willing to
emit that PTX.

### The workspace forbids the only way to talk to it

`Cargo.toml` sets `unsafe_code = "forbid"` for every crate, and the workspace
manifest published here still carries that line. Calling the CUDA driver is FFI,
and `forbid` cannot be relaxed by an inner attribute: that is what distinguishes
it from `deny`, and it is deliberate everywhere else. A repository-wide
weakening would put every crate, present and future, one edit away from unsafe
code in order to serve one module that needs it.

### The obvious link route does not work either

`CUDA_PATH/lib/x64/cuda.lib` is present, and linking against it fails:

```
error LNK2019: unresolved external symbol __imp_cuInit
error LNK2019: unresolved external symbol __imp_cuDeviceGet
```

Chasing that would have made the build depend on which toolkit happened to be
installed, the exact dependency worth not having, given the toolkit cannot
compile for the card anyway.

## Decision

1. **Compile the kernels with NVRTC 12.9, not with the installed nvcc.** NVRTC
   is published as a PyPI wheel (`nvidia-cuda-nvrtc-cu12`, 76 MB, no install
   and no administrator rights). It targets `compute_61`, warns that
   architectures before `compute_75` are deprecated, and produces PTX.
   `tools/build_ptx.py` does this and is the only supported way to rebuild a
   kernel. That script is published here; the kernel source it expects is not
   (`SCOPE.md`), so what reproduces from it is the method.

2. **Commit the PTX; let the driver do the rest.** The PTX for `compute_61` is
   committed next to the kernel it was compiled from and `include_str!`d into
   the binary. Both live in the implementation under measurement, which is not
   published in this repository, and no PTX is published here. The driver
   JIT-compiles it to `sm_61` at module load, in tens of milliseconds, reported
   separately from every timing so it never lands inside a per-query figure
   (measured: 33.2 ms cold against 0.6 ms warm,
   `measurements/gpu-jit-cold-2026-08-28.txt`). Consequence: the crate builds
   with **no CUDA toolkit present at all**, and the kernel that was measured is
   in the repository it belongs to as readable text rather than being whatever
   the local toolchain produced that day.

3. **Link the driver with `raw-dylib`, not an import library.**
   `#[link(name = "nvcuda", kind = "raw-dylib")]` on Windows resolves against
   the names the driver's own DLL exports, with no `.lib` and no `CUDA_PATH`.
   The build script consults `CUDA_PATH` only on Linux, and only if it is set.

4. **The unsafe exception is one module, not the repository.** The GPU crate of
   the implementation under measurement declares its own lint table with
   `unsafe_code = "deny"`, deliberately not `lints.workspace = true`, and
   deliberately `deny` rather than `forbid` so that a single module can opt out.
   The module holding the CUDA driver bindings carries `#![allow(unsafe_code)]`
   and its reason. Every other module in that crate, and every other crate in
   that workspace, still refuses unsafe code; the rest of the GPU crate is safe
   Rust over checked slices.

5. **The GPU runs the same scheme, not a faster one.** ADR 0001 point 1 refused
   two-server DPF-PIR on threat model, and having a GPU does not revisit that:
   the speed of those schemes comes from splitting trust across two operators,
   and one operator running both is the colluding case (`docs/threat-model.md`).
   The GPU arm is SimplePIR, the same construction, the same parameters, the
   same database shape as the CPU arm, with a different processor underneath it.
   That is what makes the two numbers comparable.

6. **Every GPU figure is verified before it is reported.** A timing is printed
   only if the GPU answer is **bit-identical** to the CPU answer, the device's
   scan sum matches the host's, and the record decodes end to end. All the
   arithmetic is `u32`/`u64` wrapping over a commutative group, so there is no
   float slop available to excuse a difference; a mismatch is a bug, not a
   rounding.

## Consequences

**An earlier internal prohibition on a GPU arm is superseded, and its reason
survives.** That prohibition was against deciding by purchase rather than by
measurement, which is the same thing ADR 0001 declined when it deferred the
FPGA. Nothing was bought. The card was already in the machine, and the CPU floor
was measured first and is reported alongside every GPU figure
(`crates/pir-bench`, `measurements/RESULTS.md`). The prohibition, read as a rule
about how the hardware question gets decided rather than as a ban on a
processor, still holds.

**The GPU arm depends on a Python script and a wheel to rebuild a kernel.** This
is a real cost. It is bounded by the PTX being committed: the wheel is needed to
*change* a kernel, never to build or run one. `tools/.nvrtc/` is gitignored. The
wheel's licence question is open and recorded as open in `LICENSES.md`, not
assumed benign.

**This route has a shelf life.** NVRTC 12.x already warns that pre-`compute_75`
architectures may be removed. When that happens the options are an older wheel
or a newer card, and the answer will probably be a newer card: by then the
question this work exists to answer will have been answered.

**Nothing here is a security claim.** The GPU kernel is not constant-time, the
error sampler is prototype-grade, and no parameter set has been checked against
a lattice estimator. `README.md` leads with a privacy claim and this ADR does
not discharge any part of it.

## Alternatives considered

**Install CUDA 12.x alongside 13.3.** Would work, since 12.x supports `sm_61`
natively, at about 3 GB, administrator rights, and two toolkits on the machine.
Rejected as heavier than the problem: what was needed was a compiler front end,
and the wheel is exactly that with none of the rest.

**Use the CUDA runtime API and a `cudart` from a wheel.** Also possible. Adds a
second moving part without removing the first, since the offline-compilation
problem is upstream of which API the host code uses.

**Write the host in Python (CuPy) instead of Rust.** The fastest route to a
number, and rejected for one reason: the CPU arm and the GPU arm have to agree
bit for bit, and that check is worth much more when both answers are produced in
one process from one database by one client. Splitting the arms across two
languages would have made the strongest verification available the weakest thing
in the build.

**Relax `unsafe_code` to `deny` across the workspace.** Rejected. One crate
needs it; a workspace-wide change would put every future crate one edit away
from unsafe code, and the point of `forbid` is precisely that it cannot be
relaxed from the inside.

**Skip the GPU and stay on the CPU baseline.** The defensible reading of the
prohibition above, and rejected because the marginal cost was an afternoon on
hardware already present, and because ADR 0001 explicitly wanted a GPU data
point before the FPGA question was reopened.

## References

- `docs/adr/0001-gpu-before-fpga-for-pir.md`, GPU before FPGA; DPF-PIR refused
  on threat model; the standing B = 1 rule.
- `docs/threat-model.md`, the single-operator constraint that point 5 applies to
  the GPU arm, stated on its own and generalised past DPF-PIR.
- The internal planning document that named SimplePIR as the first
  single-server scheme to measure, and the PIR multiplier as the metric, is not
  published here; `SCOPE.md` section 5 says so and explains what else the port
  held back.
- The CUDA driver bindings, and the only `unsafe` involved, are in the
  implementation under measurement, which is not published in this repository.
  No path inside it is cited here; `SCOPE.md` states what was held back and why.
- `tools/build_ptx.py`, the NVRTC 12.9 path, with the `nvcc` failure it exists
  to route around quoted in its docstring, and the three things a byte-for-byte
  rebuild needs.
- `measurements/gpu-jit-cold-2026-08-28.txt`, the cold and warm driver JIT
  measurement that point 2 keeps outside every per-query figure, and
  `docs/errors-caught.md` section 4 for the remembered figure it replaced.
- `docs/methodology.md`, for the host and card description every GPU figure
  carries, and `measurements/RESULTS.md` for the figures themselves.
- `LICENSES.md`, for the NVRTC wheel and CUDA driver rows, including the open
  EULA question on distributing compiled output.
- CUDA 13.0 release notes, removal of offline compilation support for Maxwell,
  Pascal and Volta.
