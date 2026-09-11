# Measurement methodology

This document is the measurement discipline behind every figure in this
repository, in one place, so that a reader can judge the rest of the repository
by it rather than by taking each number on trust.

It is worth stating the awkward part first. The SimplePIR implementation that
produced the CPU and GPU timings is not published here (`SCOPE.md` section 2),
so those figures are reported rather than reproduced. What is published is the
denominator of the headline metric (`crates/pir-bench`), the corpus sampler
(`tools/fetch_cap.py`), the PTX build tool (`tools/build_ptx.py`), the build
configuration that turned out to be a finding (`.cargo/config.toml`), and every
raw artifact in `measurements/`. So the rules below are partly an account of how
the closed numbers were produced and partly a description of code anyone can
run. Which is which is said at each point.

Throughout, comparisons are made against the **declared reference workload**
defined in `SCOPE.md` section 3: an arrival rate of **lambda = 0.06
queries/second** and **Q = 157 queries per client per rebuild period**. Those are
stated evaluation parameters, in the same sense that the database size is a
stated input, not measurements of anyone's traffic. Every latency, bandwidth and
throughput figure here is a measurement and is independent of both.

---

## 1. The floor, and the multiplier

PIR must touch every byte of the database on every query. That **is** the privacy
guarantee, not an implementation detail: touching only the relevant bytes is
exactly what leaks which record was wanted. A scheme that reads less than the
whole database has told the operator something about the query, whatever else it
does.

It follows that a plain linear scan of the same bytes is a lower bound no scheme
can beat, on any hardware, ever. That bound is measurable in an afternoon and
costs nothing, so it is measured first and everything else is expressed against
it:

> **PIR multiplier = (seconds/query under PIR) / (seconds to linearly scan the
> same database at the same size on the same host)**

**Why a ratio and not a queries/second.** Absolute throughput is a fact about the
box. Two hosts appear in this repository and they are not comparable: the same
scan loop measures roughly 10 GB/s single-threaded out of cache on a shared Xeon
vCPU at 2.80 GHz, and roughly 22 GB/s on an i5-11400 desktop. Any absolute
figure quoted from one says nothing about the other, and neither says anything
about whatever hardware a reader would rent. The floor moves with the hardware;
the multiplier is a property of the scheme, and it is the number that survives
being carried off the machine that produced it.

**The numerator and the denominator must come from the same loop on the same
run.** This is why the scan is a library (`crates/pir-bench/src/lib.rs`) rather
than a snippet copied into each benchmark. Two scan loops that are merely
*meant* to stay identical will not, and their divergence would surface as a
change in the metric rather than as a compile error. The implementation under
measurement links that library and divides by it.

**The evidence that the ratio is a property of the scheme rather than of the
size.** On one core the answer holds at 10.45 to 10.78 GB/s from 64 MB upward
while the floor sits at about 22 GB/s, so the multiplier is flat at roughly 2.1x
across a 32-fold range of database sizes (2.14x at the real index size,
m = 29312). Across twelve threads the answer reaches the memory wall and the
multiplier settles at about 1.0. On the GTX 1070 it is 1.01x, with the answer at
213.8 GB/s against a device floor of 214.9 GB/s, which is 83% of the card's
256.3 GB/s theoretical peak. Three different processors, the same scheme, and
the multiplier converging to 1 exactly as the hardware gets fast enough to make
the scheme purely memory-bound: that is the strongest available evidence that
the 2.1x on one core is a property of that core and not of SimplePIR.

**How to read a multiplier at or below 1.** It is not the cryptography beating
the memory bus.

- The 12-thread row at the real index size reads 0.99 (30.73 GB/s answer against
  a 30.54 GB/s floor). Both figures describe the same memory bus at the same
  width. The honest reading is *indistinguishable from the floor*, not *1%
  faster than it*.
- The 6.23x at m = 2048 is the opposite case: a 4.2 MB database sitting in L3,
  where the floor is 74.9 GB/s and nothing is memory-bound. It is not comparable
  to the middle of the sweep.
- The GPU rows at 4.2 MB and 16.8 MB read 0.97 and 0.91. Both kernels finish in
  well under 150 microseconds there (58 and 111 microseconds), where launch
  overhead and the scan kernel's grid-stride shape matter more than bandwidth.
  The 20% and 21% run-to-run spreads in those rows say so, against 0.1% at the
  real index size.

