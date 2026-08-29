//! Progress on the console face (plan v2.16) — the long measurements
//! stop being indistinguishable from a hang. Measured before this
//! existed: `ce churn . --days 1` ran 102 s and wrote ZERO bytes
//! until its report; the default 14-day window is 278.4 s and
//! `ce join` 265.0 s (PERF-BUDGET M5-3h). main_cli.rs opens by saying
//! help answers "what it costs" — for those three it did not, and
//! neither did the run itself.
//!
//! Two structural rules:
//!
//! 1. It rides STDERR. stdout carries the report — console lines or
//!    `--format json` — and every pipe, golden and machine consumer
//!    reads that stream; a progress byte on it would be the report
//!    lying about its own shape. The machine faces are therefore
//!    unchanged by construction, not by anyone remembering.
//! 2. The measurement names a PHASE and two counts; the words are
//!    this face's (plan v2.15 — the same split the doctor document
//!    and the join caveat now take). A loop inside churn knows how
//!    far along it is, never what to say about it nor whether
//!    anybody is watching.
//!
//! Armed once by main, beside the language switch it mirrors
//! (i18n::init). Threading a sink through churn::run's five callers
//! would have made the MCP and GUI faces' silence a matter of every
//! caller REMEMBERING to pass None; here silence is simply what an
//! unarmed process does.

use crate::i18n;
use std::io::{IsTerminal, Write};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicI64, AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

/// Which measurement is running. A TYPE rather than the bare integer
/// codes the wire families use: nothing here crosses a wire, so the
/// discriminant is an implementation detail of the words table it
/// indexes — and the seven-`pub const` first draft was a T2 twin of
/// graph/wire.rs's and docdup/spec.rs's frozen code runs the moment
/// it was written. Positions are still frozen (the table is read at
/// them) and the test at the bottom is what freezes them.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum Phase {
    Window,
    Commits,
    Survival,
    Index,
    Graph,
    Assemble,
    Measure,
}

/// One row per phase in code order, as ONE literal rather than a
/// tuple list: main_lang.rs learned that under T2 literal abstraction
/// any homogeneous tuple table matches any other, and this one
/// matched graph_ladder's and deadcode_e2e's tables the day it was
/// written. `en<TAB>zh`; the LAST row is the unknown case, which is
/// what [`i18n::coded`] falls back to — a phase the table cannot name
/// must not borrow the name of the one at index 0. Rows without `{}`
/// ignore the counts.
const PHASE_TSV: &str = "\
enumerating the commit window\t正在枚举提交窗口
commits {}/{}\t提交 {}/{}
blame survivors: {}/{} files\t存活归因：{}/{} 文件
building the clone index\t正在建立克隆索引
building the reference graph\t正在建立引用图
assembling the join\t正在装配联判
measuring commits {}/{}\t测量提交 {}/{}
working\t处理中
";

fn phases() -> &'static [(&'static str, &'static str)] {
    static ROWS: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
    ROWS.get_or_init(|| {
        PHASE_TSV
            .lines()
            .filter_map(|l| l.split_once('\t'))
            .collect()
    })
}

/// Repaint budget. A tick must cost the measurement nothing, or the
/// progress becomes part of the number it is reporting.
const PAINT_MS: u64 = 120;

/// "No phase has painted yet" — outside the table on purpose, so the
/// first tick of a span always beats the repaint throttle.
const NO_PHASE: i64 = -1;

struct Sink {
    start: Instant,
    last: AtomicU64,
    phase: AtomicI64,
    cols: AtomicUsize,
}

/// `None` = this process is not a console face. Never set means the
/// same, so a library caller — the MCP server, the GUI backend, the
/// daemon, every test — is silent without doing anything.
static SINK: OnceLock<Option<Sink>> = OnceLock::new();

