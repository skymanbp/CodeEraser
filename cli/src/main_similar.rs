//! `ce similar` — the same-role advisor's CLI face (plan v2.29 step
//! 6, spec §六): one ask (`--at file:line` / `--text` / `--unit`),
//! the associative view under `--widen`, the document under
//! `--format json`. Split from main_judge.rs at its own size gate.

use crate::main_cmds::{fail, json, or_cwd};
use crate::main_judge::JudgeArgs;
use codeeraser::report::print_doc;
use codeeraser::similar::face;
use codeeraser::similar::query::Ask;
use std::process::ExitCode;

#[derive(clap::Args)]
#[command(group(clap::ArgGroup::new("ask").required(true).args(["at", "text", "unit"])))]
pub struct SimilarArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// The unit holding `file:line` (path relative to the root)
    #[arg(long)]
    at: Option<String>,
    /// Free text: its words as name and doc evidence (no shape or
    /// callee, so the core's role bit stays false)
    #[arg(long)]
    text: Option<String>,
    /// A unit by key (`name/arity`); an ambiguous key refuses by name
    #[arg(long)]
    unit: Option<String>,
    /// Add the associative view: candidates the PPMI-widened query
    /// reaches that the bare query does not, tagged
    #[arg(long)]
    widen: bool,
}

pub fn similar_cmd(a: SimilarArgs) -> ExitCode {
    let ask = match Ask::from_parts(a.at.as_deref(), a.text.as_deref(), a.unit.as_deref()) {
        Ok(ask) => ask,
        Err(err) => return fail("similar", err),
    };
    let j = a.judge;
    match face::run(&or_cwd(j.root), j.db, &j.core, &ask, a.widen) {
        Ok(r) => {
            print_doc(
                json(j.format),
                || face::report_json(&r),
                || {
                    for l in face::console(&r) {
                        println!("{l}");
                    }
                },
            );
            ExitCode::SUCCESS
        }
        Err(err) => fail("similar", err),
    }
}
