//! Score-family subcommand bodies (`ce check`, `ce baseline`) —
//! the ADR-006 gate pair: check judges against the committed
//! baseline and fails on ratchet or floor; baseline persists the
//! core's newBaseline under THREE named acts (plan v2.18 step #14):
//! none — the violation set may only shrink; CE_ACCEPT_FENCE=1 — a
//! held FENCE condition alone (knobs_digest, rows_dropped) is accepted and the
//! min-with-old floor re-pinned under the declared knobs; and
//! CE_ACCEPT_BASELINE=1 — the wholesale re-establish, the current
//! facts become the floor. A missing file is not a fourth, unnamed
//! act (O31), and a subdirectory is never a baseline's home (O30).

use crate::main_cmds::{fail, json, or_cwd};
use crate::main_judge::JudgeArgs;
use codeeraser::i18n::{line, t};
use codeeraser::score;
use std::path::{Path, PathBuf};
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

/// The fence conditions CE_ACCEPT_FENCE=1 may accept by name: the
/// config fence, and the rows an exclusion dropped (6.4.0, O40 —
/// accepting writes the baseline without them, which is the act of
/// owning the exclusion). Anything else held — growth, a floor, a
/// budget — still needs the wholesale act.
const FENCE: [&str; 2] = ["knobs_digest", "rows_dropped"];

fn act(var: &str) -> bool {
    std::env::var(var).as_deref() == Ok("1")
}

/// The shared head of both commands — the scope, the output mode and
/// the wire options from one JudgeArgs (the ratchet caught the second
/// copy of this shape growing). The baseline is the caller's ONE read
/// (O31): score::run never reads the file itself.
fn head(judge: JudgeArgs, days: Option<u32>, floor: Option<u32>) -> (PathBuf, bool, score::Opts) {
    let opts = score::Opts {
        db: judge.db,
        core: judge.core,
        days,
        floor,
        establish: false,
        // both commands judge the committed baseline (or a true
        // re-establish); only trend pins a soft line without one
        pinned_soft: None,
        baseline: None,
    };
    (or_cwd(judge.root), json(judge.format), opts)
}

/// One score::run, printed. `name` is the command's own, so a failure
/// says `ce baseline:` when that is what ran; the folded head used to
/// print `ce check:` for both.
fn judged(
    name: &str,
    root: &Path,
    as_json: bool,
    roast: bool,
    opts: score::Opts,
) -> Result<score::Outcome, ExitCode> {
    match score::run(root, opts) {
        Ok(o) => {
            score::print(&o, as_json);
            if roast && !as_json {
                score::report::roast_line(&o);
            }
            Ok(o)
        }
        Err(err) => Err(fail(name, err)),
    }
}

/// `ce check`: judge, print, and fail on the core's word ALONE —
/// ratchet, floor, and since ADR-008 P1 the degraded case too: the
/// core's degraded reply carries fail=true itself ("a gate that
/// could not judge must never pass", said by the core), so the old
/// `|| degraded` disjunct here retired as re-derived policy. A
/// subdirectory scopes the measurement — it under-reports, and
/// persists nothing.
pub fn check_cmd(a: CheckArgs) -> ExitCode {
    let (root, as_json, mut opts) = head(a.judge, a.days, a.fail_under);
    opts.baseline = match score::baseline::read(&root) {
        Ok(committed) => committed,
        Err(err) => return fail("check", err),
    };
    match judged("check", &root, as_json, a.roast, opts) {
        Err(code) => code,
        Ok(o) if o.reply.fail => ExitCode::FAILURE,
        Ok(_) => ExitCode::SUCCESS,
    }
}

