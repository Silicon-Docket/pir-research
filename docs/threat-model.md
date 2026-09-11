# Threat model

**The adversary is the operator of the service.** Everything else in this
document follows from that sentence.

Before any of it: **nothing in this repository is a security artefact.** The
threat model below describes what the *construction* is chosen to resist. It is
not a claim that this implementation resists it, and section 7 says exactly why
not. A reader who takes only one thing from this file should take section 7.

---

## 1. The adversary

The deployment this work is measured for has **one operator**. That operator
runs the server, holds its keys, owns the machine the server process runs on,
and can read anything that process can read. The question a retrieval scheme has
to answer here is narrow and it is the whole question:

> Can the party running the server learn which record a client retrieved?

Nothing weaker is interesting. An adversary who has to break into the server to
see the query is a smaller adversary than the one who already runs it, and a
scheme that is private against the first but not the second answers a question
nobody asked. The adversary is assumed to be **semi-honest**: it follows the
protocol and reads everything that passes through it. It is also assumed to
retain what it sees, so a scheme that hides one query but leaks over a sequence
of them has not solved the problem either.

This is why the study is of **single-server computational PIR** and not of the
faster things next to it in the literature. Single-server PIR places the
guarantee on a hardness assumption (here Regev/LWE) rather than on the
behaviour of a second party, and the class of adversaries it covers includes the
operator by construction.

## 2. What the construction is chosen to hide, and what it does not

Stated as a boundary rather than as a list of features, because the parts
outside the boundary are the parts a reader is most likely to assume are inside
it.

**Inside.** Which record of the index a given query retrieves. The server's
online work is a dense matrix-vector product, every entry of which is touched
unconditionally: there is no data-dependent branch or data-dependent address on
the database side, and the cost of that is the reason the headline metric in
`README.md` is a multiplier against a linear scan rather than an absolute
throughput. Touching every byte on every query *is* the guarantee, not an
implementation detail. The measured per-query wire traffic is fixed at 117.2 KB
up and 117.2 KB down regardless of which record is fetched
(`measurements/RESULTS.md`), and the 120.1 MB hint is client-independent: one
object per database version, identical for every client.

**Outside, and not analysed here.** That a query occurred at all, and when.
Network-level metadata, including the identity of the client and the timing and
count of its requests. Integrity: a single-server PIR answer of this kind is not
authenticated, and a server that returns a wrong answer is a failure mode this
work does not address. Availability. Client-side compromise. And anything to do
with how the index was built or what is in it.

## 3. Why two-server schemes are excluded

The fastest constructions in this literature are two-server. They are excluded
here, and the reason recorded is the threat model rather than performance.

The model is stated precisely in the paper that prompted the decision. Tan,
Zeng, Feng, Peng, Wang, He, *GPU-Accelerated DPF-Based Private Information
Retrieval for Large-Scale Database*, TCHES 2026(3):933-957, DOI
10.46586/tches.v2026.i3.933-957, CC-BY 4.0.

- **Section 2.2:** *"We consider the scenario of one client C and two
  non-colluding servers P0 and P1, where each server independently holds a
  complete database t."*
- **Section 5** makes the adversary explicit: semi-honest, where *"one of the
  two servers may be corrupted."*

Privacy there holds because each server sees only one additive share of the
one-hot vector. An operator holding **both** shares reconstructs the queried
index exactly.

A single-operator deployment is therefore not a deployment in which the
non-collusion assumption is strained. It **is** the colluding case, the one the
model explicitly places outside its guarantee. The guarantee is not weakened
there, it is absent.

This is not a defect in that paper. The paper states its model plainly in the
second section and again in the fifth, and its results are correct in the model
it states. The mismatch is entirely on the deployment side, and the reason it is
written down at this length is that the numbers are extremely good and the
artifact is public, so the temptation to reach for them is real. Anyone who
reads a benchmark table, sees a two-server construction winning by an order of
magnitude and proposes it is proposing a number that does not apply to this
adversary.

## 4. The two workarounds, both rejected

Named explicitly because they are the tempting misreadings, and because a
rejection that is not written down gets re-proposed.

**Run both servers yourself.** Rejected. The numbers are excellent and the code
is public, and it provides no privacy against the operator, which is the only
adversary this threat model is about. Running P0 and P1 in two processes, two
machines, or two regions changes nothing: the party that can read one share can
read the other.

**Run the second server from a subsidiary or a separate cloud account.**
Rejected as the same thing wearing a hat. Non-collusion has to be a property of
**who controls the keys**, not of the invoice. A separate billing relationship,
a separate legal entity under common control, or a separate tenancy inside the
same administrative boundary all leave the same party able to obtain both
shares, and the assumption the scheme's proof relies on is about that party, not
about the paperwork.