/// Arm the console face. `CE_PROGRESS=1` forces it on and `=0` off;
/// otherwise a terminal on stderr decides. The override is not a
/// convenience: a TTY-only feature cannot be observed by a test or a
/// CI leg, and a feature nothing observes is one nobody notices
/// breaking. It is an env var rather than a flag because a global
/// flag would land in all twenty subcommands' help while only three
/// of them ever paint (CE_LANG / CE_BLESS / CE_ACCEPT_BASELINE are
/// the same shape).
pub fn arm() {
    let on = match std::env::var("CE_PROGRESS").as_deref() {
        Ok("0") => false,
        Ok("1") => true,
        _ => std::io::stderr().is_terminal(),
    };
    let _ = SINK.set(on.then(|| Sink {
        start: Instant::now(),
        last: AtomicU64::new(0),
        phase: AtomicI64::new(NO_PHASE),
        cols: AtomicUsize::new(0),
    }));
}

/// A measurement's progress span: ticks belong to it, and the line is
/// erased when it ends. RAII because `?` is how these functions
/// return — a `finish()` at the bottom of churn::run would be skipped
/// by exactly the error paths that most need the line gone, and one
/// per command throat would have to be re-found every time a new
/// caller appears. Spans nest: the inner one erases on the way out
/// and the outer one repaints on its next tick.
#[must_use = "a span erases the progress line when it drops"]
pub(crate) struct Span;

pub(crate) fn span() -> Span {
    Span
}

impl Drop for Span {
    fn drop(&mut self) {
        let Some(Some(s)) = SINK.get() else { return };
        let cols = s.cols.swap(0, Ordering::Relaxed);
        if cols == 0 {
            return;
        }
        s.phase.store(NO_PHASE, Ordering::Relaxed);
        let mut e = std::io::stderr().lock();
        let _ = write!(e, "\r{:cols$}\r", "");
        let _ = e.flush();
    }
}

/// A phase with no countable work: it names WHICH step is running,
/// which is the whole answer when the step is one long call.
pub fn step(phase: Phase) {
    at(phase, 0, 0);
}

/// `done` of `total` inside `phase`. Lock-free and allocation-free
/// when the face is unarmed, which is every non-CLI caller.
pub fn at(phase: Phase, done: usize, total: usize) {
    let Some(Some(s)) = SINK.get() else { return };
    let code = phase as i64;
    let ms = u64::try_from(s.start.elapsed().as_millis()).unwrap_or(u64::MAX);
    // a phase CHANGE always paints: an indeterminate step ticks
    // exactly once, so letting the throttle swallow it would leave
    // the previous phase's counter on screen for the whole of the
    // next one — the stalest possible reading of a live process
    let moved = s.phase.swap(code, Ordering::Relaxed) != code;
    if !moved && ms < s.last.load(Ordering::Relaxed).saturating_add(PAINT_MS) {
        return;
    }
    s.last.store(ms, Ordering::Relaxed);
    paint(s, &i18n::coded(code, phases(), &[&done, &total]));
}

fn paint(s: &Sink, text: &str) {
    let cols = columns(text);
    // blanks, not an ANSI erase: `\x1b[2K` needs VT processing, and a
    // console without it prints the escape as text — the one failure
    // mode worse than the silence this module exists to end
    let pad = s.cols.swap(cols, Ordering::Relaxed).saturating_sub(cols);
    let mut e = std::io::stderr().lock();
    let _ = write!(e, "\r{text}{:pad$}", "");
    let _ = e.flush();
}

/// Display columns, not chars — the zh line is what makes the
/// difference load-bearing: a CJK glyph occupies two cells, so a char
/// count would under-erase by half the width and leave a tail of the
/// previous phase on screen. The ranges are the East Asian Wide and
/// Fullwidth blocks; every other glyph these tables use is ASCII.
fn columns(s: &str) -> usize {
    s.chars()
        .map(|c| {
            1 + usize::from(matches!(c,
                '\u{1100}'..='\u{115F}' | '\u{2E80}'..='\u{A4CF}'
                | '\u{AC00}'..='\u{D7A3}' | '\u{F900}'..='\u{FAFF}'
                | '\u{FE30}'..='\u{FE6F}' | '\u{FF00}'..='\u{FF60}'
                | '\u{FFE0}'..='\u{FFE6}'))
        })
        .sum()
}

#[cfg(test)]
#[path = "../tests/unit/progress.rs"]
mod tests;
