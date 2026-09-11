# The third hint-free scheme, run on this machine

**2026-09-04.** `measurements/external-2026-08-29-ypir-via.md` ran two of the
three hint-free schemes `measurements/RESULTS.md` named and left one open:

> **HintlessPIR is still unrun**, and the reason it is ranked last is weaker
> than it looks. Its published per-query download of 3,080 KB is 130x VIA's,
> but that figure is measured at one-byte records, which is exactly the
> unchecked assumption that had VIA filed next to YPIR two paragraphs of this
> repository ago. Ranking it last is a scheduling decision, not a finding.

This runs it, at this repository's real shape: 6,712,448 records, 128 bytes
each. The instruction that closed the YPIR/VIA gap applies again, measure at
the real record width, don't extrapolate the paper's one-byte table.

Artifact: `external-2026-09-04-hintlesspir.txt`. `google/hintless_pir` is
Apache-2.0; see `LICENSES.md` for it and the four further repositories its
build pulls as source. Not vendored, not linked against, not in any
manifest, the same posture as YPIR and VIA.

The same two conventions the YPIR/VIA writeup depends on hold here. Where a
comparison below uses a query volume or an arrival rate, both come from the
**declared reference workload**: lambda = 0.06 queries/second and Q = 157
queries per client per rebuild period. Those are stated evaluation parameters
rather than measurements, and `SCOPE.md` section 3 defines them and is the only
place they are defined. And the SimplePIR implementation the "ours" rows come
from is not published here (`SCOPE.md` section 2); it is called *the
implementation under measurement*, and no path inside it is cited.

## What was run

Built via Bazel from the upstream source at `812babf` (2026-07-23), with one
addition: `hintless_simplepir/measure_main.cc`, a small binary that is not
part of upstream and calls only the library's public `Server`/`Client`/
`Database` API, no scheme code was touched. The database is filled with
random records (as the shipped benchmark itself does); this is a cost
measurement, not a security artefact, matching the posture the implementation
under measurement takes about itself (`SCOPE.md` section 4).

**Host, as `external-2026-09-04-hintlesspir.txt` records it:** an Intel Xeon at
2.10 GHz, 4 vCPU, 15 GiB RAM. That is not the desktop i5-11400 that produced
the "ours" rows below, so the per-core time comparison in the latency section
crosses hosts. It is kept, and stated as cross-host rather than smoothed,
because the gap it reports is three orders of magnitude and the clock
difference between these two hosts is a factor of 1.24 (2.60 GHz against
2.10 GHz, as `measurements/RESULTS.md` and
`external-2026-09-04-hintlesspir.txt` record them).

**The shape.** HintlessPIR parameterizes the database as `db_rows x db_cols`
matrices, one per shard, where `num_shards = record_bit_size /
lwe_plaintext_bit_size` (128 here, at the shipped default of 8-bit
plaintexts). The client's request holds one LWE ciphertext of length
`db_cols`; the response holds `num_shards` LWE ciphertexts of length
`db_rows`, plus the LinPIR ciphertexts that carry the outsourced hint
computation. So request bytes scale with `db_cols` and the response's LWE
part scales with `num_shards x db_rows`. Minimizing `db_cols + num_shards x
db_rows` subject to `db_rows x db_cols = N` gives `db_rows = sqrt(N /
num_shards)`. At `N = 6,712,448` and `num_shards = 128` that is **exactly
229**, giving `db_cols = 29312`, which is `m` in the implementation under
measurement. Not a coincidence: both numbers are the same balance point for a
database this shape. Every other parameter (`lwe_secret_dim`, the LinPIR RLWE
parameters) is left at the shipped benchmark's default; only the shape and the
record width change.

## Measured

**Gated on a correctness check, per this repository's own rule**
(`docs/adr/0002-pascal-without-a-pascal-toolkit.md`, point 6, cited again in
the YPIR/VIA writeup): `client->RecoverRecord` decoded against the database's
own stored value at the same index, and it matched.

| | |
|---|---|
| database | 6,712,448 records x 128 bytes = 859.2 MB (same as every other measurement here) |
| public params (offline, once per DB version) | **264 bytes** |
| server preprocessing | 6,092.8 s = 1.69 core-hours, single core |
| request (up, per query) | 513,535 bytes (501.5 KiB) |
| response (down, per query) | 23,751,495 bytes (22.65 MiB) |
| server `HandleRequest` | **113.65 s**, averaged over 5 reps, single core |
| client request-gen / recover | 12.8 s / 2.6 s |

