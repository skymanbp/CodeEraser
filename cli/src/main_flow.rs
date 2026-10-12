//! `ce flow` — the flow family's CLI face (plan v2.31 step 5, design
//! booklet §5.4): the dead code inside every function of the tree,
//! judged by the core's flow/1. Report-only unless `--check`: then it
//! is its own gate, never `ce check`'s — exit 1 when `[flow] tier` is
//! deny and a judged finding stands (an unused parameter is advisory
//! everywhere and never counts). Exit 2 = a core that could not judge,
//! a malformed reply, a document the core did not lay out, an unknown
//! `--kind`, an unreadable ce.toml.

use crate::main_prelude::*;

#[derive(clap::Args)]
pub struct FlowArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Exit 1 when `[flow] tier` is deny and a judged finding stands
    /// (unused parameters are advisory and never count)
    #[arg(long)]
    check: bool,
    /// Show only these kinds, comma-separated: a kind the core's flow
    /// catalogue lists; an unknown name is refused with the list. The
    /// counts stay whole
    #[arg(long, value_delimiter = ',')]
    kind: Vec<String>,
}

pub fn flow_cmd(a: FlowArgs, core: &str) -> ExitCode {
    let plumb = Plumb::gate(None, a.check);
    let answer = faces::flow(&or_cwd(a.judge.root), core, &a.kind, plumb);
    answered("flow", "flow", json(a.judge.format), answer, degraded)
}
