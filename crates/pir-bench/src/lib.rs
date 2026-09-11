//! The floor, as a library.
//!
//! `main.rs` sweeps this across the cache hierarchy and reports the knee. It is
//! also linked by `pir-simplepir`, which divides by it: the headline result of
//! this repository is
//!
//! ```text
//! PIR multiplier = (seconds/query under PIR) / (seconds/scan measured here)
//! ```
//!
//! and a multiplier is only meaningful if numerator and denominator were
//! produced by the *same* scan on the *same* run. That is the whole reason
//! these functions live in a library rather than being copied: two scan loops
//! that are meant to be identical will not stay identical, and the divergence
//! would show up as a change in the metric rather than as a compile error.

#![forbid(unsafe_code)]

use std::time::Duration;

pub mod host;

/// Touch every byte. Summation is the cheapest operation that cannot be
/// optimised away once the result is consumed, which makes this a measurement
/// of moving bytes rather than of doing arithmetic on them.
#[must_use]
pub fn scan(buf: &[u8]) -> u64 {
    let mut acc: u64 = 0;
    for c in buf.chunks_exact(8) {
        let mut b = [0u8; 8];
        b.copy_from_slice(c);
        acc = acc.wrapping_add(u64::from_le_bytes(b));
    }
    acc
}

/// The same scan split across threads. PIR parallelises trivially — every
/// record is independent — so the parallel figure is the one that describes a
/// server, and the single-threaded figure is the one that describes the
/// scheme.
#[must_use]
pub fn scan_parallel(buf: &[u8], threads: usize) -> u64 {
    if threads <= 1 || buf.is_empty() {
        return scan(buf);
    }
    // Round the split up to a multiple of 8. `scan` reads u64 at a time via
    // `chunks_exact(8)`, so a chunk boundary that is not 8-aligned makes each
    // thread drop its own tail *and* shifts every subsequent u64 relative to
    // the serial pass — the two then measure different computations. The
    // `parallel_agrees_with_serial` test exists because this was wrong first
    // time round, at threads=3.
    let per = buf.len().div_ceil(threads).next_multiple_of(8).max(8);
    std::thread::scope(|s| {
        let handles: Vec<_> = buf.chunks(per).map(|part| s.spawn(|| scan(part))).collect();
        handles
            .into_iter()
            .fold(0u64, |a, h| a.wrapping_add(h.join().unwrap_or(0)))
    })
}

/// Fill with a cheap LCG rather than zeros. A zeroed buffer can be backed by a
/// single shared page, which would measure the page table instead of the
/// memory.
#[must_use]
pub fn make_buffer(bytes: usize) -> Vec<u8> {
    let mut v = vec![0u8; bytes];
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    for chunk in v.chunks_mut(8) {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let b = state.to_le_bytes();
        for (dst, src) in chunk.iter_mut().zip(b.iter()) {
            *dst = *src;
        }
    }
    v
}

/// Spread across repeats, kept rather than averaged away: the spread is how you
/// tell a measurement from a coincidence.
#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    /// Fastest repeat. Closest to the machine's capability.
    pub best: Duration,
    /// Reported figure. Robust to a single stalled repeat.
    pub median: Duration,
    /// Slowest repeat. With `best`, this is the spread.
    pub worst: Duration,
}

impl Stats {
    /// Collapse a set of repeats. Sorts, so the caller need not.
    #[must_use]
    pub fn from(mut d: Vec<Duration>) -> Self {
        d.sort_unstable();
        let median = d.get(d.len() / 2).copied().unwrap_or_default();
        Self {
            best: d.first().copied().unwrap_or_default(),
            median,
            worst: d.last().copied().unwrap_or_default(),
        }
    }

    /// GB/s using the median, decimal GB to match how memory bandwidth is
    /// normally quoted.
    #[must_use]
    pub fn gbps(self, bytes: usize) -> f64 {
        let s = self.median.as_secs_f64();
        if s <= 0.0 {
            return 0.0;
        }
        bytes as f64 / s / 1e9
    }

    /// Spread as a fraction of the median. Above ~0.10 the box is noisy enough
    /// that adjacent sizes stop being distinguishable.
    #[must_use]
    pub fn spread(self) -> f64 {
        let m = self.median.as_secs_f64();
        if m <= 0.0 {
            return 0.0;
        }
        (self.worst.as_secs_f64() - self.best.as_secs_f64()) / m
    }

    /// The median, in seconds. The numerator and denominator of the PIR
    /// multiplier are both this.
    #[must_use]
    pub fn secs(self) -> f64 {
        self.median.as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_touches_every_byte() {
        // A changed byte must change the result, or the "touches everything"
        // claim the floor rests on is not true of this loop.
        let a = make_buffer(1024);
        let mut b = a.clone();
        let last = b.len() - 8;
        if let Some(v) = b.get_mut(last) {
            *v = v.wrapping_add(1);
        }
        assert_ne!(scan(&a), scan(&b));
    }

    #[test]
    fn parallel_agrees_with_serial() {
        let buf = make_buffer(8 * 1024);
        // Chunking must not change the sum, or the parallel figure is measuring
        // a different computation from the serial one.
        for threads in [1usize, 2, 3, 4, 7] {
            assert_eq!(
                scan(&buf),
                scan_parallel(&buf, threads),
                "threads={threads}"
            );
        }
    }

    #[test]
    fn buffer_is_not_uniform() {
        // A zeroed buffer can be backed by one shared page and would measure
        // the page table rather than memory.
        let b = make_buffer(4096);
        let first = b.first().copied().unwrap_or(0);
        assert!(b.iter().any(|&x| x != first));
    }
}
