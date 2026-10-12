//! `ce query` and `ce rules` — the code-query family's CLI faces
//! (plan v2.31 step 2, design booklet §4.6): one question answered
//! with its derivations under `--why`, a rules file's assertions
//! judged as a gate. Exit codes: a program error (lexical, syntax,
//! sort, safety, stratification) or a core that could not judge is 2
//! for both; `ce rules` answers 1 when any assertion holds a
//! violation and 0 when every one passes — a missing default file with
//! no `[rules.pack]` declared is zero assertions and 0, said aloud.

use crate::main_prelude::*;
use codeeraser::document::Answer;
use std::path::{Path, PathBuf};

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
    /// Print the built-in prelude, then the `[rules.pack]` program the
    /// core renders from ce.toml when one is declared, and exit
    #[arg(long)]
    prelude: bool,
}

#[derive(clap::Args)]
pub struct RulesArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// The rules file to judge (default: `[rules] file`, else
    /// ce.rules at the root; absent = the `[rules.pack]` assertions
    /// alone, zero when none is declared)
    #[arg(long)]
    file: Option<PathBuf>,
    /// Carry every query answer's derivation too
    #[arg(long)]
    why: bool,
}

pub fn query_cmd(a: QueryArgs, core: &str) -> ExitCode {
    let j = a.judge;
    if a.prelude {
        return prelude(&or_cwd(j.root), core, json(j.format));
    }
    let Some(body) = a.body else {
        return fail(
            "query",
            anyhow::anyhow!("a question is required (or --prelude)"),
        );
    };
    let root = or_cwd(j.root);
    let ask = (body.as_str(), a.why);
    let answer = faces::query(&root, core, ask, a.file.as_deref(), Plumb::db(j.db));
    shown("query", json(j.format), answer)
}

pub fn rules_cmd(a: RulesArgs, core: &str) -> ExitCode {
    let j = a.judge;
    let file = a.file.as_deref();
    let answer = faces::rules(&or_cwd(j.root), core, file, a.why, Plumb::db(j.db));
    shown("rules", json(j.format), answer)
}

/// The pack text (faces::pack_text): the prelude, then the program the
/// core renders from `[rules.pack]` when one is declared — the two
/// strings on the console, the document under `--format json`.
fn prelude(root: &Path, core: &str, as_json: bool) -> ExitCode {
    let doc = match faces::pack_text(root, core) {
        Ok(doc) => doc,
        Err(err) => return fail("query", err),
    };
    if as_json {
        println!("{doc}");
    } else {
        let text = |k: &str| doc[k].as_str().unwrap_or_default().to_string();
        print!("{}{}", text("prelude"), text("pack"));
    }
    ExitCode::SUCCESS
}

/// Both faces' answer shown; exit 2 when the program was not judged (a
/// program error, a degraded core), else the core's veto (`ce rules`:
/// a violation; `ce query` states none).
fn shown(name: &str, as_json: bool, answer: anyhow::Result<Answer>) -> ExitCode {
    answered(name, name, as_json, answer, |doc| {
        let errors = doc["errors"].as_array().is_some_and(|e| !e.is_empty());
        Ok(errors || !doc["degraded"].is_null())
    })
}
