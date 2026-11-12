//! When is the core wedged? The rule the link reads every exchange by
//! (plan v2.35 phase D). The core is a pure computation between reading
//! a line and writing one, so it is working exactly when its CPU clock
//! advances; wall time measures the machine, not the core. Until 2026-11
//! the link killed any core whose reply took 60 s of wall time, on the
//! grounds that every judgment answered in single-digit seconds: on a
//! loaded machine that killed a core in the middle of an honest request
//! (koreader `ce join`, 2026-11-11, the machine at 100 % with the owner's
//! game running: a `candidates` request that had used 4.71 CPU-s in its
//! 78 s), and the request's own write was never watched at all.
//!
//! An exchange is open from the request's first byte until the reply line
//! arrives, and three signals say it moves: request bytes handed to the
//! pipe, the core's CPU time (cputime.rs), the reply itself.
//! - Stalled: for S wall seconds neither the byte count nor the CPU time
//!   moved. A core that drains the request or computes is not stalled at
//!   any wall time, however starved the machine.
//! - Over budget: the core spent more than B CPU-seconds on this exchange,
//!   a core that spins.
//!
//! When the CPU clock cannot be read the byte count alone is progress, so
//! once the request is written S is a reply deadline again (the rule
//! before 2026-11), and the error says the clock was unreadable. The rule
//! is a pure fold over samples (`Watch::step`); session.rs samples it.

use std::time::Duration;

/// S, the wall seconds an exchange may go without progress: the shipped
/// 60, or `CE_CORE_DEADLINE_SECS` (a wedged-core leg cannot wait a minute).
pub(super) fn stall() -> Duration {
    crate::config::env_secs("CE_CORE_DEADLINE_SECS", 60)
}

/// B, the CPU seconds one exchange may cost: a backstop for a core that
/// spins, not a performance line. Chosen from readings, which are dated
/// observations and no invariant: the largest single request measured
/// cost 4.5 CPU-s (plan v2.35 C8's table of every family's largest
/// request, 2026-11-12, one machine, loaded), and the steepest family not
/// planned under a ceiling of its own, `document` (3.77 CPU-s on
/// 2,485,434 B), would reach about 204 CPU-s at the protocol's line of
/// 134,217,728 B; 600 is about three times that. An honest request that
/// passes it is judged by setting `CE_CORE_CPU_BUDGET_SECS`.
pub(super) fn budget() -> Duration {
    crate::config::env_secs("CE_CORE_CPU_BUDGET_SECS", 600)
}

/// One reading of an open exchange: wall time since its first byte, the
/// request bytes handed to the pipe, the core's CPU clock (None when it
/// could not be read).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Sample {
    pub at: Duration,
    pub bytes: u64,
    pub cpu: Option<Duration>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    Wait,
    Stalled,
    OverBudget,
}

/// The rule's state over one exchange: the first CPU reading (what the
/// exchange's spend is counted from), the last reading of each signal and
/// when one last moved.
pub(super) struct Watch {
    stall: Duration,
    budget: Duration,
    first_cpu: Option<Duration>,
    last: Sample,
    moved_at: Duration,
}

impl Watch {
    pub(super) fn new(stall: Duration, budget: Duration, first: Sample) -> Watch {
        Watch {
            stall,
            budget,
            first_cpu: first.cpu,
            last: first,
            moved_at: first.at,
        }
    }

    /// Read one more sample. A reading the clock could not give keeps the
    /// last one it gave, so a flicker is never counted as movement.
    pub(super) fn step(&mut self, now: Sample) -> Verdict {
        let cpu = now.cpu.or(self.last.cpu);
        let advanced = matches!((self.last.cpu, cpu), (Some(a), Some(b)) if b > a);
        if now.bytes > self.last.bytes || advanced {
            self.moved_at = now.at;
        }
        self.first_cpu = self.first_cpu.or(cpu);
        self.last = Sample { cpu, ..now };
        if self.spent().is_some_and(|s| s > self.budget) {
            Verdict::OverBudget
        } else if now.at.saturating_sub(self.moved_at) >= self.stall {
            Verdict::Stalled
        } else {
            Verdict::Wait
        }
    }

    /// The CPU this exchange cost the core so far.
    pub(super) fn spent(&self) -> Option<Duration> {
        Some(self.last.cpu?.saturating_sub(self.first_cpu?))
    }

    /// The error for a verdict that kills: the clause, its readings and
    /// the variable that sets it. `total` is the request line's bytes with
    /// its newline; `waiting` names the request (kind and bytes, C5).
    pub(super) fn explain(&self, verdict: Verdict, total: u64, waiting: &str) -> String {
        let written = format!("{} of {total} B written", self.last.bytes);
        let cpu = match self.spent() {
            Some(s) => format!("core CPU {:.2} s on it", s.as_secs_f64()),
            None => "core CPU unreadable, so the bytes written were the only progress read".into(),
        };
        match verdict {
            Verdict::OverBudget => format!(
                "core CPU budget exceeded ({:?}) on {waiting}: {cpu}, {written} — core killed (CE_CORE_CPU_BUDGET_SECS sets the budget)",
                self.budget
            ),
            _ => format!(
                "core reply deadline exceeded: no progress for {:?} waiting on {waiting}: {written}, {cpu} — core killed (CE_CORE_DEADLINE_SECS sets the wait)",
                self.stall
            ),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/corelink/watch.rs"]
mod tests;
