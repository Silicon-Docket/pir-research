# Scope

What is in this repository, what is not, and why.

The study behind this repository measures what single-server computational PIR
costs over a fixed-width existence-check index built from a public caselaw
corpus: 6,712,448 records at 128 bytes each, `m = 29312`, 859.2 MB. It was
carried out inside a private repository at Silicon Docket, and this is the
public port of it. `README.md` says what the work found and why it is public.

This file is the inventory. Every claim made in the other documents here is
either reproducible from what is published, or it is not, and the point of this
file is to say which, without the reader having to work it out.

---

## 1. What is published

### Findings and write-ups

| Path | What it carries |
|---|---|
| `README.md` | The result in short, and the constraint the whole study sits under: one operator, no second server |
| `measurements/RESULTS.md` | Every figure, with its host, its sample definition and its verification checks. The primary document |
| `docs/methodology.md` | How each number was produced, and where each sample is weak |
| `docs/threat-model.md` | What single-server PIR does and does not protect, and why two-server constructions were excluded rather than benchmarked |
| `docs/errors-caught.md` | The measurement errors found before publication, each with the outside number that caught it |
| `docs/adr/0001-gpu-before-fpga-for-pir.md` | The hardware decision, and the rules the measurements had to follow to settle it |
| `docs/adr/0002-pascal-without-a-pascal-toolkit.md` | Targeting a GPU architecture that the installed toolkit will no longer compile for |
| `measurements/external-2026-08-29-ypir-via.md` | YPIR and VIA, built and run at this index shape |
| `measurements/external-2026-09-04-hintlesspir.md` | HintlessPIR, built and run at this index shape |
| `measurements/crate-comparison-2026-08-28.md` | An independent third-party Rust SimplePIR, read and measured against ours |
| `LICENSES.md` | The terms of every corpus and third-party input, including the ones recorded as unchecked |
| `LICENSE` | Apache-2.0, governing the source code here and nothing that the source code reads |
| `docs/cap-terms-2026-08-28.txt` | The Caselaw Access Project terms, captured verbatim on the date they were read |

Two hosts appear throughout and they are not comparable to each other. A shared
Xeon vCPU at 2.80 GHz (4 threads, 33 MiB L3, no GPU) produced the first floor; a
desktop i5-11400 at 2.60 GHz (6C/12T, 12 MiB L3, GTX 1070) produced everything
after it. Every figure names its own.

### Raw measurement artifacts

All of `measurements/` is committed as produced. A measurement that lives only
in a terminal did not happen.

| Artifact | What it is |
|---|---|
| `record-size.json` | 100,394 records from 993 CAP volumes, 0 volume misses, six strata. The one corpus study here that is reproducible end to end from public CC0 data |
| `floor-2026-08-28.json` | The linear-scan floor on the shared Xeon. The third of three sweeps and the only one committed |
| `floor-2026-08-28-i5-11400.json` / `.txt` | The floor on the desktop, with a readable knee (3.11x in cache against out) |
| `cpu-simplepir-2026-08-28.json` / `.txt` | The CPU answer sweep, eight sizes, B = 1 |
| `gpu-simplepir-2026-08-28.json` / `.txt` | The GPU answer sweep, same eight sizes, same client code |
| `cpu-hint-2026-08-28.json` / `.txt` | Hint generation on the CPU, both serial and parallel |
| `gpu-hint-2026-08-28.json` / `.txt` | Hint generation on the GTX 1070, with per-launch times kept as a series |
| `baseline-target-cpu-2026-08-28.txt` | The same source built for the default target, produced into a separate directory so the two builds could never be mixed |
| `gpu-jit-cold-2026-08-28.txt` | Cold and warm driver JIT, 33.2 ms against 0.6 ms |
| `external-2026-08-29-ypir.json` / `.txt` | The YPIR reference implementation at 859.2 MB |
| `external-2026-08-29-via.txt` | The VIA reference implementation at a reduced column count |
| `external-2026-09-04-hintlesspir.txt` | The HintlessPIR reference implementation at `db_rows = 229`, `db_cols = 29312` |
| `crate-comparison-2026-08-28.txt` | Raw output behind the third-party crate comparison |

### Code

| Path | What it is |
|---|---|
| `crates/pir-bench/` | The linear-scan floor harness: the denominator of the headline ratio. No dependencies, on purpose |
| `tools/fetch_cap.py` | The corpus sampler. CAP metadata only, resumable, and it validates structurally rather than on status code because a missing volume comes back as an HTML error page, and not always with a 404 |
| `tools/build_ptx.py` | The PTX build tool: pinned NVRTC 12.9.86 to `compute_61`, with the output normalised so a second machine gets the same bytes |
| `.cargo/config.toml` | `-C target-cpu=native`, with the measurement that justifies it in the comment |
| `Cargo.toml`, `_typos.toml`, `.gitignore` | Workspace and lint configuration. Lints are configuration rather than CI flags so that local and CI runs enforce the same thing |

