//! Language identification and per-language tree-sitter grammar lookup.
//! M1 launch set (plan §6 M1): TypeScript / Python / Rust / Go / Markdown
//! (Markdown is size-only: no grammar, no functions). M5-3k adds
//! Haskell (full grammar — spike pinned in tests/it/grammar_pins.rs).
//! Plan v2.5 adds the SCAN-ONLY arm: common front-end/script
//! extensions that enter the scan's size gates, the guard's hard
//! budget and the score ratchet — and nothing else (see scan_only).
//!
//! APPEND-ONLY: `Lang as i64` is a frozen wire position (graph node
//! rows) — inserting a variant would silently relabel every language
//! code downstream (RM15). Scan-only variants sit AFTER the sentinel
//! and never reach the wire anyway (they are never indexed).
//!
//! Plan v2.30 (docs/reference/language-expansion.md) reserves six
//! more codes after the arm — C / C++ / Lua / Java / Ruby / R — and
//! promotes HTML to a judged document language. Each row turns judged
//! in its own step, together with its measurement tables, so no file
//! is ever judged or fingerprinted under a placeholder spec; the core
//! read the judged set off the wire (`judgedMask`, proto 7.2.0) instead
//! of a constant until 8.0.0, and off its own language table since. Since plan v2.32 step 2 the rows themselves —
//! extensions, report name, the arm bits — are the core's
//! (CE.Lang.Common, read off `tables/1`, crate::tables): this file keeps
//! the enum, whose codes are the frozen wire positions, and the
//! grammars, which are this binary's own.
//! Step 2 turned C and C++ (`.h` is C++ by the 2026-09-24 ruling — a
//! header parsed as C loses every class body); step 3 turned Java; step
//! 4 turned Lua and R (R takes both `.R` and `.r`, the extension match
//! being exact); step 5 turned HTML (a document language: section units,
//! attribute sites and docdup text, never a fingerprint). Step 5b-8
//! (2026-09-27) adds the PROSE-ONLY arm: plain text (`.txt`) enters
//! the index for the docdup family alone (prose_only) — no size row,
//! no judgment, no graph file node.

use std::path::Path;
use tree_sitter_language::LanguageFn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Python,     // 0
    TypeScript, // 1
    Tsx,        // 2
    Rust,       // 3
    Go,         // 4
    Markdown,   // 5
    Haskell,    // 6
    /// Wire sentinel: "extension not ours". NEVER produced by
    /// from_path (which returns None) — it exists so the graph wire's
    /// unknown-language code is 7, not Python's 0 (RM15: today an
    /// unknown extension was indistinguishable from Python on the
    /// wire; the core's contract only rejects negatives).
    LangUnknown, // 7
    // ---- the v2.5 scan-only arm (never on the wire) ----
    JavaScript, // 8
    Css,        // 9
    Html,       // 10
    Vue,        // 11
    Svelte,     // 12
    Shell,      // 13
    Yaml,       // 14
    // ---- plan v2.30 reserved codes (language-expansion.md §1):
    // each turns judged in its own step, with its tables ----
    C,    // 15
    Cpp,  // 16
    Lua,  // 17
    Java, // 18
    Ruby, // 19
    R,    // 20
    // ---- plan v2.30 step 5b-8: the prose-only arm (prose_only) ----
    /// Plain text: a docdup corpus member and nothing else — no
    /// grammar, no unit, no site, no fingerprint, no graph file node.
    Text, // 21
}

impl Lang {
    /// Every variant in wire-code order: the package's language rows are
    /// indexed the same way (crate::tables checks the codes at read).
    pub const ALL: [Lang; 22] = codes::ALL;
}

/// The variants by bare name, so the code-order list reads as the
/// enum above does.
mod codes {
    use super::Lang::{self, *};
    pub const ALL: [Lang; 22] = [
        Python,
        TypeScript,
        Tsx,
        Rust,
        Go,
        Markdown,
        Haskell,
        LangUnknown,
        JavaScript,
        Css,
        Html,
        Vue,
        Svelte,
        Shell,
        Yaml,
        C,
        Cpp,
        Lua,
        Java,
        Ruby,
        R,
        Text,
    ];
}

/// The grammar of every AST-backed language, as the `LanguageFn`
/// value its crate exports (`Language::from` converts at lookup). A
/// row is five distinct tokens, under the clone gate's diversity
/// floor; a match arm per grammar read as clones of itself from the
/// eighth arm on (tests/it/grammar_pins.rs stores its pins the same
/// way, for the same reason). The plan v2.30 reserved codes join as
/// their steps land.
const GRAMMARS: [(Lang, LanguageFn); 12] = [
    (Lang::Python, tree_sitter_python::LANGUAGE),
    (
        Lang::TypeScript,
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
    ),
    (Lang::Tsx, tree_sitter_typescript::LANGUAGE_TSX),
    (Lang::Rust, tree_sitter_rust::LANGUAGE),
    (Lang::Go, tree_sitter_go::LANGUAGE),
    (Lang::Haskell, tree_sitter_haskell::LANGUAGE),
    (Lang::C, tree_sitter_c::LANGUAGE),
    (Lang::Cpp, tree_sitter_cpp::LANGUAGE),
    (Lang::Java, tree_sitter_java::LANGUAGE),
    (Lang::Lua, tree_sitter_lua::LANGUAGE),
    (Lang::R, tree_sitter_r::LANGUAGE),
    (Lang::Html, tree_sitter_html::LANGUAGE),
];

