# ADR 0001: GPU before FPGA, and why this paper's scheme is not adopted

- Status: accepted (amended 2026-08-28, see *Amendment 1*)
- Date: 2026-08-28
- Deciders: the maintainers of the PIR prototype
- **Ported from an internal repository and renumbered.** This was ADR 0004
  there. Its own self-references are renumbered with it, and the companion ADR
  it cites is `docs/adr/0002-pascal-without-a-pascal-toolkit.md`. `SCOPE.md`
  section 5 records what the port removed and what it kept.
- **Amended 2026-08-28.** The scheme measured is SimplePIR, not BFV, and point 3
  is discharged. The original text is left standing and each superseded sentence
  is named in *Amendment 1*.

## Context

Where the PIR prototype is built was settled before this ADR, deliberately
saying nothing about *what it runs on*. An earlier internal planning document
answers that with FPGA: AWS F2 instances at up to 8 FPGAs each, HEAWS cited as
reference architecture, and a cost model scaling with queries per day. Hardware
acceleration is listed there as the third lever, after multiplicative depth and
ciphertext packing.

Tan et al., *GPU-Accelerated DPF-Based Private Information Retrieval for
Large-Scale Database* (TCHES 2026, vol. 2026 no. 3, pp. 933-957,
DOI 10.46586/tches.v2026.i3.933-957, CC-BY 4.0) was read to decide whether GPU
should displace FPGA as the first experiment. It should. But not for the reason
the abstract suggests, and the paper's actual scheme is one this deployment
cannot use.

### The scheme is two-server, and that is disqualifying

Section 2.2: *"We consider the scenario of one client C and two non-colluding
servers P0 and P1, where each server independently holds a complete database
t."* Section 5 makes the model explicit: semi-honest, where *"one of the two
servers may be corrupted."* Privacy holds because each server sees only one
additive share of the one-hot vector; an operator holding **both** shares
reconstructs the queried index exactly.

The deployment this work is about has a single operator. One organisation
running P0 and P1 is the colluding case, and the guarantee is not weakened by
it, it is absent. The property the construction is chosen to hold is that the
operator itself cannot learn which record was looked up, and under two-server
DPF with one operator that property does not hold at all. The argument is set
out at length, and generalised past DPF-PIR, in `docs/threat-model.md`.

This is not a gap in the paper. It is the model the internal planning document
already chose against, in its second paragraph: *"Unlike information-theoretic
PIR, which requires multiple non-colluding servers, single-server C-PIR relies
on the computational hardness of the underlying HE scheme's security
assumptions."* That choice was correct and this paper does not disturb it.

### The optimizations are DPF-shaped, so they do not port either

The paper's two contributions are a block-based full-domain evaluation of the
DPF binary tree (section 3, Table 1: 87.00x to 1.84x over Lam et al. on an
RTX 4090, falling as depth grows) and a distribution-aware access optimization
for the inner product, exploiting the fact that database access under a one-hot
share is probabilistic. Both are optimizations of an AES-style PRG tree
expansion.

Single-server BFV PIR is polynomial arithmetic: NTTs, modular reduction,
ciphertext packing. There is no binary tree to expand and no probabilistic
access pattern to coalesce. **None of the paper's kernels transfer.** What
transfers is the demonstration that the platform is worth the effort, and the
engineering discipline of the evaluation.

### What is genuinely useful in it

Three numbers, read against an earlier internal estimate that the
existence-check index is ~6.7M records rather than the ~350 GB of the full-text
corpus.

The paper's default configuration is 2048-bit (256 B) entries at batch size
B = 512. Its N = 2^23 row is 8.39M entries, the closest published point to an
existence index over ~6.7M CAP cases, at a per-record size comfortably above the
~100 B that earlier estimate assumed:

| At N = 2^23, entry 256 B, B = 512 | pirs/sec | Servers |
|---|---|---|
| Tan et al. Non-pipeline, RTX 4090 | 4,111 | 2 |
| Tan et al. Pipeline, RTX 4090 | 795 | 2 |
| Kales et al., re-optimized CPU (AVX-512, 32 threads) | 300 | 2 |

And two single-server reference points the paper carries for comparison:
SimplePIR at a 1G database, 3,611 pirs/sec on CPU with preprocessing (Table 3,
where the *Servers* column reads 1); and Lehmkuhl, Henzinger, Corrigan-Gibbs,
*Distributional Private Information Retrieval* (USENIX Security 2025), a non-DPF
lattice scheme, at 550 pirs/sec on a V100 at N = 2^24 (Table 2, marked as
reported in that paper rather than re-measured there).

