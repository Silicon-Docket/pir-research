# Single-server PIR over a fixed-width existence index

**What one private lookup actually costs, measured on commodity hardware.**

This repository records what single-server computational PIR costs when it is
run over a fixed-width existence-check index: 6,712,448 records at 128 bytes
each, m = 29312, 859.2 MB. It carries the measurements for our own SimplePIR
implementation on one CPU core, on twelve threads and on a 2016 consumer GPU,
and an independent evaluation of four third-party PIR implementations built from
source and re-measured at a record width their own papers never tested.

The published results are the measurements and the harness for the denominator
of the metric (`crates/pir-bench`). The SimplePIR implementation those timings
came from, referred to throughout as *the implementation under measurement*, is
not published here; `SCOPE.md` says what is in and what is out.

Nothing here is a security artefact. See *Status and limits* before citing any
number in this file.

## Why single-server

The setting has **one operator**. Any scheme whose privacy rests on two
non-colluding parties, two-server DPF-PIR, secret-shared retrieval, two-party
SMPC, provides no guarantee when both parties are the same organisation. It is
not weakened in that setting, it is **absent**: an operator holding both additive
shares of a one-hot vector reconstructs the queried index exactly.

This matters because those schemes are faster, and the literature is full of
them. Anyone who reads a benchmark table, sees a two-server construction winning
by an order of magnitude and reaches for it is reaching for a number that does
not apply to this threat model. The reasoning is in `docs/threat-model.md` and
in `docs/adr/0001-gpu-before-fpga-for-pir.md`.

The scheme measured is **SimplePIR** (Regev/LWE), chosen because its online
server cost is one pass over the database and nothing else.

## The declared reference workload

Throughout, comparisons are made against a **declared reference workload**:

- arrival rate **lambda = 0.06 queries/second** (one query every ~16.7 s),
- **Q = 157 queries per client per rebuild period**.

These are stated evaluation parameters, like a database size, not measurements.
Substitute your own: the crossovers below move with Q, and the headroom figures
move with lambda. Every latency, bandwidth and throughput figure in this
repository is a measurement and is independent of both.

## Headline results

All figures at m = 29312 (859.2 MB), **B = 1** always, on an 11th Gen Intel Core
i5-11400 @ 2.60 GHz (6C/12T, 12 MiB L3, 15.8 GB RAM) with a GeForce GTX 1070
(compute capability 6.1, 8 GiB, 256-bit bus). Multiplier is against that same
host's own linear-scan floor, measured in the same sitting.

| Online query, B = 1 | latency | throughput | multiplier | spread |
|---|---|---|---|---|
| 1 CPU core | **80.3 ms** | 10.70 GB/s | **2.14x** | 5.6% |
| 12 CPU threads | **28.0 ms** | 30.73 GB/s | 0.99x | |
| GTX 1070 | **4.02 ms** | 213.8 GB/s | **1.01x** | 0.1% |

The 0.99 row is not the answer beating the floor. Both figures describe the same
memory bus at the same width, and the honest reading is *indistinguishable from
the floor*. On the GPU, 213.8 GB/s is 83% of the card's 256.3 GB/s theoretical
peak and within 1% of the same card summing the same bytes with no cryptography
at all: the online cost is the cost of reading the database and nothing more.

Costs a latency figure hides, stated with it rather than after it:

- **Hint: 120.1 MB**, client-independent, one object per database version,
  broadcast identical to every client. Generated in **89.6 s on one core**
  (0.0249 core-hours), 25.0 s across twelve, **1.17 s on the GTX 1070**. The CPU
  rate, 9.59 MB/s/core, is 2.4x the published CPU figure for SimplePIR
  preprocessing (YPIR, ePrint 2024/270). Generation is not the cost;
  distribution is.
- **Per-query wire traffic: 117.2 KB up and 117.2 KB down to retrieve 128
  bytes**, an expansion of roughly 1,800x.
- The hint size is externally corroborated: SimplePIR's authors report a 121 MB
  hint for a 1 GB database, and this implementation, written independently from
  the construction with no shared code, produces 120.1 MB at 859 MB.

