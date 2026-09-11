# Compared against `docs.rs/simplepir`

**Date:** 2026-08-28
**Subject:** `simplepir` 1.0.1, MIT, `github.com/XiXinping/simplepir-rs`, published
2024-08-19, 2,459 downloads, badge `passively-maintained`.
**Raw:** `measurements/crate-comparison-2026-08-28.txt`
**Status:** read and measured. **Not linked against**, and not in any manifest:
the same posture `docs/adr/0001-gpu-before-fpga-for-pir.md` point 6 takes to
`Crypto-Lab-WHU/GPU-PIR`. Its MIT terms are recorded in `LICENSES.md`.

## Why this was worth doing

Our SimplePIR implementation, the implementation under measurement, is **not
published here** (`SCOPE.md` section 2). It was written from the construction
and has **no dependencies**. That is deliberate: a lattice scheme whose
arithmetic arrives through a dependency is one whose noise budget has to be
argued about with someone else's release notes. But it also means nothing
outside it had ever checked it. This closes that.

It matters most for the one claim in `measurements/RESULTS.md` that leans on an
outside number: that reproducing the SimplePIR paper's published hint and query
sizes is evidence the parameter set is right. **That claim is only worth
anything if there is no shared code**, and there is none, which is precisely
what the zero-dependency posture buys. This confirms it from the other
direction.

## The parameter sets are not the same, and the difference is structural

| | `simplepir` 1.0.1 | the implementation under measurement | the paper |
|---|---|---|---|
| ciphertext modulus `q` | 2^64 (hardcoded) | 2^32 | 2^32 |
| plaintext modulus `p` | 2^17 recommended | 2^8 | small |
| secret dimension `n` | 2048 | 1024 | 1024 |
| error sigma | 81,920 | 6.4 | 6.4 |
| error sampler | 40,961-entry CDT, rejection | centred binomial, k = 82 | discrete Gaussian |
| secret | uniform over `u64` | uniform over `u32` | (not stated) |
| database element | `u64` | `u8` | (not stated) |
| packing | 3 records per `u64` when p <= 2^21 | none | (not stated) |
| dependencies | `rand`, `rand_chacha`, `rand_distr`, `thiserror` | none | (not stated) |

**Our parameters are the paper's. Theirs are not.** The decisive check is the
hint size, because it is fixed by `n` and the word width and nothing else. The
paper reports a **121 MB** hint for a 1 GB database. Scaling our measured 120.1 MB
at 859 MB gives **129.6 MB** at 1 GB, within 7%. The same database under the
crate's parameters gives `31623 x 2048 x 8 B` = **518 MB**, 4.3x the published
figure. A parameter set that misses the paper's headline number by 4x is not the
one the paper describes.

At our real index size the gap is the same 4x:

| | their hint | our hint | their query | our query |
|---|---|---|---|---|
| m = 29312 | **480.2 MB** | **120.1 MB** | 234.5 KB | 117.2 KB |

Given `measurements/RESULTS.md` Phase 4c concludes that **hint distribution is
the entire cost of this design**, a 4x larger hint is not a detail. It is the
cost line, multiplied by four.

## Throughput, same machine, same `m`, single-threaded

**Host:** the desktop, i5-11400 at 2.60 GHz, single-threaded on both sides, both
run on 2026-08-28. Their figures are the three sizes in
`measurements/crate-comparison-2026-08-28.txt`; ours are the m = 2048 rows of
`measurements/cpu-hint-2026-08-28.json` (`gmac_per_s_single`) and
`measurements/cpu-simplepir-2026-08-28.json` (4.19 MB over `answer_single_s`).

| m = 2048 | `simplepir` 1.0.1 | ours | ratio |
|---|---|---|---|
| hint `D·A` | 1.21 GMAC/s | **9.12 GMAC/s** | 7.5x |
| answer `D·q` | 1.57 GMAC/s | **11.95 GMAC/s** | 7.6x |

**Do not read that as a like-for-like speed comparison.** Their MAC is
`u64 x u64` and ours is `u8 x u32`, so a good part of the gap is arithmetic
width rather than implementation quality. What it does establish is that the
figures in `measurements/RESULTS.md` are not slow by the standards of an
existing implementation, which, before this, nothing had shown.