Set any of these against the **declared reference workload** defined in
`SCOPE.md`: an arrival rate of **lambda = 0.06 queries/second**, one arrival
every roughly 16.7 seconds. That is a stated evaluation parameter, in the same
sense that the database size is a stated input, and not a measurement of
anyone's traffic.

That is four to five orders of magnitude below the slowest row in the table,
including the CPU row.

### The batching caveat, which cuts the other way

Every headline throughput above is measured at B = 512, and the paper says
plainly that this is the point *"at which GPU utilization is saturated."*
Batching amortizes one pass over the database across many concurrent queries.
At lambda = 0.06 queries/second there is no batch to form: queries arrive alone,
and the number that matters is single-query latency, which these tables do not
report for the full PIR (Table 1 reports it for the DPF expansion alone, 4.28 ms
at layer 23 on the RTX 4090, for a batch of 512 trees).

So the paper simultaneously suggests that PIR compute is nowhere near the
binding constraint at the reference workload, and that the metric its tables
optimize is not the metric this deployment is judged on. Both are worth writing
down before anything is rented.

## Decision

**The prototype's first hardware experiment is a rented commodity GPU, not an
FPGA. ~~The scheme stays single-server BFV.~~ The scheme stays single-server
lattice PIR: SimplePIR, not BFV; amended 2026-08-28, see *Amendment 1*. Tan et
al. is used as reference engineering, not as a scheme to port.**

Concretely:

1. **DPF-PIR is not adopted, and the reason recorded is the threat model, not
   performance.** Its throughput is not in question and is not the point. The
   full reasoning is in `docs/threat-model.md`.

2. **FPGA is deferred, not rejected.** The internal planning document's AWS F2 /
   HEAWS path stays on the table, behind the two levers that document itself
   ranks above it. Renting an F2 instance before a CPU baseline exists would be
   buying acceleration for an unmeasured workload.

3. **A CPU baseline is measured first, and may end the hardware question.** If
   single-server BFV over a ~700 MB index clears single-query latency on a CPU
   at the reference workload's arrival rate, there is no GPU decision to make
   yet. This is the cheapest experiment available and it is ordered first.

4. **Every benchmark reports B = 1 alongside any batched figure.** A throughput
   number without an arrival-rate assumption beside it is not usable evidence
   for this workload, and the tables that prompted this ADR are the reason the
   rule is written down.

5. **The measurement targets the existence-check index, not the ~350 GB
   corpus.** The existence check first, per the ordering fixed before this ADR;
   retrieval of opinion text at a cited page is the separate, expensive problem
   and gets its own numbers.

6. **The artifact is read, not linked against, until its licence is checked.**
   The source is at `https://github.com/Crypto-Lab-WHU/GPU-PIR`. The paper is
   CC-BY 4.0; the code's terms are stated nowhere in the paper. Given this
   work's posture on third-party terms, recorded in `LICENSES.md`, an unread
   licence is not a dependency.

## Consequences

**What this buys.**

- The privacy property stays true by construction. A scheme that requires a
  second non-colluding operator cannot be adopted by accident on performance
  grounds, because the reason it is refused is now written down.
- The cheapest experiment runs first. The internal cost model that motivated
  FPGA is derived from a ~350 GB database saturating an FPGA, and every input to
  it is now something the prototype measures rather than assumes.
- A commodity GPU is rentable by the hour from any provider, needs no bitstream
  toolchain, and has a public artifact to compare against. An F2 instance has
  none of those properties.

**What this gives up.**

- **The best-performing construction in the literature is off the table**, and
  it is off by a wide margin: 4,111 pirs/sec at N = 2^23 against a single-server
  lattice scheme's 550 at N = 2^24 on comparable hardware. Single-server PIR is
  simply more expensive than two-server PIR, and this decision pays that
  difference deliberately.
- **The CPU-first ordering may waste a week** if the baseline turns out to be
  hopeless. That is the cheap direction to be wrong in.
- **No conclusion is drawn about BFV on GPU specifically.** This paper contains
  no BFV measurement at all; the lattice data point is a different scheme by
  different authors, carried second-hand. The prototype's own numbers are the
  only ones that will settle it.

**What would change this decision.**

- **A genuinely independent second operator.** If a law school, a legal-aid
  organisation, or a nonprofit with an aligned mission would run P1 under an
  agreement not to collude, two-server DPF becomes available and is roughly an
  order of magnitude cheaper. The obstacle is organisational, not technical, and
  it is the single largest cost lever identified so far. It also imports a new
  failure mode: the guarantee becomes contractual as well as computational, and
  a claim resting on someone else's conduct has to be described that way
  wherever it is stated.
