//! The package's shape (VERSIONING 7.7.0; the keys are the core's
//! CE.Lang reply, one per table family): typed once at read, so every
//! reader's query is a field or a hash lookup — never a walk over JSON.
//! A per-language table is keyed by the language's report name; the
//! language rows are indexed by wire code.

use super::leak::{Leak, leaked, owned};
use crate::scan::lang::Lang;
use std::collections::HashMap;
use std::path::PathBuf;

/// Names, kinds or tokens, as the field says.
type Names = &'static [&'static str];
/// A head and its words.
type Rows = &'static [(&'static str, &'static [&'static str])];
/// One string.
type Text = &'static str;
/// Bracket characters.
type Chars = &'static [char];
/// Rows whose head is itself a name list.
type Pairs = &'static [(Names, Names)];
/// Each language's call sites.
type CallSites = ByLang<Vec<crate::graph::spec::CallSite>>;
/// Each language's protected-call wrappers and their argument counts.
type Wrappers = ByLang<Vec<(String, usize)>>;
/// Each language's docstring hosts.
type DocSpecs = ByLang<crate::docdup::spec::DocSpec>;

/// One table per judged language, by report name.
pub struct ByLang<T>(HashMap<String, T>);

impl<T: Leak> Leak for ByLang<T> {
    type Owned = HashMap<String, T::Owned>;
    fn leak(owned: Self::Owned) -> Self {
        ByLang(owned.into_iter().map(|(k, v)| (k, T::leak(v))).collect())
    }
}

impl<T> ByLang<T> {
    /// This language's table, None for one the package does not judge
    /// (the scan-only arm, plain text, the sentinel).
    pub fn get(&self, lang: Lang) -> Option<&T> {
        self.0.get(lang.name())
    }
}

// The tables read whole, already owned: a FlowSpec or SlotSpec is
// owned strings, and the four derive their own Deserialize.
owned!(
    crate::flow::spec::FlowSpec,
    crate::merge::slot::SlotSpec,
    crate::graph::spec::Specifier,
    crate::scan::spec::NameStyle
);

