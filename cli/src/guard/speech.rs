//! Every sentence the PreToolUse guard speaks, said by the core (plan
//! v2.32 step 5, ruling R7; design booklet
//! docs/reference/authority-track.md §6). A rule that fired leaves a
//! `Said` — its code, its numbers and its strings; the hook decides
//! the tier, then asks ONE `guard` document over the daemon's held
//! link (`Request::Document`, daemon proto 2.3.0) with every rule as a
//! `say` row and every string as a reference, binds the reply's lines
//! through the strings it holds, and joins them as the decision's
//! reason. The sentences and both languages are the core's
//! (`CE.Text.Guard`); the decision stays this side's. A core the hook
//! cannot reach, or a reply it cannot bind, still decides — the
//! sentence then is the one English fallback, which names the rules
//! and why the core did not phrase them.

use crate::daemon::client;
use crate::daemon::proto::Request;
use crate::document::{self, lines};
use crate::tombstone::Row;
use serde_json::Value;
use std::path::Path;

/// Why a write was judged against the shipped budgets rather than the
/// declared ones (O33): its code is the `say` row's fence column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Fence {
    None,
    Drifted,
    Unreadable,
}

/// One duplicated region the probe matched.
pub(super) struct Match {
    pub file: String,
    pub start: i64,
    pub end: i64,
    pub tokens: i64,
}

/// A rule that spoke, with what its sentence names.
pub(super) enum Said {
    /// The T1/T2 rule: the write duplicates `regions` indexed regions,
    /// the first three shown.
    Duplicate {
        file: String,
        regions: usize,
        top: Vec<Match>,
    },
    /// The hard budget: the write leaves the file past its class line.
    OverBudget {
        file: String,
        lines: usize,
        cap: usize,
        fence: Fence,
    },
    /// The graded zone (v2.7 ①, opt-in): under the hard line, `permille`
    /// into (soft, cap].
    Zone {
        file: String,
        lines: usize,
        permille: usize,
        soft: usize,
        cap: usize,
    },
    /// The tombstone class: the core judged `sites` past the budget,
    /// the first three shown (their kind code = tombstone::Kind, the
    /// core's `kindNames` order).
    Tombstone {
        sites: usize,
        budget: u32,
        fence: Fence,
        shown: Vec<Row>,
    },
    /// The flow class: `novel` new findings, the first one named.
    FlowNovel {
        file: String,
        novel: usize,
        unit: String,
        kind: u8,
        line: u32,
    },
    /// Fail-open, but never silent: a ce.toml that will not parse.
    ConfigUnreadable(String),
}

impl Said {
    /// The rule's code, the catalogue's key for its sentence.
    fn code(&self) -> &'static str {
        match self {
            Said::Duplicate { .. } => "duplicate",
            Said::OverBudget { .. } => "over_budget",
            Said::Zone { .. } => "graded_zone",
            Said::Tombstone { .. } => "tombstone_over",
            Said::FlowNovel { .. } => "flow_novel",
            Said::ConfigUnreadable(_) => "config_unreadable",
        }
    }
}

/// The sentences of every rule in `said`, in order, joined as one
/// reason — the core's, or the fallback.
pub(super) fn phrase(root: &Path, said: &[Said]) -> String {
    spoken(root, said).unwrap_or_else(|why| fallback(said, &why))
}

/// The one English sentence the hook says when the core did not phrase
/// the rules (a program-error line, not a catalogue sentence). It names
/// the rules that fired and claims no decision: the decision is the
/// hook's own allow / ask / deny channel, one spelling for every tier.
fn fallback(said: &[Said], why: &str) -> String {
    let codes: Vec<&str> = said.iter().map(Said::code).collect();
    format!(
        "ce: rule {} fired; the core could not phrase the reason: {why}",
        codes.join(", ")
    )
}

fn spoken(root: &Path, said: &[Said]) -> Result<String, String> {
    let strings = Strings::of(said);
    let request = Request::Document {
        body: strings.body(),
    };
    let reply = client::relay(root, &request)?;
    crate::corelink::judged::degraded(&reply)?;
    let (lines, _) = lines::bind_lines(&reply, &strings.lists()).map_err(|e| e.to_string())?;
    let texts: Vec<String> = lines.into_iter().map(|l| l.text).collect();
    Ok(texts.join(" "))
}

