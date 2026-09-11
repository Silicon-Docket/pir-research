# Errors caught before publication

Ten errors were found in this work's own numbers and claims before those
numbers were published. Seven were found while the work was being done; three
more were found while the work was being prepared for this repository, by
checking every sentence against the artifact behind it. Several had already been
committed to internal documents and are marked as superseded there; one was
fixed before its figure was ever committed. This file records, for each one,
what the wrong number was, what would have gone out instead, and what caught it.

It is kept for two reasons. The first is that a repository which reports only
its final numbers gives a reader nothing to judge the process by. The second is
narrower and is the argument this file exists to make: **five of the first seven
were caught by a number or an artifact from outside this work, and none of the
seven could have been caught by internal consistency.** Every one of them
produced a self-consistent result. Several produced a *plausible* one, which is
worse.

The last three sharpen that argument rather than repeating it. They were caught
by a mechanism the first seven never applied to themselves: reading each written
claim back against the committed artifact or the source file it describes, on
the way out the door. All three had survived the whole of the work that produced
them.

Conventions here are the ones used everywhere in this repository. Every figure
names its host and its sample. A correction names the sentence it supersedes
rather than deleting it, so superseded claims appear below beside the
measurements that replaced them. Two hosts appear and they are not comparable
to each other:

| Host | What it produced |
|---|---|
| Shared Xeon vCPU @ 2.80 GHz, 4 threads, 33 MiB L3, no GPU | the first linear-scan floor |
| Desktop: i5-11400 @ 2.60 GHz, 6C/12T, 12 MiB L3, 15.8 GB RAM, GTX 1070 (cc 6.1, 15 SMs @ 1.78 GHz, 8 GiB, 256-bit bus, display watchdog active) | everything below |

Where a comparison uses the **declared reference workload** (lambda = 0.06
queries/second, Q = 157 queries per client per rebuild period), those are stated
evaluation parameters rather than measurements. `SCOPE.md` section 3 defines
them and is the only place they are defined.

The SimplePIR implementation these figures came from is not published here; it
is called *the implementation under measurement* throughout, and no path inside
it is cited. `crates/pir-bench`, `tools/`, `.cargo/config.toml` and every raw
artifact under `measurements/` are published, and are cited by name.

## Summary

| # | The error | What would have been published | What is true | What caught it |
|---|---|---|---|---|
| 1 | Binary built for baseline x86-64 | PIR multiplier **3.55x** at m = 16384 | **2.13x** | A second build, plus the instruction set the first one had compiled to |
| 2 | Hint kernel written as a full-row `axpy` | 2.47 MB/s per core, **0.6x** the published figure | 9.80 MB/s per core, **2.5x** | An outside number: YPIR, ePrint 2024/270 |
| 3 | Sustained-load check not normalised for launch size | a **0.67x "speedup"** under sustained load | ratio **0.87** in ns/row, the clock held | The direction of the result: a throttling card does not speed up |
| 4 | Cold JIT quoted from an uncommitted observation | a **102 ms** cold JIT | **33.2 ms** cold against **0.6 ms** warm | Running it with the driver cache disabled, and committing the output |
| 5 | Deeper recursion assumed to shrink the hint | a **3.9 MB** hint at r = 3 | **4.3 GB** at r = 3, **4.4 TB** at r = 4 | Reading SimplePIR's own Remark 5.1 instead of extrapolating |
| 6 | VIA's plaintext modulus read as its record size | VIA filed with YPIR as unusable at 128-byte records | VIA records are **>= 512 B**, and VIA is the only scheme measured that beats the 120.1 MB hint on client bytes | Building and running `owniai/VIA` @ `f65aa9d` |
| 7 | HintlessPIR ranked last on a one-byte-record download | **3,080 KB** down per query, 130x VIA's | **22.65 MiB** down per query at 128-byte records | Building and running `google/hintless_pir` @ `812babf` |

Errors 1 and 2 are the two the `README.md` summarises. Errors 6 and 7 are the
same error one layer apart, which is the reason both are listed.

Three more were found while preparing this repository, by checking each written
claim against the artifact or the source behind it:

| # | The error | What would have been published | What is true | What caught it |
|---|---|---|---|---|
| 8 | A section heading left at an earlier draft's figure | GPU multiplier **1.03** in a heading, above a table reading 1.01 | **1.01** (raw: `multiplier` 1.0055) | Reading the heading against the table under it and against `gpu-simplepir-2026-08-28.json` |
| 9 | A run-to-run spread quoted from an uncommitted observation | **a 379% spread** at m = 23168 | **0.1%** at that row; the largest spread anywhere in the GPU sweep is 21.1% at m = 4096 | Searching the committed artifacts for the figure and not finding it |
| 10 | The harness stated the direction of its own bias backwards, in published code | "a floor that is too HIGH makes the PIR multiplier too SMALL, which flatters the scheme" | An inflated floor **inflates** the multiplier: the in-cache row at m = 2048 has a 74.90 GB/s floor and the sweep's **largest** multiplier, 6.23 | Deriving the direction from the definition, then checking it against the sweep's own rows |

---

## 1. The build target, which changed the headline by two thirds

**What would have been published: a PIR multiplier of 3.55x at m = 16384.**

Rust's default target is baseline x86-64, which is SSE2, and SSE2 has no 32-bit
vector multiply (that arrived with SSE4.1). The operation at the centre of
SimplePIR's online answer is a byte-times-word multiply-accumulate over the
whole database, so on the default target it compiles to something far narrower
than the host can issue. The floor it is divided by, a plain linear scan, is
*not* narrowed by the same gap, so the entire error lands in the numerator.

Measured on the desktop at m = 16384 (268.4 MB), both rows from the same source
one build apart, single-threaded:

| build | floor 1T | answer 1T | multiplier 1T |
|---|---|---|---|
| default target (`RUSTFLAGS=""`) | 21.69 GB/s | 6.11 GB/s | **3.55x** |
| `-C target-cpu=native` | 22.94 GB/s | 10.78 GB/s | **2.13x** |

Raw: `measurements/baseline-target-cpu-2026-08-28.txt` for the first row,
produced into a separate target directory so the two builds could never be
mixed, and `measurements/cpu-simplepir-2026-08-28.json` for the second. The
baseline run decoded (margin 69 sigma) and reports a 0.3%/0.4% spread, so it is
a clean measurement of the wrong thing rather than a noisy one.

**Without the flag the reported multiplier would have been two thirds too
large, and it would have been read as a fact about SimplePIR.** It is a fact
about the instruction set the binary happened to be built for. This is why
`.cargo/config.toml` is published rather than trimmed for portability: the file
carries these numbers in its comment, and deleting the setting would silently
inflate the metric again. The cost of keeping it is that binaries are not
portable off the build host, which is acceptable because every result in
`measurements/` already records the CPU it ran on.

**The honest part, which is what makes the comparison checkable.** The two rows
are from two runs, not one. The floor column is quoted precisely so a reader can
see what that cost: the floor moved **5.8%** between the runs, which is what a
desktop with a browser on it does, and the answer moved **76%**. The effect is
an order of magnitude larger than the drift, so the comparison survives being
assembled from two runs. Quoting both floors rather than asserting "the floor
did not move" is the difference between a claim and a checkable one, and it is
the only reason this entry is evidence rather than an anecdote.

One further figure belongs here, because it changes what the table is a
comparison of. An earlier form of the inner loop, the obvious
`iter().zip().fold()`, measured **4.50 GB/s** on the default target.
Restructuring it into fixed-size lanes helped both targets, and the table above
is from the current code so that both of its rows can be reproduced against the
same source.

**What caught it:** asking why an answer kernel was running at a third of a
floor measured by a different loop (`crates/pir-bench`) on the same host in the
same sitting, and then checking what the generated code could issue against what
the ISA provides. Internal consistency had nothing to say: the 3.55x run passes
every correctness gate this repository has, decodes, and is reproducible.

## 2. The hint kernel, where the arithmetic was right and the intensity was wrong

**What would have been published: 2.5 GMAC/s, 2.47 MB/s per core, which is 0.6x
the published CPU figure it was meant to beat.**

