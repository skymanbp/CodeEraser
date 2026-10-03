//! `ce merge` — the clone-merge family's CLI face (plan v2.31 step 7,
//! design booklet §6.4): every clone group judged by the core's merge/1
//! — its parameters, the member kept, the lines saved, feasible or why
//! not — `--group <n>` for one group on the console (the JSON document
//! is always whole). Exit codes: 0 with a judged document, 2 when the
//! core did not judge, did not lay the document out, or an argument is
//! wrong. Advisory: no gate reads
//! it, nothing enters the baseline.

use crate::main_prelude::*;
use codeeraser::merge::face;

#[derive(clap::Args)]
pub struct MergeArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Print one group alone on the console (its number in the
    /// document); the JSON document is always whole
    #[arg(long)]
    group: Option<usize>,
}

pub fn merge_cmd(a: MergeArgs, core: &str) -> ExitCode {
    let root = or_cwd(a.judge.root);
    let group = a.group;
    let answer = face::run(&root, a.judge.db, core, group);
    answered("merge", "merge", json(a.judge.format), answer, |doc| {
        let held = doc["groups"].as_array().map_or(0, Vec::len);
        match group {
            Some(k) if !degraded(doc)? && k >= held => Err(anyhow::anyhow!(
                "--group {k}: the document holds {held} group(s)"
            )),
            _ => degraded(doc),
        }
    })
}
