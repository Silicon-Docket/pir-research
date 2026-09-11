# Results

Committed as produced. A measurement that lives only in a terminal did not
happen: the container this work started in was ephemeral and was in fact
rescheduled onto a different CPU mid-session.

Two machines appear below and they are not comparable to each other. Every
figure names its own.

| Host | What it produced |
|---|---|
| Shared Xeon vCPU @ 2.80 GHz, 4 threads, 33 MiB L3, no GPU | the first floor |
| Desktop: i5-11400 @ 2.60 GHz, 6C/12T, 12 MiB L3, **GTX 1070** | everything after it |

Everything from the desktop was produced by one build of the workspace, in one
sitting, on 2026-08-28.

**The phase numbers are internal structure.** They come from an internal
planning document that is not published here, and they are kept because they are
how the work was sequenced and because several corrections below name the phase
they belong to. What each one was: **Phase 1**, the corpus record-size study;
**Phase 2**, building the index from real canonical keys, which is not part of
this work (`SCOPE.md`); **Phase 3**, building the fixed-width database, which is
unstarted; **Phase 4a**, the linear-scan floor; **Phase 4b**, the SimplePIR
online answer on CPU and on GPU; **Phase 4c**, hint generation on both arms;
**Phase 5**, the decision gate.

**The reference workload is declared, not measured.** Throughout, comparisons
are made against the declared reference workload of `SCOPE.md` section 3: an
arrival rate of **lambda = 0.06 queries/second** (one arrival every roughly 16.7
seconds) and **Q = 157 queries per client per rebuild period**. Those are stated
evaluation parameters, in the same sense that the database size is a stated
input; substitute your own. The latencies, throughputs and per-query byte counts
below are measurements and are independent of both. The headroom figures scale
with lambda, the total-client-bytes comparisons move with Q, and the crossover
points are properties of the schemes rather than of the workload.

**The implementation under measurement is not published here** (`SCOPE.md`
section 2), so the SimplePIR and hint figures are reported rather than
reproducible from this repository. The floor they are divided by is published:
`crates/pir-bench`. Every raw artifact quoted below is in `measurements/`.

> **Correction.** The four section headings below read *Phase 3a* and *Phase 3b*
> in an earlier draft of this file, and the first of them was committed that way.
> The internal plan numbers the linear-scan floor **4a** and the SimplePIR
> baseline **4b**; its Phase 3 is *build the fixed-width database*, which is
> unstarted. The superseded headings are the ones that read "Phase 3a" and
> "Phase 3b".

> **Every figure below is at m = 29312, and that size is now measured rather
> than assumed**: see *Phase 1* immediately below. The corpus the timings ran
> against is still synthetic; the *shape* it is synthetic in is not.

---

## Phase 1: the record size, settled

**Raw:** `measurements/record-size.json` · **Tool:** `tools/fetch_cap.py`
**Sample:** 100,394 records from 993 CAP volumes, **0 volume misses**,
stratified across {federal, state} x {pre-1950, 1950-2000, post-2000}.
Metadata only, no opinion text is downloaded.

Every figure in this file rested on a 1,862-record sample that was never
committed. This replaces it with one that is.

| | preliminary (1,862 records) | measured (100,394 records) |
|---|---|---|
| key | 12.0 B | **8.95 B** |
| case name | 29.2 B | **28.79 B** |
| payload | 43.2 B | **39.75 B** |
| intact at 128 B | 99.8% | **99.970%** |

**The key figure is a definition difference, not a disagreement.** 12.0 B is the
length of the citation *string*, `"100 A.2d 36"`, spaces and punctuation
included. 8.95 B is the **normalised** key, which is what the index stores. The
roughly 3 B gap is exactly the separators. The name figures agree to 1.4%, which
is the real corroboration of the earlier sample.

### m = 29312 does not move

At 128 B/entry and 6.7M records the database is 857.6 MB, so m is 29,285 before
rounding and **29,312** after rounding up to a multiple of 128: 859.2 MB,
holding 6,712,448 records. That is the value every measurement in this file
already used. **Nothing above needs re-running**, and
`docs/adr/0001-gpu-before-fpga-for-pir.md`'s discharge of its point 3, which was
explicitly conditional on this, stands as written.

### The worry that motivated the wider sample is dead

The earlier estimate predicted that modern state-court names would run far
longer, and that post-2000 state volumes were where the estimate was weakest.
Measured, that stratum is the *smallest* of the three post-2000 groups:

| stratum | n | mean | median | p95 | max |
|---|---|---|---|---|---|
| federal/pre-1950 | 42,216 | 41.50 | 39 | 67 | 167 |
| federal/1950-2000 | 2,883 | 36.88 | 34 | 59 | 104 |
| federal/post-2000 | 14,084 | 44.19 | 36 | 83 | 156 |
| state/pre-1950 | 24,672 | 36.16 | 31 | 59 | **186** |
| state/1950-2000 | 9,316 | 37.03 | 29 | 75 | 145 |
| state/post-2000 | 7,223 | **37.70** | 29 | 74 | 134 |

The longest record in the whole sample is a **pre-1950 state** case at 186 B.
Modern federal names are the long ones, not modern state names.

### Where this sample is weak, stated rather than smoothed

**The strata are not equally represented.** Round-robin exhausted the thin ones,
so federal/pre-1950 contributes 42,216 records and federal/1950-2000 only 2,883.
The overall mean is therefore weighted toward pre-1950. The check that matters:
the **unweighted mean of the six stratum means is 38.91 B** against a weighted
39.75 B, and every stratum sits between 36.16 and 44.19. The figure survives
reweighting, which is what the stratification was for.