The sweep exists so that all of these are visible at once rather than one of
them being reported alone.

**The direction of the error matters, and getting it backwards is easy.** The
multiplier is `answer_seconds / scan_seconds` and `scan_seconds` is
`bytes / floor`, so the multiplier scales **with** the floor. Overstate the floor
and the multiplier comes out too large, which makes the scheme look more
expensive than it is. Understate the floor and the multiplier comes out too
small, which flatters it.

The sweep's own data shows this directly. At m = 2048 the 4.2 MB database sits
inside the host's 12 MiB L3, the floor is measured at an inflated 74.90 GB/s,
and the multiplier is **6.23**, the largest in the sweep, against roughly 2.1
once the database leaves cache. An inflated floor inflates the multiplier.

So when `crates/pir-bench` cannot certify its own knee it now says which way the
bias runs *for each of the two causes it cannot tell apart*: a working set that
never left cache overstates the floor, making the multiplier an upper bound; a
scan loop that caps below the machine's memory bandwidth understates it, making
the multiplier a lower bound and flattering the scheme. The ratio alone does not
distinguish them, which is why both verdicts read "either ... or".

> **Correction, and it is in the published code rather than only in prose.**
> `crates/pir-bench/src/main.rs` stated this backwards: *"a floor that is too
> HIGH makes the PIR multiplier too SMALL, which flatters the scheme. Treat any
> multiplier derived from this run as a lower bound on the true one."* That is
> inverted, and it also gave a single direction for two causes that bias
> opposite ways. The superseded sentence is the one just quoted. No committed
> figure is affected: the branch prints only when the knee ratio is below 1.15,
> and every committed sweep passed it (3.11x on the i5-11400). It was a wrong
> diagnostic, not a wrong measurement.

---

## 2. Sweep across the cache cliff

A database that fits in L3 is not the database. The real existence index is
859.2 MB and is DRAM-bound; a benchmark at 100 MB on a host with a large
last-level cache is not a conservative simplification of it, it is a wrong
answer with a plausible shape. So the sweep straddles the cache boundary in both
directions and the report names the knee.

**A curve that is flat across the host's L3 boundary is measuring cache. It is a
failed run, not a fast one.** `crates/pir-bench` encodes that as an acceptance
test rather than as advice: in-cache mean against out-of-cache mean, pass at
ratio >= 1.15, and below that it prints NOT A USABLE FLOOR with the reason (a
working set that is not leaving cache, or a scan loop slower than the machine's
memory and therefore itself the binding constraint). A ratio below 1.0, where
throughput *rises* with size, is reported separately as inverted, because that
is not a cache effect at all: it means the small sizes are dominated by per-run
fixed cost.

**The concrete evidence that this rule is not theoretical.** The original sweep
started at 16 MiB. On the desktop's 12 MiB L3 that point is *already outside*
cache, so the curve came back flat and the knee was unreadable, not because the
machine was odd but because the sweep never sampled cache at all. Adding 2 and
4 MiB made it readable: **69.55 GB/s in cache against 22.39 GB/s out, a ratio of
3.11x**, and the acceptance test passed for the first time. The floor is the
out-of-cache figure, about 22 GB/s on one thread and about 30 GB/s across
twelve. 16 MiB straddles the 12 MiB L3 and is excluded from both sides of the
knee by the margin rule (a size counts as inside only if twice its bytes fit in
L3, and as outside only if it is at least twice L3), so a straddling point can
neither rescue nor sink the verdict.

**L3 is not a constant, and it is not even constant within a session.** The
container this work started in was rescheduled onto a different part mid-work:

```
measured first : 4 cores, Xeon @ 2.10 GHz, L2 8 MiB, L3 260 MiB, no GPU
measured later : 4 cores, Xeon @ 2.80 GHz, L2 4 MiB, L3  33 MiB, no GPU
```

Those are the same session. L3 moved by nearly an order of magnitude, which
moves the knee by nearly an order of magnitude with it. So the harness reads
cache size and CPU model **at runtime** and records them with every result
(`crates/pir-bench/src/host.rs`); nothing hardcodes a host. Where the host does
not offer a value cheaply, the run continues and the *report* degrades and says
so, rather than the measurement degrading silently: with L3 unknown the harness
prints "knee cannot be located" instead of guessing a boundary.

