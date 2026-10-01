//! The console face of the query document (booklet §4.6): the
//! errors first (a program with an error answers nothing), the
//! degraded reason if the core did not judge, then each goal — a
//! question with its column names and answers, an assertion with its
//! witnesses — every proof row indented by its depth in the
//! derivation, and one counts line. Every sentence has its Chinese
//! twin (i18n::line); the program's own words (predicates, columns,
//! paths) stay as written.

use super::report::{AnswerFace, ProofFace, Report};
use crate::i18n::line;

pub fn console(r: &Report) -> Vec<String> {
    let mut out = Vec::new();
    for e in &r.errors {
        out.push(line("error at {}: {}", "错误 {}：{}", &[&e.at, &e.what]));
    }
    if let Some(why) = &r.degraded {
        out.push(line(
            "query: degraded — {} (no judgment)",
            "查询：已降级——{}（未判决）",
            &[why],
        ));
    }
    if !r.judged() {
        return out;
    }
    for g in &r.goals {
        let rows: Vec<&AnswerFace> = r.answers.iter().filter(|a| a.goal == g.goal).collect();
        let cols = g.columns.join(", ");
        out.push(if g.kind == "assert" {
            let name = g.name.clone().unwrap_or_default();
            if rows.is_empty() {
                line("assert {}({}): ok", "断言 {}({})：通过", &[&name, &cols])
            } else {
                line(
                    "assert {}({}): {} violation(s)",
                    "断言 {}({})：{} 条违规",
                    &[&name, &cols, &rows.len()],
                )
            }
        } else {
            line(
                "?- {}: {} answer(s)",
                "?- {}：{} 个答案",
                &[&cols, &rows.len()],
            )
        });
        for (i, a) in rows.iter().enumerate() {
            out.push(format!("  {}", a.values.join("  ")));
            out.extend(chain(&r.proof, g.goal, i));
        }
    }
    out.push(line(
        "{} rule(s), {} fact row(s), {} derived, {} proof node(s){}",
        "{} 条规则、{} 行事实、{} 个派生元组、{} 个推导结点{}",
        &[
            &r.counts["rules"],
            &r.counts["facts"],
            &r.counts["derived"],
            &r.counts["proofNodes"],
            &truncated(r),
        ],
    ));
    out
}

/// One answer's derivation, each node under its parent.
fn chain(proof: &[ProofFace], goal: usize, answer: usize) -> Vec<String> {
    let rows: Vec<&ProofFace> = proof
        .iter()
        .filter(|p| p.goal == goal && p.answer == answer)
        .collect();
    let depth_of = |node: i64| -> usize {
        let mut depth = 0;
        let mut at = node;
        while let Some(p) = rows.iter().find(|p| p.node == at) {
            if p.parent < 0 {
                break;
            }
            depth += 1;
            at = p.parent;
        }
        depth
    };
    rows.iter()
        .filter(|p| p.parent >= 0)
        .map(|p| {
            let by = if p.rule < 0 {
                line(" (fact)", "（事实）", &[])
            } else {
                line(" (clause {})", "（第 {} 条子句）", &[&p.rule])
            };
            format!(
                "{}{}({}){by}",
                "  ".repeat(depth_of(p.node) + 1),
                p.pred,
                p.args.join(", ")
            )
        })
        .collect()
}

fn truncated(r: &Report) -> String {
    match r.counts.get("proofTruncated") {
        Some(n) if *n > 0 => line(
            "; {} proof tree(s) past the budget left out",
            "；{} 棵推导树超预算未出",
            &[n],
        ),
        _ => String::new(),
    }
}
