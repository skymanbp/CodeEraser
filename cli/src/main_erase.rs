//! `ce erase` command body (M9 batch 3) — its own file: main_cmds and
//! main_judge both sit at the repo's 300-line gate, and the eraser is
//! a family of its own (plan by default, --apply behind the contract
//! preconditions, --check as the CI zero-rows gate).

use crate::main_cmds::{fail, json, or_cwd};
use crate::main_judge::JudgeArgs;
use crate::main_prelude::{answered, whole};
use codeeraser::erase::{self, document, document::Run};
use std::process::ExitCode;

#[derive(clap::Args)]
pub struct EraseArgs {
    #[command(flatten)]
    pub(crate) judge: JudgeArgs,
    /// Actually erase what the plan names (requires a git repository,
    /// a clean worktree, and unchanged targets; default is dry-run)
    #[arg(long)]
    pub(crate) apply: bool,
    /// Gate mode: exit 1 when the plan holds ANY eraseable row (the
    /// self-repo keeps itself clean)
    #[arg(long)]
    pub(crate) check: bool,
    /// Read the audit trail of applied erases (.ce/erase-log.ndjson)
    /// instead of planning; exit 1 when a line cannot be read
    #[arg(long, conflicts_with_all = ["apply", "check"])]
    pub(crate) log: bool,
}

pub fn erase_cmd(a: EraseArgs, core: &str) -> ExitCode {
    let root = or_cwd(a.judge.root);
    let as_json = json(a.judge.format);
    if a.log {
        let answer = erase::log::read(&root).and_then(|l| document::trail_answer(core, &l));
        return answered("erase", "erase-trail", as_json, answer, whole);
    }
    // the diff is rendered before anything is applied: the hash check
    // names a file that moved since planning, and an applied file no
    // longer holds the bytes the plan showed
    let planned = erase::planned(&root, a.judge.db.clone(), core)
        .and_then(|(p, held)| Ok((document::Diffs::of(&root, &p)?, p, held)));
    let (diffs, plan, held) = match planned {
        Ok(planned) => planned,
        Err(e) => return fail("erase", e),
    };
    let mut run = Run {
        check: a.check,
        applied: None,
    };
    // `--check` with an eraseable row is the veto, and a vetoed plan is
    // never applied
    let vetoed = a.check && plan.counts.eraseable > 0;
    let applied = (a.apply && !vetoed).then(|| erase::apply_plan(&root, a.judge.db, core, &plan));
    let refused = match applied {
        Some(Ok(n)) => {
            run.applied = Some(n);
            None
        }
        Some(Err(e)) => Some(e),
        None => None,
    };
    let answer = document::answer(core, held, &plan, &diffs, run);
    let code = answered("erase", "erase", as_json, answer, whole);
    match refused {
        Some(e) => fail("erase", e),
        None => code,
    }
}