Hint generation is `H = D.A`, O(m squared n), which is 8.8 x 10^11
multiply-accumulates at the real index size and four orders of magnitude more
work than one query. The first form of that kernel was slower than the number in
the literature it was being compared against, and it was slower for a reason
that has nothing to do with the arithmetic being wrong.

The cause was arithmetic *intensity*. Written as the obvious full-row `axpy`,
every multiply-accumulate needs a load of `A`, a load of `H` and a store of `H`:
three memory operations per MAC, so the multiplier unit never binds and the
kernel measures the memory system. Strip-mining `n`, which means holding 64
words of `H` in registers across the whole contraction, removes `H` from the
inner loop entirely and leaves one `A` load per eight MACs.

| form | 1 thread, desktop | vs published |
|---|---|---|
| full-row `axpy` | 2.5 GMAC/s, 2.47 MB/s per core | **0.6x** |
| strip-mined | 10.0 GMAC/s, 9.80 MB/s per core | **2.5x** |

**A factor of four**, and the first version would have been published as a fact
about SimplePIR's preprocessing cost rather than about one loop nest.

**Where this comparison is weak, stated rather than smoothed.** The superseded
form is not among the committed artifacts; only the form that replaced it is, in
`measurements/cpu-hint-2026-08-28.json` / `.txt`. The 2.5 GMAC/s figure is
quoted from the record made at the time, and the record does not name the `m` the
two forms were run at. What *is* committed is the shipped form across all eight
swept sizes, where the rate is flat at **8.90 to 9.91 MB/s per core** (a
512-fold range of databases for an 11% spread), and **9.59 MB/s per core, 89.6 s
for one full rebuild** at m = 29312. Entry 4 below is the reason that
distinction is worth drawing out rather than glossing.

**What caught it:** having a number from outside this repository to check
against. YPIR (ePrint 2024/270) reports under 4 MB/s per core as the published
CPU throughput for SimplePIR preprocessing. A kernel at 2.47 MB/s per core is
not obviously broken; it is a plausible number, it passes every bit-identity and
decode check, and nothing internal to this work had an opinion about whether it
was fast. The outside figure is what made 0.6x legible as a defect rather than
as a result.

## 3. The sustained-load check that was reassuring and meaningless

**What would have been published: a 0.67x "speedup" offered as evidence that the
GPU held its clock.**

The GPU answer kernel runs for about 4 ms at the real index size, which is far
too short to show thermal or power behaviour. The hint kernel is different: it
runs for about a second in total, in launches sized against a wall-clock budget,
and a roughly 1 s integer kernel on a card that is also driving a display can
drop off boost. So a sustained-load check was added to the hint arm.

The first version of that check compared the first launch to the last without
normalising for how much work each launch did. **The last launch is short,
because it covers whatever rows remain**, so the check reported the series as a
0.67x speedup. A card that throttles does not get faster. The number was not
just unhelpful, it pointed the wrong way, and it would have been quoted as
reassurance.

Normalised to ns/row the same series reads:

| m = 29312, GTX 1070, 10 launches x 2944 rows | first | last | ratio |
|---|---|---|---|
| per-launch cost, normalised | 41522 ns/row | 36229 ns/row | **0.87** |

so the clock held. Per-launch wall times for that row are min 102.0, median
117.6, max 127.2 ms against a 500 ms budget. Raw:
`measurements/gpu-hint-2026-08-28.json`, fields `drift_first_ns_per_row` and
`drift_last_ns_per_row`, with `measurements/gpu-hint-2026-08-28.txt` carrying
the same series in the printed sweep.

The budget is checked on **wall** time rather than CUDA event time, because on
WDDM a launch queued behind display work accumulates time that events never see,
and a launch past roughly two seconds kills the display driver, which destroys
the CUDA context, after which every remaining call fails and a sweep that
carries on emits rows that look like measurements. The hint routine in the
implementation under measurement therefore probes with 512 rows and sizes its
launches from what the probe actually cost rather than from an assumed rate; at
the real index size that is the 10 launches of 2944 rows above.

