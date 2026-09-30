//! `ce flow` — the flow family's CLI face (plan v2.31 step 5, design
//! booklet §5.4): the dead code inside every function of the tree,
//! judged by the core's flow/1. Report-only unless `--check`: then it
//! is its own gate, never `ce check`'s — exit 1 when `[flow] tier` is
//! deny and a judged finding stands (an unused parameter is advisory
//! everywhere and never counts). Exit 2 = a core that could not judge,
//! a malformed reply, an unknown `--kind`, an unreadable ce.toml.

use crate::main_cmds::{fail, json, or_cwd};
use crate::main_judge::JudgeArgs;
use codeeraser::config::Config;
use codeeraser::flow_report::{console, face};
use codeeraser::report::print_doc;
use std::process::ExitCode;

#[derive(clap::Args)]
pub struct FlowArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Exit 1 when `[flow] tier` is deny and a judged finding stands
    /// (unused parameters are advisory and never count)
    #[arg(long)]
    check: bool,
    /// Show only these kinds, comma-separated (unreachable, dead_store,
    /// unused_local, unused_param); the counts stay whole
    #[arg(long, value_delimiter = ',')]
    kind: Vec<String>,
}

pub fn flow_cmd(a: FlowArgs) -> ExitCode {
    let root = or_cwd(a.judge.root);
    let tier = match Config::load(&root) {
        Ok(c) => c.flow.tier().to_string(),
        Err(e) => return fail("flow", anyhow::anyhow!(e)),
    };
    let r = match face::run(&root, &a.judge.core, &a.kind) {
        Ok(r) => r,
        Err(e) => return fail("flow", e),
    };
    print_doc(
        json(a.judge.format),
        || face::report_json(&r),
        || {
            for l in console::console(&r) {
                println!("{l}");
            }
        },
    );
    if r.degraded.is_some() {
        ExitCode::from(2)
    } else if a.check && tier == "deny" && r.judged() > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