**Five first-series regional reporters are absent** from `static.case.law` under
the slugs tried (`a`, `p`, `ne`, `nw`, `se`), so pre-1950 state coverage comes
from early second-series volumes rather than the true first series.

**6.7M is CAP's corpus size, not something this sample measured.** The sample
fixes the record *size*; the record *count* is taken from CAP's own description
of its coverage.

**The "post-2000" stratum is bounded above by CAP's coverage, which ends in
2019.** This was not stated when the stratum was defined and it should have
been, because the worry the wider sample was built to test is specifically that
*modern* names run long. Checked against `static.case.law` on 2026-09-11, the
last volume in each current-series reporter:

| reporter | volumes | year range |
|---|---|---|
| `a3d` | 253 | 2004-2019 |
| `p3d` | 447 | 2000-2019 |
| `sw3d` | 579 | 1993-2018 |
| `ne3d` | 130 | 2003-2018 |
| `f3d` | 935 | 1993-2019 |
| `f-supp-3d` | 386 | 2005-2019 |
| `us` | 572 | 1798-2017 |
| `a2d` | 967 | 1943-2010 |
| `so2d` | 999 | 1941-2009 |

There is also no third series at all for two regions: `nw3d` and `se3d` both
return 404, which is why `tools/fetch_cap.py` carries `nw2d` and `se2d` in its
third-series line. Those two slugs therefore appear twice in `REPORTERS`, and
the tool deduplicates with `dict.fromkeys` so they are not double-weighted.

So "post-2000" means **2001 to 2019**, and the conclusion of *The worry that
motivated the wider sample is dead* above, that the long-modern-names worry is
dead, holds over that window and says nothing about cases decided after 2019.
Nothing in this file depends on the missing years: m = 29312 is fixed by the
payload mean, the mean is stable to within 1.4% between two independently drawn
samples, and every stratum sits between 36.16 and 44.19 B. But the bound belongs
in the sample definition rather than in a reader's inference from a reporter
list.

### The 96 B option, named and not taken

| entry width | intact | truncated of 6.7M | DB | hint |
|---|---|---|---|---|
| 96 B | 99.563% | ~29,300 | 643 MB | 104 MB |
| **128 B** | **99.970%** | **~2,010** | **859 MB** | **120 MB** |

96 B is a 25% smaller database, a 25% smaller hint and a 25% cheaper query, for
about 27,000 more records whose *name* is truncated. Only the name truncates,
never the key: a truncated name is a display nuisance, and a truncated key is a
wrong answer.

**Not taken here**, because 128 B is what every committed measurement used and
changing it would invalidate them for a saving that is not currently binding on
anything. Recorded because it is a real engineering trade and the numbers to
make it with are now in hand.

---

## Phase 4a: the linear-scan floor (shared Xeon vCPU)

**Date:** 2026-08-28
**Host:** Intel Xeon @ 2.80 GHz, 4 threads, L1d 32K/core, L2 1024K/core,
L3 33 MiB shared, 15 GB RAM. Shared vCPU.
**Raw:** `measurements/floor-2026-08-28.json`, the third of three sweeps and the
only one whose raw data is committed. The headline spread is quoted from that
file (1% to 2%); the wider 0.6% to 3.7% band below spans all three sweeps and
cannot be re-derived from this repository.
**Status:** usable for the out-of-cache figure. **Not** usable for the knee.

### The number

| | single-thread | 4-thread |
|---|---|---|
| **Out-of-cache floor** (>= 512 MiB) | **~10 GB/s** | ~19 to 34 GB/s |
| Run-to-run spread there | 1% to 2% | 14% to 27% |

Derived, for the 858 MB existence index the record-size measurement implies:
**~86 ms per full scan, single-threaded.**

### What was not usable, and why

**The knee verdict was unreliable on that box.** Three runs of the same sweep
produced in-cache/out-of-cache ratios of **1.00x, 0.70x and 1.46x**, flipping
the harness's own pass/fail verdict each time. The cause was the in-cache
point, not the floor:

| 16 MiB (in-cache), single-thread, across three runs | 9.95 · 7.18 · 14.42 GB/s |
|---|---|
| >= 512 MiB (out-of-cache), same three runs | 9.5 to 10.9 GB/s, spread 0.6% to 3.7% |

At 16 MiB a pass finished in ~2 ms, which on a shared vCPU is short enough that
scheduling noise and frequency ramp dominate. That 0.6% to 3.7% is the band the
**Raw** note above refers to: it spans all three sweeps, only one of which is
committed, so it cannot be re-derived from this repository.

**Both of that section's "Next" items are now done**, and together they are what
made the knee readable on the second host:

1. A **minimum sample duration** of 50 ms. Each size is now repeated until it
   has run that long, and the reported time is the total divided by the repeat
   count.
2. The sweep now includes **2 and 4 MiB**. The original started at 16 MiB,
   which on a 12 MiB L3 is *already outside cache*, so the curve came back
   flat and the knee unreadable, not because the machine was odd but because
   the sweep never sampled cache at all.

---

## Phase 4a again: the floor on the desktop

**Host:** 11th Gen Intel Core i5-11400 @ 2.60 GHz, 6C/12T, L3 12 MiB, 15.8 GB
RAM. Not shared, but running a desktop.
**Raw:** `measurements/floor-2026-08-28-i5-11400.json` / `.txt`