- **The CPU baseline missing latency targets.** Then GPU, then, only if GPU also
  misses at a plausible arrival rate, FPGA.
- **Retrieval of opinion text becoming in scope.** That is the ~350 GB problem
  the original cost model describes, and it may reach FPGA economics directly,
  without passing through the existence check's conclusions.

## Alternatives considered

- **Adopt DPF-PIR and run both servers ourselves.** Rejected, and worth naming
  explicitly because it is the tempting misreading of the paper: the numbers are
  excellent and the code is public. It provides no privacy against the operator,
  which is the only adversary this work's claims are about.

- **Adopt DPF-PIR with the second server run by a subsidiary or a separate
  cloud account.** Rejected as the same thing wearing a hat. Non-collusion has
  to be a property of who controls the keys, not of the invoice.

- **Go straight to FPGA, per the internal planning document.** Rejected as
  premature rather than wrong. Every figure supporting it assumes a ~350 GB
  database and a saturated pipeline; the existence-check-first ordering puts the
  first target three orders of magnitude below that, and this paper's CPU row
  suggests the workload may not need acceleration at all at the reference
  workload's arrival rate.

- **Port the paper's GPU optimizations to BFV.** Rejected as a category error.
  Block-based binary-tree expansion and probabilistic-access coalescing are
  properties of DPF evaluation; BFV has neither structure.

- **Wait for a single-server GPU PIR paper.** Rejected. Lehmkuhl et al. already
  exists and is cited here; the prototype's own CPU baseline is cheaper than
  another literature review and answers a question about *this* index.

## Amendment 1, 2026-08-28: SimplePIR, not BFV; and point 3 answered

The house rule this work is written under is that any correction to an ADR
"lands as an amendment naming the superseded sentence". This is that amendment.
Nothing above is deleted.

### (a) The five superseded sentences

The scheme this ADR was written around was BFV. The scheme built and measured is
**SimplePIR** (Regev/LWE), which the internal planning document had already
named as the first single-server construction to measure. Substitute it in each
of these:

| Where | Superseded sentence | Reads instead |
|---|---|---|
| Decision line | "The scheme stays single-server BFV." | single-server SimplePIR (Regev/LWE) |
| *The optimizations are DPF-shaped* | "Single-server BFV PIR is polynomial arithmetic: NTTs, modular reduction, ciphertext packing." | SimplePIR's server side is a dense byte-times-word matrix-vector product; there are no NTTs and no packing |
| Decision point 3 | "If single-server BFV over a ~700 MB index clears single-query latency..." | SimplePIR over the 859 MB index, and it does; see (c) |
| *What is genuinely useful in it* | "**No conclusion is drawn about BFV on GPU specifically.**" | none is drawn now either. The GPU figure in `measurements/RESULTS.md` is SimplePIR's, on a GTX 1070, and says nothing about BFV |
| *Alternatives considered* | "**Port the paper's GPU optimizations to BFV.**" | ...to SimplePIR |

### (b) The reasoning survives, and is strengthened

Substituting SimplePIR does **not** weaken this ADR's central argument that none
of the paper's kernels transfer. It strengthens it. BFV at least shares
polynomial arithmetic with some accelerated schemes; SimplePIR shares nothing
with DPF: no binary tree to expand, no probabilistic access pattern to coalesce,
just a dense product every entry of which is touched unconditionally. The
"category error" verdict in *Alternatives considered* holds verbatim under the
substitution.

**Point 1 is untouched.** DPF-PIR is still refused, still on threat model, and
the reasoning is unchanged: privacy there comes from two non-colluding servers,
and one operator holding both shares recovers the queried index exactly. That
constraint, single server, one operator, is what this ADR was actually about,
and it is scheme-independent.

### (c) Point 3 is discharged

Point 3 said a CPU baseline "may end the hardware question". Quoted:

> **A CPU baseline is measured first, and may end the hardware question.** If
> single-server BFV over a ~700 MB index clears single-query latency on a CPU at
> the reference workload's arrival rate, there is no GPU decision to make yet.

**It does end it.** At m = 29312 (859.2 MB, 6,712,448 records), B = 1:

| | latency | multiplier vs its own linear-scan floor |
|---|---|---|
| 1 core, i5-11400 | 80.3 ms | 2.14x |
| 12 threads | 28.0 ms | 1.0x (indistinguishable from the floor) |
| GTX 1070 | 4.02 ms | 1.01x |

