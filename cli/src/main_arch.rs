//! `ce arch` — the architecture family's CLI face (plan v2.31 step 9,
//! design booklet §7.3): the layers, the cuts, the clusters, the
//! misplaced files and the directory metrics of the tree, and under
//! `--impact` the files a change to the named ones reaches. Advisory:
//! exit 0 with a document, 2 when the core could not judge (the
//! document names why) or a named path is not a measured file.

use crate::main_prelude::*;
use codeeraser::arch::{console, face};
use std::path::{Path, PathBuf};

#[derive(clap::Args)]
pub struct ArchArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Trace the files that depend on this one (root-relative;
    /// repeatable)
    #[arg(long = "impact")]
    impact: Vec<PathBuf>,
}

/// A focus path as the index spells it: root-relative, forward
/// slashes, no leading `./`.
fn spelled(root: &Path, path: &Path) -> String {
    let rel = codeeraser::scan::walk::rel_str(root, path);
    rel.strip_prefix("./").map_or(rel.clone(), str::to_string)
}

pub fn arch_cmd(a: ArchArgs) -> ExitCode {
    let j = a.judge;
    let root = or_cwd(j.root);
    let focus: Vec<String> = a.impact.iter().map(|p| spelled(&root, p)).collect();
    let r = match face::run(&root, j.db, &j.core, &focus) {
        Ok(r) => r,
        Err(err) => return fail("arch", err),
    };
    if json(j.format) {
        println!("{}", face::report_json(&r));
    } else {
        console::console(&r).iter().for_each(|l| println!("{l}"));
    }
    if r.degraded.is_some() {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}