The consequence for reading this repository is in the results themselves: two
hosts appear, they are not comparable to each other, and every figure names its
own.

---

## 3. Minimum sample duration

A number produced in two milliseconds on a shared vCPU describes the scheduler,
not the memory.

At 16 MiB a pass finished in about 2 ms, short enough that scheduling noise and
frequency ramp dominate. Three runs of the same sweep on that host produced
in-cache/out-of-cache ratios of **1.00x, 0.70x and 1.46x**, flipping the
harness's own pass/fail verdict each time. The instability was entirely in the
in-cache point, not in the floor:

| across the same three runs | single-thread |
|---|---|
| 16 MiB (in cache) | 9.95, 7.18, 14.42 GB/s |
| >= 512 MiB (out of cache) | 9.5 to 10.9 GB/s, spread 0.6 to 3.7% |

So the fix was not "run it on a quieter machine". Each size is now repeated until
it has run for at least **50 ms**, and the reported per-pass time is the total
divided by the repeat count. Three such samples are taken per size, so the
spread across them is reported as well (section 8).

Four smaller rules travel with it, all in `crates/pir-bench`:

- One untimed pass before the timed ones, so the first timed pass is not also
  paying for first touch of freshly mapped pages.
- Buffers are filled from a cheap LCG rather than zeroed. A zeroed buffer can be
  backed by a single shared page, which would measure the page table instead of
  the memory.
- The scan sums `u64` words and the result is consumed, because summation is the
  cheapest operation that cannot be optimised away: this is a measurement of
  moving bytes, not of doing arithmetic on them. A unit test asserts that
  changing one byte changes the result, since the "touches everything" claim the
  floor rests on is a claim about this loop.
- A debug build refuses to run at all. A floor from an unoptimised build
  describes the optimiser's absence, not the machine.

One limitation belongs with this section rather than after it. The 0.6 to 3.7%
band quoted above spans all three sweeps on that host, and **only the third is
committed** (`measurements/floor-2026-08-28.json`), so that band cannot be
re-derived from this repository. It is quoted because it is the honest
description of what was seen, and flagged because the artifact behind it does
not exist here.

---

## 4. B = 1 is the headline

**Batched throughput may appear beside single-query latency. It may never appear
instead of it.** Every headline figure in this repository is at B = 1.

The justification is the declared reference workload. At lambda = 0.06
queries/second, one arrival every roughly 16.7 seconds, no batch forms: queries
arrive alone, and a batch-512 figure describes a workload that does not exist
here. The general form of the rule, recorded in
`docs/adr/0001-gpu-before-fpga-for-pir.md`, is that a throughput number without
an arrival-rate assumption beside it is not usable evidence, and that a reader
who substitutes their own lambda should be able to see which figures move.

The rule was written because of a specific table. The GPU DPF-PIR work (Tan,
Zeng, Feng, Peng, Wang, He, *GPU-Accelerated DPF-Based Private Information
Retrieval for Large-Scale Database*, TCHES 2026(3):933 to 957) reports its
headline throughputs at B = 512 and says plainly that this is the point "at
which GPU utilization is saturated". Its tables do not report single-query
latency for the full PIR at all; Table 1 reports it for the DPF expansion alone
(4.28 ms at layer 23 on an RTX 4090, for a batch of 512 trees). So that paper
simultaneously suggests PIR compute is nowhere near binding at this arrival rate
and optimises a metric that this arrival rate does not select. Both are worth
writing down. Separately, and more importantly, those figures are two-server:
they are excluded on the threat model rather than on speed, which is
`docs/threat-model.md`.

What makes B = 1 tolerable here is a property of the scheme rather than a
concession. SimplePIR streams the whole database for a *single* query, so one
query already saturates a 2016 gaming card: **4.02 ms per query at B = 1 on a
GTX 1070, a multiplier of 1.01x, run-to-run spread 0.1%**, at 83% of the card's
theoretical peak bandwidth. There is no headroom left to batch into. A scheme
whose per-query work is sub-linear in the database would need batching to fill a
GPU, and would be a different scheme with a different guarantee.