/// The request's tables and the strings its references name.
#[derive(Default)]
struct Strings {
    files: Vec<String>,
    match_files: Vec<String>,
    place_files: Vec<String>,
    units: Vec<String>,
    errors: Vec<String>,
    say: Vec<[i64; 7]>,
    matches: Vec<[i64; 4]>,
    places: Vec<[i64; 3]>,
}

/// A count or a line as a row integer.
fn n(x: usize) -> i64 {
    i64::try_from(x).unwrap_or(i64::MAX)
}

/// A shown list as rows, each row's first column its path's index in
/// `paths` (the matches, the judged sites); the first row's index.
fn shown_rows<'a, const N: usize>(
    rows: &mut Vec<[i64; N]>,
    paths: &mut Vec<String>,
    items: impl Iterator<Item = (&'a String, [i64; N])>,
) -> i64 {
    let first = n(rows.len());
    for (path, mut row) in items {
        row[0] = push(paths, path);
        rows.push(row);
    }
    first
}

/// A string's index in `list`, once pushed.
fn push(list: &mut Vec<String>, s: &str) -> i64 {
    list.push(s.to_string());
    n(list.len() - 1)
}

impl Strings {
    fn of(said: &[Said]) -> Self {
        let mut s = Strings::default();
        for x in said {
            let row = s.row(x);
            s.say.push(row);
        }
        s
    }

    /// One `say` row: [rule, a, b, c, d, e, f] (CE.Guard.Document).
    fn row(&mut self, said: &Said) -> [i64; 7] {
        match said {
            Said::Duplicate { .. } | Said::Tombstone { .. } => self.listed(said),
            Said::OverBudget {
                file,
                lines,
                cap,
                fence,
            } => {
                let f = push(&mut self.files, file);
                [1, f, n(*lines), n(*cap), *fence as i64, 0, 0]
            }
            Said::Zone {
                file,
                lines,
                permille,
                soft,
                cap,
            } => {
                let f = push(&mut self.files, file);
                [2, f, n(*lines), n(*permille), n(*soft), n(*cap), 0]
            }
            Said::FlowNovel {
                file,
                novel,
                unit,
                kind,
                line,
            } => {
                let (f, u) = (push(&mut self.files, file), push(&mut self.units, unit));
                [4, f, n(*novel), u, i64::from(*kind), i64::from(*line), 0]
            }
            Said::ConfigUnreadable(e) => [5, push(&mut self.errors, e), 0, 0, 0, 0, 0],
        }
    }

    /// The `say` row of a rule that shows a list (the duplicate's
    /// matches, the tombstone's sites), the list's rows pushed first.
    fn listed(&mut self, said: &Said) -> [i64; 7] {
        match said {
            Said::Duplicate { file, regions, top } => {
                let rows = top.iter().map(|m| (&m.file, [0, m.start, m.end, m.tokens]));
                let first = shown_rows(&mut self.matches, &mut self.match_files, rows);
                let f = push(&mut self.files, file);
                [0, f, n(*regions), first, n(top.len()), 0, 0]
            }
            Said::Tombstone {
                sites,
                budget,
                fence,
                shown,
            } => {
                let rows = shown
                    .iter()
                    .map(|p| (&p.file, [0, n(p.line), p.kind as i64]));
                let first = shown_rows(&mut self.places, &mut self.place_files, rows);
                let (b, fence) = (i64::from(*budget), *fence as i64);
                [3, n(*sites), b, fence, first, n(shown.len()), 0]
            }
            _ => unreachable!("only the two rules that show a list reach here"),
        }
    }

    fn body(&self) -> Value {
        document::Request::new("guard")
            .range("files", self.files.len())
            .range("matches", self.matches.len())
            .range("places", self.places.len())
            .range("units", self.units.len())
            .range("errors", self.errors.len())
            .rows("say", &self.say)
            .rows("matches", &self.matches)
            .rows("places", &self.places)
            .body()
    }

    /// The strings the request's references name, one list per class.
    fn lists(self) -> document::Lists {
        document::Lists(vec![
            ("file", self.files),
            ("match_file", self.match_files),
            ("place_file", self.place_files),
            ("unit", self.units),
            ("error", self.errors),
        ])
    }
}

#[cfg(test)]
#[path = "../../tests/unit/guard/speech.rs"]
mod tests;