| size | 1-thread | spread | 12-thread | spread |
|---|---|---|---|---|
| 2 MiB | 65.10 GB/s | 0.6% | 5.03 GB/s | 14.4% |
| 4 MiB | 74.00 GB/s | 14.9% | 9.29 GB/s | 14.2% |
| 16 MiB | 33.05 GB/s | 3.9% | 33.74 GB/s | 16.0% |
| 64 MiB | 23.30 GB/s | 0.7% | 29.06 GB/s | 9.3% |
| 256 MiB | 22.68 GB/s | 2.1% | 27.53 GB/s | 6.6% |
| 512 MiB | 22.82 GB/s | 3.5% | 30.52 GB/s | 3.3% |
| 1024 MiB | 22.92 GB/s | 2.3% | 31.23 GB/s | 3.5% |
| 2048 MiB | 20.23 GB/s | 11.6% | 30.94 GB/s | 2.7% |

> **knee: 69.55 GB/s in cache against 22.39 GB/s out, ratio 3.11x.** The
> acceptance test passes for the first time. The floor is the out-of-cache
> figure: **~22 GB/s single-thread, ~30 GB/s across 12 threads.**

16 MiB straddles the 12 MiB L3 and is excluded from both sides of the knee by
the existing margin rule. The spreads above 10% are a desktop with a browser on
it; they do not touch the knee, whose two sides differ by 3.1x.

---

## Phase 4b: SimplePIR, CPU

**Raw:** `measurements/cpu-simplepir-2026-08-28.json` / `.txt`
**Scheme:** SimplePIR (Regev/LWE), n = 1024, p = 256, q = 2^32, error centred
binomial k = 82 (sigma approximately 6.40). **B = 1**, always.
**Corpus:** synthetic. Shape faithful (128 B fixed-width records, m/128 per
column, 6,712,448 records at m = 29312); contents are not case law.

| m | DB | floor 1T | answer 1T | **x** | floor 12T | answer 12T | **x** |
|---|---|---|---|---|---|---|---|
| 2048 | 4.2 MB | 74.90 GB/s | 11.95 GB/s | 6.23 | 8.56 GB/s | 9.58 GB/s | 0.89 |
| 4096 | 16.8 MB | 32.90 GB/s | 10.54 GB/s | 3.12 | 26.46 GB/s | 27.64 GB/s | 0.96 |
| 8192 | 67.1 MB | 23.40 GB/s | 10.57 GB/s | 2.21 | 27.42 GB/s | 27.18 GB/s | 1.01 |
| 16384 | 268.4 MB | 22.94 GB/s | 10.78 GB/s | 2.13 | 29.77 GB/s | 29.36 GB/s | 1.01 |
| 23168 | 536.8 MB | 22.69 GB/s | 10.78 GB/s | 2.10 | 30.51 GB/s | 29.86 GB/s | 1.02 |
| **29312** | **859.2 MB** | 22.87 GB/s | **10.70 GB/s** | **2.14** | 30.54 GB/s | **30.73 GB/s** | **0.99** |
| 32768 | 1073.7 MB | 22.72 GB/s | 10.68 GB/s | 2.13 | 30.70 GB/s | 30.92 GB/s | 0.99 |
| 46336 | 2147.0 MB | 22.10 GB/s | 10.45 GB/s | 2.11 | 30.94 GB/s | 30.86 GB/s | 1.00 |

**At the real index size: 80.3 ms/query on one core, 28.0 ms across twelve.**
Spread at that size was 7.6% on the floor and 5.6% on the answer.

Every size decoded: the record recovered through the query, answer and
hint-subtraction path equalled the record stored in the database, at all eight
sizes.

**Single-threaded, the answer is compute-bound and the multiplier is ~2.1x.**
The answer holds at 10.45 to 10.78 GB/s from 64 MB upward while the floor sits
at ~22 GB/s, so the ratio is flat across sizes: a property of the loop, not of
the database size. **Across twelve threads the answer reaches the memory wall**
and the multiplier settles at ~1.0, 30.73 GB/s against a 30.54 GB/s floor at the
real index size.

That row reads 0.99, and **0.99 is not the answer beating the floor.** Both
figures describe the same memory bus at the same width; the ratio is 1.0 with a
percent of noise on it, and the honest reading is *indistinguishable from the
floor*, not *1% faster than it*. The 6.23x at m = 2048 is the opposite case: a
4 MB database sitting in L3, where the floor is 74.9 GB/s and nothing is
memory-bound. Neither row is comparable to the middle of the sweep, and the
sweep exists so that both are visible rather than one being reported alone.

### The build setting that changes the headline by two thirds

Rust's default target is baseline x86-64, which is SSE2 and has no 32-bit vector
multiply, so the byte-times-word multiply-accumulate at the centre of SimplePIR
compiles to something much narrower than the machine can do. Measured at
m = 16384, both from the current source, one build apart:

| build | floor 1T | answer 1T | multiplier 1T |
|---|---|---|---|
| default target (`RUSTFLAGS=""`) | 21.69 GB/s | 6.11 GB/s | **3.55x** |
| `-C target-cpu=native` | 22.94 GB/s | 10.78 GB/s | **2.13x** |

Raw: `measurements/baseline-target-cpu-2026-08-28.txt` for the first row,
produced into a separate target directory so the two builds could never be
mixed; `measurements/cpu-simplepir-2026-08-28.json` for the second.

**The two rows are from two runs, and the floor column is there so you can see
what that cost.** The floor moved 5.8% between them: this is a desktop, not a
quiet machine. The answer moved **76%**, an order of magnitude more than the
drift, so the comparison survives being assembled from two runs. Quoting the
floors rather than asserting "the floor did not move" is the difference between
a claim and a checkable one.