---

## 5. Verification gates: a timing is printed only if it passes

A benchmark that reports a time without checking that the computation was
correct is reporting how fast the machine produces wrong answers. Every printed
timing in the answer and hint sweeps is gated on the checks below, and they
passed at all eight sizes on both arms. Each is listed with what it does **not**
cover, because a check that is credited with more than it does is worse than no
check.

**Answer path.**

1. **The GPU answer is bit-identical to the CPU answer.** Not close, equal.
   Every operation is `u32` wrapping arithmetic over a commutative group, so the
   two reduction orders must agree exactly and a mismatch would be a bug rather
   than a rounding.
   *Does not cover:* anything the two arms get wrong in the same way. This is
   the **only** check on the answer kernel, and it is what the claim that the
   kernel reads every byte rests on.
2. **The device scan sum equals the host scan sum.**
   *Does not cover:* the answer kernel, at all. It validates the *scan* kernel,
   which is the denominator of the GPU multiplier, and nothing else.
3. **A record is decoded end to end from the answer**, at every one of the eight
   sizes on the CPU arm and from the GPU's own answer on the GPU arm: the record
   recovered through query, answer and hint subtraction equals the record stored
   in the database.
   *Does not cover:* the layout. It is **one record per size, at the midpoint of
   the database**. Four positions (`[0, 1, 17, count-1]`) are checked by a test
   inside the implementation under measurement, but only at m = 256. "decode OK"
   in a sweep row is not exhaustive over the layout, and should not be read as
   if it were.

**Hint path.**

4. **The serial and parallel CPU hints agree bit for bit.** Every word of `H` is
   written by one thread and `A` is read-only, so this is true by construction:
   it is asserted rather than hoped.
   *Does not cover:* the definition of `H`. It checks the parallel decomposition
   only.
5. **The GPU hint is bit-identical to the CPU hint**, all 30,015,488 words at
   the real index size.
   *Does not cover:* an error in generating `A`. Both arms expand `A` from the
   same seed through the same routine, so bit-identity **structurally** cannot
   catch an `A`-generation bug, however many words agree.
6. **`H.s == D.(A.s)` at full size.** This is the check that carries the most
   weight and is the easiest to under-rate. The right-hand side walks the same
   seed through a path that never materialises `A`, so it is the only thing
   tying the two arms together at m = 29312, and the only thing that closes the
   gap gate 5 structurally leaves open.
7. **The record decodes through the materialised hint**, not through the
   shortcut. Every decode before this one went through a path that is exact but
   needs `D`, and is therefore available to no client. This is the real client
   path.

Two more properties of the gating, rather than of any one gate:

- **A failure aborts the entire run** rather than moving to the next size. On
  the GPU this is not fussiness: a display-watchdog kill destroys the CUDA
  context, so every call after it fails, and a sweep that carries on would emit
  rows that look like measurements.
- **The decode margin is asserted, and it is a correctness property rather than
  a security one.** It runs at 41 sigma to 196 sigma across the sweep. A
  shrinking margin is where a wrong answer would come from, which is why it is
  checked; it says nothing about the security of any parameter set, and no
  parameter set here has been through a lattice estimator.

---

## 6. Check against a number from outside

Self-consistency is not evidence. Ten errors were found in this work's own
numbers and claims before publication, and `docs/errors-caught.md` records every
one. Five of the seven found during the work were caught by the same mechanism,
and it was not review or testing: an external figure existed that our own number
could be held against, and it disagreed. Two of those five are where this
section's rule comes from.

- Built at Rust's default x86-64 target, which has no 32-bit vector multiply,
  the answer measured 6.11 GB/s and the multiplier came out at **3.55x** instead
  of 2.13x at m = 16384: two thirds too large, and it would have been published
  as a fact about SimplePIR rather than about a build flag.
- The first form of the hint kernel, the obvious full-row `axpy`, measured 2.5
  GMAC/s and 2.47 MB/s per core, which is **0.6x** the published CPU throughput
  it was meant to beat. Strip-mining the contraction reached 10.0 GMAC/s and
  9.80 MB/s per core, **2.5x**. A factor of four, and the slow version was
  plausible enough to publish.