The declared reference workload's arrival rate is lambda = 0.06 queries/second,
so **one desktop core has ~208x of headroom**. No FPGA, and no GPU either. That
ratio is against lambda and moves with lambda and with nothing else; the
latencies themselves are measurements and move with neither. The GPU arm retains
its value as a *bound* rather than a purchase: the multiplier converging to 1.01
on hardware fast enough to make the scheme purely memory-bound is the strongest
available evidence that the 2.14 measured on one core is a property of that core
and not of SimplePIR.

Artifacts: `measurements/cpu-simplepir-2026-08-28.json`,
`measurements/gpu-simplepir-2026-08-28.json`, written up as the decision gate
(Phase 5) in `measurements/RESULTS.md`.

Point 2 stands as written: FPGA remains deferred rather than rejected, and the
trigger named there, retrieval of opinion text becoming in scope, which is the
~350 GB problem and not this index, has not fired.

### (d) The caveat that kept the discharge honest, and what has since replaced it

As first written on 2026-08-28, this part read that the index size the discharge
rests on, 859 MB, came from a **1,862-record sample that is not committed**, and
that this amendment therefore did not touch the earlier internal ~700 MB index
estimate. The superseded sentences are the one that called the sample "not
committed" and the one that left the ~700 MB estimate standing.

**The sample is now committed, and it is 100,394 records**, from 993 CAP volumes
with 0 volume misses, stratified across {federal, state} x {pre-1950, 1950-2000,
post-2000}, metadata only. It is `measurements/record-size.json`, produced by
`tools/fetch_cap.py`, and written up as Phase 1 in `measurements/RESULTS.md`.
Against the preliminary 1,862-record figures: case name 28.79 B against 29.2 B,
payload 39.75 B against 43.2 B, and 99.970% of records intact at a 128 B entry
width against 99.8%. The key figure moved from 12.0 B to 8.95 B, and that is a
definition difference rather than a disagreement: 12.0 B is the length of the
citation string with its spaces and punctuation, 8.95 B is the normalised key
that the index actually stores, and the gap is exactly the separators.

**m = 29312 did not move.** At 128 B/entry and 6.7M records the database is
857.6 MB, so m is 29,285 before rounding and 29,312 after rounding up to a
multiple of 128: 859.2 MB, holding 6,712,448 records. That is the value every
measurement in `measurements/RESULTS.md` already used, so nothing needed
re-running, and this section's discharge of point 3, which was explicitly
conditional on the size holding, stands as written.

What the wider sample does supersede is the ~700 MB figure. The earlier internal
estimate of ~700 MB for this index is superseded by the measured 859.2 MB, and
the *References* below is amended to say so rather than to keep citing the
estimate as live.

The margin statement is unchanged and is the reason the conditional was
survivable in the first place: point 3 is answered at the *measured* size, and
it has roughly two orders of magnitude of margin to spend before the answer
could change.

## References

- Tan, Zeng, Feng, Peng, Wang, He, *GPU-Accelerated DPF-Based Private
  Information Retrieval for Large-Scale Database*, TCHES 2026(3):933-957,
  DOI 10.46586/tches.v2026.i3.933-957, CC-BY 4.0. Section 2.2 (two
  non-colluding servers), section 5 (semi-honest, one corrupted server), Table 1
  (full-domain evaluation latency), Table 2 (batch throughput and GMEM), Table 3
  (comparison with non-GPU schemes, including the *Servers* column), section 6
  (V100 32 GB, RTX 4090 24 GB, Xeon Gold 5418Y; artifact URL).
- Lehmkuhl, Henzinger, Corrigan-Gibbs, *Distributional Private Information
  Retrieval*, USENIX Security 2025, 3377-3396. Cited in Table 2 as [LHC25].
- Henzinger et al., SimplePIR, cited in Table 3 as the fastest known
  single-server PIR.
- `docs/threat-model.md`, where the single-operator constraint that decides
  point 1 is stated on its own, generalised past DPF-PIR, and given the two
  rejected workarounds in full.
- `SCOPE.md` section 3, where the declared reference workload this ADR compares
  against (lambda = 0.06 queries/second, Q = 157 queries per client per rebuild
  period) is defined once, as a stated evaluation parameter rather than a
  measurement.
- The ~700 MB index estimate this ADR originally read the paper's tables
  against was an earlier internal figure. It is superseded by the measured
  859.2 MB at m = 29312 and 6,712,448 records: `measurements/record-size.json`
  and Phase 1 of `measurements/RESULTS.md`.
- `docs/adr/0002-pascal-without-a-pascal-toolkit.md`, the toolchain consequence
  of running the GPU arm on a Pascal card.
