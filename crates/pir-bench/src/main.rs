//! The linear-scan floor.
//!
//! PIR must touch every byte of the database on every query. That is the
//! privacy guarantee itself — touching only the relevant bytes is exactly what
//! leaks which record was wanted — and not an artefact of any particular
//! scheme. So the fastest conceivable PIR on a given machine is bounded below
//! by how fast that machine can stream the database through a trivial
//! operation, and no amount of cryptographic cleverness gets underneath it.
//!
//! This binary measures that floor, and it exists to be divided into. The
//! headline result of this repository is a ratio:
//!
//! ```text
//! PIR multiplier = (seconds/query under PIR) / (seconds/scan measured here)
//! ```
//!
//! Absolute throughput on a shared vCPU transfers to nothing. The multiplier is
//! a property of the scheme rather than of the box, so it is the number that
//! survives being carried to whatever hardware gets rented.
//!
//! The scan itself lives in `lib.rs`, because `pir-simplepir` divides by it and
//! a floor that is merely *meant* to be the same loop will not stay the same
//! loop.
//!
//! # Why the sweep, and why it is not optional
//!
//! A database that fits in cache is not the database. The real existence-check
//! index is roughly 858 MB, so a benchmark run at a size that happens to fit in
//! the host's last-level cache reports a number that is optimistic by a wide
//! margin, silently. Measuring one size is not a conservative simplification;
//! it is a wrong answer with a plausible shape.
//!
//! So the sweep straddles the cache boundary in both directions and the report
//! names the knee. **A curve that is flat across L3 means the harness is
//! measuring cache, and is a failed run rather than a fast one.**
//!
//! # Why every sample runs for at least 50 ms
//!
//! The first run of this harness could not read its own knee: three sweeps gave
//! in-cache/out-of-cache ratios of 1.00x, 0.70x and 1.46x, flipping the verdict
//! each time. The cause was the in-cache point, which finished in ~2 ms and was
//! dominated by scheduling noise and frequency ramp. Every sample is now
//! repeated until it has run for at least `MIN_SAMPLE`, and the reported
//! per-pass time is the total divided by the repeat count.

#![forbid(unsafe_code)]

use std::hint::black_box;
use std::time::{Duration, Instant};

use pir_bench::{Stats, host, make_buffer, scan, scan_parallel};

/// Sizes swept, in MiB. Chosen to straddle an L3 in both directions rather than
/// to be round numbers: 2 and 4 MiB sit inside a desktop part's 12 MiB, 64 MiB
/// and up sit outside anything currently shipping. The original sweep started
/// at 16 MiB, which on a 12 MiB L3 is *already outside* — the curve came back
/// flat and the knee unreadable, not because the machine was odd but because
/// the sweep never sampled cache.
const DEFAULT_SIZES_MIB: &[usize] = &[2, 4, 16, 64, 256, 512, 1024, 2048];

/// Repeats per size. Three is enough to see whether the box is quiet: the
/// spread across repeats is reported, and a spread comparable to the gap
/// between adjacent sizes means the machine is too noisy to conclude from.
const REPEATS: usize = 3;

/// Minimum wall time per timed sample. Below this the number describes the
/// scheduler rather than the memory.
const MIN_SAMPLE: Duration = Duration::from_millis(50);

const MIB: usize = 1024 * 1024;

fn main() {
    if cfg!(debug_assertions) {
        eprintln!(
            "refusing to run: this is a debug build.\n\
             A floor measurement from an unoptimised build describes the \
             optimiser's absence, not the machine.\n\
             Run: cargo run --release --bin pir-bench"
        );
        std::process::exit(2);
    }

    let sizes = parse_sizes(std::env::args().skip(1));
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let l3 = host::l3_bytes();
    let cpu = host::cpu_model();

    print_header(threads, l3, cpu.as_deref());

    let mut rows = Vec::new();
    for &mib in &sizes {
        rows.push(measure(mib, threads));
    }

    print_table(&rows);
    print_knee(&rows, l3);
    print_json(&rows, threads, l3, cpu.as_deref());
}