impl Lang {
    /// This language's row of the package — total by construction (the
    /// package carries one row per code, checked at read).
    fn row(self) -> &'static crate::tables::LangRow {
        &crate::tables::get().languages.rows[self as usize]
    }

    /// The language a path's extension names. A `.txt` name a
    /// specification reserves for a machine format (the package's
    /// `machine_txt`: CMake's list file, clang's flat compilation
    /// database, the robots file) is no language of ours.
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        let pack = crate::tables::get();
        let machine = |n: &std::ffi::OsStr| pack.languages.machine_txt.iter().any(|m| n == *m);
        if path.file_name().is_some_and(machine) {
            return None;
        }
        pack.ext_code(ext).map(|code| Self::ALL[code])
    }

    /// from_path in the judgment surfaces' form: None for unknown
    /// extensions, for the scan-only arm AND for the prose-only arm
    /// (plan v2.30 step 5b-8). fourclass, churn, structure, mention
    /// and tombstone inputs come through here; the scan and the
    /// guard's budget read sized_path, the index walk indexed_path.
    pub fn judged_path(path: &Path) -> Option<Self> {
        Self::from_path(path).filter(|l| !l.scan_only() && !l.prose_only())
    }

    /// from_path in the size surfaces' form (plan v2.30 step 5b-8):
    /// the judged set and the scan-only arm — every file the scan
    /// measures — and never the prose-only arm, which no size gate,
    /// hard budget, ratchet row or provenance entity counts.
    pub fn sized_path(path: &Path) -> Option<Self> {
        Self::from_path(path).filter(|l| !l.prose_only())
    }

    /// from_path in the index walk's form (plan v2.30 step 5b-8): the
    /// judged set and the prose-only arm — every file the index holds
    /// a row for — and never the scan-only arm.
    pub fn indexed_path(path: &Path) -> Option<Self> {
        Self::from_path(path).filter(|l| !l.scan_only())
    }

    /// A path of the prose-only arm (plan v2.30 step 5b-8): the graph
    /// loader's file-universe filter and the check universe's second
    /// tier ask by path.
    pub fn prose_path(path: &Path) -> bool {
        Self::from_path(path).is_some_and(Self::prose_only)
    }

    /// The judged-language set as a wire bitmask (H1 slice 2,
    /// 2.29.0): bit = Lang wire code, set = judged — the package's
    /// `judged` column (plan v2.32 step 2). No request carries it since
    /// 8.0.0 (plan v2.32 step 6): the core reads CE.Lang itself; the
    /// facts registry and the per-language ledgers read it here.
    pub fn judged_mask() -> i64 {
        crate::tables::get().languages.mask(|row| row.judged)
    }

    /// The plan v2.5 boundary predicate: a scan-only language enters
    /// the scan (size gates), the guard's hard budget and the score
    /// ratchet — and NEVER the dedup index, the graph, or any
    /// judgment family. Markdown is not in this class: it is
    /// grammar-less but fully judged (docdup corpus, graph ladder).
    pub fn scan_only(self) -> bool {
        self.row().scan_only
    }

    /// The plan v2.30 step 5b-8 boundary predicate: a prose-only
    /// language is read by the docdup family alone. Its files enter
    /// the index as docsegs rows and the check's file universe as
    /// docdup opportunities — and never the scan's size gates, the
    /// guard's budget, the ratchet, the graph's file nodes (a page
    /// naming one lands an asset node, as before), churn, fourclass,
    /// the mention domain, the tombstone reading or the audit's LOC
    /// ledger: it has no unit, no site, no name and no fingerprint.
    /// Plain text is the one member.
    pub fn prose_only(self) -> bool {
        self.row().prose_only
    }

    /// Grammar for AST-backed languages; None = grammar-less (Markdown,
    /// plain text, the scan-only arm), the wire sentinel (never walked) or a plan
    /// v2.30 reserved code whose grammar lands with its own step.
    pub fn grammar(self) -> Option<tree_sitter::Language> {
        GRAMMARS
            .iter()
            .find(|(l, _)| *l == self)
            .map(|&(_, f)| f.into())
    }

    /// The languages that parse, in GRAMMARS order — one per
    /// tree-sitter grammar (TypeScript and TSX are two grammars from
    /// one crate). The docs' grammar count reads this face (plan v2.30
    /// step 7); before it the registry scraped this file's text.
    pub fn with_grammar() -> impl Iterator<Item = Lang> {
        GRAMMARS.iter().map(|&(l, _)| l)
    }

    /// The T1/T2/T3 population (plan v2.30 §2): a grammar to tokenize
    /// with AND a code language. HTML parses — docdup segments, the
    /// reference ladder, section anchors — but never fingerprints: its
    /// leaves carry no identifiers, so any two pages would hash into
    /// one clone pair. The index refresh, the guard's clone probe and
    /// the unit cache read this; every other grammar consumer keeps
    /// reading `grammar()`.
    pub fn fingerprints(self) -> bool {
        self.grammar().is_some() && self != Self::Html
    }

    pub fn name(self) -> &'static str {
        self.row().name
    }

    /// The extensions this language's row claims — the face an
    /// exam table's scope is held to (plan v2.31 step 4: the flow
    /// exams walk one language's files, and their extension list must
    /// be this row's, not a hand copy of it).
    pub fn extensions(self) -> &'static [&'static str] {
        self.row().exts
    }
}

#[cfg(test)]
#[path = "../../tests/unit/scan/lang.rs"]
mod tests;
