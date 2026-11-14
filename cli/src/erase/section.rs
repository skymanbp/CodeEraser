//! The links a doc segment's section receives (release 1.10.0, erase's
//! `linked_section`): of two byte-identical documentation segments,
//! the one a link's fragment lands on is referenced, so the side with
//! FEWER such links is the candidate and the count rides to the core,
//! which refuses a linked candidate. Erasing a segment leaves dangling
//! every link that lands on its section or on a target the erase
//! removes, so a segment answers for these targets:
//! - Markdown: every heading slug (the ladder's own slug rule,
//!   md_slug.rs) and every raw-HTML anchor (`<a id>`, `<a name>`,
//!   `<hN id>`, md_head.rs `anchors`) on a row from the nearest heading
//!   at or above the segment's first row through its last row (from the
//!   segment's first row when no heading is above it: then only the
//!   targets inside the segment count). A parent heading further up is
//!   not one.
//! - HTML: the innermost element carrying an `id` that holds the
//!   segment's first row, and every element carrying an `id` whose first
//!   row lies inside the segment (the section units' reading,
//!   fourclass/units.rs).
//!
//! The count is the link sites that land on any of them, each distinct
//! target once (a link site names one unit, so no site counts twice).
//! The links are the index's resolved edges that name that slug or id
//! as their unit (graph/load.rs `unit_links`), same-file links included.

use crate::scan::lang::Lang;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Link sites per (file, unit), as the index resolved them.
pub type Links = BTreeMap<(String, String), i64>;

/// How many link sites land on the targets of the segment on rows
/// `first..=last` of `path` (`text` its content).
pub fn links_to(links: &Links, path: &str, text: &str, (first, last): (i64, i64)) -> i64 {
    targets(path, text, first, last)
        .into_iter()
        .filter_map(|unit| links.get(&(path.to_string(), unit)))
        .sum()
}

/// The units a link's fragment may name that erasing the segment on rows
/// `first..=last` would leave dangling or empty.
fn targets(path: &str, text: &str, first: i64, last: i64) -> BTreeSet<String> {
    let (Ok(first), Ok(last)) = (usize::try_from(first), usize::try_from(last)) else {
        return BTreeSet::new();
    };
    match Lang::from_path(Path::new(path)) {
        Some(Lang::Html) => {
            let units = crate::fourclass::units::segments(text, Lang::Html);
            let holder = units
                .iter()
                .filter(|u| u.start_line <= first && first <= u.end_line)
                .max_by_key(|u| u.start_line);
            let inside = units
                .iter()
                .filter(|u| first <= u.start_line && u.start_line <= last);
            holder
                .into_iter()
                .chain(inside)
                .filter_map(|u| u.key.strip_prefix('#').map(str::to_string))
                .collect()
        }
        Some(Lang::Markdown) => {
            let slugs = crate::graph::ladder::md_slug::heading_slugs(text);
            let lo = slugs
                .iter()
                .take_while(|(row, _)| *row <= first)
                .last()
                .map_or(first, |(row, _)| *row);
            slugs
                .into_iter()
                .chain(crate::graph::ladder::md_head::anchors(text))
                .filter(|(row, _)| lo <= *row && *row <= last)
                .map(|(_, id)| id)
                .collect()
        }
        _ => BTreeSet::new(),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/erase/section.rs"]
mod tests;
