//! The lowering's entry (plan v2.31 step 4 A2; design booklet
//! docs/reference/analysis-track.md §5.1): every unit scan extracts,
//! lowered into the four integer tables the core reads — statements
//! (tree.rs, tree_arms.rs, tree_jumps.rs), variables (scope.rs) and
//! accesses (access.rs) — and a legend naming what the integers stand
//! for (names and places never cross the wire). A unit whose statements
//! cannot take a legal shape is not sent: it is an `Unlowered` with its
//! reason (rule 11).

use super::build::Lowerer;
use super::finish::{finish, noreturn_in_file};
use super::spec::spec;
use crate::scan::functions;
use crate::scan::lang::Lang;

/// One file's units, lowered, and the ones that could not be.
pub struct Lowered {
    pub lang: Lang,
    pub units: Vec<Unit>,
    pub unlowered: Vec<Unlowered>,
}

/// A unit left out, and why.
pub struct Unlowered {
    pub nth: usize,
    pub name: String,
    pub start_line: u32,
    pub reason: String,
}

/// One unit's tables.
pub struct Unit {
    /// The unit's place in scan's extraction order.
    pub nth: usize,
    /// functions::name_of's spelling.
    pub name: String,
    /// 1-based, inclusive.
    pub start_line: u32,
    pub end_line: u32,
    /// The parameters in `vars` (the units row's `params`).
    pub params: u32,
    /// Some statement carries the dynamic flag: the core judges nothing.
    pub dynamic: bool,
    /// `[seq, parent, kind, flags, aux]`, seq from 0 in pre-order.
    pub stmts: Vec<[i64; 5]>,
    /// `[v, declSeq, flags]`.
    pub vars: Vec<[i64; 3]>,
    /// `[seq, v, mode]`, seq not decreasing, evaluation order within.
    pub uses: Vec<[i64; 3]>,
    pub legend: Legend,
}

/// What the integers stand for: for the faces and the exam, never the
/// wire.
pub struct Legend {
    /// Each seq's 1-based (line, column); a synthetic statement takes
    /// the place of the source node it stands for.
    pub stmt_at: Vec<(u32, u32)>,
    /// Each seq's last source line, 1-based: where an unreachable run
    /// ending at it ends (LEG-1; a face's lineEnd).
    pub stmt_end: Vec<u32>,
    /// Each seq's first source line, trimmed, at most 80 characters; a
    /// synthetic statement reads `<synthetic:…>`.
    pub stmt_text: Vec<String>,
    /// Each v's name.
    pub var_name: Vec<String>,
    /// Each v's declaring place (a parameter's: the parameter).
    pub var_at: Vec<(u32, u32)>,
}

/// The file's units lowered: None when the language has no flow table,
/// no units when the text does not parse.
pub fn lower_file(text: &str, lang: Lang) -> Option<Lowered> {
    let flow = spec(lang)?;
    let mut out = Lowered {
        lang,
        units: Vec::new(),
        unlowered: Vec::new(),
    };
    let Some(tree) = crate::scan::ast::parse_lang(text, lang) else {
        return Some(out);
    };
    let (root, src) = (tree.root_node(), text.as_bytes());
    let scan = crate::scan::spec::spec(lang);
    let doomed = noreturn_in_file(root, src, flow);
    for (nth, unit) in functions::extract(root, src, scan).into_iter().enumerate() {
        let lowered = Lowerer::run(unit.node, src, flow, scan, &doomed);
        let (start, end) = (unit.start_line as u32, unit.end_line as u32);
        match lowered.and_then(|done| finish(done, nth, unit.name.clone(), start, end)) {
            Ok(u) => out.units.push(u),
            Err(reason) => out.unlowered.push(Unlowered {
                nth,
                name: unit.name,
                start_line: start,
                reason,
            }),
        }
    }
    Some(out)
}

#[cfg(test)]
#[path = "../../tests/unit/flow/shape.rs"]
pub(crate) mod shape;

#[cfg(test)]
#[path = "../../tests/unit/flow/lang.rs"]
mod lang;
