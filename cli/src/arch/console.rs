//! The console face of the arch document (design booklet §7.3): one
//! counts line, the layers from the top level down, every cut arc
//! with the file references under it, the misplaced files, the impact
//! walk when a focus was named, and the metrics table. An empty
//! section says so in one sentence; a degraded document is one line
//! naming why. Every sentence has its Chinese twin (i18n::line); the
//! paths stay as written.

use super::face::slashed;
use super::report::Report;
use crate::i18n::line;
use std::collections::BTreeMap;

pub fn console(r: &Report) -> Vec<String> {
    if let Some(why) = &r.degraded {
        return vec![line("arch: degraded — {}", "arch：已降级——{}", &[why])];
    }
    let c = &r.counts;
    let mut out = vec![line(
        "{} files, {} directories, {} file references, {} package references: {} cut(s), {} cluster(s), {} misplaced",
        "{} 个文件、{} 个目录、{} 条文件引用、{} 条包引用：{} 条切边、{} 个簇、{} 个错位",
        &[
            &c["files"],
            &c["dirs"],
            &c["edges"],
            &c["pkgEdges"],
            &c["cuts"],
            &c["clusters"],
            &c["misplaced"],
        ],
    )];
    out.extend(layers(r));
    out.extend(cuts(r));
    out.extend(misplaced(r));
    out.extend(impact(r));
    out.extend(metrics(r));
    out
}

fn shown(dir: &str) -> &str {
    if dir.is_empty() { "." } else { dir }
}

/// One line per level, the highest first.
fn layers(r: &Report) -> Vec<String> {
    let mut by: BTreeMap<i64, Vec<&str>> = BTreeMap::new();
    for l in &r.layers {
        by.entry(l.level).or_default().push(shown(&l.dir));
    }
    by.iter()
        .rev()
        .map(|(level, dirs)| line("level {}: {}", "第 {} 层：{}", &[level, &dirs.join(", ")]))
        .collect()
}

fn cuts(r: &Report) -> Vec<String> {
    if r.cuts.is_empty() {
        return vec![line("no cycles among directories", "目录之间没有环", &[])];
    }
    let mut out = Vec::new();
    for c in &r.cuts {
        let how = if c.exact {
            line("exact", "精确", &[])
        } else {
            line("greedy", "贪心", &[])
        };
        out.push(line(
            "cut: {} → {}  ({} refs, {})",
            "切边：{} → {}（{} 处引用，{}）",
            &[&slashed(&c.from), &slashed(&c.to), &c.refs, &how],
        ));
        for f in &c.files {
            out.push(format!("  {} → {}  ({})", f.from, f.to, f.refs));
        }
    }
    out
}

fn misplaced(r: &Report) -> Vec<String> {
    if r.misplaced.is_empty() {
        return vec![line("no misplaced file", "没有错位的文件", &[])];
    }
    r.misplaced
        .iter()
        .map(|m| {
            line(
                "misplaced: {} (in {}, cluster majority {})",
                "错位：{}（在 {}，簇多数在 {}）",
                &[&m.path, &shown(&m.dir), &shown(&m.majority)],
            )
        })
        .collect()
}

/// Only when a focus was named: the files a change reaches, by depth.
fn impact(r: &Report) -> Vec<String> {
    if r.counts["focus"] == 0 {
        return Vec::new();
    }
    let mut out = vec![line("impact (depth):", "影响面（深度）：", &[])];
    out.extend(
        r.impact
            .iter()
            .map(|i| format!("  {}  {}", i.path, i.depth)),
    );
    out
}

/// `dir  fanIn  fanOut  instability`, `-` where nothing touches the
/// directory; the directory column padded to its widest.
fn metrics(r: &Report) -> Vec<String> {
    let width = r
        .metrics
        .iter()
        .map(|m| shown(&m.dir).len())
        .max()
        .unwrap_or(3)
        .max(3);
    let mut out = vec![format!("{:width$}  fanIn  fanOut  instability", "dir")];
    for m in &r.metrics {
        let i = m
            .instability
            .map_or_else(|| "-".to_string(), |i| i.to_string());
        out.push(format!(
            "{:width$}  {:>5}  {:>6}  {:>11}",
            shown(&m.dir),
            m.fan_in,
            m.fan_out,
            i
        ));
    }
    out
}
