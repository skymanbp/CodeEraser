//! `ce query` and `ce rules` — the code-query family's CLI faces
//! (plan v2.31 step 2, design booklet §4.6): one question answered
//! with its derivations under `--why`, a rules file's assertions
//! judged as a gate. Exit codes: a program error (lexical, syntax,
//! sort, safety, stratification) or a core that could not judge is 2
//! for both; `ce rules` answers 1 when any assertion holds a
//! violation and 0 when every one passes — a missing default file is
//! zero assertions and 0, said aloud.

use crate::main_prelude::*;
use codeeraser::query::face::{self, Ask};
use codeeraser::query::{PRELUDE, rules_source};
use std::path::PathBuf;

#[derive(clap::Args)]
pub struct QueryArgs {
    /// The question, a query body (`dead(F)`; `?-` and the final
    /// `.` may be left out); required unless --prelude
    body: Option<String>,
    #[command(flatten)]
    judge: JudgeArgs,
    /// Carry every answer's derivation (a violation always does)
    #[arg(long)]
    why: bool,
    /// The rules file whose rules the question builds on (default:
    /// `[rules] file`, else ce.rules at the root when it exists)
    #[arg(long)]
    file: Option<PathBuf>,
    /// Print the built-in prelude and exit
    #[arg(long)]
    prelude: bool,
}

#[derive(clap::Args)]
pub struct RulesArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// The rules file to judge (default: `[rules] file`, else
    /// ce.rules at the root; absent = zero assertions)
    #[arg(long)]
    file: Option<PathBuf>,
    /// Carry every query answer's derivation too
    #[arg(long)]
    why: bool,
}

pub fn query_cmd(a: QueryArgs, core: &str) -> ExitCode {
    if a.prelude {
        print!("{PRELUDE}");
        return ExitCode::SUCCESS;
    }
    let Some(body) = a.body else {
        return fail(
            "query",
            anyhow::anyhow!("a question is required (or --prelude)"),
        );
    };
    judge("query", (a.judge, core), a.file, Some(body), a.why)
}

pub fn rules_cmd(a: RulesArgs, core: &str) -> ExitCode {
    judge("rules", (a.judge, core), a.file, None, a.why)
}

/// One judgment for both faces: the rules file resolved, the core
/// asked, its lines or document shown; exit 2 when the program was not
/// judged (a program error, a degraded core), else the core's veto
/// (`ce rules`: a violation; `ce query` states none).
fn judge(
    name: &str,
    (j, core): (JudgeArgs, &str),
    file: Option<PathBuf>,
    query: Option<String>,
    why: bool,
) -> ExitCode {
    let root = or_cwd(j.root);
    let rules = match rules_source(&root, file.as_deref()) {
        Ok(r) => r,
        Err(err) => return fail(name, err),
    };
    let ask = Ask { rules, query, why };
    let answer = face::run(&root, j.db, core, &ask);
    answered(name, name, json(j.format), answer, |doc| {
        let errors = doc["errors"].as_array().is_some_and(|e| !e.is_empty());
        Ok(errors || !doc["degraded"].is_null())
    })
}
