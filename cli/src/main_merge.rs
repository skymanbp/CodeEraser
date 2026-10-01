//! `ce merge` — the clone-merge family's CLI face (plan v2.31 step 7,
//! design booklet §6.4): every clone group judged by the core's merge/1
//! — its parameters, the member kept, the lines saved, feasible or why
//! not — `--group <n>` for one group on the console (the JSON document
//! is always whole). Exit codes: 0 with a judged document, 2 when the
//! core did not judge or an argument is wrong. Advisory: no gate reads
//! it, nothing enters the baseline.

use crate::main_prelude::*;
use codeeraser::merge::{console, face};

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
    let r = match face::run(&root, a.judge.db, core) {
        Ok(r) => r,
        Err(err) => return fail("merge", err),
    };
    if let Some(k) = a.group
        && r.degraded.is_none()
        && k >= r.groups.len()
    {
        let err = anyhow::anyhow!(
            "--group {k}: the document holds {} group(s)",
            r.groups.len()
        );
        return fail("merge", err);
    }
    print_doc(
        json(a.judge.format),
        || face::report_json(&r),
        || {
            for l in console::console(&r, a.group) {
                println!("{l}");
            }
        },
    );
    if r.degraded.is_some() {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}