Both of those are written up, with the numbers they would have published, in
`docs/errors-caught.md`, alongside the other eight. The generalisation is the
rule this section states:
**where an outside number exists, quote it and check against it; where none
exists, say so and report against a ceiling instead.**

Three outside numbers are used that way here:

| Ours | Outside |
|---|---|
| 120.1 MB hint at 859 MB, 234 KB per query round trip | SimplePIR's authors report 121 MB at 1 GB and 242 KB per query |
| 120.1 MB hint, independently measured | YPIR's own internal server-side SimplePIR hint at this size is 117.4 MB, agreeing to within 2.2% |
| 9.59 MB/s/core hint generation, flat at 8.90 to 9.91 across a 512-fold range of sizes | under 4 MB/s/core published for SimplePIR preprocessing (YPIR, ePrint 2024/270) |

And one place where the outside number does not exist, stated as such: no
published work reports a throughput for GPU-accelerated SimplePIR *offline*
preprocessing, so the 1.17 s rebuild is reported against two hardware ceilings
instead (1139 GMAC/s for `sm_61`'s emulated `IMAD`, 3418 for `DP4A` at full
rate), at 57 to 71% utilisation of the IMAD ceiling across the sweep. That is
what says the figure describes the card rather than the kernel. It is weaker
than an independent measurement and is labelled weaker.

The same rule is why the four third-party implementations were **built and
re-run at this repository's record width** rather than read off their own
tables. Every published figure for them is at 1-bit or 1-byte records, and
record width is exactly where the first candidate collapsed: at Q = 157 queries
per rebuild period, YPIR falls behind our hint after 1.1 queries, HintlessPIR
after 5.0, and VIA stays ahead until 335. Those crossovers are properties of the
schemes; Q only decides which side of each one a reader is standing on. The runs
are in `measurements/external-2026-08-29-ypir-via.md`,
`measurements/external-2026-09-04-hintlesspir.md` and
`measurements/crate-comparison-2026-08-28.md`.

---

## 7. Commit as produced, and correct by naming the superseded sentence

**A measurement that lives only in a terminal did not happen.** This is not a
slogan here: the container this work started in was ephemeral and was in fact
rescheduled onto a different CPU mid-session, taking its terminal history and
its L3 with it. Everything in `measurements/` is the artifact as the run emitted
it, JSON and text, committed at the time. The write-ups quote those files; they
do not restate remembered numbers.

The cost of not doing this is on record too. An earlier draft claimed a cold
driver JIT of 102 ms, from an observation that was never committed. Measured, it
is **33.2 ms** cold against 0.6 ms warm. The claim was not merely unsupported,
it was wrong, and nothing but a committed artifact would have shown that.

**Corrections name the sentence they supersede rather than deleting it.** That
is the house style throughout, and it is why the ported documents read as
layered rather than clean. A reader will meet, in the same file, a claim and the
measurement that overturned it:

- "Hint generation was not benchmarked" and "the largest unmeasured quantity in
  this repository", superseded by 89.6 s on one core, 25.0 s across twelve and
  1.17 s on the GTX 1070.
- Record-size figures from a preliminary 1,862-record sample that was never
  committed, superseded by 100,394 records from 993 volumes.
- A reading that filed VIA beside YPIR as a one-byte-record scheme, superseded
  by the measurement that its records are 512 bytes, which is what makes it the
  only external scheme ahead of us on total client bytes.
- Section headings that read "Phase 3a" and "Phase 3b", superseded by 4a and 4b.
  Some phase numbering survives here from an internal document that is not
  published; it is left in place because renumbering a correction detaches it
  from what it corrected.

A correction that deletes its own subject is not checkable. The point of the
style is that a reader can see what the error was, not merely that one was
fixed.

One case of it is worth repeating because the error never reached a committed
figure and is recorded anyway. The sustained-load check on the hint kernel, which
exists because a roughly one-second integer kernel on a display-driving card can
drop off boost, originally compared the first launch to the last without
normalising for length. The last launch is short, since it covers whatever rows
remain, so the check reported a 0.67x "speedup", which is not something a
throttling card does. Normalised to ns/row the series runs 41522 to 36229, a
ratio of 0.87, so the clock held. The unnormalised version was fixed before the
figure was committed, and it is written down because **a check that reads as
reassuring and is meaningless is worse than no check**, and the only way to stop
one being rediscovered is to describe it.

