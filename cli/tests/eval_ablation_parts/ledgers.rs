//! The ablation's two itemized ledgers — the review food behind the
//! matrix numbers. A summary count says a variant dropped N sites;
//! the ledger says WHICH, with content, so the winner decision reads
//! evidence rather than totals.
#![allow(dead_code)]

use super::variants::{Ctx, QUALITY_ALNUM, anchor_alnum};
use super::{Mark, ShadowBlock};
use crate::eval_l2_parts as parts;
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Sites the quality floor drops. On the self corpus every row is a
/// potential recall hole (547/547 is a hard bar); on requests the
/// invented stations must appear here for the floor to win.
pub fn kill_ledger(
    sha: &str,
    texts: &parts::Texts,
    blocks: &[ShadowBlock],
    ctx: &Ctx,
) -> Vec<Value> {
    blocks
        .iter()
        .filter(|b| anchor_alnum(b, ctx) < QUALITY_ALNUM)
        .map(|b| {
            let lines: Vec<&str> = b.hashes.iter().map(|h| ctx.contents[h].as_str()).collect();
            json!({
                "sha": &sha[..9],
                "from": texts[b.from_pair].2["before"],
                "to": texts[b.to_pair].2["after"],
                "max_alnum": anchor_alnum(b, ctx) as u64,
                "lines": lines,
            })
        })
        .collect()
}

/// Lines only the edge-FREE deletion-side attribution marks (attack
/// review F4's width), with GT membership: true/false where the GT
/// carries line identities, null on reviewed-correction files (count
/// GT cannot say which line), false where the file has no cross GT.
pub fn width_ledger(
    sha: &str,
    texts: &parts::Texts,
    gt: &parts::CrossGt,
    base_outs: &BTreeSet<Mark>,
    edge_outs: &BTreeSet<Mark>,
) -> Vec<Value> {
    base_outs
        .difference(edge_outs)
        .map(|&(p, l)| {
            let file = texts[p].2["before"].as_str().expect("before");
            let text: Vec<&str> = texts[p].0.lines().collect();
            let key = ("out".to_string(), file.to_string());
            let in_gt = match gt.lines.get(&key) {
                Some(want) => json!(want.contains(&l)),
                None if gt.counts.contains_key(&key) => Value::Null,
                None => json!(false),
            };
            json!({"sha": &sha[..9], "file": file, "line": l as u64,
                   "content": text[l - 1].trim(), "in_gt": in_gt})
        })
        .collect()
}

/// The width ledger's summary shape, re-derivable by the CI gate.
pub fn width_summary(width: &[Value]) -> Value {
    let count = |want: &Value| width.iter().filter(|r| r["in_gt"] == *want).count() as u64;
    json!({
        "lines": width.len() as u64,
        "in_gt": count(&json!(true)),
        "extra": count(&json!(false)),
        "unknown": count(&Value::Null),
    })
}
