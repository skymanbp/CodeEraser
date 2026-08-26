//! bench harness support (M9 batch 5): timing, nearest-rank
//! percentiles, the bench.json row grammar and its merge. The
//! conventions are inherited, each from a named precedent — sorted
//! nearest-rank P50/P95 (the retired perf_budget.rs shape), refuse
//! debug-build numbers outright (PERF-BUDGET.md:60-62), every row
//! bound to {version, commit, dirty, host} (attack-review F1), a
//! failed measurement reported by name and never recorded as a
//! number (the trend measure() stance).

#![allow(dead_code)] // shared across the bench targets; each binary uses its slice

pub mod render;

use serde_json::{Value, json};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub const BENCH_SCHEMA: &str = "ce.bench/0.1.0";

/// contracts/bench/bench.json relative to the cli/ test cwd.
pub fn bench_path() -> std::path::PathBuf {
    std::path::PathBuf::from("../contracts/bench/bench.json")
}

/// Nearest-rank percentile over a SORTED sample vector — rtts[94] on
/// n=100 is the repo's convention; generalized as ceil(p*n)-1.
pub fn pctl(sorted: &[u128], p: f64) -> u128 {
    let idx = ((p * sorted.len() as f64).ceil() as usize).saturating_sub(1);
    sorted[idx.min(sorted.len() - 1)]
}

/// Run one command to completion, timed. Exit 0 AND exit 1 are both
/// the product SPEAKING (0 = pass, 1 = a verdict — `ce check` on a
/// mid-batch tree honestly fails its own ratchet, and that run's
/// latency is a real measurement); exit ≥ 2 is infrastructure and a
/// named failure, never a recorded number. Returns (ms, stdout).
pub fn timed(mut cmd: Command, what: &str) -> anyhow::Result<(u128, String)> {
    let t0 = Instant::now();
    let out = cmd.output()?;
    let ms = t0.elapsed().as_millis();
    anyhow::ensure!(
        matches!(out.status.code(), Some(0 | 1)),
        "{what}: exit {:?}\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    Ok((ms, String::from_utf8_lossy(&out.stdout).into_owned()))
}

/// n timed runs of a freshly built command → sorted millisecond
/// samples plus the LAST stdout (for count extraction).
pub fn samples(
    n: usize,
    mut mk: impl FnMut() -> Command,
    what: &str,
) -> anyhow::Result<(Vec<u128>, String)> {
    let mut ms = Vec::with_capacity(n);
    let mut last = String::new();
    for _ in 0..n {
        let (t, out) = timed(mk(), what)?;
        ms.push(t);
        last = out;
    }
    ms.sort_unstable();
    Ok((ms, last))
}

/// Host fingerprint: enough to see a mixed-machine series, no PII.
pub fn host() -> String {
    format!(
        "{}/{}/{}cpu",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::thread::available_parallelism().map_or(0, |n| n.get())
    )
}

/// One series row. `release` comes from the MEASURED binary's build,
/// asserted by the caller — a debug number is refused before here.
#[allow(clippy::too_many_arguments)] // the row grammar IS this arity; a struct would re-state it
pub fn row(
    version: &str,
    commit: &str,
    dirty: bool,
    metric: &str,
    unit: &str,
    sorted: &[u128],
    count: Option<i64>,
    date: &str,
) -> Value {
    json!({
        "version": version,
        "commit": commit,
        "dirty": dirty,
        "metric": metric,
        "unit": unit,
        "n": sorted.len(),
        "p50": pctl(sorted, 0.50),
        "p95": pctl(sorted, 0.95),
        "max": sorted.last(),
        "count": count,
        "host": host(),
        "measured_at": date,
        "harness": BENCH_SCHEMA,
    })
}

/// Merge rows into bench.json: the key is (version, metric, host) —
/// a re-measure REPLACES its own row (the trend stamp stance: a row
/// from another toolchain is exactly the one this run re-measured),
/// rows from other versions/hosts are never touched. Rows sort by
/// (version, metric) so the file diffs stably.
pub fn merge_rows(new_rows: Vec<Value>) -> anyhow::Result<()> {
    let path = bench_path();
    let mut doc: Value = match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s)?,
        Err(_) => json!({"schema": BENCH_SCHEMA, "rows": [], "frozen": []}),
    };
    let mut rows: Vec<Value> = serde_json::from_value(doc["rows"].take())?;
    let sv = |r: &Value, k: &str| r[k].as_str().unwrap_or("").to_string();
    for nr in new_rows {
        let key = |r: &Value| (sv(r, "version"), sv(r, "metric"), sv(r, "host"));
        let k = key(&nr);
        rows.retain(|r| key(r) != k);
        rows.push(nr);
    }
    // Version keys sort NUMERICALLY — a string sort files 1.10.0
    // before 1.2.0, and latest() in the renderer trusts this order
    // for its headline version.
    let ver_key = |v: &Value| -> (u64, u64, u64) {
        let mut it = v["version"]
            .as_str()
            .unwrap_or("")
            .split('.')
            .map(|p| p.parse().unwrap_or(0));
        let mut next = || it.next().unwrap_or(0);
        (next(), next(), next())
    };
    rows.sort_by_key(|r| (ver_key(r), r["metric"].as_str().unwrap_or("").to_string()));
    doc["rows"] = Value::Array(rows);
    std::fs::create_dir_all(path.parent().expect("bench dir"))?;
    std::fs::write(&path, serde_json::to_string_pretty(&doc)? + "\n")?;
    Ok(())
}

/// The measured binary's own version line ("ce X.Y.Z") — the row is
/// stamped by what RAN, never by what the harness thinks it built.
pub fn version_of(ce: &Path) -> anyhow::Result<String> {
    let (_, out) = timed(
        {
            let mut c = Command::new(ce);
            c.arg("--version");
            c
        },
        "version",
    )?;
    Ok(out
        .split_whitespace()
        .nth(1)
        .unwrap_or("unknown")
        .to_string())
}

/// today, YYYY-MM-DD (UTC) — provenance, not a comparability key.
pub fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    // civil-date conversion (Howard Hinnant's days algorithm), no deps
    let days = (secs / 86400) as i64;
    let (mut y, mut doy) = (1970i64, days);
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let len = if leap { 366 } else { 365 };
        if doy < len {
            break;
        }
        doy -= len;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let ml = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0usize;
    while doy >= ml[m] {
        doy -= ml[m];
        m += 1;
    }
    format!("{y:04}-{:02}-{:02}", m + 1, doy + 1)
}
