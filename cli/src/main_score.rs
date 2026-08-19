//! Score-family subcommand bodies (`ce check`, `ce baseline`) —
//! the ADR-006 gate pair: check judges against the committed
//! baseline and fails on ratchet or floor; baseline writes the
//! core's newBaseline back, refusing to GROW the violation set
//! unless CE_ACCEPT_BASELINE=1 says the growth is a decided
//! acceptance (the only-shrink invariant as set inclusion).

use crate::main_cmds::{fail, json, or_cwd};
use crate::main_judge::JudgeArgs;
use codeeraser::score;
use std::process::ExitCode;

#[derive(clap::Args)]
pub struct CheckArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Churn window in days (omit = churn tables stay empty)
    #[arg(long)]
    days: Option<u32>,
    /// Fail when the score lands under this per-mille floor
    #[arg(long)]
    fail_under: Option<u32>,
    /// Append a roast line to the console verdict (easter egg)
    #[arg(long)]
    roast: bool,
}

#[derive(clap::Args)]
pub struct BaselineArgs {
    #[command(flatten)]
    judge: JudgeArgs,
    /// Churn window in days (omit = churn tables stay empty)
    #[arg(long)]
    days: Option<u32>,
}

/// The shared judge-and-print head of both commands — root, output
/// mode, wire options, one score::run (the ratchet caught the second
/// copy of this shape growing).
fn judged(
    judge: JudgeArgs,
    days: Option<u32>,
    floor: Option<u32>,
    establish: bool,
    roast: bool,
) -> Result<(std::path::PathBuf, score::Outcome), ExitCode> {
    let (root, as_json) = (or_cwd(judge.root), json(judge.format));
    let opts = score::Opts {
        db: judge.db,
        core: judge.core,
        days,
        floor,
        establish,
    };
    match score::run(&root, opts) {
        Ok(o) => {
            score::print(&o, as_json);
            if roast && !as_json {
                score::report::roast_line(&o);
            }
            Ok((root, o))
        }
        Err(err) => Err(fail("check", err)),
    }
}

/// `ce check`: judge, print, and fail on the core's word ALONE —
/// ratchet, floor, and since ADR-008 P1 the degraded case too: the
/// core's degraded reply carries fail=true itself ("a gate that
/// could not judge must never pass", said by the core), so the old
/// `|| degraded` disjunct here retired as re-derived policy.
pub fn check_cmd(a: CheckArgs) -> ExitCode {
    match judged(a.judge, a.days, a.fail_under, false, a.roast) {
        Err(code) => code,
        Ok((_root, o)) => {
            if o.reply.fail {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
    }
}

/// `ce baseline`: persist the core's newBaseline. Without
/// CE_ACCEPT_BASELINE=1 both halves may only improve — a grown
/// discrete set OR a busted continuous ceiling is refused (the
/// newBaseline is min(current, old), so persisting past an `over`
/// would freeze a permanently red ceiling). With the env set the
/// judgment runs against a NULL baseline: re-establish, the current
/// facts become the floor — growth accepted DELIBERATELY.
pub fn baseline_cmd(a: BaselineArgs) -> ExitCode {
    let accepted = std::env::var("CE_ACCEPT_BASELINE").as_deref() == Ok("1");
    let (root, o) = match judged(a.judge, a.days, None, accepted, false) {
        Err(code) => return code,
        Ok(pair) => pair,
    };
    if o.reply.degraded.is_some() {
        eprintln!(
            "{}",
            codeeraser::i18n::t(
                "baseline: refusing to persist a degraded judgment",
                "baseline：拒绝落盘一次降级判决",
            )
        );
        return ExitCode::FAILURE;
    }
    // the core's fail bit IS the only-shrink refusal (ADR-008 P2):
    // this wire carries no floor and no dedup pair, so fail ≡
    // added∨over — the set re-interpretation retires, the counts
    // below are reporting
    if o.reply.fail {
        eprintln!(
            "{}",
            codeeraser::i18n::line(
                "baseline: {} new member(s), {} ceiling(s) busted — both halves only \
                 improve; set CE_ACCEPT_BASELINE=1 to re-establish from the current tree",
                "baseline：新增成员 {} 个、突破天花板 {} 处 — 两半都只准改善；\
                 设 CE_ACCEPT_BASELINE=1 方可按当前树重立",
                &[&o.reply.added.len(), &o.reply.over.len()],
            )
        );
        return ExitCode::FAILURE;
    }
    match score::baseline::write(&root, &o.reply.new_baseline) {
        Ok(()) => {
            println!(
                "{}",
                codeeraser::i18n::line(
                    "baseline written: {}",
                    "基线已写入：{}",
                    &[&score::baseline::BASELINE_FILE],
                )
            );
            ExitCode::SUCCESS
        }
        Err(err) => fail("baseline", err),
    }
}
