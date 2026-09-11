# Two hint-free schemes, run on this machine

**2026-08-29.** `measurements/RESULTS.md` closed Phase 4c with hint
*distribution* as the entire cost of this design, listed three hint-free schemes
as the way to attack it, and then said the thing that mattered:

> **Check record width before believing any of the figures above.** Every one is
> measured at 1-bit or 1-byte records, and 128-byte records are exactly where
> DoublePIR collapsed. [...] The honest way to settle it is to run the reference
> implementations at `m = 29312` with 128-byte records, not to extrapolate a
> table.

Two of the three were run. The instruction was right, and it cut both ways: the
table's framing was wrong about **both** schemes, in opposite directions.

Artifacts: `external-2026-08-29-ypir.{txt,json}`, `external-2026-08-29-via.txt`.
Both projects are MIT; see `LICENSES.md`. Neither is vendored, linked against,
or in any manifest, which is the same posture `LICENSES.md` records for every
third-party implementation read here, including the `simplepir` crate written up
in `measurements/crate-comparison-2026-08-28.md`.

Two conventions this file depends on. Where a comparison below uses a query
volume or an arrival rate, both come from the **declared reference workload**:
lambda = 0.06 queries/second and Q = 157 queries per client per rebuild period.
Those are stated evaluation parameters rather than measurements, and
`SCOPE.md` section 3 is their reference definition; where this file restates
them, it restates those same two values rather than justifying them
independently. And
the SimplePIR implementation the "ours" rows come from is not published here
(`SCOPE.md` section 2); it is called *the implementation under measurement*, and
no path inside it is cited.

## What was run

| | YPIR | VIA |
|---|---|---|
| source | `menonsamir/ypir` @ `a73e550` | `owniai/VIA` @ `f65aa9d` |
| built with | rustc, stable; Zig 0.16.0 as the C toolchain for `matmul.cpp` | g++-15 15.2.0, `-O3 -march=native -mavx512dq -mavx512ifma`, Intel HEXL |
| database | **859.2 MB**, the same size every sweep reported in this repository uses | **256 MB**, reduced; see below |
| trials | 3 | 3 |

VIA could not be run at its shipped parameters. `LOG_ROW = 8`, `LOG_COL = 13`
allocates `ROW * COL = 2^21` polynomials of `DEGREE1 = 2048` `uint64`, which is
**32 GiB of NTT coefficients for a 4 GiB logical database, an 8x expansion**,
and is OOM-killed on this 16 GB machine. The authors' machine has 128 GB.
`LOG_COL` was dropped to 9 (256 MB, 2.1 GB allocated), which fits; `core.hpp`
was restored afterwards.

## Measured

**One caveat on the communication rows, because the rule everywhere here is that
a measurement living only in a terminal did not happen.** YPIR's `uploadBytes`
and `downloadBytes` count real serialised buffers. VIA's do not: `printInfoVIA`
*derives* them in closed form from the gadget parameters, and nothing in the run
serialises a query. They are arithmetic on the scheme, checked against the
source, not bytes observed on a wire, and the conclusion below rests entirely on
them. The timings on both sides are genuine wall-clock.

**YPIR**, 859,193,344 items x 8 bits:

| | |
|---|---|
| client offline download | **0 bytes**, the hint-free claim holds |
| server preprocessing | 20.8 s (16.7 s of it SimplePIR prep) |
| online upload / download | 866.3 KB / 12.3 KB |
| server time | **155 ms** (170 / 133 / 162) |

Its *internal, server-side* SimplePIR hint is 117.4 MB, against the 120.1 MB the
implementation under measurement produces independently at the same database
size: agreement within 2.2%, and a useful check on our own parameter set.

**VIA**, 256 MB, no blinded extraction:

| | |
|---|---|
| client offline upload / download | **0 / 0** |
| server database setup | 0.487 s |
| online upload / download | 525.875 KiB / 23 KiB |
| total answer time | **279.2 ms** (285.2 / 279.2 / 278.6) |
| of which First Dimension | 235.2 ms, 84% |
| client query gen / recover | 2.6 to 3.8 ms / 16 microseconds |

## The record-width check, which is the whole point

**YPIR's items are one byte.** `src/params.rs` rejects `ITEM_SIZE_BITS` above 8.
A 128-byte record costs **128 queries**.

**VIA's records are at least 512 bytes.** This is the correction.
`LOG_MODULUS_P = 8` is the plaintext *modulus*, bits per coefficient, not the
record size. A VIA record is `DEGREE2 = 512` coefficients, and the run confirms
it arithmetically: 256 MB divided by `N = 524,288` is 512 B. An earlier reading
in this work took `LOG_MODULUS_P = 8` to mean one-byte records and filed VIA
alongside YPIR. That was wrong (`docs/errors-caught.md` section 6). **One VIA
query covers one of our 128-byte records.**

*"At least", because the source leaves it open which of three it is.* A single
`client.Query(20471, qu)` is answered by `Recover(RlweSampleQ2 ans[8], int64_t*
result[8])`, which decodes **eight** `DEGREE2` blocks, while `N` counts single
blocks. So either a record is 4 KB and `N` overcounts by 8, or it is 512 B and
the download carries 8x more than the query asked for, or the eight blocks are
the record. Nothing below depends on which: every reading gives >= 512 B, the
claim needs >= 128 B, and the byte figures come from the printed communication
totals rather than from the record size.

So the two schemes land in opposite places:

