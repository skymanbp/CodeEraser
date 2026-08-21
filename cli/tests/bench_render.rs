//! The bench dashboard's three rendered faces are GENERATED from
//! contracts/bench/bench.json and byte-gated here (the docs_gate
//! pattern: CE_BLESS=1 rewrites, a plain run compares) — a
//! hand-edited surface number re-opens the D7 defect class the plan
//! forbids ("禁手填"). docs/BENCH.md is written whole; the two site
//! pages own only the block between their bench markers.

mod bench_support;
mod common;

use serde_json::Value;

fn doc() -> Value {
    let text = std::fs::read_to_string(bench_support::bench_path()).expect("bench.json");
    serde_json::from_str(&text).expect("bench.json parses")
}

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v[k].as_str().unwrap_or("")
}

/// docs/BENCH.md, whole file.
fn render_md(d: &Value) -> String {
    let mut out = String::from(
        "# Benchmarks — replayed, never hand-filled\n\n\
         > Generated from [contracts/bench/bench.json](../contracts/bench/bench.json) by\n\
         > `cli/tests/bench_render.rs` (`CE_BLESS=1` to regenerate). Every series row was\n\
         > measured by `cli/tests/bench.rs` (`bench_append` for a checkout, `bench_backfill`\n\
         > per release tag — each tag's OWN binaries, release builds only, fresh index per\n\
         > cold run). Frozen points carry their sealed-ledger source; points that cannot\n\
         > honestly become a per-version series say why in their epoch clause.\n\n\
         ## Latency series (self repository)\n\n\
         | version | metric | p50 ms | p95 ms | n | host | measured |\n|---|---|---|---|---|---|---|\n",
    );
    for r in d["rows"].as_array().expect("rows") {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {}{} |\n",
            s(r, "version"),
            s(r, "metric"),
            r["p50"],
            r["p95"],
            r["n"],
            s(r, "host"),
            s(r, "measured_at"),
            if r["dirty"] == Value::Bool(true) {
                " (dirty)"
            } else {
                ""
            },
        ));
    }
    out.push_str("\n## Frozen evaluation points\n\n| metric | value | source |\n|---|---|---|\n");
    for f in d["frozen"].as_array().expect("frozen") {
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            s(f, "metric"),
            s(f, "value"),
            s(f, "source"),
        ));
    }
    out.push_str("\nPer-point detail, freeze dates and epoch clauses live in the JSON itself.\n");
    out
}

/// The newest series version present (rows are sorted by version).
fn latest(d: &Value) -> &str {
    d["rows"]
        .as_array()
        .and_then(|r| r.last())
        .map(|r| s(r, "version"))
        .unwrap_or("")
}

fn latest_p50(d: &Value, metric: &str) -> String {
    let v = latest(d);
    d["rows"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|r| s(r, "version") == v && s(r, "metric") == metric)
        .map(|r| r["p50"].to_string())
        .unwrap_or_else(|| "—".into())
}

/// A frozen point's value BY LOOKUP — the first draft hard-coded the
/// two frozen chips in this renderer, which is precisely the
/// hand-filled surface the whole batch exists to forbid.
fn frozen_value(d: &Value, metric: &str) -> String {
    d["frozen"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|f| s(f, "metric") == metric)
        .map(|f| s(f, "value").to_string())
        .unwrap_or_else(|| "—".into())
}

/// The site's bench block (between the page's bench markers). One
/// renderer, two languages — the labels differ, the numbers cannot.
fn render_site(d: &Value, zh: bool) -> String {
    let v = latest(d);
    let chip = |label: &str, val: String, unit: &str| {
        format!(
            "<div class=\"install\"><span class=\"k\">{label}</span><code>{val} {unit}</code></div>\n"
        )
    };
    let (h, note, hook, scan, dedup, fpr, prec) = if zh {
        (
            "实测",
            "每个数字由回放产生（cli/tests/bench.rs），单源 contracts/bench/bench.json——绝不手填。",
            "hook 探针 p50",
            "全仓扫描 p50",
            "增量索引 p50 (warm)",
            "守卫误报（人裁 630 事件）",
            "docdup 判准（冻结黄金集）",
        )
    } else {
        (
            "Measured",
            "Every number is produced by replay (cli/tests/bench.rs) from one source, contracts/bench/bench.json — never hand-filled.",
            "hook probe p50",
            "full scan p50",
            "incremental index p50 (warm)",
            "guard false positives (630 arbitrated events)",
            "docdup precision (frozen golden set)",
        )
    };
    format!(
        "<h2>{h} · v{v}</h2>\n<div class=\"installs\">\n{}{}{}{}{}</div>\n<p class=\"cap\">{note}</p>\n",
        chip(hook, latest_p50(d, "hook_probe"), "ms"),
        chip(scan, latest_p50(d, "scan"), "ms"),
        chip(dedup, latest_p50(d, "dedup_warm"), "ms"),
        chip(fpr, frozen_value(d, "guard_fpr_per500"), ""),
        chip(prec, frozen_value(d, "docdup_d3_precision"), ""),
    )
}

/// Splice one page's marker block; CE_BLESS writes, a plain run
/// byte-compares the block only (the rest of the page is authored).
fn gate_site(path: &str, zh: bool) {
    let (b, e) = ("<!-- bench:begin -->", "<!-- bench:end -->");
    let page = std::fs::read_to_string(path)
        .expect(path)
        .replace("\r\n", "\n");
    let (head, rest) = page
        .split_once(b)
        .unwrap_or_else(|| panic!("{path}: no {b}"));
    let (block, tail) = rest
        .split_once(e)
        .unwrap_or_else(|| panic!("{path}: no {e}"));
    let want = format!("\n{}", render_site(&doc(), zh));
    if std::env::var("CE_BLESS").as_deref() == Ok("1") {
        std::fs::write(path, format!("{head}{b}{want}{e}{tail}")).expect("bless site");
        return;
    }
    assert_eq!(
        block, want,
        "{path}: bench block drifted — CE_BLESS=1 to regenerate"
    );
}

#[test]
fn bench_md_matches_its_regeneration() {
    common::assert_matches_golden(&render_md(&doc()), std::path::Path::new("../docs/BENCH.md"));
}

#[test]
fn site_bench_blocks_match() {
    gate_site("../site/index.html", false);
    gate_site("../site/zh/index.html", true);
}
