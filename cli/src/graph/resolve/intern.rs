//! The request's segment table (plan v2.33 wave W2a): every string the
//! resolve/1 search needs — a directory or file name, a piece of a
//! specifier, a word of the core's vocabulary — gets one dense id, and
//! only ids cross the wire. Equal strings are one id, so the core
//! compares names by comparing integers.

use std::collections::HashMap;

#[derive(Default)]
pub struct Intern {
    ids: HashMap<String, i64>,
}

impl Intern {
    /// The id of a string, minted on first sight.
    pub fn id(&mut self, s: &str) -> i64 {
        let next = self.ids.len() as i64;
        *self.ids.entry(s.to_string()).or_insert(next)
    }

    /// Every piece of a string split at one separator, empty pieces
    /// kept (`"a//b"` is three pieces), each as its id — a raw
    /// directory's spelling, or a path the core will read.
    pub fn pieces(&mut self, s: &str, sep: char) -> Vec<i64> {
        s.split(sep).map(|p| self.id(p)).collect()
    }

    /// A normalized repo-relative directory's pieces: the root is none.
    pub fn dir(&mut self, s: &str) -> Vec<i64> {
        s.split('/')
            .filter(|p| !p.is_empty())
            .map(|p| self.id(p))
            .collect()
    }

    /// How many ids exist (the request's `segs`).
    pub fn count(&self) -> usize {
        self.ids.len()
    }
}
