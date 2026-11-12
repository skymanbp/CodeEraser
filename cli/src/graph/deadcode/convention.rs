//! The tool conventions (lane v235_licence, part 4A): files a tool finds
//! by where it sits, which nothing in the tree references, so without a
//! role of their own the graph would call every one dead. The rows are
//! the core's (CE.Lang.Common.Graph `flags.entry_conventions`, read off
//! `tables/1`): each a name and a position written as its tool's
//! documentation states it. This side holds no convention of its own —
//! only how each of the three position kinds reads a root-relative path.

use serde::Deserialize;

/// Where a convention's file sits. A row naming another kind, or a key
/// its kind does not take, does not read: the package is refused.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Position {
    /// The file's own name, at any depth.
    Basename { names: Vec<String> },
    /// The name below the FIRST directory segment named `segment`, where
    /// no later directory segment begins with `no_later_prefix` (no such
    /// bar when it is empty).
    Under {
        segment: String,
        names: Vec<String>,
        #[serde(default)]
        no_later_prefix: String,
    },
    /// Root-relative paths, compared without ASCII case when `nocase`.
    Exact {
        paths: Vec<String>,
        #[serde(default)]
        nocase: bool,
    },
}

impl Position {
    /// Whether a root-relative path sits where this position names.
    /// Names are literal: nothing in a row is a pattern.
    pub fn holds(&self, path: &str) -> bool {
        let (dirs, base) = match path.rsplit_once('/') {
            Some((dir, base)) => (dir.split('/').collect::<Vec<_>>(), base),
            None => (Vec::new(), path),
        };
        let named = |names: &[String]| names.iter().any(|n| n == base);
        match self {
            Position::Basename { names } => named(names),
            Position::Under {
                segment,
                names,
                no_later_prefix,
            } => {
                let barred = |d: &&str| {
                    !no_later_prefix.is_empty() && d.starts_with(no_later_prefix.as_str())
                };
                let first = dirs.iter().position(|d| d == segment);
                first.is_some_and(|i| !dirs[i + 1..].iter().any(barred)) && named(names)
            }
            Position::Exact { paths, nocase } => paths.iter().any(|p| {
                if *nocase {
                    p.eq_ignore_ascii_case(path)
                } else {
                    p == path
                }
            }),
        }
    }
}

/// Whether a root-relative path is a file some convention of the
/// definition package names.
pub(super) fn conventional(path: &str) -> bool {
    crate::tables::get()
        .flags
        .entry_conventions
        .iter()
        .any(|c| c.position.holds(path))
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/deadcode/convention.rs"]
mod tests;
