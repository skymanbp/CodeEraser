//! The CLI flags that are NOT document options (plan v2.33 step 10),
//! one row each with the reason — the only list the three-face parity
//! gate subtracts. Every other long flag of a report subcommand changes
//! the document it prints, so the MCP tool and the Tauri command of
//! the same family take it too (spelled with `_` for `-`, or by its
//! machine name in `ALIASES`); a flag missing here and missing from a
//! machine face is a missing cell, red in the gate
//! (`cli/tests/it/face_parity*.rs`).

/// `(flag, why)`: the long flag without its dashes, and why no machine
/// face takes it.
pub const EXEMPT: &[(&str, &str)] = &[
    (
        "db",
        "plumbing: the index path; the machine faces read the project's default index",
    ),
    (
        "format",
        "rendering: console lines or the document; a machine face always hands out the document",
    ),
    ("lang", "rendering: the console's language"),
    (
        "core",
        "plumbing: the ce-core binary; the machine faces resolve it the daemon's way",
    ),
    ("roast", "rendering: one roast line on the console verdict"),
    (
        "group",
        "rendering: one merge group on the console; the document stays whole",
    ),
    (
        "check",
        "exit gate: the machine faces are report-only by the standing rule",
    ),
    (
        "apply",
        "mutation: the GUI asks through its own confirm flow; MCP has none by design",
    ),
    (
        "yes",
        "mutation: the GUI asks through its own confirm flow; MCP has none by design",
    ),
    (
        "installer",
        "mutation: the GUI asks through its own confirm flow; MCP has none by design",
    ),
    (
        "prelude",
        "a mode, not an option: the pack text is its own document, reached by the MCP `query` \
         tool's `prelude` and the GUI's Prelude button",
    ),
    (
        "mentions",
        "CLI only: the census behind the symbol advisory, which every face of `deadcode` carries",
    ),
];

/// A document option whose CLI spelling differs from the machine
/// faces' name — both shipped before the faces were compared, and a
/// rename would break the callers of one of them: `(cli flag, machine
/// name)`.
pub const ALIASES: &[(&str, &str)] = &[("fail-under", "floor"), ("split-candidates", "split")];