Two of those are worth a sentence each.

**The floor is a library, not a script.** PIR must touch every byte of the
database on every query, which is the privacy guarantee itself rather than an
implementation detail, so a plain linear scan is a floor no scheme can get
underneath. `crates/pir-bench` measures that floor, sweeps it across the host's
cache hierarchy and reports the knee, and it exposes the scan as a library
because the implementation under measurement divides by it: numerator and
denominator come from the same loop on the same run, and two scan loops that are
merely meant to stay identical will not. The library is published. The thing
that divided by it is not, which is section 2.

**`.cargo/config.toml` is a finding as much as a setting.** Rust's default
target is baseline x86-64, which has no 32-bit vector multiply, so the
byte-times-word multiply-accumulate at the centre of SimplePIR compiles far
narrower than the machine can issue. Measured at `m = 16384`, both rows from the
same source one build apart: the default target gives 21.69 GB/s floor, 6.11
GB/s answer, multiplier 3.55x; `target-cpu=native` gives 22.94 GB/s floor, 10.78
GB/s answer, multiplier 2.13x. The floor moved 5.8% between the two runs and the
answer moved 76%, so the comparison survives being assembled from two runs, and
quoting the floors is what makes that checkable rather than asserted. Without
the setting the reported multiplier would have been two thirds too large and
would have been read as a fact about SimplePIR. The file is published because
deleting it for portability would silently inflate the metric again.

---

## 2. What is held back, and why

**Our SimplePIR implementation is not published.** That covers the CPU server
and hint generation, the GPU arm that runs the same scheme through the CUDA
driver API, and the CUDA kernel the GPU arm compiles. Where those must be
referred to in the other documents, they are called "the implementation under
measurement" or "our SimplePIR implementation", and no path inside them is
cited.

The reason is that the retrieval scheme is the part of this work the
organisation treats as proprietary. That is a commercial decision and it was
made deliberately rather than by default: the alternative, publishing the
implementation and letting reviewers re-run every figure, was considered and
declined, and the cost of declining it is set out in this section instead of
being left for a reader to discover. Everything that could be published without
publishing the scheme was published, including the raw artifacts that would let
someone check our arithmetic even though they cannot check our code.

### What a reader therefore cannot do

**You cannot re-run the SimplePIR numbers or the hint numbers.** Specifically,
the following are reported here and are not reproducible from this repository:

- The answer sweep. 80.3 ms/query on one core, 28.0 ms across twelve threads and
  4.02 ms on the GTX 1070, all at B = 1 and `m = 29312`, with multipliers of
  2.14x, 0.99x and 1.01x against the floor at the same size.
- Hint generation. 89.6 s on one core, 25.0 s across twelve, 1.17 s on the GTX
  1070, which is 0.0249 core-hours for a whole index rebuild.
- The hint and the wire cost. 120.1 MB of hint per database version, and 117.2
  KB up and 117.2 KB down per query to retrieve 128 bytes.
- The verification checks behind those figures: the GPU answer being bit
  identical to the CPU answer at all eight sizes, all 30,015,488 words of the
  hint agreeing between the two arms, `H.s == D.(A.s)` at full size, and the
  decode margins.

Those are reported figures, not reproduced ones. The distinction is the point of
saying it here.

**You also cannot regenerate the committed PTX byte for byte**, because the
kernel source it is compiled from is part of what is held back.
`tools/build_ptx.py` is published for its method rather than for our output.

### What a reader can do

1. **Re-run the floor.** `crates/pir-bench` is the harness that produced every
   denominator here, and it runs on any x86-64 machine with no dependencies. It
   sweeps 2 MiB to 2048 MiB, runs each size for at least 50 ms, and fails a run
   whose curve is flat across the host's last-level cache rather than reporting
   it as a fast one. On the desktop it reports roughly 22 GB/s single-thread and
   30 GB/s across twelve threads out of cache, with a knee of 3.11x.
2. **Re-derive the record-size study from the public corpus.**
   `tools/fetch_cap.py` samples CAP metadata under CC0 and computes the same
   statistics: 100,394 records, 993 volumes, 0 misses, a 39.75 B payload, and
   99.970% of records intact at a 128 B entry width, from which `m = 29312` and
   859.2 MB follow. Nothing in this path depends on anything held back.