Since the floor is essentially unchanged and the answer is not, this changed
only the numerator. Without it the reported multiplier would have been **two
thirds too large** and would have been read as a fact about SimplePIR. It is
set in `.cargo/config.toml`, with these numbers in the comment, because deleting
it for portability would silently inflate the metric again.

An earlier form of the inner loop, the obvious `iter().zip().fold()`, measured
4.50 GB/s on the default target. Restructuring it into fixed-size lanes helped
both targets; the table above is from the current code so that both rows can be
reproduced.

---

## Phase 4b: SimplePIR, GPU

**Raw:** `measurements/gpu-simplepir-2026-08-28.json` / `.txt`
**Device:** NVIDIA GeForce GTX 1070 · compute capability 6.1 · 15 SMs @ 1.78 GHz
· 8 GiB, 256-bit bus, **256 GB/s theoretical peak** · L2 2 MiB · display
watchdog active.
**Toolchain:** PTX for `compute_61` from NVRTC 12.9, JIT to `sm_61` by the
driver. The installed CUDA 13.3 cannot target Pascal at all:
`docs/adr/0002-pascal-without-a-pascal-toolkit.md`.

Same scheme, same parameters, same database shape, same client code. Only the
processor changes.

| m | DB | device floor | GPU answer | **x** | vs 12T CPU | upload (once) |
|---|---|---|---|---|---|---|
| 2048 | 4.2 MB | 70.4 GB/s | 72.8 GB/s | 0.97 | 21.7x | 2 ms |
| 4096 | 16.8 MB | 137.2 GB/s | 150.7 GB/s | 0.91 | 18.8x | 5 ms |
| 8192 | 67.1 MB | 197.6 GB/s | 190.1 GB/s | 1.04 | 8.7x | 20 ms |
| 16384 | 268.4 MB | 207.9 GB/s | 205.5 GB/s | 1.01 | 7.3x | 59 ms |
| 23168 | 536.8 MB | 211.6 GB/s | 217.6 GB/s | 0.97 | 8.3x | 120 ms |
| **29312** | **859.2 MB** | 214.9 GB/s | **213.8 GB/s** | **1.01** | **7.0x** | 199 ms |
| 32768 | 1073.7 MB | 217.6 GB/s | 219.6 GB/s | 0.99 | 7.2x | 239 ms |
| 46336 | 2147.0 MB | 219.8 GB/s | 217.7 GB/s | 1.01 | 7.1x | 469 ms |

**At the real index size: 4.02 ms/query at B = 1, a multiplier of 1.01x**, with
a run-to-run spread of 0.1%.

### What a multiplier of 1.01 means

> **Correction.** This heading read *"What a multiplier of 1.03 means"* in the
> draft this file was ported from. The table above it reads 1.01, the sentence
> above it reads 1.01x, and the raw artifact records `"multiplier": 1.0055` at
> m = 29312. The superseded text is the heading that read 1.03. A heading is the
> part of a section a reader remembers and the part no check touches:
> `docs/errors-caught.md` entry 8.

**SimplePIR's online cost on this GPU is the cost of reading the database, and
nothing more.** 213.8 GB/s is 83% of the card's 256.3 GB/s theoretical peak, and
it is within 1% of the same card summing the same bytes with no cryptography at
all. There is no headroom left to optimise into: the answer is memory-bound, and
the memory wall is the floor no PIR scheme can pass.

The 0.97 and 0.91 at 4 and 17 MB are not the answer beating the floor. Both
kernels finish in well under 150 microseconds there (58 and 111 microseconds),
where launch overhead and the scan kernel's grid-stride shape matter more than
bandwidth; the 20% and 21% spreads in those rows say so, against 0.1% at the
real index size. Ignore those two.

**This is the result the B = 1 rule in
`docs/adr/0001-gpu-before-fpga-for-pir.md` was written for.** The DPF-PIR paper
needed batch 512 to saturate an RTX 4090. SimplePIR streams the whole database
for a *single* query, so one query already saturates a 2016 gaming card, and
B = 1 is exactly what the declared reference workload describes: at lambda =
0.06 queries/second, no batch forms.

### What each check does and does not cover

A timing is printed only if all three pass, and they passed at all eight sizes.

1. **GPU answer bit-identical to the CPU answer.** Not close, equal. Every
   operation is `u32` wrapping arithmetic over a commutative group, so the two
   reduction orders must agree exactly, and a mismatch would be a bug rather
   than a rounding. **This is the only check on the answer kernel**, and it is
   what the claim that it reads every byte rests on.
2. **Device scan sum equal to the host scan sum.** This validates the *scan*
   kernel (the denominator) and says nothing about the answer kernel.
3. **Record decoded end to end** from the GPU's answer. **One record per size**,
   at the midpoint of the database. Four records at `[0, 1, 17, count-1]` are
   checked by a test inside the implementation under measurement, but only at
   m = 256. "decode OK" in a sweep is not exhaustive over the layout.

### Two device details worth recording

**The JIT is cached by the driver, and the cold figure is now measured rather
than remembered.** `measurements/gpu-jit-cold-2026-08-28.txt`, same binary and
same PTX, seconds apart:

| | JIT to `sm_61` |
|---|---|
| cold (`CUDA_CACHE_DISABLE=1`) | **33.2 ms** |
| warm (default) | **0.6 ms** |

A factor of about 55. Both are start-up costs and neither is inside a query, but
a reader comparing two runs needs to know why the number moves.

An earlier draft of this section claimed a cold JIT of **102 ms**, from an
observation that was never committed. The measured figure is 33.2 ms, so that
claim was not merely unsupported, it was wrong. The superseded sentence is the
one that said "102 ms".