leaked! {
    /// One language row (`languages.rows`), in wire-code order.
    LangRow { code: usize, name: Text, exts: Names, scan_only: bool, prose_only: bool,
              judged: bool, document: bool, flow_judged: bool }
    /// The language rows and the two extension tables beside them.
    Languages { rows: &'static [LangRow], machine_txt: Names, mention_whole_run_exts: Names }
    /// The calls that open a site, the protected-call wrappers and R's
    /// formals.
    Calls { calls: CallSites, protected: Wrappers, r_formals: Rows }
    /// The four-class register's kind tables: each language's extra
    /// unit kinds, then the shared ones.
    Fourclass { extra: ByLang<Names>, redeclaring: Names, package_level: Names,
                java_fields: Names, c_variable: Text, ts_lexical: Names, bodied: Names }
    /// GHC's global package table: each package and its modules.
    HsLadder { boot: Rows }
    /// The JDK's exported packages and `java.lang`'s public types.
    JavaLadder { packages: Names, lang: Names }
    /// Node's builtin modules and the ones only `node:` reaches.
    TsLadder { builtins: Names, prefix_only: Names }
    /// A ladder's one name table.
    StdLadder { std: Names }
    /// A ladder's standard-library table.
    StdlibLadder { stdlib: Names }
    /// Rust's toolchain crates.
    RsLadder { builtin: Names }
    /// The ladders' name tables.
    Ladder { hs: HsLadder, java: JavaLadder, ts: TsLadder, go: StdLadder, py: StdlibLadder,
             lua: StdlibLadder, rs: RsLadder }
    /// The walk's two built-in exclude lists.
    Walk { secret_globs: Names, builtin_excludes: Names }
    /// The docdup family's docstring hosts, markers, kind names and
    /// numbers.
    Docdup { doc_spec: DocSpecs, license_markers: Names, skeleton_prefixes: Names,
             allow_marker: Text, kind_names: Names, min_doc_tokens: usize,
             license_head_lines: i64, verbatim_floor: usize, doc_shingle: usize,
             doc_line_cap: usize }
    /// The resolver configuration names and the TypeScript twins.
    Keys { config_names: Names, twin_exts: Rows }
    /// The entry-file names and directories.
    Flags { entry_names: Names, entry_dirs: Rows }
    /// The tombstone reading's vocabularies and numbers.
    Tombstone { negations: Names, keywords: Names, en_prefix: Names, en_suffix: Names,
                zh_prefix: Names, zh_suffix: Names, marks_en: Names, marks_zh: Names,
                stop_en: Names, min_ascii_name: usize, min_wide_name: usize,
                join_max: usize, open: Chars, close: Chars }
    /// The compilation database's flag tables.
    Compdb { gnu: Names, skip: Names }
    /// The protocol names a loader, a framework or a platform calls.
    Protocol { py_names: Names, py_prefixes: Names, ts_by_stem: Pairs, ts_pages: Names,
               ts_routes: Names, java_names: Names, c_names: Names, c_prefixes: Names,
               lua_names: Names, love_names: Names, r_names: Names }
    /// One flow kind, a `[name, advisory, en, zh]` row: the name `--kind`
    /// and the feeds read, whether the kind is advisory in every
    /// language, and its display label in each language (plan v2.32
    /// step 5, R8: the GUI's hub reads the labels here).
    KindRow { name: Text, advisory: bool, en: Text, zh: Text }
    /// The flow document's catalogue entry: the kinds by code.
    DocFlow { kinds: &'static [KindRow] }
    /// The check document's catalogue entry (7.9.0): the fail-condition
    /// names and the degraded reasons by code — a face sends the codes.
    DocCheck { failed: Names, reasons: Names }
    /// A degrading family's catalogue entry (7.9.0): its reasons by code.
    DocReasons { reasons: Names }
    /// The entries this side reads of the report documents' catalogue
    /// (7.8.0): the schema ids and empty documents are the core's
    /// statement and never bound here.
    DocEntries { flow: DocFlow, check: DocCheck, join: DocReasons, deadcode: DocReasons,
                 graphscreen: DocReasons }
    /// The index's storage tables (7.9.0): the site kinds by their
    /// frozen storage code.
    Store { site_kinds: Names }
    /// Every table, as the core answers them less the envelope.
    Tables {
        languages: Languages,
        scan: ByLang<crate::scan::spec::LangSpec>,
        flow: ByLang<Option<crate::flow::spec::FlowSpec>>,
        slot: ByLang<Option<crate::merge::slot::SlotSpec>>,
        sites: ByLang<Vec<crate::graph::spec::SiteKind>>,
        calls: Calls, fourclass: Fourclass, ladder: Ladder, walk: Walk, outputs: Rows,
        docdup: Docdup, keys: Keys, flags: Flags, tombstone: Tombstone, compdb: Compdb,
        protocol: Protocol, document: DocCatalogue, store: Store,
    }
}

/// The report documents' catalogue: the entries this side reads,
/// whether each family's `--format json` face prints its document
/// indented (`pretty`, 7.10.0, plan v2.32 step 5 ruling R5) — a
/// catalogue fact, never a table here — and the churn entry's pairing
/// cap, which the measurement uses and the console names
/// (`cochangeFileCap`).
pub struct DocCatalogue {
    entries: DocEntries,
    pretty: HashMap<String, bool>,
    pub cochange_file_cap: usize,
}

impl std::ops::Deref for DocCatalogue {
    type Target = DocEntries;
    fn deref(&self) -> &DocEntries {
        &self.entries
    }
}

impl DocCatalogue {
    /// Whether `family`'s document prints indented; None = an entry
    /// that states no `pretty` (a pre-7.10.0 catalogue).
    pub fn pretty(&self, family: &str) -> Option<bool> {
        self.pretty.get(family).copied()
    }
}

/// The catalogue read whole: the typed entries, and every family's
/// `pretty` beside them.
#[derive(serde::Deserialize)]
#[serde(try_from = "serde_json::Value")]
pub struct DocCatalogueOwned {
    entries: <DocEntries as Leak>::Owned,
    pretty: HashMap<String, bool>,
    cochange_file_cap: usize,
}

impl TryFrom<serde_json::Value> for DocCatalogueOwned {
    type Error = serde_json::Error;
    fn try_from(v: serde_json::Value) -> Result<Self, Self::Error> {
        let pretty = v
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(family, entry)| Some((family.clone(), entry.get("pretty")?.as_bool()?)))
            .collect();
        let cap = v["churn"]["cochangeFileCap"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok());
        let Some(cochange_file_cap) = cap else {
            return Err(serde::de::Error::custom(
                "document.churn: no cochangeFileCap",
            ));
        };
        Ok(DocCatalogueOwned {
            entries: serde_json::from_value(v)?,
            pretty,
            cochange_file_cap,
        })
    }
}

impl Leak for DocCatalogue {
    type Owned = DocCatalogueOwned;
    fn leak(owned: DocCatalogueOwned) -> Self {
        DocCatalogue {
            entries: DocEntries::leak(owned.entries),
            pretty: owned.pretty,
            cochange_file_cap: owned.cochange_file_cap,
        }
    }
}

impl Languages {
    /// The rows one column picks, as a bitmask at their wire codes —
    /// the judged set (`judged`) and the flow family's (`flow_judged`).
    pub fn mask(&self, pick: impl Fn(&LangRow) -> bool) -> i64 {
        self.rows
            .iter()
            .filter(|row| pick(row))
            .fold(0, |m, row| m | (1 << row.code))
    }
}

/// Every table, read, with where it came from.
pub struct Pack {
    /// The core's digest of the package (its `digest`, its hello's
    /// `tablesDigest`).
    pub digest: u64,
    /// The cache file this package was read from or written to.
    pub source: PathBuf,
    /// Extension to wire code, built at read.
    by_ext: HashMap<&'static str, usize>,
    tables: Tables,
}

impl std::ops::Deref for Pack {
    type Target = Tables;
    fn deref(&self) -> &Tables {
        &self.tables
    }
}

impl Pack {
    /// Read the package text; Err = serde's own words (a missing key is
    /// "missing field `x`"), or language rows out of wire-code order.
    pub(super) fn read(text: &str, digest: u64, source: PathBuf) -> Result<Pack, String> {
        let owned: <Tables as Leak>::Owned =
            serde_json::from_str(text).map_err(|e| e.to_string())?;
        let tables = Tables::leak(owned);
        let rows = tables.languages.rows;
        if let Some((i, row)) = rows.iter().enumerate().find(|(i, row)| row.code != *i) {
            return Err(format!("languages row {i} carries code {}", row.code));
        }
        if rows.len() != Lang::ALL.len() {
            return Err(format!(
                "{} language rows for {} codes",
                rows.len(),
                Lang::ALL.len()
            ));
        }
        let by_ext = rows
            .iter()
            .flat_map(|row| row.exts.iter().map(move |&e| (e, row.code)))
            .collect();
        Ok(Pack {
            digest,
            source,
            by_ext,
            tables,
        })
    }

    /// The wire code an extension belongs to.
    pub fn ext_code(&self, ext: &str) -> Option<usize> {
        self.by_ext.get(ext).copied()
    }
}