That the normalised check is a real check and not a rubber stamp is visible in
the committed column, which is not uniformly favourable: at m = 32768 it runs
42412.1 to 45948.4 ns/row, a ratio above 1, and at m = 23168 it runs 44977.0 to
29603.0.

**What caught it:** the direction of the result. This one was fixed before the
figure was ever committed, so it is the entry with the least external support
and it is recorded anyway, because **an unnormalised version of this check reads
as reassuring and is meaningless**, and a reader has no way to tell the two
apart from the printed output alone.

## 4. The cold JIT figure that was remembered rather than measured

**What would have been published: a cold PTX JIT of 102 ms.** That claim was
committed, in an earlier draft of the GPU section of `measurements/RESULTS.md`,
and it came from an observation that was never committed as an artifact.

Measured properly, same binary, same PTX, same card, one run each, seconds
apart, the only difference being whether the driver's compute cache is allowed
to answer:

| | JIT of `compute_61` PTX to `sm_61` |
|---|---|
| cold (`CUDA_CACHE_DISABLE=1`) | **33.2 ms** |
| warm (default) | **0.6 ms** |

A factor of about 55. Raw: `measurements/gpu-jit-cold-2026-08-28.txt`, whose
header states the condition under which the run would have proved nothing ("if
the two numbers do not differ materially, this run proved nothing and says so
rather than being presented as evidence") before either number was in hand.

**The superseded sentence is the one that said "102 ms".** It was not merely
unsupported, it was wrong, and wrong by about 3x. Both figures are start-up
costs and neither is inside a query, so nothing downstream moved; the reason the
entry is here is the provenance rather than the magnitude. A number that lives
only in a terminal did not happen, and this is the one case in this repository
where a number that lived only in a terminal was quoted anyway and turned out to
be false.

**The limit on the correction, stated with it:** one run each. That establishes
the order of magnitude and the sign, not the 55x to two figures. Repeating it
would cost seconds and has not been done.

**What caught it:** the repository's own rule about what counts as a
measurement, applied to a sentence rather than to a table. Nothing about 102 ms
was internally inconsistent.

## 5. The recursion error, wrong by about 1,100x in the unsafe direction

**What would have been published: that a three-dimensional SimplePIR variant
gives a 3.9 MB hint.**

Phase 4c concluded that hint *generation* is not a cost and hint *distribution*
is the entire cost, so anything that shrinks the 120.1 MB hint is worth
checking. Deeper recursion, which is what "multi-dimensional SimplePIR" means,
looks like the obvious move. An earlier reading of this work's own numbers
guessed that going to three dimensions would give `m = N^(1/3)` and therefore a
3.9 MB hint.

**That is wrong.** It assumed the recursion shrinks `m` while leaving the hint
formula alone, when what the recursion actually does is trade the `sqrt(N)` term
for an `n^r` one. SimplePIR's Remark 5.1, verbatim:

> "After *r* levels of recursion, the cost of the recursive PIR scheme, on
> lattice dimension *n* and database size *N*, would be (hiding constants):
> one-time download **n^r** in the preprocessing step, as well as per-query
> upload r·N^(1/r) and download **n^(r−1)**. For *r > 2*, the communication is
> likely too large for databases of interest."

The hint becomes independent of `N` and grows as `n^r` in the lattice dimension:

| r | hint |
|---|---|
| 2 (DoublePIR) | 4.2 MB, or 16.8 MB with kappa = 4, which matches their published 16 MB |
| **3** | **4.3 GB** |
| 4 | 4.4 TB |

3.9 MB against 4.3 GB is a factor of about 1,100, **in the unsafe direction**:
the guess made a dead end look like the cheapest option in the repository. The
r = 2 row is the check on the reading, because it reproduces a published figure
the authors state independently.

**A related correction, from the same source and the same day.** The item that
said DoublePIR "shrinks the hint to roughly 16 MB" is superseded for the same
class of reason: that figure is measured at **one-byte** records. At 128-byte
records, SimplePIR's own Table 16 gives DoublePIR 43 MB of amortized per-query
communication against SimplePIR's 1.4 MB, and their section 5.2 large-record
construction gives a hint of `d.kappa.n^2`, which at `d = 128`, `kappa = 4`,
`n = 1024` is a **2.1 GB hint**, 18x larger than the 120.1 MB already paid, with
a 4.2 MB per-query download against 117.2 KB. SimplePIR is flat in record width
and DoublePIR is linear in it. The property that makes DoublePIR attractive on
one-byte records is the property that makes it useless at this record width.

Both of these are arithmetic on published formulas rather than measurements, and
are labelled as such wherever they appear.

**What caught it:** reading the primary source instead of extrapolating from
this work's own numbers. The guess was internally consistent with everything
else measured here; it was inconsistent only with the paper being extrapolated
from, which is a document no internal check consults.

## 6. VIA, misfiled on a parameter name

**What would have been published: that VIA is a one-byte-record scheme, filed
alongside YPIR as unusable at 128-byte records.** The superseded reading is the
one that listed VIA's "sub-KB queries" beside YPIR and treated the two as the
same case.

`LOG_MODULUS_P = 8` was read as meaning one-byte records. It is the plaintext
*modulus*, bits per coefficient, and says nothing about how many coefficients a
record occupies. **A VIA record is `DEGREE2 = 512` coefficients**, and the run
confirms it arithmetically: 256 MB divided by `N = 524,288` is 512 B. So one VIA
query covers one 128-byte record, and the scheme was never in the same category
as YPIR at all.

The consequence of the correction is the largest single change of conclusion in
this repository. Per client, per rebuild period, at Q = 157 and 128-byte
records, all three rows at this index size:

| | database | offline | Q = 157 queries | total | vs ours |
|---|---|---|---|---|---|
| ours (SimplePIR) | 859.2 MB | 120.1 MB hint | 36.8 MB | **156.9 MB** | |
| **VIA**, `LOG_COL = 11` | 1 GiB | 0 | 93.2 MB | **93.2 MB** | **1.7x better** |
| YPIR | 859.2 MB | 0 | 17,656 MB | **17,656 MB** | 113x worse |

**VIA is the only scheme measured here that beats the 120.1 MB hint on total
client bytes.** It stays ahead until **335 queries per rebuild period**, against
a declared Q of 157; YPIR falls behind after **1.1 queries**, because its items
are one byte (`src/params.rs` rejects `ITEM_SIZE_BITS` above 8) so a 128-byte
record costs 128 separate queries. The crossovers are properties of the schemes
and do not move with Q.

**What is weak in the correction, carried with it rather than after it.** The
VIA byte counts are *computed* by `printInfoVIA` in closed form from the gadget
parameters, not counted off a wire: nothing in the run serialises a query. They
were checked against the source, and the closed form reproduces the printed
figure exactly at the `LOG_COL = 9` that was run, which is the check that it is
being read correctly; the 556.5 KiB at `LOG_COL = 11` is that same formula
evaluated at this index's shape. The run itself is at a reduced `LOG_COL`,
because VIA's shipped `LOG_ROW = 8`, `LOG_COL = 13` allocates 32 GiB of `uint64`
NTT coefficients for a 4 GiB logical database and is OOM-killed on this 16 GB
machine. And "512 bytes" is a floor rather than an exact figure: `Recover`
decodes eight `DEGREE2` blocks while `N` counts single blocks, so a record is
either 4 KB, or 512 B with the download carrying eight times what the query
asked for, or the eight blocks together. Nothing in the conclusion depends on
which, because every reading gives at least 512 B and the claim needs 128 B.

Two defects in the artifact were found in the same run and are recorded with it
rather than used to dismiss the scheme: VIA's First Dimension loop reads
`database[0]` on every iteration rather than streaming the database, so 84% of
its measured answer time is a floor and not a measurement, and its harness never
checks that the record recovered is the record that was stored. Full write-up,
including the 11.7x per-byte server-time gap and the 8x RAM expansion that is the
real obstacle: `measurements/external-2026-08-29-ypir-via.md`.

**What caught it:** building `owniai/VIA` @ `f65aa9d` and running it, instead of
reading its paper's table. The misfiling was consistent with every other
document here, and would have stayed consistent forever.

## 7. The same error one layer up: HintlessPIR ranked on an untested number

**What would have been published: HintlessPIR ranked last of the three
hint-free schemes, on a published per-query download of 3,080 KB, 130x VIA's.**

That figure is measured at one-byte records. **Which is exactly the unchecked
assumption that had just misfiled VIA**, two entries of this work earlier. The
ranking was a scheduling decision, an ordering of what to run next, rather than
a finding, and the write-up that made it recorded it as one **before** the run,
in `measurements/external-2026-08-29-ypir-via.md`:

> **HintlessPIR is still unrun**, and the reason it is ranked last is weaker
> than it looks. Its published per-query download of 3,080 KB is 130x VIA's,
> but that figure is measured at one-byte records, which is exactly the
> unchecked assumption that had VIA filed next to YPIR two paragraphs of this
> repository ago. Ranking it last is a scheduling decision, not a finding.

Run at this repository's real shape (`db_rows = 229`, `db_cols = 29312`, the
bandwidth-optimal split for 128-byte records at this record count, not a round
number chosen for convenience), `google/hintless_pir` @ `812babf`, built via
Bazel, single core on the desktop, gated on a correctness check per
`docs/adr/0002-pascal-without-a-pascal-toolkit.md` point 6:

| | measured |
|---|---|
| public params, offline, once per DB version | **264 bytes** |
| request, per query | 513,535 bytes (501.5 KiB) |
| response, per query | 23,751,495 bytes (**22.65 MiB**) |
| server `HandleRequest`, 1 core, 5 reps | **113.65 s** |
| server preprocessing, 1 core | 6,092.8 s = 1.69 core-hours |

The hint-free claim holds exactly: 264 bytes replaces a 120.1 MB hint and does
not grow with the database. Everything else is worse than the published table
suggested in one direction and better in another. **Naively scaling the
published 3,080 KB by 128 would predict about 385 MB per query; the measured
figure is 23.75 MB, about 7.7x smaller than that scaling**, because the
published number was measured at a different row/column shape as well as a
different record width. Neither extrapolation was right, which is the point.

At Q = 157 that is **3,809.6 MB against 156.9 MB, 24x worse**, and it falls
behind the hint after **5.0 queries**, landing between VIA (93.2 MB, still the
only scheme ahead) and YPIR (17,656 MB). On server time it is **about 1,415x
slower per query than SimplePIR on one core**: 113.65 s against 80.3 ms. Against
the declared lambda of 0.06 queries/second, one arrival every roughly 16.7 s,
that is 6.8x the entire inter-arrival budget on a single core, a backlog that
never drains rather than a latency figure with margin. The reference binary
spawns no threads of its own, so this describes what was run and not a ceiling
on what further engineering could do.