**The card drives a display, so medians are reported with the spread printed
beside them.**

> **Correction, and it is the same error as the cold-JIT one directly above.**
> This paragraph read: *"One row had a 379% spread (m = 23168) while its median
> stayed in line with its neighbours. The card is driving a display; that is a
> stall, not a measurement."* The committed artifacts contradict it. That row's
> spread is **0.1%** (`gpu_answer_spread: 0.0008` in
> `measurements/gpu-simplepir-2026-08-28.json`, `spread 0.1%` in the `.txt`),
> and the string `379` appears nowhere under `measurements/`. The superseded
> sentence is the one that said 379%. `docs/errors-caught.md` entry 9 records it
> beside the cold-JIT entry it repeats.

What survives that correction is the practice rather than the row. The largest
run-to-run spread anywhere in the GPU answer sweep is 21.1%, at m = 4096:

| m | 2048 | 4096 | 8192 | 16384 | 23168 | 29312 | 32768 | 46336 |
|---|---|---|---|---|---|---|---|---|
| spread | 20.1% | **21.1%** | 14.9% | 2.6% | 0.1% | 0.1% | 0.1% | 0.0% |

Those two leading spreads are exactly what makes the 4 and 17 MB rows
excludable above, and they are visible only because the spread is printed beside
the median rather than smoothed away. On a display-driving card that is not
fussiness: a stall is not a measurement, and the only way to tell one from the
other in a committed artifact is to have recorded the dispersion with the point.

## Phase 4c: hint generation, both arms

**Raw:** `measurements/cpu-hint-2026-08-28.json` and
`measurements/gpu-hint-2026-08-28.json`
**Scheme:** the same SimplePIR. `H = D·A`, O(m^2 · n): **8.8 x 10^11**
multiply-accumulates at the real index size, four orders of magnitude more work
than one query.

> **Correction.** This file said hint generation "was not benchmarked" and
> called it "the largest unmeasured quantity in this repository". Both sentences
> are superseded by this section.

### At the real index size (m = 29312, 859.2 MB)

| | one full rebuild | rate |
|---|---|---|
| 1 CPU core | **89.6 s** = 0.0249 core-hours | 9.59 MB/s/core |
| 12 CPU threads | **25.0 s** | 35.3 GMAC/s |
| GTX 1070 | **1.17 s** in 10 launches | 751 GMAC/s |

**The CPU figure has an outside number to be judged against and beats it.**
Under 4 MB/s per core is the published CPU throughput for SimplePIR
preprocessing (YPIR, ePrint 2024/270); this measures **9.59, or 2.4x**. The rate
is flat at 8.90 to 9.91 MB/s/core across all eight swept sizes, a 512-fold range
of databases for an 11% spread, which is what makes it a property of the core
rather than of the working set, and therefore portable to another machine.

**The GPU figure has nothing to be judged against.** No published work reports a
throughput for GPU-accelerated SimplePIR *offline* preprocessing, so it is
reported against two hardware ceilings instead: 1139 GMAC/s for `sm_61`'s
emulated `IMAD`, and 3418 for `DP4A` at full rate. Utilisation is **57% to 71%
of the IMAD ceiling** across the sweep, which is what says the number describes
the card rather than the kernel.

`DP4A` is named as follow-up with its prediction written down rather than left
to be rediscovered: `A = Σ_b 2^{8b} A_b` decomposes exactly over the integers
and therefore mod 2^32, so a byte-plane implementation is sound and has roughly
**3x of headroom**.

### The finding

**Hint generation is not a cost. Hint distribution is the entire cost.**

A rebuild takes 0.0249 core-hours. Against a fleet of **1,000 clients** each
pulling the 120.1 MB hint (a stated scale parameter, like the reference
workload), one rebuild is 120 GB of egress.

The committed figures are core-hours and wall-seconds, and the conclusion does
not depend on any assumed price: compute is **1.17 seconds of a consumer GPU**,
and no plausible rate makes that comparable to broadcasting 120 GB. Any dollar
figure either way would be arithmetic on assumptions rather than a measurement,
and none originates in this work.

This retires the open item and turns the remaining question into a bandwidth one
rather than a throughput one. What answers it is **not** DoublePIR: see
*The hint, and the two dead ends* below.

### The kernel that would have been published instead

The first form of the CPU hint measured **2.5 GMAC/s, 2.47 MB/s per core, 0.6x
the published figure.** Slower than the number it was meant to beat.

The cause was arithmetic intensity, not the arithmetic. Written as the obvious
full-row `axpy`, every multiply-accumulate needs a load of `A`, a load of `H`
and a store of `H`: three memory operations per MAC, and the multiplier never
binds. Strip-mining `n`, holding 64 words of `H` in registers across the whole
contraction, removes `H` from the inner loop, leaving one `A` load per eight
MACs.

| form | 1 thread | vs published |
|---|---|---|
| full-row axpy | 2.5 GMAC/s, 2.47 MB/s/core | **0.6x** |
| strip-mined | 10.0 GMAC/s, 9.80 MB/s/core | **2.5x** |

A factor of four, and the first version would have been published as a fact
about SimplePIR's preprocessing cost. **It was caught by having a number from
outside this work to check against**, the same mechanism and the same class of
error as the `target-cpu=native` finding above.

### What the watchdog forced, and what it cost

The card drives a display. A full-size hint kernel would run for about a second,
and on WDDM a launch past roughly two seconds kills the display driver, which
also destroys the CUDA context, so every call after it fails and a sweep that
carries on emits rows that look like measurements.