/// One size's result. Times are for a full pass over the buffer.
#[derive(Debug, Clone)]
struct Row {
    mib: usize,
    single: Stats,
    parallel: Stats,
}

/// Time one pass, repeating until the sample has run for at least
/// `MIN_SAMPLE`, then dividing. Returns the per-pass time.
fn timed(mut pass: impl FnMut()) -> Duration {
    let probe = Instant::now();
    pass();
    let one = probe.elapsed().max(Duration::from_nanos(1));

    let iters = u32::try_from(MIN_SAMPLE.as_nanos() / one.as_nanos()).unwrap_or(1).max(1);

    let t = Instant::now();
    for _ in 0..iters {
        pass();
    }
    t.elapsed() / iters
}

fn measure(mib: usize, threads: usize) -> Row {
    let bytes = mib * MIB;
    let buf = make_buffer(bytes);

    // One untimed pass so the first timed pass is not also paying for first
    // touch of freshly mapped pages.
    black_box(scan(&buf));

    let mut single = Vec::with_capacity(REPEATS);
    for _ in 0..REPEATS {
        single.push(timed(|| {
            black_box(scan(&buf));
        }));
    }

    let mut parallel = Vec::with_capacity(REPEATS);
    for _ in 0..REPEATS {
        parallel.push(timed(|| {
            black_box(scan_parallel(&buf, threads));
        }));
    }

    Row {
        mib,
        single: Stats::from(single),
        parallel: Stats::from(parallel),
    }
}

fn parse_sizes<I: Iterator<Item = String>>(args: I) -> Vec<usize> {
    let parsed: Vec<usize> = args.filter_map(|a| a.parse::<usize>().ok()).collect();
    if parsed.is_empty() {
        DEFAULT_SIZES_MIB.to_vec()
    } else {
        parsed
    }
}

fn print_header(threads: usize, l3: Option<usize>, cpu: Option<&str>) {
    println!("pir-bench — linear-scan floor");
    println!();
    println!("cpu:  {}", cpu.unwrap_or("unknown"));
    match l3 {
        Some(b) => println!(
            "host: {threads} threads · L3 {:.0} MiB (knee expected here)",
            b as f64 / MIB as f64
        ),
        None => println!("host: {threads} threads · L3 unknown (knee cannot be located)"),
    }
    println!(
        "repeats per size: {REPEATS} · minimum sample {} ms",
        MIN_SAMPLE.as_millis()
    );
    println!();
}

fn print_table(rows: &[Row]) {
    println!(
        "{:>8}  {:>10}  {:>8}  {:>10}  {:>8}",
        "size", "1-thread", "spread", "n-thread", "spread"
    );
    println!(
        "{:->8}  {:->10}  {:->8}  {:->10}  {:->8}",
        "", "", "", "", ""
    );
    for r in rows {
        let bytes = r.mib * MIB;
        println!(
            "{:>6} M  {:>7.2} GB/s  {:>7.1}%  {:>7.2} GB/s  {:>7.1}%",
            r.mib,
            r.single.gbps(bytes),
            r.single.spread() * 100.0,
            r.parallel.gbps(bytes),
            r.parallel.spread() * 100.0,
        );
    }
    println!();

    let noisy = rows.iter().filter(|r| r.single.spread() > 0.10).count();
    if noisy > 0 {
        println!(
            "WARNING: {noisy} size(s) vary by >10% across repeats. On a shared vCPU that \
             can exceed the gap between adjacent sizes, which makes the knee \
             unreadable. Re-run on a quiet machine before concluding."
        );
        println!();
    }
}

