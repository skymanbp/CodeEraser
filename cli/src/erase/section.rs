//! The links a doc segment's section receives (release 1.10.0, erase's
//! `linked_section`): of two byte-identical documentation segments,
//! the one a link's fragment lands on is referenced, so the side with
//! FEWER such links is the candidate and the count rides to the core,
//! which refuses a linked candidate. A segment's section is the one its
//! first line falls in: the nearest heading at or above it in Markdown
//! (its slug by the ladder's own slug rule, md_slug.rs), the innermost
//! element carrying an `id` that holds it in HTML (the section units'
//! reading, fourclass/units.rs). A segment above the first heading has
//! no section and counts 0. The links are the index's resolved edges
//! that name that slug or id as their unit (graph/load.rs
//! `unit_links`), same-file links included.

use crate::scan::lang::Lang;
use std::collections::BTreeMap;
use std::path::Path;

/// Link sites per (file, unit), as the index resolved them.
pub type Links = BTreeMap<(String, String), i64>;

/// How many link sites land on the section holding line `line` of
/// `path` (`text` its content).
pub fn links_to(links: &Links, path: &str, text: &str, line: i64) -> i64 {
    section_of(path, text, line)
        .and_then(|unit| links.get(&(path.to_string(), unit)).copied())
        .unwrap_or(0)
}

/// The unit a link's fragment names for the section holding `line`.
fn section_of(path: &str, text: &str, line: i64) -> Option<String> {
    let line = usize::try_from(line).ok()?;
    match Lang::from_path(Path::new(path))? {
        Lang::Html => crate::fourclass::units::segments(text, Lang::Html)
            .into_iter()
            .filter(|u| u.start_line <= line && line <= u.end_line)
            .max_by_key(|u| u.start_line)
            .and_then(|u| u.key.strip_prefix('#').map(str::to_string)),
        Lang::Markdown => crate::graph::ladder::md::slug::heading_slugs(text)
            .into_iter()
            .take_while(|(row, _)| *row <= line)
            .last()
            .map(|(_, slug)| slug),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/erase/section.rs"]
mod tests;