The 264-byte public-params figure is the entire point of this scheme: it
replaces this repository's 120.1 MB hint, and it does not grow with the
database. That part of the hint-free claim holds, exactly as it does for
YPIR and VIA.

## The record-width check, again

**The published 3,080 KB per-query download figure is for one-byte records**
(`num_shards = 1`). At our 128 bytes (`num_shards = 128`) the measured
response is 23.75 MB: **7.7x the published figure, not the 128x** a naive
scaling would predict (3,080 KB x 128 is about 385 MB). Not because the scheme
is cheaper at width than expected, but because the published number was
measured at a different `db_rows`/`db_cols` shape too, and this repository's
own shape (`db_rows = 229`) keeps the LWE part of the response small; the
LinPIR part, which does scale with `num_shards`, is what the 23.75 MB is
actually made of. The honest number is the one measured here, not either
extrapolation.

Per client, per rebuild period, at the declared reference workload of Q = 157
queries, at 128-byte records, same methodology as the YPIR/VIA table:

| | database | offline | Q = 157 queries | **total** | vs ours |
|---|---|---|---|---|---|
| **VIA**, `LOG_COL = 11` | 1 GiB | 0 | 93.2 MB | **93.2 MB** | 1.7x better |
| **ours (SimplePIR)** | 859.2 MB | 120.1 MB hint | 36.8 MB | **156.9 MB** | |
| **HintlessPIR** | 859.2 MB | 0.0003 MB | 3,809.6 MB | **3,809.6 MB** | **24.3x worse** |
| **YPIR** | 859.2 MB | 0 | 17,656 MB | **17,656 MB** | 113x worse |

HintlessPIR falls behind our hint after **5.0 queries** (120.1 MB divided by
(24.265 MB/query minus 0.2344 MB/query), the same crossover arithmetic as the
YPIR/VIA table). Like the other crossovers, it is a property of the scheme
rather than of the workload; Q only decides which side of it a reader is
standing on. It lands between VIA and YPIR: eliminating the hint entirely
costs far less bandwidth than YPIR's 128-separate-queries workaround, but far
more than the 120.1 MB hint this repository already pays once per rebuild
period.

## What this scheme costs that the other two don't: latency

VIA traded bandwidth for server time and stayed inside this repository's
headroom (11.7x slower per byte, against 208x of measured slack). HintlessPIR
does not:

| | database | time/query, 1 core | per MB |
|---|---|---|---|
| ours | 859.2 MB | 80.3 ms | 0.093 ms |
| HintlessPIR | 859.2 MB (logical, across all 128 shards) | **113,651 ms** | 132.3 ms |

**HintlessPIR is ~1,415x slower per query than SimplePIR, per core.** The
declared reference workload puts arrivals at lambda = 0.06 queries/second, one
query every roughly 16.7 s, against which a single core has 208x of headroom
(`SCOPE.md` section 3, `measurements/RESULTS.md`). HintlessPIR's 113.65 s per
query is **6.8x that entire inter-arrival budget**, on the same core count.
Unlike VIA, this is not a cost the existing headroom absorbs: a single core
cannot keep up with even the reference arrival rate, let alone bursts, and the
measurement never had to reach for a batching or concurrency argument to
reveal that, it shows up in the `HandleRequest` reps directly. Whether a
production-grade, multi-threaded implementation of the LinPIR step would close
most of this gap is not something this run answers; the reference binary
issues no threads of its own, and this number describes what was actually run,
not what the construction could do under further engineering.

## What this changes

1. **HintlessPIR is measured, not ranked by a placeholder.** The instruction
   that closed the YPIR/VIA gap applies here too, and the correction runs the
   same direction as YPIR's: the paper's headline figures are real at one-byte
   records and do not survive 128-byte ones, though far less catastrophically
   than YPIR's 128-separate-queries structural wall.
2. **On total client bytes, HintlessPIR is out.** It sits between VIA (still
   the only scheme so far that beats this repository's own hint) and YPIR
   (still disqualified outright), worse than what this repository already
   runs by 24x, at our record width and at the reference query volume.
3. **On server time, HintlessPIR is out more decisively than the bandwidth
   number alone suggests.** At ~1,415x the per-query cost of SimplePIR on one
   core, it does not clear the reference arrival rate the way VIA's
   slower-but-affordable answer time does. This is a second, independent
   reason to prefer VIA over HintlessPIR if a hint-free scheme is ever
   adopted, not a restatement of the bandwidth finding.
4. **The open item named in the previous writeup is unchanged by this run:**
   "does VIA's throughput survive actually reading the database, and is 8x
   RAM acceptable" is still the question that decides whether any hint-free
   scheme replaces what this repository already measures.