/// The acceptance test. Compare throughput comfortably inside cache against
/// throughput comfortably outside it. A flat curve is a failed run.
fn print_knee(rows: &[Row], l3: Option<usize>) {
    let Some(l3) = l3 else {
        println!("knee: not evaluated (L3 size unknown).");
        return;
    };

    let inside: Vec<&Row> = rows.iter().filter(|r| r.mib * MIB * 2 <= l3).collect();
    let outside: Vec<&Row> = rows.iter().filter(|r| r.mib * MIB >= l3 * 2).collect();

    if inside.is_empty() || outside.is_empty() {
        println!(
            "knee: not evaluated — the sweep does not straddle L3 ({:.0} MiB) \
             with margin on both sides. Widen the sizes.",
            l3 as f64 / MIB as f64
        );
        return;
    }

    let avg = |v: &[&Row]| -> f64 {
        let s: f64 = v.iter().map(|r| r.single.gbps(r.mib * MIB)).sum();
        s / v.len() as f64
    };
    let (a, b) = (avg(&inside), avg(&outside));
    let ratio = if b > 0.0 { a / b } else { 0.0 };

    println!("knee: {a:.2} GB/s in cache vs {b:.2} GB/s out — ratio {ratio:.2}x");
    if ratio >= 1.15 {
        println!(
            "  Curve behaves as expected. Use the out-of-cache figure as the floor — \
             it is the one the real index is subject to."
        );
    } else if ratio >= 1.0 {
        println!(
            "  NOT A USABLE FLOOR. The curve is flat across L3: either the working set \
             is not leaving cache, or the scan loop is slower than this machine's \
             memory and is the binding constraint. Either way the number describes the \
             loop, not the memory."
        );
    } else {
        println!(
            "  NOT A USABLE FLOOR, and inverted: throughput *rises* with size. That is \
             not a cache effect. It means the small sizes are dominated by per-run \
             fixed cost (they finish too fast to time well) and/or the scan loop caps \
             below this machine's memory bandwidth, so the memory wall is never \
             reached."
        );
    }
    if ratio < 1.15 {
        println!(
            "  Consequence for the headline metric: a floor that is too HIGH makes the \
             PIR multiplier too SMALL, which flatters the scheme. Treat any multiplier \
             derived from this run as a lower bound on the true one."
        );
    }
    println!();
}

/// Emit the same numbers as JSON so a run can be committed to `measurements/`
/// rather than living in a terminal that gets closed.
fn print_json(rows: &[Row], threads: usize, l3: Option<usize>, cpu: Option<&str>) {
    println!("--- json ---");
    println!("{{");
    match cpu {
        Some(c) => println!("  \"cpu\": \"{}\",", c.replace('"', "'")),
        None => println!("  \"cpu\": null,"),
    }
    println!("  \"threads\": {threads},");
    match l3 {
        Some(b) => println!("  \"l3_bytes\": {b},"),
        None => println!("  \"l3_bytes\": null,"),
    }
    println!("  \"repeats\": {REPEATS},");
    println!("  \"min_sample_ms\": {},", MIN_SAMPLE.as_millis());
    println!("  \"rows\": [");
    for (i, r) in rows.iter().enumerate() {
        let bytes = r.mib * MIB;
        let comma = if i + 1 == rows.len() { "" } else { "," };
        println!(
            "    {{\"mib\": {}, \"bytes\": {}, \"single_gbps\": {:.4}, \
             \"single_spread\": {:.4}, \"parallel_gbps\": {:.4}, \
             \"parallel_spread\": {:.4}}}{}",
            r.mib,
            bytes,
            r.single.gbps(bytes),
            r.single.spread(),
            r.parallel.gbps(bytes),
            r.parallel.spread(),
            comma
        );
    }
    println!("  ]");
    println!("}}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_default_when_no_args() {
        assert_eq!(parse_sizes(std::iter::empty()), DEFAULT_SIZES_MIB.to_vec());
        assert_eq!(
            parse_sizes(["32".to_string(), "nonsense".to_string()].into_iter()),
            vec![32]
        );
    }
}