The fairer question is wall-clock for equivalent capacity. To hold the same
859 MB of payload at 17 bits per entry they need m approximately 20,101, so a
hint of `20101^2 x 2048` = 8.3 x 10^11 MACs at 1.21 GMAC/s is approximately
**684 s**, against our **89.6 s**, and a 329 MB hint against our 120 MB. With
their 3x packing enabled (51 bits per `u64`) it improves to m approximately
11,608, approximately **228 s** and a 190 MB hint. Still slower and still
larger, but the packing is a real advantage and is discussed below.

## What their design does better, and what we should consider taking

**Record packing.** `Database::compress` puts three plaintext records into one
`u64` when `p <= 2^21`. That cuts server work per record by 3x and shrinks `m`,
and therefore the hint, by sqrt(3). We have no equivalent.

Whether it transfers is a genuine question and the answer is not obvious:

- Our answer kernel is **memory-bound**, 1.01x the device's own linear-scan
  floor. Packing three 8-bit symbols into a `u32` would raise memory traffic from
  1.0 to 1.33 bytes per symbol, so it would make the answer roughly **33%
  slower**, not faster.
- Our hint kernel is **compute-bound**, so the same packing would make it about
  **2.3x faster** and the hint **1.73x smaller**.
- But packing raises the magnitude of a database entry from 2^8 to 2^24, and the
  noise a client decodes through is the sum over `j` of `D[i][j]·e[j]`. Our
  margin is 52 sigma at m = 29312; a 2^16 increase in entry magnitude destroys it
  outright. Recovering it needs a larger `q`, which is exactly why the crate uses
  2^64, and exactly why its hint is 4x ours.

So their design is **a coherent alternative point in the same trade space, not a
mistake**: it buys the ability to pack more plaintext per entry by paying for a
64-bit modulus, and pays for that in hint size. Ours buys a small hint by
staying at 8-bit entries and a 32-bit modulus.

Which end of that trade is better is a property of the workload, not of the
scheme. Against the **declared reference workload** (lambda = 0.06
queries/second and Q = 157 queries per client per rebuild period, `SCOPE.md`
section 3, stated evaluation parameters rather than measurements), ours is the
end that costs less, for two reasons that are both in
`measurements/RESULTS.md`: Phase 4c finds hint *distribution* is the entire cost
of this design, and Phase 5 finds a single desktop core already carries
approximately 208x the arriving load at lambda = 0.06. Buying server throughput
with a 4x larger hint spends the scarce resource to buy the abundant one. Raise
lambda far enough, or raise Q far enough that the hint amortises over many more
queries, and the crate's end becomes the better one. It is a trade, and it was
made here without knowing the other end existed.

## Two observations on the crate, offered without prejudice

It is passively maintained and two years old; neither of these is a criticism of
a working implementation.

- **`gauss_sample()` constructs `ChaCha20Rng::from_entropy()` on every call**
  (`src/regev/gauss.rs`). One OS entropy draw and one CSPRNG seeding per error
  sample. At m = 2048 that is approximately 24 microseconds per sample; at our
  index size it would be approximately 0.7 s of noise generation per query,
  against a 4 ms answer.
- **The sampler is rejection sampling against a 40,961-entry `f64` CDT table**
  with a `SKIP` factor of 40, which makes `src/regev/gauss.rs` 674 KB of the
  crate's source. It is the reason the sampler is not constant-time; ours is not
  either, and neither is claiming to be.

## What this does and does not settle

**Settles:** our parameter set is the paper's and the crate's is not; our
throughput is not anomalously slow; the "we reproduce the paper's published
sizes" claim in `measurements/RESULTS.md` is not circular, because there is no
shared code.

**Does not settle:** whether either parameter set is *secure*. A back-of-envelope
root-Hermite estimate puts the crate's n = 2048 / q = 2^64 choice at a
**smaller** delta than our n = 1024 / q = 2^32, that is, plausibly the more
conservative of the two, despite its much smaller noise-to-modulus ratio,
because it doubles the dimension. That is a heuristic and not an estimator run.
**Neither parameter set has been through a lattice estimator**, and that remains
the top open item in `measurements/RESULTS.md`. This comparison does not change
it; if anything it sharpens it, because two implementations of the same paper
disagree by a factor of two in `n`.

**Not a cross-implementation test vector.** The two schemes differ in element
type, packing and modulus, so their `answer` outputs are not comparable
bit-for-bit and no such check was attempted. That would be the strictly stronger
form of this comparison and it does not exist here.
