//! The dedup report faces as the core lays them out (plan v2.32 step
//! 5; CE.Dedup.Document, CE.Dedup.Lines): this side sends the verified
//! blocks, the groups and their members as integers over one path
//! table, the walk's counters and the two report filters, and under
//! `--check` the budget and verdict/1's fail bit; the document, the
//! console lines (the ratchet's among them) and the exit bit come back.
//! The SARIF face re-spells the bound document's blocks.

use super::{Summary, budget::Gate, pairs};
use crate::document::{self, Answer, Paths, Request};
use crate::sarif::{location, num, text};
use crate::scan::Format;
use anyhow::Result;
use serde_json::Value;

/// The dedup document, its lines and its veto for one measurement;
/// `gate` is the `--check` judgment when one ran.
pub fn answer(
    core: &str,
    found: &pairs::Blocks,
    s: &Summary,
    gate: Option<Gate>,
) -> Result<Answer> {
    let mut paths = Paths::default();
    let blocks: Vec<[i64; 8]> = found
        .blocks
        .iter()
        .map(|b| {
            [
                paths.id(&b.a_file),
                b.a_start as i64,
                b.a_end as i64,
                paths.id(&b.b_file),
                b.b_start as i64,
                b.b_end as i64,
                b.tokens as i64,
                b.distinct as i64,
            ]
        })
        .collect();
    let mut members = Vec::new();
    let groups: Vec<[i64; 3]> = found
        .groups
        .iter()
        .enumerate()
        .map(|(g, group)| {
            for m in &group.members {
                members.push([g as i64, paths.id(&m.file), m.start as i64, m.end as i64]);
            }
            [g as i64, group.blocks as i64, group.tokens as i64]
        })
        .collect();
    let gate = gate.unwrap_or_default();
    let req = Request::new("dedup")
        .range("paths", paths.list.len())
        .range("groups", groups.len())
        .fact("files", s.files)
        .fact("refreshed", s.refreshed)
        .fact("removed", s.removed)
        .fact("hot_chained", found.hot_chained)
        .fact("stale_skipped", found.stale_skipped)
        .fact("low_diversity_suppressed", found.low_diversity_suppressed)
        .fact("min_tokens", s.min_tokens)
        .fact("min_distinct", s.min_distinct)
        .fact("check", u8::from(gate.checked))
        .fact("budget", gate.budget)
        .fact("fail", u8::from(gate.fail))
        .rows("blocks", blocks)
        .rows("groups", groups)
        .rows("members", members);
    document::assemble(core, req.text("path", &paths.list))
}

/// The answer in the requested form: console lines, the document, or
/// its SARIF projection — each followed by the stream-1 lines.
pub(super) fn emit(format: Format, answer: &Answer) -> Result<()> {
    match format {
        Format::Console => document::emit("dedup", answer, false),
        Format::Json => document::emit("dedup", answer, true),
        Format::Sarif => {
            document::emit_projected(
                answer,
                &crate::sarif::projected(&answer.document, "blocks", block)?,
            );
            Ok(())
        }
    }
}

/// One clone block of the bound document as a SARIF "note" (the
/// `--format sarif` face, sarif::projected) — blocks are budget-gated
/// facts, not per-block failures, so the ratchet's tolerated debt must
/// not masquerade as errors on a scanning dashboard. The message is the
/// console face's English line verbatim (SARIF is a machine face,
/// never translated).
fn block(b: &Value) -> Value {
    let (a, bf) = (text(b, "a_file"), text(b, "b_file"));
    let (a_at, b_at) = (
        (num(b, "a_start"), num(b, "a_end")),
        (num(b, "b_start"), num(b, "b_end")),
    );
    let message = format!(
        "dup {a}:{}-{} <-> {bf}:{}-{} ({} tokens)",
        a_at.0,
        a_at.1,
        b_at.0,
        b_at.1,
        num(b, "tokens")
    );
    let (here, there) = (location(a, a_at.0, a_at.1), location(bf, b_at.0, b_at.1));
    crate::sarif::result("ce.dedup/clone-block", "note", &message, here, vec![there])
}

/// The dedup report as a document, for the faces that hold no core
/// path of their own (the daemon's `dedup` reply, the MCP tool and the
/// GUI's `faces::dedup`): the process's core answers it.
pub fn report_json(found: &pairs::Blocks, s: &Summary) -> Result<Value> {
    Ok(answer(crate::tables::core_flag(), found, s, None)?.document)
}