---

## 8. Report the median, with the spread beside it

The harness keeps best, median and worst across repeats and reports the
**median** with the **spread**, defined as (worst minus best) divided by the
median. Averaging would hide exactly what the spread is for.

The case that justifies it is the two smallest GPU rows. At m = 4096 the
run-to-run spread is **21.1%**, the largest anywhere in the GPU answer sweep,
and at m = 2048 it is **20.1%**, against 0.1% at m = 23168 and 0.1% at the real
index size (`measurements/gpu-simplepir-2026-08-28.json`, field
`gpu_answer_spread`). A mean would have absorbed those two rows into the curve;
the median with the spread beside it is what makes them excludable rather than
smoothed away. Nothing was dropped.

> **Correction.** This paragraph read: *"one GPU row (m = 23168) had a 379%
> spread while its median stayed in line with its neighbours. The card drives a
> display, so that is a stall, not a measurement."* The committed artifact
> records `"gpu_answer_spread": 0.0008` at that row, which is 0.1%, and the
> string `379` appears nowhere under `measurements/`. The superseded sentence is
> the one that said 379%, and `docs/errors-caught.md` entry 9 records it.

The spread also does the work in three other places:

- **It says when a machine is too noisy to conclude from.** Above roughly 10%,
  adjacent sizes stop being distinguishable, and `crates/pir-bench` prints a
  warning naming how many sizes crossed that line. On the desktop sweep, several
  did; that is a desktop with a browser on it. They do not touch the knee, whose
  two sides differ by 3.1x, and saying so is a claim a reader can check against
  the printed spreads rather than one they have to accept.
- **It licenses, or refuses, a comparison assembled from two runs.** The
  build-flag finding in section 6 compares two builds measured one run apart. The
  floor moved 5.8% between them and the answer moved 76%, an order of magnitude
  more than the drift, so the comparison survives. Quoting both floors rather
  than asserting "the floor did not move" is the difference between a claim and a
  checkable one.
- **It separates a fast row from a short one.** The GPU rows at 4.2 MB and
  16.8 MB carry 20% and 21% spreads against 0.1% at the real index size, which
  is how those rows are known to be dominated by launch overhead rather than
  reporting a real multiplier below 1.

---

## 9. Every figure carries its sample definition

**The hosts.** Two, not comparable to each other, named at every figure:

| Host | What it produced |
|---|---|
| Shared Xeon vCPU @ 2.80 GHz, 4 threads, 33 MiB L3, no GPU | the first floor |
| Desktop i5-11400 @ 2.60 GHz, 6C/12T, 12 MiB L3, 15.8 GB RAM, GTX 1070 | everything after it |

The GPU is a GeForce GTX 1070: compute capability 6.1, 15 SMs at 1.78 GHz, 8 GiB
over a 256-bit bus (256.3 GB/s theoretical peak), 2 MiB L2, display watchdog
active. Its kernels are PTX for `compute_61` from NVRTC 12.9, JIT-compiled to
`sm_61` by the driver, for the reasons in
`docs/adr/0002-pascal-without-a-pascal-toolkit.md`. Everything from the desktop
was produced by one build, in one sitting, on 2026-08-28.

**The corpus the timings ran against is synthetic, and the shape is what
matters.** 128-byte fixed-width records, m/128 records per column, 6,712,448
records at m = 29312, 859.2 MB. The timings depend only on the shape, so the
synthesis does not weaken them. The records recovered by the decode gates are a
test of the pipeline and are **not facts about case law**.

**The record-size study is real data, and this is its full sample definition.**
100,394 records from 993 volumes of the Caselaw Access Project (CC0; terms as
retrieved are in `docs/cap-terms-2026-08-28.txt`), **0 volume misses**,
stratified across {federal, state} x {pre-1950, 1950-2000, post-2000}. Metadata
only: no opinion text is downloaded. `tools/fetch_cap.py` is resumable and
validates structurally rather than on status code, because a missing volume
comes back as an HTML error page and not always with a 404.

