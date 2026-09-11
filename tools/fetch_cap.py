#!/usr/bin/env python3
"""Sample the Caselaw Access Project's metadata, and measure the record size.

Every figure in `measurements/RESULTS.md` rests on m = 29312, which came from
a 1,862-record sample that was never committed. Hint generation costs O(m^2 n),
so a 20% error in the record size is a 44% error in what an index rebuild
costs. This settles it, against a 100,394-record sample that is committed as
`measurements/record-size.json`.

What it fetches, and what it does not
-------------------------------------
Only `VolumesMetadata.json` per reporter and `CasesMetadata.json` per volume.
**No opinion text.** The existence-check index needs the citation, the
abbreviated case name and the decision year, and all three are in the metadata;
the ~350 GB text corpus is a different download for a different problem.

Two traps, both hit before
--------------------------
1. **CAP answers a missing volume with a ~27 KB HTML error page.** Sometimes
   with a 404 and sometimes not, so a status-code check silently ingests an
   error page as if it were data and corrupts the statistics rather than
   failing. Every response here is validated *structurally* — it must parse as
   JSON and be a list — and anything else is discarded and recorded as a miss.
2. **`jq` is not installed on the measurement machine and Python is.**
   This began life as a shell script doing `jq -e 'type=="array"'`; this is
   that same check, in the language the repository already depends on for
   `build_ptx.py`.

Sampling
--------
Stratified by era and jurisdiction rather than by convenience, because
convenience means whichever reporters are easiest, and those skew old and
federal. Six strata: {federal, state} x {pre-1950, 1950-2000, post-2000}.
Volumes are ordered deterministically inside each stratum — no RNG, so a rerun
with the same target samples the same volumes and the number is reproducible.

Usage
-----
    python tools/fetch_cap.py                  # sample and write the JSON
    python tools/fetch_cap.py --target 20000   # smaller run
    python tools/fetch_cap.py --analyse-only   # recompute from the cache
"""

from __future__ import annotations

import argparse
import json
import pathlib
import statistics
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORPUS = ROOT / "corpus"           # gitignored
OUT = ROOT / "measurements" / "record-size.json"
BASE = "https://static.case.law"

# The fixed-width year the index stores. The payload is key + name + this; the
# year is not tabulated with the other two because it is fixed width, and it is
# the 2 B that takes the measured 8.95 + 28.79 to 39.75.
YEAR_BYTES = 2

# Candidate reporters. Probed at run time; whatever is absent is skipped and
# reported, so this list can be wrong without being fatal.
REPORTERS = [
    # Federal
    "us", "f2d", "f3d", "f-supp", "f-supp-2d", "f-supp-3d", "fed-cl",
    # Regional, first series (mostly pre-1950)
    "a", "p", "sw", "ne", "nw", "so", "se",
    # Regional, second series (mostly 1930-2000)
    "a2d", "p2d", "sw2d", "ne2d", "nw2d", "so2d", "se2d",
    # Regional, third series (mostly post-1990)
    "a3d", "p3d", "sw3d", "ne3d", "nw2d", "se2d",
]

ERAS = (("pre-1950", 0, 1949), ("1950-2000", 1950, 2000), ("post-2000", 2001, 9999))


def fetch(url: str, dest: pathlib.Path) -> object | None:
    """Fetch and structurally validate. Returns a list, or None.

    Uses curl rather than urllib: on the machine this was written for, urllib
    was observed to take a 403 through a proxy where curl on the same URL
    succeeded.
    """
    if dest.exists():
        try:
            data = json.loads(dest.read_text(encoding="utf-8"))
            return data if isinstance(data, list) else None
        except (ValueError, OSError):
            dest.unlink(missing_ok=True)

    dest.parent.mkdir(parents=True, exist_ok=True)
    r = subprocess.run(
        ["curl", "-s", "--max-time", "60", "-o", str(dest), url],
        capture_output=True,
    )
    if r.returncode != 0 or not dest.exists():
        dest.unlink(missing_ok=True)
        return None
    try:
        data = json.loads(dest.read_text(encoding="utf-8"))
    except (ValueError, OSError):
        # This is the ~27 KB error page, not JSON. Discard it so a later run
        # does not treat the cache as authoritative.
        dest.unlink(missing_ok=True)
        return None
    if not isinstance(data, list):
        dest.unlink(missing_ok=True)
        return None
    return data


def era_of(year: int) -> str | None:
    for name, lo, hi in ERAS:
        if lo <= year <= hi:
            return name
    return None


def normalise_key(cite: str) -> str:
    """volume + canonical reporter + first page, lowercased alphanumerics.

    The index this feeds is keyed on the normalised citation, so what matters
    here is the *length* of that key rather than the key itself: the measurement
    is how wide a fixed-width PIR record has to be to hold one, and a scheme
    that normalises differently by a separator or two lands in the same place.
    """
    return "".join(ch for ch in cite.lower() if ch.isalnum())


def official_cite(rec: dict) -> str | None:
    cites = rec.get("citations") or []
    for c in cites:
        if c.get("type") == "official" and c.get("cite"):
            return c["cite"]
    return cites[0].get("cite") if cites and cites[0].get("cite") else None