Both rejections generalise past DPF-PIR to anything whose privacy rests on two
parties not colluding: secret-shared retrieval, two-party SMPC, and
information-theoretic PIR with multiple replicas.

## 5. What would change this

**A genuinely independent second operator.** If an organisation that is not this
one, a law school, a legal-aid organisation, or a nonprofit with an aligned
mission, would run the second server under an agreement not to collude, then
two-server constructions become available and are roughly an order of magnitude
cheaper. The obstacle is organisational rather than technical, and it is the
single largest cost lever identified in this work.

It also imports a new failure mode, and the honest description of it is the
point of this paragraph: **the guarantee becomes contractual as well as
computational.** A claim that rests on another party's conduct is a different
kind of claim from one that rests on a hardness assumption. It can be broken by
a subpoena, an acquisition, an insider, or a quiet change of hosting
arrangements, none of which are events a security proof reasons about. If that
route is ever taken, the guarantee has to be described that way wherever it is
stated, not described as though it were the computational one.

**A better single-server construction.** That is a measurement question rather
than a threat-model one, and it is what
`measurements/external-2026-08-29-ypir-via.md` and
`measurements/external-2026-09-04-hintlesspir.md` exist to answer. Every scheme
evaluated there is single-server, which is why it was eligible for evaluation at
all.

## 6. The cost of this choice, stated rather than hidden

Single-server PIR is simply more expensive than two-server PIR, and this
decision pays that difference deliberately.

The comparison, as carried in Tan et al. All rows are that paper's figures at
its default configuration of 256 B entries and batch size B = 512, on the
hardware its section 6 names (RTX 4090 24 GB, V100 32 GB, Xeon Gold 5418Y):

| Scheme | pirs/sec | N | Hardware | Servers |
|---|---|---|---|---|
| Tan et al., non-pipeline | 4,111 | 2^23 | RTX 4090 | 2 |
| Tan et al., pipeline | 795 | 2^23 | RTX 4090 | 2 |
| Kales et al., re-optimized CPU (AVX-512, 32 threads) | 300 | 2^23 | CPU | 2 |
| SimplePIR with preprocessing (Table 3) | 3,611 | 1 GB database | CPU | 1 |
| Lehmkuhl, Henzinger, Corrigan-Gibbs (Table 2, marked as reported rather than re-measured) | 550 | 2^24 | V100 | 1 |

The headline contrast is the first row against the last: **4,111 pirs/sec at
N = 2^23 for two-server DPF-PIR on an RTX 4090, against 550 at N = 2^24 for a
single-server lattice scheme on a V100** (Lehmkuhl, Henzinger, Corrigan-Gibbs,
*Distributional Private Information Retrieval*, USENIX Security 2025,
3377-3396). The two rows differ in database size and in GPU generation as well
as in scheme, so this is an indication of the size of the gap and not a
controlled comparison of it. It is quoted in that form because it is how the gap
appears in the literature, and because an uncontrolled comparison that favours
the option being rejected is the safe direction for it to be uncontrolled in.

Two caveats belong with those numbers, and they cut in opposite directions.

**The batching caveat.** Every throughput figure above is measured at B = 512,
which the paper says plainly is the point *"at which GPU utilization is
saturated."* Batching amortizes one pass over the database across many
concurrent queries. Against the declared reference workload defined in
`SCOPE.md` (arrival rate lambda = 0.06 queries/second, Q = 157 queries per
client per rebuild period, both stated evaluation parameters and not
measurements), no batch forms: queries arrive alone, and the number that matters
is single-query latency, which these tables do not report for the full PIR.
Table 1 reports it for the DPF tree expansion alone, 4.28 ms at layer 23 on the
RTX 4090, for a batch of 512 trees. So the same tables that show the gap are
optimizing a metric this workload does not have, and the gap at B = 1 is not
established by them.

**The cost is affordable at this workload, which is why paying it was
available.** Our own single-server measurement, at m = 29312 (859.2 MB,
6,712,448 records), B = 1, on an i5-11400 at 2.60 GHz (6C/12T, 12 MiB L3),
answers a query in 80.3 ms on one core, which is roughly 208x headroom over
lambda. Twelve threads give roughly 596x and the GTX 1070 roughly 4,150x. Those
are ratios against lambda, so they move with lambda and with nothing else. A
workload ten times denser divides them by ten and the ordering of the schemes
does not change. The point is that at the declared reference workload the more
expensive construction is not the binding constraint, so choosing it costs
throughput that was not going to be used rather than costing service.

This is what makes the decision cheap rather than making it right. If the
arrival rate were four orders of magnitude higher the threat-model argument
above would be unchanged and the bill would not be, and the honest statement of
the trade-off is that it was made where it happened to be affordable.