So HintlessPIR did end up last on bandwidth among the two survivors and out on
both counts. **The ranking was right and the reason for it was not**, and the
two are worth separating: the published figure that justified it would have
predicted an ordering (130x VIA's download) that the measurement does not
support at this record width. Full write-up:
`measurements/external-2026-09-04-hintlesspir.md`.

**What caught it:** building and running the third reference implementation, and
before that, writing down that the ranking was unearned at the moment it was
made. The second half matters as much as the first: the error was visible as an
error while it was still only a scheduling decision, and it was recorded then
rather than discovered later.

---

---

# Found while preparing this repository

The three below were not found by running anything. They were found by reading
each written claim back against the artifact or the source file it describes,
while porting this work into a public repository. All three had survived the
whole of the work that produced them, and two had been committed and read many
times.

They are listed with the first seven rather than in a separate file because they
are the same kind of failure: a number that was right when it was written down,
or never right at all, and that nothing afterwards was obliged to re-check.

## 8. A heading left at an earlier draft's figure

The GPU section carried the heading **"What a multiplier of 1.03 means"**. The
table immediately above it reads 1.01, the sentence immediately above it reads
"a multiplier of 1.01x", and the raw artifact agrees:

```
gpu-simplepir-2026-08-28.json, m = 29312: "multiplier": 1.0055
```

**1.01 is correct and the heading was stale.** The superseded text is the
heading that read 1.03.

This is the smallest error in this file and it is listed because of where it
sat. A heading is the part of a section a reader remembers, and it is the part
no check touches: the decode gates, the bit-identity checks and the spread
figures all constrain the table, and none of them constrains the sentence above
the table. The prose and the artifact had drifted apart by 2% in the direction
that makes the scheme look worse, and nothing in the pipeline was going to
notice, because nothing in the pipeline reads prose.

## 9. A spread quoted from an observation that was never committed

The same section said:

> **One row had a 379% spread** (m = 23168) while its median stayed in line with
> its neighbours. The card is driving a display; that is a stall, not a
> measurement.

**The committed artifacts contradict it.** That row's spread is 0.1%:

```
gpu-simplepir-2026-08-28.json, m = 23168: "gpu_answer_spread": 0.0008
gpu-simplepir-2026-08-28.txt,  line 35:   spread 0.1%
```

The string `379` does not appear anywhere under `measurements/`. The largest
run-to-run spread in the entire GPU answer sweep is 21.1%, at m = 4096:

| m | 2048 | 4096 | 8192 | 16384 | 23168 | 29312 | 32768 | 46336 |
|---|---|---|---|---|---|---|---|---|
| spread | 20.1% | **21.1%** | 14.9% | 2.6% | 0.1% | 0.1% | 0.1% | 0.0% |

The superseded sentence is the one that said 379%.

**This is entry 4 happening a second time, in the same section of the same
file.** Entry 4 is a cold JIT figure quoted from an observation that was never
committed, and its correction in `measurements/RESULTS.md` states the rule it
broke. The 379% claim broke the same rule, sat four paragraphs away from the
correction that named it, and survived anyway.

What survives the correction is the methodological point the sentence was making,
which is true and is supported by different rows: medians are reported with the
spread printed beside them, so a noisy row is visible rather than smoothed away.
The 20.1% and 21.1% spreads at the two smallest sizes are what make those rows
excludable, and `measurements/RESULTS.md` already uses them for exactly that. The
sentence did not need an invented row to make its point, which is the most
uncomfortable part of it.

## 10. The harness stated the direction of its own bias backwards

This one is in **published code**, and a reader can check it:
`crates/pir-bench/src/main.rs`, in the branch that runs when the knee cannot be
certified. It read:

> Consequence for the headline metric: a floor that is too HIGH makes the PIR
> multiplier too SMALL, which flatters the scheme. Treat any multiplier derived
> from this run as a lower bound on the true one.

**That is inverted.** The multiplier is `answer_seconds / scan_seconds`, and
`scan_seconds` is `bytes / floor`, so the multiplier scales *with* the floor.
Overstate the floor and the multiplier comes out too large, which makes the
scheme look more expensive than it is. Understate it and the multiplier comes
out too small, which is the direction that flatters.

The sweep's own data settles it without any argument from definitions. At
m = 2048 the 4.2 MB database sits inside the host's 12 MiB L3, so its floor is
measured in cache and is inflated:

| m | DB | floor | answer | multiplier |
|---|---|---|---|---|
| **2048** | **4.2 MB (in L3)** | **74.90 GB/s** | 11.95 GB/s | **6.23** |
| 16384 | 268.4 MB | 22.94 GB/s | 10.78 GB/s | 2.13 |
| 29312 | 859.2 MB | 22.87 GB/s | 10.70 GB/s | 2.14 |

The inflated floor produces the largest multiplier in the sweep, not the
smallest.

There is a second problem in the same sentence, independent of the direction. It
gives **one** consequence for **two** causes that bias opposite ways, and the two
verdicts printed immediately above it already concede that the ratio cannot
separate them ("either the working set is not leaving cache, or the scan loop is
slower than this machine's memory"). A working set that never left cache
overstates the floor and makes the multiplier an upper bound; a scan loop that
caps below memory bandwidth understates it and makes the multiplier a lower
bound. The text asserted the second for both.

The superseded sentence is the one quoted above. The replacement gives both
directions and says the ratio does not distinguish them.

**No committed figure moves.** That branch prints only when the knee ratio is
below 1.15, and every committed sweep passed it: 3.11x on the i5-11400, and
2.47x on the container this repository was assembled on. It was a wrong
diagnostic rather than a wrong measurement, which is precisely why nothing
caught it: it had never run.

## The pattern

**Of the seven found during the work, five were caught by an external number or
an external artifact.**

| | what was outside this work |
|---|---|
| 1 | what the host's instruction set can issue, checked against what the binary compiled to |
| 2 | YPIR's published CPU throughput for SimplePIR preprocessing (ePrint 2024/270) |
| 5 | SimplePIR's own Remark 5.1, and its Table 16 for the related DoublePIR correction |
| 6 | `owniai/VIA` @ `f65aa9d`, built and run |
| 7 | `google/hintless_pir` @ `812babf`, built and run |

The other two were caught by something that is also not self-consistency:
entry 3 by a physical impossibility (a card that is throttling does not get
faster), and entry 4 by a rule about what counts as a measurement, applied to a
sentence that had no artifact behind it.

**Self-consistency could not have caught any of them.** This is the part worth
stating plainly, because it is easy to assume that a repository with
bit-identity checks, decode checks and end-to-end verification is a repository
that catches its own errors. Those checks are real and they are described where
each figure is, in `measurements/RESULTS.md`. They constrain correctness, which
is whether the scheme computes what it claims to compute. Every one of the seven
errors above is a *cost* error, and every one of them passed correctness:

- The 3.55x build decoded, with a 69 sigma margin and a 0.3% spread. It was a
  clean measurement of a binary that was not the one the conclusion was about.
- The `axpy` hint kernel produced a bit-identical `H`. It was the same
  arithmetic, arranged so the memory system rather than the multiplier set the
  rate.
- The unnormalised drift check ran on a series of launches that were all
  correct.
- 102 ms was never checked against anything because there was nothing to check
  it against.
- The 3.9 MB recursion figure was arithmetic on this work's own numbers, and
  agreed with them.
- VIA and HintlessPIR were misfiled from published tables that are accurate at
  the record width they were measured at.

The practice that follows from this is the one the external write-ups state as a
rule: **check record width before believing any published figure, and run the
reference implementation rather than extrapolating its table.** It is the same
discipline in each case, and it is why four third-party implementations were
built from source and re-measured here rather than cited. Three of the findings
above exist only because that was done; two others exist because a figure from
outside was available to compare against; and the remaining two exist because a
number with no artifact behind it is treated here as a claim rather than as a
measurement.

The corresponding limit, stated rather than smoothed: this mechanism only finds
errors in quantities that somebody outside has also measured. Where no outside
number exists, this work reports against hardware ceilings instead and says so,
as the GPU hint arm does (57% to 71% of the `sm_61` emulated-`IMAD` ceiling
across the sweep, because no published work reports a throughput for
GPU-accelerated SimplePIR offline preprocessing). That is weaker than an
independent measurement, and nothing in this file should be read as claiming
otherwise.

### The three found on the way out are a different mechanism, and a gap

Entries 8, 9 and 10 were not caught by anything external and not caught by
running anything. They were caught by reading each written claim back against
the committed artifact or the source file it describes, once, at the point of
publication.

That is worth naming because of what it says about the first seven. Every
verification gate in this work constrains a *computation*: bit-identity between
arms, the `H.s == D.(A.s)` shortcut, decode through the materialised hint. None
of them constrains a *sentence*. So a heading could drift 2% from its own table
(entry 8), a spread could be quoted that no artifact contains (entry 9), and a
diagnostic could state its own bias backwards in shipped code (entry 10), and
every gate in the repository would still pass, because none of them reads prose
and entry 10's branch had never executed.

Two of those three are instances of a failure this work had already named. Entry
9 is entry 4 repeating, in the same section of the same file, four paragraphs
from the correction that states the rule it breaks. **Writing a rule down did
not enforce it.** The check that enforced it was mechanical and took one search:
look for the figure in `measurements/` and find nothing.

The gap this leaves is not closed. There is no automated check in this
repository that a number in prose matches the artifact it cites, and the three
entries above were found by hand. A reader should treat the prose here as
carefully checked at one point in time rather than as continuously verified, and
the raw artifacts under `measurements/` as the authority wherever the two
disagree.