def build_pool() -> tuple[list[dict], list[str]]:
    pool, missing = [], []
    for slug in dict.fromkeys(REPORTERS):
        vols = fetch(f"{BASE}/{slug}/VolumesMetadata.json", CORPUS / slug / "VolumesMetadata.json")
        if vols is None:
            missing.append(slug)
            continue
        for v in vols:
            year = v.get("publication_year") or v.get("start_year") or 0
            era = era_of(int(year or 0))
            if era is None:
                continue
            juris = [j.get("name", "") for j in (v.get("jurisdictions") or [])]
            klass = "federal" if "U.S." in juris else "state"
            pool.append({
                "reporter": slug,
                "volume": str(v.get("volume_number")),
                "year": int(year),
                "era": era,
                "class": klass,
                "stratum": f"{klass}/{era}",
            })
    return pool, missing


def sample(pool: list[dict], target: int) -> tuple[list[dict], dict]:
    """Round-robin across strata until the record target is met."""
    strata: dict[str, list[dict]] = {}
    for v in pool:
        strata.setdefault(v["stratum"], []).append(v)
    # Deterministic order inside each stratum: no RNG, so a rerun is a rerun.
    for k in strata:
        strata[k].sort(key=lambda v: (v["reporter"], int(v["volume"]) if v["volume"].isdigit() else 0))

    records: list[dict] = []
    per_stratum: dict[str, int] = {k: 0 for k in strata}
    volumes_used = 0
    misses = 0
    idx = {k: 0 for k in strata}
    order = sorted(strata)
    while len(records) < target and any(idx[k] < len(strata[k]) for k in order):
        for k in order:
            if len(records) >= target or idx[k] >= len(strata[k]):
                continue
            v = strata[k][idx[k]]
            idx[k] += 1
            cases = fetch(
                f"{BASE}/{v['reporter']}/{v['volume']}/CasesMetadata.json",
                CORPUS / v["reporter"] / v["volume"] / "CasesMetadata.json",
            )
            if cases is None:
                misses += 1
                continue
            volumes_used += 1
            for rec in cases:
                cite = official_cite(rec)
                name = rec.get("name_abbreviation")
                if not cite or not name:
                    continue
                key = normalise_key(cite)
                records.append({
                    "stratum": k,
                    "key": len(key.encode("utf-8")),
                    "name": len(name.encode("utf-8")),
                })
                per_stratum[k] += 1
            if len(records) % 20000 < len(cases):
                print(f"  {len(records):,} records from {volumes_used} volumes ...")
    return records, {"volumes_used": volumes_used, "volume_misses": misses,
                     "per_stratum": per_stratum, "strata_available": {k: len(v) for k, v in strata.items()}}


def stats(values: list[int]) -> dict:
    if not values:
        return {}
    v = sorted(values)
    return {
        "n": len(v),
        "mean": round(statistics.fmean(v), 2),
        "median": v[len(v) // 2],
        "p95": v[min(len(v) - 1, int(0.95 * len(v)))],
        "max": v[-1],
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--target", type=int, default=100_000)
    ap.add_argument("--analyse-only", action="store_true")
    args = ap.parse_args()

    print("building the volume pool ...")
    pool, missing = build_pool()
    print(f"  {len(pool)} volumes across {len(set(v['reporter'] for v in pool))} reporters"
          f"{'; absent: ' + ', '.join(missing) if missing else ''}")

    print(f"sampling to {args.target:,} records ...")
    records, meta = sample(pool, args.target)
    if not records:
        raise SystemExit("no records sampled")

    payloads = [r["key"] + r["name"] + YEAR_BYTES for r in records]
    widths = {}
    for w in (64, 96, 128, 256):
        intact = sum(1 for p in payloads if p <= w)
        widths[str(w)] = {
            "records_intact_pct": round(100.0 * intact / len(payloads), 3),
            "db_bytes_at_6_7m": w * 6_700_000,
        }

    by_stratum = {}
    for k in sorted(set(r["stratum"] for r in records)):
        sel = [r for r in records if r["stratum"] == k]
        by_stratum[k] = {
            "key": stats([r["key"] for r in sel]),
            "name": stats([r["name"] for r in sel]),
            "payload": stats([r["key"] + r["name"] + YEAR_BYTES for r in sel]),
        }

    out = {
        "phase": "1",
        "source": "Caselaw Access Project, static.case.law, metadata only",
        "sample_definition": {
            "target_records": args.target,
            "records": len(records),
            "volumes_used": meta["volumes_used"],
            "volume_misses": meta["volume_misses"],
            "reporters_absent": missing,
            "stratified_by": "jurisdiction class x era, round-robin, deterministic order",
            "strata_available_volumes": meta["strata_available"],
            "records_per_stratum": meta["per_stratum"],
        },
        "year_bytes": YEAR_BYTES,
        "overall": {
            "key": stats([r["key"] for r in records]),
            "name": stats([r["name"] for r in records]),
            "payload": stats(payloads),
        },
        "by_stratum": by_stratum,
        "entry_widths": widths,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8", newline="\n")

    p = out["overall"]["payload"]
    print(f"\n{len(records):,} records from {meta['volumes_used']} volumes "
          f"({meta['volume_misses']} volume misses)")
    print(f"payload: mean {p['mean']} B, median {p['median']}, p95 {p['p95']}, max {p['max']}")
    for w, d in widths.items():
        print(f"  {w:>4} B entry: {d['records_intact_pct']:6.3f}% intact  "
              f"-> {d['db_bytes_at_6_7m']/1e6:,.0f} MB at 6.7M records")
    print(f"\nwrote {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
