//! `ce arch` — the architecture family's CLI face (plan v2.31 step 9,
//! design booklet §7.3): the layers, the cuts, the clusters, the
//! misplaced files and the directory metrics of the tree, and under
//! `--impact` the files a change to the named ones reaches. Advisory:
//! exit 0 with a document, 2 when the core could not judge (the
//! document names why), could not lay the document out, or a named
//! path is not a measured file.

use crate::main_prelude::*;
use std::path::PathBuf;

#[derive(clap::Args)]
pub struct ArchArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Trace the files that depend on this one (root-relative;
    /// repeatable)
    #[arg(long = "impact")]
    impact: Vec<PathBuf>,
}

/// The `--impact` paths go as typed: arch::face::run reads them as the
/// index spells them, on every face (plan v2.33 step 10).
pub fn arch_cmd(a: ArchArgs, core: &str) -> ExitCode {
    let j = a.judge;
    let focus: Vec<String> = a.impact.iter().map(|p| p.display().to_string()).collect();
    let answer = faces::arch(&or_cwd(j.root), core, &focus, Plumb::db(j.db));
    answered("arch", "arch", json(j.format), answer, degraded)
}
