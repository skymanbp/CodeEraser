//! The PreToolUse leg of the flow class (plan v2.31 step 5; design
//! booklet docs/reference/analysis-track.md §5.4): this write's
//! (on-disk, applied) pair, each side lowered (flow::lower) and judged
//! over the daemon's core link (flow/1), the findings placed through
//! each side's legend, and the after side's findings the before side
//! does not account for (flow_novel.rs) counted as NOVEL — the guard's
//! own semantics: a finding the file already carried is not this
//! write's doing. Only a judged language's non-advisory findings make
//! the condition; the class speaks at its OWN tier (`[flow] tier`,
//! default observe) and never on a degraded side. A `flow` line lands
//! when the after side has a finding or a side could not be judged
//! (no core, a daemon without the family: named, never a decision).

use super::envelope::Envelope;
use super::flow_novel;
use crate::config::{Config, FLOW_DEFAULT};
use crate::daemon::client;
use crate::daemon::proto::{FlowTables, Request, Response};
use crate::flow::lower::{Lowered, lower_file};
use crate::flow::wire;
use crate::flow_report::{self as fr, Placed};
use serde_json::{Value, json};
use std::path::Path;

/// What the leg leaves for the hook: the feed line, and the class's
/// reason at its own tier when the condition holds.
pub(super) struct Pending {
    pub line: Value,
    pub speak: Option<(&'static str, String)>,
}

/// Measure and judge one Write/Edit: a judged language with a flow
/// table, inside the config's walk, whose applied text parses into at
/// least one unit. A new file's before side is empty.
pub(super) fn observe(root: &Path, env: &Envelope, cfg: Option<&Config>) -> Option<Pending> {
    let (lang, before_text) = env.judged_pair(root, cfg)?;
    let after = lower_file(&super::budget::resulting_text(env)?, lang)?;
    if after.units.is_empty() {
        return None;
    }
    let before = lower_file(&before_text, lang)?;
    let (flow, novel) = match (placed(root, &before), placed(root, &after)) {
        (Ok(_), Ok(a)) if a.is_empty() => return None,
        (Ok(b), Ok(a)) => measured(&b, &a, &after),
        (Err(why), _) | (_, Err(why)) => (degraded(&after, &why), Vec::new()),
    };
    let tier = tier(cfg);
    let file = &env.tool_input.file_path;
    let armed = tier != FLOW_DEFAULT && fr::lang_judged(lang);
    let speak = armed
        .then(|| super::say::flow_novel(file, &novel.iter().collect::<Vec<_>>()))
        .flatten()
        .map(|why| (tier, why));
    let line = json!({ "event": "flow", "file": file, "mode": tier, "flow": flow });
    Some(Pending { line, speak })
}

/// The feed object over two judged sides, and the write's novel
/// non-advisory findings (the condition, when the language is judged).
fn measured(b: &[Placed], a: &[Placed], after: &Lowered) -> (Value, Vec<Placed>) {
    let novel: Vec<Placed> = flow_novel::novel(b, a)
        .into_iter()
        .filter(|p| !fr::advisory(p.kind))
        .cloned()
        .collect();
    let obj = json!({
        "units": after.units.len(),
        "before": b.len(),
        "after": a.len(),
        "novel": novel.len(),
        "kinds": fr::kinds_json(a.iter().map(|p| p.kind)),
        "judged": fr::lang_judged(after.lang),
    });
    (obj, novel)
}

fn degraded(after: &Lowered, why: &str) -> Value {
    json!({
        "units": after.units.len(),
        "judged": fr::lang_judged(after.lang),
        "degraded": why,
    })
}

/// One side's findings, placed: each request batch of its units asked
/// over the daemon (the tables flow::wire::body assembled, forwarded
/// as they are), the reply consumed against what was sent. A side with
/// no unit asks nothing; a degraded reply is its named reason.
fn placed(root: &Path, file: &Lowered) -> Result<Vec<Placed>, String> {
    let files = std::slice::from_ref(file);
    let mut out = Vec::new();
    for sent in wire::plan(files).0 {
        let tables: FlowTables =
            serde_json::from_value(wire::body(&sent)).map_err(|e| e.to_string())?;
        let reply = match client::request(root, &Request::Flow(tables)) {
            Ok(Response::FlowReport { reply }) => reply,
            Ok(other) => return Err(format!("daemon answered {other:?}")),
            Err(e) => return Err(format!("daemon: {e}")),
        };
        crate::corelink::judged::degraded(&reply)?;
        let judged = wire::consume(&reply, &sent)?;
        let nth = |f: &wire::Finding| sent.units[f.u].0.nth;
        out.extend(
            judged
                .findings
                .iter()
                .filter_map(|f| fr::place(file, nth(f), f)),
        );
    }
    Ok(out)
}

/// The class's tier as declared (valid by load), or the route default.
fn tier(cfg: Option<&Config>) -> &'static str {
    crate::config::static_tier(cfg.map_or(FLOW_DEFAULT, |c| c.flow.tier()), FLOW_DEFAULT)
}
