//! The structure document read back (plan v2.32 step 4): the core
//! lays the document out (structure/document.rs, CE.Structure.Document)
//! and this face prints it — the machine faces the bound document
//! itself, the console its lines through i18n::line (English bytes
//! stay identical under the default, CE_LANG=zh picks whole Chinese
//! lines). JSON is never translated.

use crate::i18n::{line, t};
use crate::report::colon_pairs;
use serde::Deserialize;
use serde_json::Value;

/// The document as the console reads it; `doc` is the document itself.
#[derive(Debug, Deserialize)]
pub struct Report {
    pub score: i64,
    #[serde(rename = "scoreScale")]
    pub scale: i64,
    pub entropy: Vec<[i64; 2]>,
    pub axes: Vec<[i64; 2]>,
    pub findings: Vec<Finding>,
    pub dirs: usize,
    pub divergence: Option<i64>,
    pub deviations: Vec<Deviation>,
    #[serde(rename = "declaredDirs")]
    pub declared: usize,
    pub deep: bool,
    pub days: Option<u32>,
    pub split: bool,
    #[serde(rename = "splitCandidates", default)]
    pub split_candidates: Vec<Candidate>,
    #[serde(rename = "sizeExempt", default)]
    pub size_exempt: Vec<Exempt>,
    #[serde(skip)]
    pub doc: Value,
}

#[derive(Debug, Deserialize)]
pub struct Finding {
    pub dir: String,
    pub axis: i64,
}

#[derive(Debug, Deserialize)]
pub struct Deviation {
    pub dir: String,
    pub kind: i64,
}

/// A viable seam: the file, the line it falls after, the unit before
/// it, and both sides of the price.
#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub path: String,
    #[serde(rename = "afterLine")]
    pub after: u64,
    pub unit: String,
    #[serde(rename = "benefitMilli")]
    pub benefit: i64,
    #[serde(rename = "costMilli")]
    pub cost: i64,
}

/// A file past the soft line with no viable seam (0/0 = no seam).
#[derive(Debug, Deserialize)]
pub struct Exempt {
    pub path: String,
    #[serde(rename = "benefitMilli")]
    pub benefit: i64,
    #[serde(rename = "costMilli")]
    pub cost: i64,
}

crate::report::bound!(Report, print_console);

fn print_console(r: &Report) {
    println!(
        "{}",
        line(
            "structure score {}/{} | entropy {} | axes {} | {} dirs",
            "结构分数 {}/{} | 熵 {} | 判轴 {} | {} 目录",
            &[
                &r.score,
                &r.scale,
                &colon_pairs(&r.entropy),
                &colon_pairs(&r.axes),
                &r.dirs,
            ],
        )
    );
    if r.declared > 0 {
        match r.divergence {
            Some(d) => println!(
                "{}",
                line(
                    "layout divergence {}‰ over {} declared dirs",
                    "布局偏离 {}‰（{} 个已声明目录）",
                    &[&d, &r.declared],
                )
            ),
            None => println!(
                "{}",
                line(
                    "layout divergence undefined: mass outside the {} declared dirs",
                    "布局偏离未定义：质量落在 {} 个已声明目录之外",
                    &[&r.declared],
                )
            ),
        }
    }
    print_findings_tail(r);
}

/// The deviation + finding lines (split at the 50-line fn gate).
fn print_findings_tail(r: &Report) {
    for d in &r.deviations {
        let label = if d.kind == 0 {
            t("undeclared territory", "未声明领地")
        } else {
            t("declared but empty", "已声明但为空")
        };
        println!(
            "{}",
            line("deviation {}  {}", "偏离 {}  {}", &[&d.dir, &label])
        );
    }
    for f in &r.findings {
        println!(
            "{}",
            line(
                "finding {}  axis {}",
                "发现 {}  判轴 {}",
                &[&f.dir, &f.axis]
            )
        );
    }
    if r.split {
        print_split(r);
    }
}

/// The v0.6 §C advisory lines: a candidate names its seam and both
/// sides of the price; an exemption is the machine-written why —
/// cohesive length is legitimate, and the numbers say so.
fn print_split(r: &Report) {
    // ROI leads: it is the quantity the core's verdict is made on
    // (viable iff benefit >= cost), with the operands as its receipt
    // — and those are absolute milli-penalty prices, not ratios, so
    // the ‰ glyph comes off them (batch 9 P15; the χ² divergence
    // above keeps ‰, where it is literally a per-mille ratio)
    let roi = |b: i64, c: i64| format!("{:.1}x", b as f64 / c as f64);
    for s in &r.split_candidates {
        println!(
            "{}",
            line(
                "split {}: seam after line {} ({}) — ROI {} (recover {} vs cost {})",
                "拆分 {}：缝在 {} 行后（{}）— ROI {}（回收 {} 对成本 {}）",
                &[
                    &s.path,
                    &s.after,
                    &s.unit,
                    &roi(s.benefit, s.cost),
                    &s.benefit,
                    &s.cost
                ],
            )
        );
    }
    for e in &r.size_exempt {
        let why = if e.cost == 0 {
            t("no seam at all (single unit)", "根本无缝（单一单元）")
        } else {
            t("cohesive: best seam under ROI 1", "内聚：最优缝 ROI 不足 1")
        };
        println!(
            "{}",
            line(
                "size-exempt {}: {} (recover {} vs cost {})",
                "尺寸豁免 {}：{}（回收 {} 对成本 {}）",
                &[&e.path, &why, &e.benefit, &e.cost],
            )
        );
    }
}
