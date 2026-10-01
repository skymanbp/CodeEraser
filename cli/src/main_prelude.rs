//! The imports every judged-family CLI face opens with (plan v2.31
//! steps 5-9): the argument helpers, the shared judge arguments, the
//! document printer and the exit code; and, since plan v2.32 step 3,
//! the one road a core-laid document takes to the terminal. `ce flow`, `ce query` /
//! `ce rules`, `ce similar`, `ce merge` and `ce arch` each read these four names
//! and then their own family's modules; the four lines lived at the
//! head of every face until the clone gate read two heads as one block
//! (main_flow.rs against main_merge.rs), and one owner is the fix, not
//! a reordered import. main_judge.rs is at its own size line, so the
//! imports live here.

pub(crate) use crate::main_cmds::{fail, json, or_cwd};
pub(crate) use crate::main_judge::JudgeArgs;
pub(crate) use codeeraser::report::print_doc;
pub(crate) use std::process::ExitCode;

/// A document face (plan v2.32 step 3): the bound document read into
/// its reader, the exit code the face's own rule reads off the reader,
/// then the document printed as it is under `--format json` or as the
/// console's lines. A run, a read or a rule that refused the reader
/// (merge's `--group` past the last group) is `ce <name>: why`, exit
/// 2, and nothing on stdout.
pub(crate) fn document_face<T: serde::de::DeserializeOwned>(
    name: &str,
    doc: anyhow::Result<serde_json::Value>,
    as_json: bool,
    console: impl FnOnce(&T) -> Vec<String>,
    exit: impl FnOnce(&T) -> anyhow::Result<ExitCode>,
) -> ExitCode {
    let read = doc.and_then(|doc| {
        let r = T::deserialize(&doc)?;
        let code = exit(&r)?;
        Ok((doc, r, code))
    });
    match read {
        Ok((doc, r, code)) => {
            print_doc(
                as_json,
                || doc,
                || console(&r).iter().for_each(|l| println!("{l}")),
            );
            code
        }
        Err(err) => fail(name, err),
    }
}
