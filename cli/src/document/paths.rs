//! A request's path table: each path once, in first-named order, its
//! index the integer a row carries and a `path` reference resolves
//! back through. One owner for every face that names paths in a
//! document request (join since plan v2.32 step 4, churn since step 5
//! R0) — the two copies were one clone block.

use std::collections::HashMap;

/// The path table: each path once, in first-named order.
#[derive(Default)]
pub struct Paths {
    pub list: Vec<String>,
    at: HashMap<String, usize>,
}

impl Paths {
    /// The path's index, entering it on first sight.
    pub fn id(&mut self, path: &str) -> i64 {
        if let Some(&i) = self.at.get(path) {
            return i as i64;
        }
        self.list.push(path.to_string());
        self.at.insert(path.to_string(), self.list.len() - 1);
        (self.list.len() - 1) as i64
    }

    /// The path's index if the table already holds it.
    pub fn find(&self, path: &str) -> Option<i64> {
        self.at.get(path).map(|&i| i as i64)
    }
}