So the hint routine **probes** with 512 rows, sizes the launches from what the
probe actually cost rather than from an assumed rate, and checks the result
against a 500 ms budget on **wall** time rather than CUDA event time: WDDM
counts time queued behind display work, which events never see. At the real
index size that is 10 launches of 2944 rows, wall max **127 ms**.

A watchdog kill aborts the entire run rather than moving to the next size.

**Sustained-load check.** A ~1 s integer kernel on a display-driving card can
drop off boost, which the 4 ms answer kernel never ran long enough to show.
Per-launch times are kept as a series and normalised to ns/row: 41522 to 36229,
a ratio of 0.87, so the clock held.

> The first version of that check compared the first launch to the last without
> normalising. The last launch is short (it covers whatever rows remain) so it
> reported that as a 0.67x speedup, which is not something a throttling card
> does. Fixed before the figure was committed; recorded because an unnormalised
> version of this check reads as reassuring and is meaningless.

### Verification

Every printed timing is gated on all of these, and they passed at all eight
sizes on both arms:

1. **Serial and parallel CPU hints agree bit for bit.** Every word of `H` is
   written by one thread and `A` is read-only, so this is true by construction:
   asserted rather than hoped.
2. **The GPU hint is bit-identical to the CPU's**, all 30,015,488 words at the
   real index size.
3. **`H·s == D·(A·s)` at full size.** This is the one that carries the most
   weight and is easiest to under-rate. Both arms expand `A` from the same seed
   through the same routine, so (2) *structurally* cannot catch an
   `A`-generation bug. The right-hand side walks that same seed through a path
   that never materialises `A`, and is the only thing tying the two together at
   m = 29312.
4. **The record decodes through the materialised hint.** New. Every decode in
   this work until now went through a shortcut path that is exact but needs `D`,
   and so is available to no client. This is the real client path, and running
   it closes the loop this file had been leaving open.

---

## Phase 5: the decision gate

Phase 5 of the internal plan asks one question: **does a CPU clear single-query
latency at this index size?**

**Yes, with a very large margin, and the GPU is not needed.**

| | latency, B = 1 | queries/s | headroom over lambda = 0.06 q/s |
|---|---|---|---|
| 1 CPU core | 80.3 ms | 12.5 | ~208x |
| 12 CPU threads | 28.0 ms | 35.8 | ~596x |
| GTX 1070 | 4.02 ms | 249 | ~4,150x |

`docs/adr/0001-gpu-before-fpga-for-pir.md` point 3 said this measurement might
end the hardware question outright. It does. **No FPGA, and no GPU either**: a
single desktop core answers a private existence-check query over the whole
859 MB index in under a tenth of a second, against a declared arrival of one
query every roughly 16.7 seconds. The headroom column scales with lambda and
nothing else; the latencies are measurements and do not.

The GPU arm keeps its value as a bound rather than as a purchase. It shows the
multiplier converging to 1.01 once the hardware is fast enough to make the
scheme purely memory-bound, which is the strongest evidence available that the
2.1x measured on one CPU core is a property of that core and not of SimplePIR.

---

## The hint, and the two dead ends

Phase 4c concludes hint *distribution* is the entire cost. Two obvious ways to
attack it were checked against primary sources on 2026-08-28, and **both are
dead ends at this record size**. Recorded so they are not re-derived.

### Dead end 1: DoublePIR, because our records are 128 bytes

DoublePIR's headline is a **16 MB hint, independent of database size**, and that
is real. It is also measured on **one-byte records**. SimplePIR's own Table 16
gives amortized per-query communication against record width, at a 1 GB
database over 100 queries:

| record width | DoublePIR | SimplePIR |
|---|---|---|
| 1 bit | 0.5 MB | 1.4 MB |
| 8 B | 3.1 MB | 1.4 MB |
| **128 B, ours** | **43 MB** | **1.4 MB** |
| 4 KB | 690 MB | 1.4 MB |

**SimplePIR is flat in record width and DoublePIR is linear in it.** Their section 5.2
large-record construction says why: for records of `d` elements in `Z_p`, the
hint is `d·kappa·n^2` and the online download `d·kappa·(2n+1)`. At our
`d = 128`, `kappa = 4`, `n = 1024` that is a **2.1 GB hint**, 18x larger than
the 120 MB we already pay, and a **4.2 MB** per-query download against our
117 KB.

The reason is structural and worth stating, because it is also why our layout is
a good one: a SimplePIR answer returns an entire *column*, so laying a 128-byte
record down one column gets all 128 bytes for the price of one query.
DoublePIR's second level exists precisely to avoid returning a whole column, so
it must pay per byte. **The property that makes DoublePIR attractive on one-byte
records is the property that makes it useless on ours.**

### Dead end 2: deeper recursion, which is what "multi-dimensional SimplePIR" means

Going to `r > 2` levels looks like it should shrink the hint further. It does the
opposite. SimplePIR's Remark 5.1, verbatim:

> "After *r* levels of recursion, the cost of the recursive PIR scheme, on
> lattice dimension *n* and database size *N*, would be (hiding constants):
> one-time download **n^r** in the preprocessing step, as well as per-query
> upload r·N^(1/r) and download **n^(r−1)**. For *r > 2*, the communication is
> likely too large for databases of interest."

The hint becomes independent of `N` but grows as `n^r` in the lattice dimension:

| r | hint |
|---|---|
| 2 (DoublePIR) | 4.2 MB, or 16.8 MB with kappa = 4, which matches their published 16 MB |
| **3** | **4.3 GB** |
| 4 | 4.4 TB |