Against the declared reference workload, one desktop core answers a private
query over the whole index with **~208x headroom** over lambda, twelve threads
with ~596x, the GPU with ~4,150x. No FPGA is needed, and no GPU either.

Full tables, sample definitions, verification checks and the costs excluded:
`measurements/RESULTS.md`.

## The PIR multiplier

The headline is a ratio, not an absolute queries/second, because a rented vCPU's
absolute throughput transfers to nothing:

> **PIR multiplier = (seconds/query under PIR) / (seconds to linearly scan the
> same database on the same host)**

PIR must touch every byte of the database on every query. That *is* the privacy
guarantee, not an implementation detail, so a plain linear scan is the floor no
scheme can beat. The floor moves with whatever hardware you rent; the multiplier
is a property of the scheme. `crates/pir-bench` measures the denominator, and it
is published so the denominator can be checked on other hardware.

Two rules the floor imposes, both in `docs/methodology.md`: sweep across the
cache cliff (a curve flat across the host's L3 boundary is measuring cache and
is a failed run, not a fast one), and report B = 1 (batched throughput may
appear beside it, never instead of it; at lambda = 0.06 queries/second no batch
forms).

## What this repository demonstrates

**Four third-party implementations built from source and re-measured at 128-byte
records**, which is the width at which published one-byte tables stop predicting
anything:

| | source | result at our shape |
|---|---|---|
| `simplepir` 1.0.1 | `XiXinping/simplepir-rs`, MIT | different parameter set (q = 2^64, n = 2048): 480.2 MB hint against our 120.1 MB, 4x the paper's published size |
| **YPIR** (USENIX Sec '24) | `menonsamir/ypir` @ `a73e550` | hint-free claim holds (0 bytes), 155 ms server, but one-byte items, so a 128-byte record costs 128 queries: **behind our hint after 1.1 queries** |
| **VIA** (IEEE S&P '26) | `owniai/VIA` @ `f65aa9d` | records are >= 512 B, so one query covers one of ours: 93.2 MB against our 156.9 MB at Q = 157, and **stays ahead to 335 queries per rebuild period** |
| **HintlessPIR** (CRYPTO '24) | `google/hintless_pir` @ `812babf` | 264-byte public params, but 22.65 MiB down per query and 113.65 s server time per query: **behind our hint after 5.0 queries** |

None is vendored, linked against, or in any manifest. Terms for every one are in
`LICENSES.md`. Write-ups: `measurements/crate-comparison-2026-08-28.md`,
`measurements/external-2026-08-29-ypir-via.md`,
`measurements/external-2026-09-04-hintlesspir.md`.

**Two defects found in a third-party artifact.** VIA's First Dimension loop
reads `database[0]` on every iteration rather than streaming the database, so
84% of its measured answer time is a floor and not a measurement of the work the
scheme actually requires; and its harness never checks that the record it
recovered is the record that was stored. Both are recorded with the run rather
than used to dismiss the scheme: VIA is still the only external scheme ahead of
our hint on total client bytes, and its throughput gap to us is a lower bound
until it reads a database it actually touches.

**Ten of our own errors, caught before publication and recorded rather than
quietly fixed** (`docs/errors-caught.md`). Seven were found while the work was
being done and three more while it was being prepared for this repository, by
reading each written claim back against the artifact behind it. The two the
summary above rests on: building at Rust's
default x86-64 target, which has no 32-bit vector multiply, would have reported
a multiplier of 3.55x instead of 2.13x at m = 16384: two thirds too large, and
it would have been read as a fact about SimplePIR rather than about a build
flag. The first form of the hint kernel, the obvious full-row `axpy`, measured
2.5 GMAC/s, 0.6x the published figure it was meant to beat; strip-mining the
contraction reached 10.0 GMAC/s, 2.5x. Both were caught by the same mechanism,
having a number from outside this repository to check against, which is also the
mechanism that caught the two VIA defects.

**A record-size study over a public corpus.** 100,394 records from 993 volumes
of the Caselaw Access Project (CC0), 0 volume misses, stratified across
{federal, state} x {pre-1950, 1950-2000, post-2000}. Metadata only, no opinion
text. Measured payload 39.75 B/record, and at a 128 B fixed-width entry
**99.970% of records are held intact**, which is where m = 29312 and 859.2 MB
come from. Where the sample is weak is stated in `measurements/RESULTS.md` rather
than smoothed: the strata are not equally represented, and five first-series
regional reporters are absent from the source under the slugs tried.

## Layout

| Path | What it is |
|---|---|
| `SCOPE.md` | What is published here and what is held back, and why |
| `docs/threat-model.md` | One operator, and what that rules out |
| `docs/methodology.md` | B = 1, the cache-cliff sweep, sample definitions, the multiplier |
| `docs/errors-caught.md` | Our own measurement errors, with the numbers they would have published |
| `docs/adr/0001-gpu-before-fpga-for-pir.md` | Why a GPU was measured before an FPGA was rented, and why a two-server scheme is not adopted |
| `docs/adr/0002-pascal-without-a-pascal-toolkit.md` | Running the GPU arm on Pascal after the toolkit dropped Pascal |
| `docs/cap-terms-2026-08-28.txt` | CAP's terms, as retrieved on that date |
| `measurements/RESULTS.md` | Every figure, with its host, its sample definition and its verification checks |
| `measurements/crate-comparison-2026-08-28.md` | The `simplepir` crate, read and measured |
| `measurements/external-2026-08-29-ypir-via.md` | YPIR and VIA, built and run |
| `measurements/external-2026-09-04-hintlesspir.md` | HintlessPIR, built and run |
| `measurements/*.json`, `*.txt` | Raw artifacts, committed as produced |
| `crates/pir-bench` | The linear-scan floor: the denominator of the multiplier |
| `tools/fetch_cap.py` | Fetches CAP volume metadata for the record-size study, resumable, validates structurally |
| `tools/build_ptx.py` | Compiles CUDA kernel source to PTX (ADR 0002); the kernel itself is not published here |
| `.cargo/config.toml` | `target-cpu=native`, carrying the measurement that says why deleting it would inflate the metric |
| `LICENSES.md` | Terms of every corpus and third-party input, including the ones recorded as unchecked |
| `LICENSE` | Apache-2.0 |

## Why this is public

Silicon Docket publishes this repository as evidence of practice: applied
cryptographic engineering, an explicit threat model, reproducible measurement,
and independent evaluation of third-party cryptographic implementations,
including of our own errors. It supports an application to Anthropic's Cyber
Verification Program, which vets organisations for legitimate dual-use security
work. That is the occasion for publishing, not the content: everything here was
measured for its own sake first.

## Status and limits

Stated plainly, because each one bounds what these numbers are good for.

- **Nothing here is a security artefact.** The error sampler is a centred
  binomial, the secret comes from a general-purpose PRNG, and **no parameter set
  has been through a lattice estimator**. That is the top open item. A
  back-of-envelope root-Hermite estimate is not an estimator run.
- **Neither the CPU loop nor the GPU kernel is constant-time.** Neither is the
  third-party crate's sampler. None of them claims to be.
- **The corpus the timings ran against is synthetic.** The *shape* is faithful,
  128 B fixed-width records, 6,712,448 of them, and the timings depend only on
  the shape. The recovered records are a test of the pipeline and are not facts
  about case law. The record-size study over CAP is separate and is real data.
- **Two hosts appear in the measurements and they are not comparable to each
  other.** Every figure names its own host, because the container this work
  started in was rescheduled onto a different CPU mid-session and L3 moved by
  nearly an order of magnitude.
- **The implementation under measurement is not published here**, so the CPU and
  GPU SimplePIR timings cannot be reproduced from this repository alone. The
  floor harness, the raw artifacts and the external-scheme write-ups can be.
  `SCOPE.md` is the record of that boundary.
- Verification of the published figures is described where each figure is: the
  decode checks, the CPU/GPU bit-identity checks and what each does *not* cover
  are in `measurements/RESULTS.md`, and one of them ("decode OK" in a sweep
  checks one record per size) is weaker than it looks.