3. **Rebuild the PTX toolchain, against any kernel.** `tools/build_ptx.py`
   takes the kernel, the output path and the target architecture as arguments;
   it fetches a pinned NVRTC 12.9.86 into a gitignored directory, compiles for
   an architecture the installed nvcc may refuse, rewrites the `.file` directive
   NVRTC emits from an absolute path to one relative to the working directory,
   and writes LF, so that a second machine compiling the same kernel gets the
   same bytes. `--check` recompiles and compares rather than writing, so a stale
   PTX is caught rather than shipped.

   It was verified end to end while this repository was assembled, on a machine
   with **no CUDA toolkit and no GPU at all**: the wheel fetched, a scratch
   kernel compiled for `compute_61`, the `.file` directive normalised, and
   `--check` correctly rejected the PTX after the kernel changed. The kernel it
   was originally written for is part of what is held back, so what reproduces
   is the method, which is the subject of
   `docs/adr/0002-pascal-without-a-pascal-toolkit.md`.
4. **Rebuild all four third-party implementations and re-run them at this
   shape.** This is how every comparative claim here was made. Each external
   scheme was built and run at `m = 29312` with 128-byte records, because every
   published figure for these schemes is measured at 1-bit or 1-byte records and
   record width is exactly where the first candidate collapsed. The commits are
   recorded in `LICENSES.md` and repeated in the external write-ups:

   | Implementation | Commit or version | Terms | Write-up |
   |---|---|---|---|
   | `github.com/XiXinping/simplepir-rs` (`simplepir` crate) | 1.0.1 | MIT | `measurements/crate-comparison-2026-08-28.md` |
   | `github.com/menonsamir/ypir` | `a73e550` | MIT | `measurements/external-2026-08-29-ypir-via.md` |
   | `github.com/owniai/VIA` | `f65aa9d` | MIT | `measurements/external-2026-08-29-ypir-via.md` |
   | `github.com/google/hintless_pir` | `812babf` | Apache-2.0 | `measurements/external-2026-09-04-hintlesspir.md` |

   None of the four is vendored, linked against, or present in any manifest.
   `LICENSES.md` records what each build pulled in, and what was not checked:
   the HEXL version behind the VIA run was not recorded and is not recoverable
   from the static library, which is a defect in that measurement and is stated
   as one.

### What an outside reader can still check about the held-back code

Three of this repository's numbers have independent counterparts that anyone can
obtain without our source:

- The SimplePIR authors report a 121 MB hint and 242 KB of communication per
  query for a 1 GB database. This implementation, written from the construction
  with no shared code, produces 120.1 MB for an 859 MB database and 234 KB per
  round trip.
- YPIR's own internal server-side SimplePIR hint at this database size is 117.4
  MB, which agrees with our independently measured 120.1 MB to within 2.2%.
- The published CPU throughput for SimplePIR preprocessing (YPIR, ePrint
  2024/270) is under 4 MB/s per core. Ours measures 9.59 MB/s per core, and the
  rate is flat at 8.90 to 9.91 MB/s per core across a 512-fold range of database
  sizes, which is what makes it a property of the core rather than of the
  working set.

Landing on published figures from a different direction is evidence that the
parameter set is the one the scheme intends, and it is the strongest check
available while the code is closed. It is weaker than releasing the code. Both
halves of that sentence are meant.

---

## 3. The declared reference workload

Throughout this repository, comparisons are made against a **declared reference
workload**:

> **lambda = 0.06 queries/second** (one arrival every roughly 16.7 seconds),
> and **Q = 157 queries per client per rebuild period.**

These are **evaluation parameters, not measurements**. They are stated inputs in
the same sense that the database size is a stated input, they were not derived
from any observed traffic, and no claim is made that they describe anyone's real
demand. Substitute your own. This section is the single place they are defined;
the other documents refer here rather than redefining them.

**What depends on lambda.** Every headroom figure. A single desktop core answers
a query over the whole 859 MB index in 80.3 ms, which is roughly 208x the
arrival rate; twelve threads give roughly 596x, and the GTX 1070 roughly 4,150x.
The same parameter is what makes HintlessPIR's 113.65 s of single-core server
time per query a backlog rather than a latency figure: it is 6.8x the entire
inter-arrival budget, so the queue never drains.

**What depends on Q.** Every total-client-bytes comparison. Over one rebuild
period our own design sends the 120.1 MB hint plus Q round trips at 117.2 KB
each way, which is 156.9 MB. Against that, VIA sends 93.2 MB, HintlessPIR sends
3,809.6 MB (24x worse) and YPIR sends 17,656 MB (113x worse).

