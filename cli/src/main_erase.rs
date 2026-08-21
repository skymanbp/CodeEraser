//! `ce erase` command body (M9 batch 3) — its own file: main_cmds and
//! main_judge both sit at the repo's 300-line gate, and the eraser is
//! a family of its own (plan by default, --apply behind the contract
//! preconditions, --check as the CI zero-rows gate).

use crate::main_cmds::{fail, json, or_cwd};
use crate::main_judge::JudgeArgs;
use codeeraser::erase;
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
}

pub fn erase_cmd(a: EraseArgs) -> ExitCode {
    let root = or_cwd(a.judge.root);
    let plan = match erase::plan(&root, a.judge.db.clone(), &a.judge.core) {
        Ok(p) => p,
        Err(e) => return fail("erase", e),
    };
    if let Err(e) = erase::render::print(&root, &plan, json(a.judge.format)) {
        return fail("erase", e);
    }
    if a.check && plan.counts.eraseable > 0 {
        eprintln!(
            "erase check: {} eraseable row(s) planned",
            plan.counts.eraseable
        );
        return ExitCode::FAILURE;
    }
    if a.apply {
        return match erase::apply_plan(&root, a.judge.db, &a.judge.core, &plan) {
            Ok(n) => {
                println!(
                    "{}",
                    codeeraser::i18n::line(
                        "erase applied: {} row(s); audit trail in .ce/erase-log.ndjson",
                        "擦除已执行：{} 行；审计轨迹在 .ce/erase-log.ndjson",
                        &[&n],
                    )
                );
                ExitCode::SUCCESS
            }
            Err(e) => fail("erase", e),
        };
    }
    ExitCode::SUCCESS
}
