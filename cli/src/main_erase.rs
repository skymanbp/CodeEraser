//! `ce erase` command body (M9 batch 3) — its own file: main_cmds and
//! main_judge both sit at the repo's 300-line gate, and the eraser is
//! a family of its own (plan by default, --apply behind the contract
//! preconditions, --check as the CI zero-rows gate).

use crate::main_cmds::{fail, json, or_cwd};
use crate::main_judge::JudgeArgs;
use crate::main_prelude::{answered, whole};
use codeeraser::erase;
use codeeraser::faces::{self, Plumb};
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
    let j = a.judge;
    let root = or_cwd(j.root);
    let as_json = json(j.format);
    if a.log {
        let answer = faces::erase_log(&root, core);
        return answered("erase", "erase-trail", as_json, answer, whole);
    }
    // the one mutation, handed to the read-only face as a closure: the
    // face plans, renders the diff, holds a `--check` veto, then asks
    // this closure — and reports the count it returns
    let db = j.db.clone();
    let apply = |plan: &erase::Plan, _: &erase::document::Diffs| {
        a.apply.then(|| erase::apply_plan(&root, db, core, plan))
    };
    let gated = faces::erase(&root, core, Plumb::gate(j.db, a.check), apply);
    let (answer, refused) = match gated {
        Ok((answer, refused)) => (Ok(answer), refused),
        Err(e) => return fail("erase", e),
    };
    let code = answered("erase", "erase", as_json, answer, whole);
    match refused {
        Some(e) => fail("erase", e),
        None => code,
    }
}
