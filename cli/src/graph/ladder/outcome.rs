//! The outcome vocabulary of a resolution site (moved out of
//! ladder/mod.rs, plan v2.33 wave W2a): the rung that answered, the
//! frozen refusal reasons with their resolve/1 codes, and the terminal
//! outcome every ladder (and the core's reply) produces.

/// Which rung answered (1-based per the design §4 table); stored on
/// every edge — ammunition for the per-level cut table.
pub type Rung = u8;

/// Unresolved reasons — the frozen design §4 vocabulary. `Dynamic`
/// and `Macro` stay structurally empty so far: the site detector
/// never opens dynamic imports or macro output (py.rs / rs.rs module
/// headers state each mechanism).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Dynamic,
    AmbiguousPaths,
    AmbiguousRoot,
    AmbiguousWorkspace,
    AmbiguousExports,
    Macro,
    ConfigDepth,
    OutOfScope,
    Unsupported,
    /// A degenerate specifier (`import ""`): the site referenced
    /// nothing, and says so in the ledger rather than vanishing at
    /// detection (L step #15, O60).
    Empty,
    /// The name is declared in the referencing file itself — Java's
    /// own compilation unit (the core's CE.Resolve.JavaPick `ownUnit`): no other file is
    /// referenced, so no edge is drawn, and the ledger says why
    /// (plan v2.30 step 5).
    OwnUnit,
}

impl Reason {
    /// The reasons in declaration order: a code is its index (the
    /// resolve/1 reply's reason column).
    const ALL: [Reason; 11] = [
        Reason::Dynamic,
        Reason::AmbiguousPaths,
        Reason::AmbiguousRoot,
        Reason::AmbiguousWorkspace,
        Reason::AmbiguousExports,
        Reason::Macro,
        Reason::ConfigDepth,
        Reason::OutOfScope,
        Reason::Unsupported,
        Reason::Empty,
        Reason::OwnUnit,
    ];

    pub fn from_code(code: i64) -> Option<Reason> {
        usize::try_from(code)
            .ok()
            .and_then(|i| Self::ALL.get(i).copied())
    }
}

/// Terminal state of one site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Exactly one in-scope target (repo-relative, forward slashes).
    Resolved {
        path: String,
        rung: Rung,
    },
    /// Exactly one in-scope PACKAGE directory (Go import, Markdown
    /// directory link): the node identity is (pkg_dir, "") and
    /// granularity is package — collapsing to a single file would be
    /// a guess (design §4 row 4).
    ResolvedPackage {
        dir: String,
        rung: Rung,
    },
    /// Markdown doc target (design §4 row 5): node identity is
    /// (path, slug), granularity section. `slug: None` is the
    /// anchored link whose section claim could not be confirmed
    /// against the target's ATX slug set — the design's "degrade to
    /// file level + ambiguous_anchor": the edge lands file-level
    /// (dst_unit ""), and the refusal to guess a section stays
    /// visible here instead of vanishing into a plain Resolved.
    ResolvedSection {
        path: String,
        slug: Option<String>,
        rung: Rung,
    },
    /// Resolved THROUGH a terminal file's re-export surface (§4 R5
    /// as amended 2026-08-18: one hop to the definition file; the
    /// via_reexport mark rides the edge row). Same edge semantics as
    /// Resolved everywhere except the stored flag.
    ResolvedVia {
        path: String,
        rung: Rung,
    },
    /// Resolved to an in-scope target that must NOT count as a
    /// reference (H1 slice 16, 2.29.0): the unused reference
    /// definition — user decision D3 made it ledger-visible, and
    /// this variant is the outcome→edge-kind channel that lets the
    /// edge TRAVEL (EDGE_REFDEF_UNUSED) while the CORE owns the
    /// liveness exclusion (inert kinds beside assetKind).
    ResolvedInert {
        path: String,
        rung: Rung,
    },
    /// Outside the corpus by design (registry dep, node_modules).
    External {
        rung: Rung,
    },
    Unresolved(Reason),
}