An earlier reading of this work's own numbers guessed that a three-dimensional
variant would give `m = N^(1/3)` and therefore a 3.9 MB hint. **That is wrong**,
and wrong by a factor of about 1,100 in the unsafe direction: it assumed the
recursion shrinks `m` while leaving the hint formula alone, when in fact it
trades the `sqrt(N)` term for an `n^r` one. The authors leave "recursive
LWE-based PIR with total communication n·N^(1/r)" as an explicit open question,
so the gap is real, but it is open, not solved.

Both rows of this section are arithmetic on published formulas rather than
measurements, and are labelled as such wherever they appear.

### What is left, and the one thing to check first

The schemes that actually attack this eliminate the hint rather than shrinking
it, and all three are single-server, so the single-operator threat model
(`docs/threat-model.md`) is intact:

| | offline download | at 1 GB: up / down | throughput |
|---|---|---|---|
| **YPIR** (USENIX Sec '24) | **none** | 846 KB / 12 KB | 7.8 GB/s; 12.1 GB/s at 32 GB |
| **HintlessPIR** (CRYPTO '24) | **none** | 453 KB / 3,080 KB | 1.75 GB/s |
| **VIA** (IEEE S&P '26) | none; VIA-C has a 14.8 MB part, flat 1 to 32 GB | sub-KB queries | ~7x below SimplePIR |

Trading roughly 7x of server throughput to delete 120 GB of egress per rebuild
is the direction the measurements point: Phase 5 records 208x of headroom over
lambda on one core and 4,150x on the GPU.

**Check record width before believing any of the figures above.** Every one is
measured at 1-bit or 1-byte records, and 128-byte records are exactly where
DoublePIR collapsed. YPIR has a large-record variant (YPIR+SP, 32 KB records,
2.4 GB/s at 1 GB); VIA discusses extension but does not benchmark VIA-C at
128 B. The honest way to settle it is to run the reference implementations at
`m = 29312` with 128-byte records, not to extrapolate a table, which is the
same discipline that caught the `target-cpu=native` and strip-mining errors
above.

### Discharged, 2026-08-29: two of the three were run

`measurements/external-2026-08-29-ypir-via.md`, with
`measurements/external-2026-08-29-ypir.{txt,json}` and
`measurements/external-2026-08-29-via.txt`.

The instruction above was right and it cut both ways: the table's framing was
wrong about **both** schemes, in opposite directions.

- **YPIR is out.** Run at our exact 859.2 MB: 0 bytes of client hint, 155 ms
  server, 866.3 KB up / 12.3 KB down. But its items are one byte and
  `src/params.rs` rejects anything wider, so a 128-byte record costs 128
  queries: **113x worse than the hint** at Q = 157, and behind after 1.1
  queries. Its internal server-side SimplePIR hint of 117.4 MB agrees with our
  independently measured 120.1 MB to within 2.2%.
- **VIA is in.** ~~"VIA ... sub-KB queries"~~ The superseded reading is the one
  that put VIA beside YPIR as a one-byte-record scheme. It does not belong
  there. A VIA record is `DEGREE2 = 512` coefficients, **512 bytes**, so one
  query covers one of our 128-byte records. At Q = 157, both sides at our index
  size, VIA sends **93.2 MB against our 156.9 MB**, and stays ahead until 335
  queries per rebuild period. It is the first thing found that beats the
  120.1 MB hint on total client bytes. Note the asymmetry: VIA's byte counts are
  *computed* by `printInfoVIA` in closed form from its gadget parameters, not
  counted off a wire. The timings on both sides are wall-clock.

What VIA costs for that: **11.7x our server time per byte** (measured, both
arms), which the 208x headroom absorbs, and **8x our server RAM**, which is the
real obstacle. Its shipped configuration stores a 4 GiB database as 32 GiB of
`uint64` NTT coefficients and is OOM-killed on this 16 GB machine; the run above
is at a reduced `LOG_COL`. Two caveats belong with the timing: VIA's First
Dimension loop reads `database[0]` on every iteration rather than streaming the
database, so 84% of its answer time is a floor and not a measurement; and its
harness never checks that the right record came back.

The open item is therefore not "is there a hint-free scheme that survives
128-byte records", there is, but **"does VIA's throughput survive actually
reading the database, and is 8x RAM acceptable."** HintlessPIR remained unrun,
and the reason it ranked last was weaker than it looked: its 3,080 KB per-query
download is measured at one-byte records, the same unchecked assumption that
filed VIA next to YPIR.

### Discharged, 2026-09-04: the third scheme was run

`measurements/external-2026-09-04-hintlesspir.md`, with
`measurements/external-2026-09-04-hintlesspir.txt`.

Run at our real shape (`db_rows = 229`, `db_cols = 29312`, the bandwidth-optimal
split for 128-byte records at our record count, not a round number chosen for
convenience) HintlessPIR's public params are **264 bytes**, so the hint-free
claim holds exactly as it does for YPIR and VIA. But the per-query cost the
paper's one-byte table hid is worse than either of the other two turned out to
be, in two independent ways:

- **Bandwidth.** 501.5 KiB up / 22.65 MiB down per query. At Q = 157 that is
  **3,809.6 MB against our 156.9 MB, 24x worse**, and it falls behind our hint
  after 5.0 queries. It lands between VIA (still the only scheme that beats us,
  at 93.2 MB) and YPIR (17,656 MB, still disqualified).
- **Server time.** 113.65 s per query, single core, against our 80.3 ms:
  **~1,415x slower**, not the roughly 7x to 11x VIA's slower-but-affordable
  answer costs. Phase 5's 208x of headroom absorbs VIA's slowdown; it does not
  absorb this one. At lambda = 0.06 queries/second (one arrival every roughly
  16.7 s), 113.65 s per query is 6.8x the entire inter-arrival budget on a
  single core: a backlog that never drains, not a latency figure with margin to
  spare. The reference binary spawns no threads of its own, so this describes
  what was run, not a ceiling on what further engineering could do.

**All three hint-free schemes are now measured at the record width that
matters, and none replaces what this work already runs.** VIA remains the only
one ahead of the 120.1 MB hint on bytes, and its own open item (RAM and the
un-streamed First Dimension loop) is unchanged by this result.

## What these numbers do not include

Stated plainly, because each is a real cost that a latency figure hides.

**The hint is 120.1 MB, computed once per database version and downloaded by
every client.** SimplePIR's hint is `m x n` words. It is *client-independent*:
`H = D·A` depends only on the database and the public `A`, so the server
computes it once per corpus version and broadcasts identical bytes to everyone.
That distinction matters twice: generation is a one-off amortised across the
whole fleet rather than a per-client cost, and the artifact is a single
cacheable object rather than N distinct ones.

Across a fleet of 1,000 clients that is ~120 GB of download per rebuild, against
~18 GB of query download for the same fleet over the same period at Q = 157.
**The hint, not the queries, is the bandwidth cost of this design**, and it is a
distribution cost rather than a compute one.

**This size is externally corroborated, which is worth more than an internal
check.** SimplePIR's authors report a **121 MB** hint for a 1 GB database and
242 KB of communication per query. This implementation, independently written
from the construction, produces **120.1 MB** for an 859 MB database and 234 KB
per query round trip. Landing on the published figures from a different
direction is evidence that the parameter set here is the one the scheme
intends, which no amount of self-consistency could give.

**Figures quoted for full-text retrieval are about a different database, and
they are easy to mistake for this one.** Retrieval of opinion text at a cited
page is a hundreds-of-gigabytes problem with its own economics, and it is
explicitly out of scope here. Per-client-per-rebuild figures derived from that
problem, in the tens of gigabytes, do not describe this index: the existence
check is one 859 MB matrix with one 120 MB hint, so the figure for *this*
database is 120 MB. Both kinds of number may be true at once; they are not true
of the same database.

~~**Hint *generation* was not benchmarked.**~~ **Measured 2026-08-28, see
Phase 4c.** 89.6 s on one core, 25.0 s across twelve, 1.17 s on the GTX 1070.
The superseded sentence is the one that called it "the largest unmeasured
quantity in this repository". It is now the smallest cost in it: 0.0249
core-hours per rebuild, against 120 GB of egress to distribute the result.

**Per-query traffic is 117.2 KB up and 117.2 KB down, to retrieve 128 bytes**,
an expansion of roughly 1,800x. Small in absolute terms at this arrival rate,
and worth stating as what privacy costs on the wire.

**The corpus the timings ran against is synthetic, and shape-faithful.** 128 B
fixed-width records, m/128 records per column, 6,712,448 records at m = 29312.
The timings depend only on that shape, so the synthesis does not weaken them.
The records recovered by the end-to-end decode checks are a test of the pipeline
and are **not facts about case law**. Building the index from real canonical
keys is not part of this work. The record-size study of Phase 1 is the separate
thing: real CAP metadata, and the reason `m` is measured rather than assumed.

**Nothing here is a security artefact.** The error sampler is a centred
binomial, the secret comes from a general-purpose PRNG, no parameter set has
been checked against a lattice estimator, and neither the CPU loop nor the GPU
kernel is constant-time. The decode margin is asserted (41 sigma to 196 sigma
across the sweep) because a shrinking margin is where a wrong answer would come
from; that is a correctness property, not a security one.

## Next

1. ~~**Measure hint generation**, on both arms.~~ **Done, Phase 4c.** It was
   the only cost thought large enough to change a deployment decision, and it
   turned out not to be one. What replaces it: **implement `DP4A`** for the hint
   kernel, which the ceiling arithmetic says is worth about 3x, and which is
   worth doing only if rebuild wall-clock ever becomes binding.
2. ~~Consider **DoublePIR** if the 120 MB hint turns out to matter. It shrinks
   the hint to ~16 MB.~~ **Superseded 2026-08-28: DoublePIR is worse than what
   we already run, at our record size.** The superseded claim is the one that
   said it "shrinks the hint to roughly 16 MB"; that figure is for **one-byte**
   records and does not survive contact with 128-byte ones. See *The hint, and
   the two dead ends* above.
3. ~~**Evaluate the hint-free schemes at 128-byte records**: YPIR,
   HintlessPIR, VIA.~~ **Done, all three now measured.** YPIR (2026-08-29): out
   on one-byte items, 113x worse than our hint. VIA (2026-08-29): **93.2 MB
   against our 156.9 MB at Q = 157**, the only one ahead of us on bytes.
   HintlessPIR (2026-09-04,
   `measurements/external-2026-09-04-hintlesspir.md`): 264-byte public params,
   but 24x worse than our hint on bytes and ~1,415x slower per query than
   SimplePIR, out on both counts, and more decisively on latency than on
   bandwidth.
4. **Measure VIA against a database it actually reads, and decide whether 8x
   RAM is acceptable.** This is what item 3 turned into, and is now the only
   open item left from it: HintlessPIR's run closed the other branch. VIA's
   shipped First Dimension loop reads `database[0]` on every iteration, so its
   11.7x throughput gap to us is a floor; and it stores a 4 GiB database as
   32 GiB of `uint64` NTT coefficients, which is why it cannot run here
   unmodified. Both are answerable on this machine.