**What depends on neither.** The crossover points are properties of the schemes
rather than of the workload: YPIR falls behind our hint after 1.1 queries,
HintlessPIR after 5.0, and VIA stays ahead until 335 queries per rebuild period.
Q only determines which side of each crossover you are standing on. Also
independent of both parameters: the multipliers, the floor, the hint size,
per-query bytes, and per-query server time.

So a reader who substitutes a different workload should expect the headroom
figures to scale with lambda and the byte totals to move with Q, while the
crossovers stay where they are and tell them what the new ordering is. A
workload ten times denser divides the headroom by ten; it does not change which
scheme is cheapest at 335 queries.

---

## 4. Standing limitations

These hold over everything in this repository and are not repeated at every
figure.

### Nothing here is a security artefact

The error sampler is a centred binomial, the secret comes from a general-purpose
PRNG, no parameter set has been checked against a lattice estimator, and neither
the CPU loop nor the GPU kernel is constant-time. The decode margin is asserted
(41 sigma to 196 sigma across the sweep) because a shrinking margin is where a
wrong answer would come from: that is a correctness property, not a security
one. No privacy claim is made to anyone on the strength of any of this.

### The corpus the timings ran against is synthetic

It is shape faithful and nothing more: 128-byte fixed-width records, `m/128`
records per column, 6,712,448 records at `m = 29312`. The timings depend only on
the shape, so the synthesis does not weaken them. The records recovered by the
end-to-end decode checks are a test of the pipeline and are **not facts about
case law**. Building the index from real canonical keys is not part of this
work.

### The record-size study is real data with stated weaknesses

That study is over actual CAP metadata rather than synthesis, and its limits are
stated rather than smoothed. The strata are not equally represented, because
round-robin sampling exhausted the thin ones: federal/pre-1950 contributes
42,216 records and federal/1950-2000 only 2,883, so the overall mean is weighted
toward pre-1950. The check that matters is that the unweighted mean of the six
stratum means is 38.91 B against a weighted 39.75 B, and every stratum sits
between 36.16 and 44.19. Five first-series regional reporters are absent from
the source under the slugs tried, so pre-1950 state coverage comes from early
second-series volumes. The record count of 6.7M is CAP's own description of its
coverage and is not something this sample measured. Full statement in
`docs/methodology.md`.

### The measurement environment

The desktop was running a desktop, and several spreads above 10% say so; they do
not touch the knee, whose two sides differ by 3.1x. The GPU drives a display,
which cost one row a 379% spread with its median in line with its neighbours,
and which is why the hint kernel probes and sizes its launches against a wall
clock budget rather than an assumed rate. Of the three shared-Xeon floor sweeps
only one is committed, so the 0.6% to 3.7% band quoted for that host cannot be
re-derived from this repository. In the sweeps, one record per size is decoded
end to end, at the midpoint of the database; four positions are checked only at
`m = 256`, so "decode OK" in a sweep row is not exhaustive over the layout.

---

## 5. Provenance

This material is ported from a private internal repository at Silicon Docket.

**The numbers are unchanged.** Nothing was re-derived, re-rounded or re-run for
publication. Where a figure appears here it is quoted from the artifact that
produced it, and the raw artifacts in `measurements/` are the same files.

**What was removed** is material specific to the organisation's commercial work:
product framing, the internal interfaces the real index was to be built from,
and deployment and cost planning. No measurement was removed. Where a figure's
original justification was specific to that commercial work, the figure is kept
and restated against the declared reference workload of section 3, which is why
lambda and Q are presented as stated parameters rather than as findings.

**The ADRs are renumbered.** What was private ADR 0004 is
`docs/adr/0001-gpu-before-fpga-for-pir.md` here, and what was private ADR 0005
is `docs/adr/0002-pascal-without-a-pascal-toolkit.md`. Their internal
self-references are renumbered with them, and every citation of an ADR elsewhere
in this repository uses the new numbers.

**The house style is preserved, including the parts that look untidy.** A
correction names the sentence it supersedes rather than deleting it, so the
write-ups carry claims that turned out to be wrong alongside the measurements
that replaced them: a cold JIT figure of 102 ms that was remembered rather than
measured and is really 33.2 ms, a line saying hint generation was the largest
unmeasured quantity here, record-size figures from a 1,862-record sample that
was never committed, and a reading that filed VIA beside YPIR as a one-byte
scheme when its records are 512 bytes. A reader will therefore meet superseded
sentences from drafts that were never public. They are kept because a correction
that deletes its own subject is not checkable, and because the sequence of
errors is itself part of what `docs/errors-caught.md` reports. For the same
reason some phase numbering survives from an internal planning document that is
not published here.
