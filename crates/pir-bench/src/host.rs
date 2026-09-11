//! Which machine produced the number.
//!
//! Cache size and CPU model are read at runtime and never hardcoded, because
//! they are not properties of this code. During the session that started this
//! repository the container was rescheduled mid-work and the CPU changed
//! underneath it — from a 2.10 GHz part with 260 MiB of L3 to a 2.80 GHz part
//! with 33 MiB — which moves the knee by nearly an order of magnitude. A
//! measurement that does not name the machine it ran on cannot be compared
//! with any other measurement.
//!
//! Two hosts are supported because two hosts have been used: Linux (sysfs and
//! `/proc/cpuinfo`, the rented container) and Windows (the registry and CIM,
//! the desktop with the GTX 1070 that Phase 4b's GPU arm needs). Neither path
//! is allowed to fail the run — a missing value degrades the *report*, and the
//! report says so, rather than degrading the measurement silently.

use std::process::Command;

/// CPU model string, or `None` if this host does not offer one cheaply.
#[must_use]
pub fn cpu_model() -> Option<String> {
    linux_cpu_model().or_else(windows_cpu_model)
}

/// Last-level cache in bytes. Used only to name where the knee is expected;
/// nothing about the measurement depends on it.
#[must_use]
pub fn l3_bytes() -> Option<usize> {
    linux_l3_bytes().or_else(windows_l3_bytes)
}

fn linux_cpu_model() -> Option<String> {
    let raw = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    raw.lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim().to_owned())
}

fn linux_l3_bytes() -> Option<usize> {
    for idx in [3usize, 2] {
        let p = format!("/sys/devices/system/cpu/cpu0/cache/index{idx}/size");
        let Ok(raw) = std::fs::read_to_string(&p) else {
            continue;
        };
        let t = raw.trim();
        let (num, mult) = match t.chars().last() {
            Some('K') => (t.trim_end_matches('K'), 1024),
            Some('M') => (t.trim_end_matches('M'), 1024 * 1024),
            Some('G') => (t.trim_end_matches('G'), 1024 * 1024 * 1024),
            _ => (t, 1),
        };
        if let Ok(n) = num.parse::<usize>() {
            return Some(n * mult);
        }
    }
    None
}

/// `reg query` rather than CIM: it answers in ~50 ms where PowerShell costs
/// ~1.4 s, and the model string is the value most likely to be wanted.
fn windows_cpu_model() -> Option<String> {
    let out = Command::new("reg")
        .args([
            "query",
            r"HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0",
            "/v",
            "ProcessorNameString",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .find(|l| l.contains("ProcessorNameString"))
        .and_then(|l| l.split("REG_SZ").nth(1))
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

/// The registry does not carry cache sizes, so this one costs a PowerShell
/// start (~1.4 s). Paid once per run, before any timing begins.
fn windows_l3_bytes() -> Option<usize> {
    let out = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_Processor | Select-Object -First 1).L3CacheSize",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    // Win32_Processor reports L3CacheSize in KiB.
    text.trim().parse::<usize>().ok().map(|kib| kib * 1024)
}