Per client, per rebuild period, at the declared reference workload of Q = 157
queries and at 128-byte records. **All three rows are at our index size**, which
for VIA means correcting off the run: its `OnlineUpload` grows with `LOG_COL`,
and our 6,712,448 x 128 B index needs `ROW = 256, LOG_COL = 11` (1 GiB, the
smallest VIA configuration that holds it), not the `LOG_COL = 9` that fits in
this machine's RAM. Evaluating
`(2*GADGET1_L*(LOG_ROW-3)+8)*sizePolyDeg1Q1 + GADGET_RSK_L*sizePolyDeg1Q2 +
(GADGET2_L_1+GADGET2_L_2)*(LOG_COL-3)*sizePolyDeg2Q2` gives **556.5 KiB** at
`LOG_COL = 11` against the 525.875 KiB printed at 9. The formula reproduces the
printed figure exactly at `LOG_COL = 9`, which is the check that it is being
read correctly. Download does not depend on `LOG_COL` and stays at 23 KiB.

| | database | offline | Q = 157 queries | **total** | vs ours |
|---|---|---|---|---|---|
| **ours (SimplePIR)** | 859.2 MB | 120.1 MB hint | 36.8 MB | **156.9 MB** | |
| **VIA**, `LOG_COL = 11` | 1 GiB | 0 | 93.2 MB | **93.2 MB** | **1.7x better** |
| **YPIR** | 859.2 MB | 0 | 17,656 MB | **17,656 MB** | 113x worse |

YPIR falls behind after **1.1 queries**. VIA stays ahead until **335 queries per
rebuild period**; the reference workload is 157. At the `LOG_COL = 9` size
actually run the crossover is 366: the margin narrows with database size, and
does not close. Both crossovers are properties of the schemes rather than of the
workload, so Q only decides which side of each one a reader is standing on.

## What VIA costs to get that

Server time per byte of database, both measured, neither extrapolated:

| | database | time/query | per MB |
|---|---|---|---|
| ours, 1 core | 859.2 MB | 80.3 ms | 0.093 ms |
| VIA, 1 core | 256 MB | 279.2 ms | 1.09 ms |

**VIA is 11.7x slower per byte.** That is affordable, because Phase 5 records
208x of headroom on one core against the reference arrival rate of lambda = 0.06
queries/second (`SCOPE.md` section 3, `measurements/RESULTS.md`), and it is
*cheaper* than it looks, because 1.7x less client egress is the cost this design
is actually made of.

Two things make it not a decision yet.

**The 8x memory expansion is the real blocker, not the CPU.** Our 859.2 MB index
occupies 859.2 MB of server RAM plus a 120.1 MB hint. The same index under VIA
occupies **6.9 GB**, because every one-byte plaintext coefficient is stored as a
`uint64` NTT coefficient. That is an infrastructure cost of a different kind
from egress, and it is the reason the shipped configuration would not run here
at all.

**The First Dimension number is not a memory measurement.** In
`src/VIAServer.hpp` the First Dimension loop runs the full `COL x ROW` iteration
count, but every operand is literal index `0`: `database[0]`, `FirstDimVec[0]`,
`FirstDimResMasks[0]`. The counters `i` and `j` index nothing. The operation
count is right; the memory traffic is not, because one 16 KB polynomial is
re-read from L1 instead of the database being streamed. `src/VIA_CServer.hpp`
does the same.

The README discloses this ("the First Dimension operations will simulate memory
usage for the full database") but scopes it to databases above the allocation
cap, and the code applies it unconditionally, at every size, including this run.
For a scheme whose cost story is that the server must touch every entry, the 84%
of answer time spent in First Dimension is therefore a floor rather than a
figure. Scaled to our 859.2 MB, with First Dimension and Ring Switch linear in
`COL` and the rest flat, VIA's answer extrapolates to ~0.86 s. Streaming its
6.9 GB of NTT coefficients once per query adds **at least** 0.30 s at this
machine's measured single-core scan floor of 22.8 GB/s out of cache
(`floor-2026-08-28-i5-11400.json`, 512 MiB row; the 1024 MiB row reads 22.92
GB/s), which would put VIA nearer 15x ours rather than 11.7x. Any real figure is
above that floor, not below it.

That is a statement about the shipped harness, not about the paper. We do not
have whatever harness produced the paper's tables and cannot say what it did.

**The harness does not check its own answer.** `test.cpp` calls
`client.Recover(ans, result)` and never compares `result` to anything; the
decrypted value is discarded. The implementation under measurement gates every
printed timing on a correctness check
(`docs/adr/0002-pascal-without-a-pascal-toolkit.md`, point 6) precisely because
a PIR timing without one measures an unconstrained loop. Noted as a property of
the artifact.

## What this changes

1. **YPIR is out**, on the measurement rather than the argument. One-byte items
   are structural in its parameter validation, and 128 queries per record is not
   a gap that tuning closes.
2. **VIA is in**, as a real candidate rather than a table row: the first thing
   found that beats the 120.1 MB hint on total client bytes at our record size
   and at the reference query volume.
3. **The open item moves** from "is there a hint-free scheme that survives
   128-byte records", answered, yes, to **"does VIA's throughput survive
   actually reading the database, and is 8x RAM acceptable."** Both are
   measurable here, and the second is answerable from the source alone.
4. **HintlessPIR is still unrun**, and the reason it is ranked last is weaker
   than it looks. Its published per-query download of 3,080 KB is 130x VIA's,
   but that figure is measured at one-byte records, which is exactly the
   unchecked assumption that had VIA filed next to YPIR two paragraphs of this
   repository ago. Ranking it last is a scheduling decision, not a finding.

> **Since superseded, in the direction this item asked for.** HintlessPIR was
> built and run at this index shape on 2026-09-04:
> `measurements/external-2026-09-04-hintlesspir.md`. Item 4 is left as written,
> because the ranking it recorded as a scheduling decision was one.
