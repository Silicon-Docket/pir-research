# Inputs, and their terms

Every corpus and third-party input this work touches, with what has been checked
and what has not. `LICENSE` governs the source code in this repository
(Apache-2.0); this file governs everything that source code reads, builds,
links against or merely observes.

The record is part of the result rather than housekeeping. This repository
measures other people's cryptographic implementations: four of them were fetched,
built from source and re-measured at a record width their own papers never
tested. A reader deciding what those numbers are worth needs to know exactly what
was read, what was built, what was linked against and what was only looked at,
and under whose terms each of those happened. A loose record of inputs would make
the measurements harder to trust, not just harder to audit.

## Corpora

| Input | Terms | Checked | Notes |
|---|---|---|---|
| **Caselaw Access Project** (`static.case.law`) | **CC0 1.0**, for the caselaw data *and metadata* | **Yes (2026-08-28)**, read verbatim from `case.law/terms` and captured at `docs/cap-terms-2026-08-28.txt` | Closes what was, until that date, the single highest-priority open item for this work. The page is a JavaScript module and its HTML carries none of the wording, which is why an earlier attempt recorded it as uncapturable; the text is in `case.law/templates/cap-terms-page.js`. Harvard states it does **not** impose legally binding conditions, and separately *requests* attribution to Harvard and CAP as Community Norms: not a licence condition, cheap to honour, and a matter of practice rather than a legal obligation. The CC BY-SA 4.0 on that page governs the **site text**, not the data; do not carry it into anything about the corpus. |
| **CourtListener bulk exports** | CC BY-ND 4.0 | Yes | **Not used here.** The ND term is precisely why CAP is the base corpus for the record-size study. Recorded so the choice is visible rather than accidental. |

CAP's per-volume `CasesMetadata.json` is the only thing read, by
`tools/fetch_cap.py`. It carries the citation, the abbreviated case name and the
decision date, which is the whole existence-check record. No opinion text is
downloaded.

## Code and research

| Input | Terms | Checked | Notes |
|---|---|---|---|
| Tan et al., TCHES 2026(3):933–957 | CC-BY 4.0 (the paper) | Yes | Read and analysed in `docs/adr/0001-gpu-before-fpga-for-pir.md`. Its *scheme* is two-server and is not adopted. |
| `github.com/Crypto-Lab-WHU/GPU-PIR` | **Unknown** | **No** | The artifact accompanying that paper. **Read for technique only. Not cloned, not vendored, not linked against.** The posture recorded in `docs/adr/0001-gpu-before-fpga-for-pir.md`: an unread licence is not a dependency. If it is ever to be used, its terms come into this table first. |
| **`simplepir` 1.0.1** (crates.io) | **MIT** | **Yes (2026-08-28)** | An independent Rust SimplePIR, `github.com/XiXinping/simplepir-rs`. **Read and measured for comparison; not linked against and not in any manifest**, the same posture taken to `GPU-PIR`. MIT would permit linking, and this work still does not: the implementation under measurement (not published here, see `SCOPE.md`) has no dependencies on purpose. The comparison is `measurements/crate-comparison-2026-08-28.md`; it found our parameters match the paper's and the crate's do not. |
| **`github.com/menonsamir/ypir`** @ `a73e550` | **MIT** | **Yes (2026-08-29)** | The YPIR reference implementation (USENIX Security '24). **Built and run at our 859.2 MB, not linked against and not in any manifest**, same posture as the `simplepir` crate. MIT would permit vendoring; the implementation under measurement still has zero dependencies on purpose. Result: `measurements/external-2026-08-29-ypir-via.md`. Its one-byte items rule it out for us. |
| **`github.com/owniai/VIA`** @ `f65aa9d` | **MIT** | **Yes (2026-08-29)** | The VIA reference implementation (IEEE S&P '26). Built and run at a reduced `LOG_COL`; not linked against, not in any manifest. **It links Intel HEXL** (`lib/libhexl.a`, Apache-2.0), and that dependency comes into this table too if VIA is ever adopted, along with HEXL's own transitive set. **The HEXL version used for this run was not recorded** and is not recoverable from the static library; if the measurement is ever repeated, pin it. Result: `measurements/external-2026-08-29-ypir-via.md`. The only external scheme so far that beats our hint on total client bytes at 128-byte records. |
| **`github.com/google/hintless_pir`** @ `812babf` | **Apache-2.0** | **Yes (2026-09-04)** | The HintlessPIR reference implementation (Li, Micciancio, Raykova, Schultz-Wu, CRYPTO '24 / IACR eprint 2023/1733), the third of the three hint-free schemes named in `measurements/RESULTS.md`. Built via Bazel and run at our real shape (`db_rows=229`, `db_cols=29312`, 128-byte records); not linked against, not in any manifest. **Its own build pulls four further repos as source**, each read only for this measurement, none vendored or redistributed: `google/protobuf` (BSD-3-Clause), `tink-crypto/tink-cc` (Apache-2.0), `google/highway` (Apache-2.0), `google/shell-encryption` (Apache-2.0), the last of which in turn needed `google/glog` (BSD-3-Clause) and `gflags/gflags` (BSD-3-Clause) resolved the same way. Result: `measurements/external-2026-09-04-hintlesspir.md`. Worse than our own hint on both total client bytes (24×) and server time per query (~1,415×) at our record width. |
| **NVIDIA CUDA driver** (`nvcuda.dll` / `libcuda.so`) | NVIDIA driver licence, as installed by the user | **No** | Linked at runtime, via `raw-dylib`, by the GPU arm of the implementation under measurement (not published here). Not redistributed and not vendored: it is the driver already on the machine. No CUDA *toolkit* is required to build or run. |
| **`nvidia-cuda-nvrtc-cu12`** (PyPI wheel, 12.9.86) | NVIDIA CUDA Toolkit EULA | **No, open item** | Used by `tools/build_ptx.py` as a *compiler*, because CUDA 13.3 cannot target this GPU's `sm_61` (`docs/adr/0002-pascal-without-a-pascal-toolkit.md`). Downloaded into gitignored `tools/.nvrtc/`, **not vendored and not redistributed**. The EULA question attaches to the compiler's *output*, PTX compiled from our own kernel source, and that artifact is committed only alongside the kernel, which is not published here (`SCOPE.md`). **No PTX is published in this repository.** What is published is `tools/build_ptx.py` itself: our own code, which invokes NVRTC rather than redistributing it, and which fetches the pinned wheel at run time into a gitignored directory. The EULA's terms on distributing compiled output have still not been read, so the row stays **unchecked**, the same posture the CAP row was in until it was read: recorded as unchecked rather than assumed benign. |
| **This repository's own code** (`crates/pir-bench`, `tools/`, `.cargo/config.toml`, the documents and the raw artifacts under `measurements/`) | **Apache-2.0** | n/a | `LICENSE`. Ours to license, and licensed permissively on purpose so the floor harness and the raw measurement artifacts can be re-run and re-checked without asking. |

## The rule

A dependency arrives in this table *before* it arrives in a manifest. An unread
licence is not a dependency, and a permissive one is not a reason to skip the
reading: anything built, linked against or redistributed has its terms looked at
first and written down here, including when the answer is "not yet". Three rows
above are marked **No** for exactly that reason, and are left marked **No** rather
than waved through because they happened to look benign.

There is no cargo-deny gate in this repository. The discipline is the point, not
the tooling.