## 7. What is not claimed

This section is the one that bounds everything above.

**Nothing in this repository is a security artefact.**

- **The error sampler is a centred binomial.** The parameter set the
  measurements ran at is n = 1024, p = 256, q = 2^32, centred binomial error
  with k = 82 (sigma approximately 6.40), recorded with the runs in
  `measurements/RESULTS.md`.
- **The secret comes from a general-purpose PRNG**, not from a cryptographic
  one seeded as a cryptographic one.
- **No parameter set has been checked against a lattice estimator.** That is the
  top open item in this work. A back-of-envelope root-Hermite estimate is not an
  estimator run, and no security level in bits is claimed here, stated, or
  implied by any figure in this repository.
- **Neither the CPU loop nor the GPU kernel is constant-time.** Neither is the
  third-party crate's sampler measured in
  `measurements/crate-comparison-2026-08-28.md`. None of them claims to be.
- **The decode margin is a correctness property, not a security one.** It is
  asserted at 41 sigma to 196 sigma across the sweep because a shrinking margin
  is where a wrong answer would come from. It says nothing about how hard the
  underlying problem is, and it must not be read as though it did.

The construction described in sections 1 to 6 is chosen to resist an operator
who wants to know which record was retrieved. **The implementation under
measurement is not offered as evidence that it does.** It is a prototype built
to produce timings, its corpus is synthetic, and it is not published here at all
(`SCOPE.md` section 2 is the record of that boundary). No privacy claim is made
to anyone on the strength of any of it.

## Provenance

This document lifts the threat-model reasoning out of
`docs/adr/0001-gpu-before-fpga-for-pir.md` so that it stands on its own, because
that reasoning is the constraint the whole study sits under and reading a
hardware decision to find it is the wrong way round. The ADR keeps the hardware
decision and the rest of its argument.

One detail of that ADR's history is worth carrying across, in the house style of
naming a superseded sentence rather than deleting it. The ADR was written around
a different single-server scheme: its decision line read *"The scheme stays
single-server BFV"*, and it was later amended to single-server SimplePIR
(Regev/LWE), which is what was built and measured. **The threat-model argument
was untouched by that amendment**, and the amendment says so: privacy under
two-server DPF-PIR comes from two non-colluding servers, one operator holding
both shares recovers the queried index exactly, and that constraint is a
property of the deployment rather than of the scheme chosen inside it. A
threat-model conclusion that survives a change of construction is worth more
than one that does not, which is the reason for mentioning it rather than
quietly presenting the current text as though it had always read that way.

`docs/adr/0002-pascal-without-a-pascal-toolkit.md` point 5 applies the same
constraint to the GPU arm: having a GPU does not reopen the two-server question,
because the speed of those schemes comes from splitting trust across two
operators and not from the processor underneath them.

## References

- Tan, Zeng, Feng, Peng, Wang, He, *GPU-Accelerated DPF-Based Private
  Information Retrieval for Large-Scale Database*, TCHES 2026(3):933-957, DOI
  10.46586/tches.v2026.i3.933-957, CC-BY 4.0. Section 2.2 (one client, two
  non-colluding servers, each holding a complete database), section 5
  (semi-honest, one of the two servers may be corrupted), Table 1 (full-domain
  evaluation latency), Table 2 (batch throughput), Table 3 (comparison with
  non-GPU schemes, including the *Servers* column), section 6 (V100 32 GB,
  RTX 4090 24 GB, Xeon Gold 5418Y, and the artifact URL).
- The artifact accompanying that paper, `github.com/Crypto-Lab-WHU/GPU-PIR`, was
  read for technique only. Its licence is stated nowhere in the paper, it is not
  cloned, vendored or linked against, and the posture is recorded in
  `LICENSES.md`: an unread licence is not a dependency.
- Lehmkuhl, Henzinger, Corrigan-Gibbs, *Distributional Private Information
  Retrieval*, USENIX Security 2025, 3377-3396. The single-server GPU data point
  above, carried second-hand through Tan et al.'s Table 2 rather than
  re-measured here.
- `docs/adr/0001-gpu-before-fpga-for-pir.md`, the hardware decision this
  reasoning was originally recorded inside.
- `docs/adr/0002-pascal-without-a-pascal-toolkit.md`, which restates the
  constraint for the GPU arm.
- `SCOPE.md`, for the declared reference workload, what is published, and what
  is held back.
- `measurements/RESULTS.md`, for every figure quoted here with its host, its
  sample definition and its verification checks.
- `docs/methodology.md`, for the B = 1 rule and the cache-cliff sweep that the
  batching caveat in section 6 is an instance of.
