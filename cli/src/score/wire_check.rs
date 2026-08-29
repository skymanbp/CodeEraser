//! The reply's self-consistency (plan v2.18 step #14, O32; 6.4.0).
//! `judge` used to check that the core judged with the knob rows it
//! was SENT and nothing else — an echo is a bookkeeping check, not a
//! verdict check, and a same-major core that forgot the fence would
//! have answered `fail: false` with the digest silently ignored.
//! Every invariant here is computable from the request this side
//! built and the reply it parsed, so the function is pure: no disk,
//! no root, no graph — a table of constructed replies tests it whole
//! (tests/unit/score/wire_check.rs), which no fake core could.

use super::knobs;
use super::wire::{Reply, Request};
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::collections::BTreeMap;

/// Every reply, degraded or not, must hold: (1) the fail bit is the
/// disjunction of the named conditions; (2) a degraded reply names
/// `degraded`; (4) the digest echoes in newBaseline on every reply,
/// absent exactly when none was sent; (7) `dropped` is answered
/// exactly when `present` rode. A JUDGED reply must also hold: (3)
/// the fence held exactly when the committed digest and the declared
/// one differ; (5) newBaseline is the document baseline::write will
/// persist; (6) every knob row sent is the one judged with.
pub(crate) fn check_reply(r: &Request, reply: &Reply) -> Result<()> {
    ensure!(
        reply.fail != reply.failed.is_empty(),
        "core said fail={} with failed={:?} — the bit is the disjunction of the names",
        reply.fail,
        reply.failed
    );
    if let Some(reason) = &reply.degraded {
        ensure!(
            reply.failed.iter().any(|c| c == "degraded"),
            "core degraded ({reason}) without naming degraded among {:?}",
            reply.failed
        );
    }
    digest_echo(r.knobs_digest, &reply.new_baseline)?;
    ensure!(
        r.present.is_some() == reply.dropped.is_some(),
        "{}",
        if r.present.is_some() {
            "core answered no ratchet.dropped — a pre-6.4.0 core cannot judge the provenance table"
        } else {
            "core answered ratchet.dropped to a request that sent no present table"
        }
    );
    if reply.degraded.is_some() {
        return Ok(());
    }
    drift_policy(r, &reply.failed)?;
    new_baseline_shape(r, &reply.new_baseline)?;
    knob_echoes(r, reply)
}

/// The fence is Maybe-equality between the committed digest and the
/// declared one — computable here, so a core that skipped the
/// comparison is refused. A null baseline (establish) never drifts;
/// an object baseline without the key recorded None.
fn drift_policy(r: &Request, failed: &[String]) -> Result<()> {
    let recorded = r.baseline.get("knobsDigest").and_then(Value::as_u64);
    let expected = r.baseline.is_object() && recorded != r.knobs_digest;
    let held = failed.iter().any(|c| c == "knobs_digest");
    ensure!(
        held == expected,
        "core {} knobs_digest (baseline recorded {recorded:?}, ce declared {:?})",
        if held { "held" } else { "did not hold" },
        r.knobs_digest
    );
    Ok(())
}

/// The newBaseline echo is the digest the request sent, and absent
/// exactly when none was — the 6.4.0 stance on every reply, the
/// degraded one included (a re-pin from a degraded reply is refused
/// upstream, but the document must never carry a digest it was not
/// told).
fn digest_echo(sent: Option<u64>, new_baseline: &Value) -> Result<()> {
    let echoed = new_baseline.get("knobsDigest").map(Value::as_u64);
    ensure!(
        echoed == sent.map(Some),
        "core echoed knobsDigest {:?} in newBaseline, ce sent {sent:?} — absent must mean none sent",
        new_baseline.get("knobsDigest")
    );
    Ok(())
}

/// newBaseline is what baseline::write persists: an object whose
/// continuous and discrete are arrays, whose softLine is the
/// committed one carried verbatim — or, at establish, derived (>= 1)
/// exactly when the judged LOC set had a positive value to derive
/// it from. `[]` or `{}` used to persist as four nulls.
fn new_baseline_shape(r: &Request, nb: &Value) -> Result<()> {
    ensure!(
        nb.is_object() && nb["continuous"].is_array() && nb["discrete"].is_array(),
        "core answered a newBaseline that is not a baseline document: {nb}"
    );
    let soft = nb.get("softLine").and_then(Value::as_u64);
    if r.baseline.is_null() {
        let derivable = r.judged_loc.iter().any(|&l| l > 0);
        ensure!(
            soft.is_some() == derivable && soft.is_none_or(|s| s >= 1),
            "core derived softLine {:?} from a judged LOC set of {} values",
            nb.get("softLine"),
            r.judged_loc.len()
        );
    } else {
        let carried = r.baseline.get("softLine").and_then(Value::as_u64);
        ensure!(
            soft == carried,
            "core answered softLine {soft:?}, the committed baseline carries {carried:?}"
        );
    }
    Ok(())
}

/// Every knob row sent must be the one judged with (ADR-008 P4; the
/// weights joined at 2.8.0, review C3; the class rows echo whole,
/// 3.1.0). A degraded reply never judged, so it is not asked.
fn knob_echoes(r: &Request, reply: &Reply) -> Result<()> {
    assert_echo(&knobs::CEILING_KEYS, &r.ceilings, &reply.knobs)?;
    assert_echo(&knobs::THRESHOLD_KEYS, &r.thresholds, &reply.knobs)?;
    assert_echo(&knobs::TOLERANCE_KEYS, &r.tolerance, &reply.knobs)?;
    let dw = *reply.knobs.get("defaultWeight").context("defaultWeight")?;
    assert_weights(&r.weights, &reply.weights, dw)?;
    ensure!(
        reply.class_knobs == r.class_knobs,
        "core judged with classKnobs {:?}, ce sent {:?}",
        reply.class_knobs,
        r.class_knobs
    );
    if r.judged_mask != 0 {
        let echoed = *reply.knobs.get("judgedMask").context("judgedMask")?;
        ensure!(
            echoed == r.judged_mask,
            "core echoed judgedMask={echoed}, ce sent {}",
            r.judged_mask
        );
    }
    Ok(())
}

/// Sent [axis, w] rows against the reply's effective weight table:
/// every axis 0..6 must echo the sent override or the default — a
/// dead or mis-indexed channel reddens here at every judged run.
fn assert_weights(sent: &[[i64; 2]], echoed: &[[i64; 2]], default_w: i64) -> Result<()> {
    ensure!(
        echoed.len() == 7,
        "weights echo has {} axes, want 7",
        echoed.len()
    );
    for (axis, row) in echoed.iter().enumerate() {
        let want = sent
            .iter()
            .find(|[c, _]| *c == axis as i64)
            .map(|[_, w]| *w)
            .unwrap_or(default_w);
        ensure!(
            row[0] == axis as i64 && row[1] == want,
            "core weighted axis {axis} at {:?}, ce sent {want}",
            row
        );
    }
    Ok(())
}

/// Sent [code, value] rows against the reply's knob echo, names
/// resolved through the SAME index-is-code registry that assembled
/// them (score::knobs).
fn assert_echo(keys: &[&str], sent: &[[i64; 2]], got: &BTreeMap<String, i64>) -> Result<()> {
    for &[code, v] in sent {
        let key = keys
            .get(code as usize)
            .with_context(|| format!("no echo key for knob code {code}"))?;
        let echoed = got
            .get(*key)
            .with_context(|| format!("reply echo missing {key}"))?;
        ensure!(*echoed == v, "core judged with {key}={echoed}, ce sent {v}");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/score/wire_check.rs"]
mod tests;
