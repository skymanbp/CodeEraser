//! `ce query` and `ce rules` — the code-query family's CLI faces
//! (plan v2.31 step 2, design booklet §4.6): one question answered
//! with its derivations under `--why`, a rules file's assertions
//! judged as a gate. Exit codes: a program error (lexical, syntax,
//! sort, safety, stratification) or a core that could not judge is 2
//! for both; `ce rules` answers 1 when any assertion holds a
//! violation and 0 when every one passes — a missing default file is
//! zero assertions and 0, said aloud.

use crate::main_prelude::*;
use codeeraser::query::face::{self, Ask, Report};
use codeeraser::query::{PRELUDE, console, rules_source};
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
    judge("query", (a.judge, core), a.file, Some(body), a.why, |r| {
        if r.judged() {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(2)
        }
    })
}

pub fn rules_cmd(a: RulesArgs, core: &str) -> ExitCode {
    judge("rules", (a.judge, core), a.file, None, a.why, |r| {
        if !r.judged() {
            ExitCode::from(2)
        } else if r.violations() > 0 {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        }
    })
}

/// One judgment for both faces: the rules file resolved, the core
/// asked, the document shown, the exit code read off the report by
/// the face's own rule.
fn judge(
    name: &str,
    (j, core): (JudgeArgs, &str),
    file: Option<PathBuf>,
    query: Option<String>,
    why: bool,
    exit: impl Fn(&Report) -> ExitCode,
) -> ExitCode {
    let root = or_cwd(j.root);
    let rules = match rules_source(&root, file.as_deref()) {
        Ok(r) => r,
        Err(err) => return fail(name, err),
    };
    let rules_face = query.is_none();
    let ask = Ask { rules, query, why };
    match face::run(&root, j.db, core, &ask) {
        Ok(r) => {
            show(&r, json(j.format), rules_face);
            exit(&r)
        }
        Err(err) => fail(name, err),
    }
}

fn show(r: &Report, as_json: bool, rules: bool) {
    print_doc(
        as_json,
        || face::report_json(r, rules),
        || {
            if rules && r.rules_file.is_none() {
                println!(
                    "{}",
                    codeeraser::i18n::line(
                        "rules: no rules file — zero assertions",
                        "rules：没有规则文件——零断言",
                        &[]
                    )
                );
            }
            for l in console::console(r) {
                println!("{l}");
            }
        },
    );
}