Measured: key 8.95 B, case name 28.79 B, payload 39.75 B, and **99.970% of
records intact at a 128 B fixed-width entry**, from which m = 29312 and 859.2 MB
follow. Per stratum, in bytes:

| stratum | n | mean | median | p95 | max |
|---|---|---|---|---|---|
| federal/pre-1950 | 42,216 | 41.50 | 39 | 67 | 167 |
| federal/1950-2000 | 2,883 | 36.88 | 34 | 59 | 104 |
| federal/post-2000 | 14,084 | 44.19 | 36 | 83 | 156 |
| state/pre-1950 | 24,672 | 36.16 | 31 | 59 | **186** |
| state/1950-2000 | 9,316 | 37.03 | 29 | 75 | 145 |
| state/post-2000 | 7,223 | 37.70 | 29 | 74 | 134 |

Where this sample is weak, stated rather than smoothed:

- **The strata are not equally represented.** Round-robin sampling exhausted the
  thin ones, so federal/pre-1950 contributes 42,216 records and federal/1950-2000
  only 2,883, and the overall mean is weighted toward pre-1950. The check that
  matters is that the **unweighted mean of the six stratum means is 38.91 B**
  against a weighted 39.75 B, and every stratum sits between 36.16 and 44.19.
  The figure survives reweighting, which is what the stratification was for.
- **Five first-series regional reporters are absent** from the source under the
  slugs tried, so pre-1950 state coverage comes from early second-series volumes
  rather than the true first series.
- **6.7M is the corpus's own stated coverage, not something this sample
  measured.** The sample fixes the record *size*; the record *count* is taken
  from the Caselaw Access Project's description of itself.
- **The post-2000 stratum is bounded above by the corpus's own coverage, which
  ends in 2019.** This was not stated when the stratum was defined and it should
  have been, because the worry the wider sample was built to test is
  specifically that *modern* names run long. Checked against the source on
  2026-09-11, the last volume in each current-series reporter
  `tools/fetch_cap.py` pulls runs no later than 2019: `a3d` 2004-2019 (253
  volumes), `p3d` 2000-2019 (447), `sw3d` 1993-2018 (579), `ne3d` 2003-2018
  (130), `f3d` 1993-2019 (935), `f-supp-3d` 2005-2019 (386). Two regions have no
  third series in the corpus at all, so `nw2d` and `se2d` stand in on the
  third-series line. So *post-2000* means 2001 to 2019.
- The sample also **overturned the worry that motivated it**. The prediction was
  that modern state-court names would run far longer and that post-2000 state
  volumes were where the estimate was weakest. Measured, that stratum has the
  smallest mean of the three post-2000 groups, and the longest record in the
  whole sample is a pre-1950 state case at 186 B. Modern federal names are the
  long ones. That holds over the 2001 to 2019 window the stratum actually
  covers, and says nothing about cases decided after 2019. The superseded
  sentence is the one that said the post-2000 state slice is where the long
  names are.

---

## 10. What this methodology does not give you

Stated here so that it is not inferred from the care taken above.

- **It is not a security evaluation.** No parameter set has been through a
  lattice estimator, the error sampler is a centred binomial, the secret comes
  from a general-purpose PRNG, and neither the CPU loop nor the GPU kernel is
  constant-time. None of the gates in section 5 is a security check; they are
  correctness checks.
- **It does not make the closed figures reproducible.** The implementation under
  measurement is not published, so the answer sweep, the hint figures and the
  gates that ran against them are reported rather than reproduced. `SCOPE.md`
  section 2 lists exactly which figures that covers, and what a reader can check
  instead.
- **It does not certify the schemes it compares against.** The external runs
  measure the reference implementations as shipped, at this index shape. Where
  one of those implementations has a defect that bounds its own number (VIA's
  First Dimension loop reads `database[0]` on every iteration rather than
  streaming the database, so 84% of its answer time is a floor rather than a
  measurement), the defect is recorded with the run and the figure is read as a
  bound, not used to dismiss the scheme.
- **It does not replace running the code.** The floor harness and the corpus
  sampler are published precisely because the denominator and the corpus study
  are the parts a reader can re-derive without us. Everything else in this
  repository should be read as what it is: measurements taken carefully, with
  their limits attached, by the people who also wrote the thing being measured.