/// `ce baseline`: persist the core's newBaseline under a named act.
/// In order: the root preflight (O30), the one read (O31), the act
/// the missing-file case demands, the judgment, the degraded refusal,
/// then what held against which act — a fence condition alone under
/// CE_ACCEPT_FENCE=1 persists the min-with-old newBaseline (no
/// ceiling rises, the discrete set is the current one, the digest is
/// the declared one); anything else held refuses by name unless the
/// judgment was the wholesale one (a NULL baseline: nothing holds).
pub fn baseline_cmd(a: BaselineArgs) -> ExitCode {
    let (root, as_json, mut opts) = head(a.judge, a.days, None);
    if let Err(code) = preflight(&root) {
        return code;
    }
    let (wholesale, fence) = (act("CE_ACCEPT_BASELINE"), act("CE_ACCEPT_FENCE"));
    opts.baseline = match score::baseline::read(&root) {
        Ok(committed) => committed,
        Err(err) => return fail("baseline", err),
    };
    if opts.baseline.is_none() && !wholesale {
        eprintln!(
            "{}",
            line(
                "baseline: no committed ce-baseline.json at {} — establishing one anchors every \
                 ceiling; set CE_ACCEPT_BASELINE=1 to do that by name",
                "baseline：{} 下没有已提交的 ce-baseline.json——首次建立即锚定全部天花板；\
                 设 CE_ACCEPT_BASELINE=1 具名为之",
                &[&root.display()],
            )
        );
        return ExitCode::FAILURE;
    }
    opts.establish = wholesale;
    let o = match judged("baseline", &root, as_json, false, opts) {
        Err(code) => return code,
        Ok(o) => o,
    };
    if o.reply.degraded.is_some() {
        eprintln!(
            "{}",
            t(
                "baseline: refusing to persist a degraded judgment",
                "baseline：拒绝落盘一次降级判决",
            )
        );
        return ExitCode::FAILURE;
    }
    // the core's fail bit IS the only-shrink refusal (ADR-008 P2):
    // this wire carries no floor and no dedup pair, so what holds is
    // growth or a fence, named — and only a fence is the narrow act's
    let fence_only = o.reply.failed.iter().all(|c| FENCE.contains(&c.as_str()));
    if o.reply.fail && !(fence && fence_only) {
        return refuse_held(&o.reply.failed, fence);
    }
    persist(&root, &o, o.reply.fail)
}

/// O30: a baseline is a per-project fact, so `ce baseline pkg` is
/// refused BY NAME before anything is read, measured or spawned —
/// a scoped run keys its rows below the scope and used to overwrite
/// the project's floor with a partial one. `ce check pkg` stays a
/// scoped measurement.
fn preflight(given: &Path) -> Result<(), ExitCode> {
    let (anchor, moved) = codeeraser::root::resolve(given);
    if !moved {
        return Ok(());
    }
    eprintln!(
        "{}",
        line(
            "baseline: {} is inside project {} — a baseline is a per-project fact; run \
             `ce baseline` at {}",
            "baseline：{} 位于项目 {} 之内——基线是逐项目事实；请在 {} 运行 `ce baseline`",
            &[&given.display(), &anchor.display(), &anchor.display()],
        )
    );
    Err(ExitCode::FAILURE)
}

/// The refusal names what held (the console's words, O36) and which
/// act would clear it — and, under CE_ACCEPT_FENCE=1, exactly which
/// held condition the narrow act may not launder.
fn refuse_held(held: &[String], fence: bool) -> ExitCode {
    let extra: Vec<&str> = held
        .iter()
        .map(String::as_str)
        .filter(|c| !FENCE.contains(c))
        .collect();
    let msg = if fence {
        line(
            "baseline: CE_ACCEPT_FENCE=1 accepts {} alone; also held: {} — growth needs \
             CE_ACCEPT_BASELINE=1 to re-establish from the current tree",
            "baseline：CE_ACCEPT_FENCE=1 只接受 {}；另有成立条件：{}——增长须 \
             CE_ACCEPT_BASELINE=1 按当前树重立",
            &[&FENCE.join(", "), &extra.join(", ")],
        )
    } else {
        line(
            "baseline: refused by {} — without a named act both halves only improve: \
             CE_ACCEPT_FENCE=1 accepts a fence condition alone ({}), CE_ACCEPT_BASELINE=1 \
             re-establishes from the current tree",
            "baseline：被 {} 拒绝——无具名动作时两半都只准改善：CE_ACCEPT_FENCE=1 只接受\
             围栏条件（{}），CE_ACCEPT_BASELINE=1 按当前树重立",
            &[&held.join(", "), &FENCE.join(", ")],
        )
    };
    eprintln!("{msg}");
    ExitCode::FAILURE
}

/// The persistence tail every accepted road shares: write at the
/// RESOLVED path and name it (a floor is a per-project fact, and the
/// operator must see which project just got one); the fence road
/// says which fence it accepted.
fn persist(root: &Path, o: &score::Outcome, fenced: bool) -> ExitCode {
    match score::baseline::write(root, &o.reply.new_baseline) {
        Ok(path) => {
            if fenced {
                println!(
                    "{}",
                    line(
                        "baseline: fence accepted by name ({}) — ceilings min-with-old, members \
                         current",
                        "baseline：围栏已具名接受（{}）——天花板取新旧之小，成员取当前",
                        &[&o.reply.failed.join(", ")],
                    )
                );
            }
            println!(
                "{}",
                line("baseline written: {}", "基线已写入：{}", &[&path.display()])
            );
            ExitCode::SUCCESS
        }
        Err(err) => fail("baseline", err),
    }
}
