//! The imports every judged-family CLI face opens with (plan v2.31
//! steps 5-9): the argument helpers, the shared judge arguments and the
//! exit code; and, since plan v2.32 step 5, the one road a core-laid
//! document takes to the terminal (`answered`). `ce flow`, `ce query` /
//! `ce rules`, `ce similar`, `ce merge` and `ce arch` each read these four names
//! and then their own family's modules; the four lines lived at the
//! head of every face until the clone gate read two heads as one block
//! (main_flow.rs against main_merge.rs), and one owner is the fix, not
//! a reordered import. main_judge.rs is at its own size line, so the
//! imports live here.

pub(crate) use crate::main_cmds::{fail, json, or_cwd};
pub(crate) use crate::main_judge::JudgeArgs;
pub(crate) use std::process::ExitCode;

use codeeraser::document::Answer;
use serde_json::Value;

/// A face the core answered (plan v2.32 step 5, rulings R3 / R4): the
/// answer printed — its lines on the console, the document under
/// `--format json` — then the exit: 2 when the face's own rule reads
/// the bound document as a judgment that did not happen (`unjudged`),
/// else 1 iff the core's veto. A run, or a rule that refused the
/// document before anything printed (merge's `--group` past the last
/// group), is `ce <name>: why`, exit 2, and nothing on stdout — `name`
/// the command, `family` the document's (`ce erase --log` lays out
/// `erase-trail`, `ce clone --units` `clone-units`).
pub(crate) fn answered(
    name: &str,
    family: &str,
    as_json: bool,
    answer: anyhow::Result<Answer>,
    unjudged: impl FnOnce(&Value) -> anyhow::Result<bool>,
) -> ExitCode {
    let shown = answer.and_then(|a| {
        let unjudged = unjudged(&a.document)?;
        codeeraser::document::emit(family, &a, as_json)?;
        Ok(if unjudged { 2 } else { u8::from(a.fail) })
    });
    match shown {
        Ok(code) => ExitCode::from(code),
        Err(err) => fail(name, err),
    }
}

/// A family whose document is never unjudged on its own reading: the
/// core's veto alone decides the exit.
pub(crate) fn whole(_: &Value) -> anyhow::Result<bool> {
    Ok(false)
}

/// A document whose `degraded` names a reason: the judgment did not
/// happen (arch, flow, merge).
pub(crate) fn degraded(doc: &Value) -> anyhow::Result<bool> {
    Ok(!doc["degraded"].is_null())
}
