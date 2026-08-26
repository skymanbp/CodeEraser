//! The how page's family-count reconciliation note, gated (K step 9).
//! The note exists precisely because three "family" counts ship on
//! this site meaning different things — and the note itself then
//! drifted: K step 7 grew the MCP catalog 11 → 13 (erase + doctor)
//! and both language mirrors kept saying eleven, with a clause
//! ("erase plans and doctor deliberately get none") that the same
//! batch made false. A count that restates a code-derived number is
//! ungated prose until something re-derives it; this does.
//!
//! The wire-capability count ("ten") is deliberately NOT gated here:
//! its authority is the core's dispatch, which no cheap Rust-side
//! probe reaches, and it moves only in wire-major work that rewrites
//! these pages anyway.

mod common;
use common::repo_root;

/// The number spelled the way the note spells it. Both mirrors write
/// words, not digits, so the gate speaks words too; a count outside
/// the table is the gate telling its maintainer to extend it, not a
/// reason to weaken the match.
fn words(n: usize) -> (&'static str, &'static str) {
    match n {
        10 => ("ten", "十"),
        11 => ("eleven", "十一"),
        12 => ("twelve", "十二"),
        13 => ("thirteen", "十三"),
        14 => ("fourteen", "十四"),
        15 => ("fifteen", "十五"),
        _ => panic!("extend the words table: the count reached {n}"),
    }
}

/// MCP tools = `tool!(` rows inside the TOOLS table. Counted from the
/// source slice between the table's opening and its closing bracket,
/// because the macro's own definition above it also spells `tool!(`
/// — the docs_consts harvest-from-source stance (probe, don't
/// guess), scoped so the probe cannot count the definition.
fn mcp_tool_count() -> usize {
    let src = std::fs::read_to_string(repo_root().join("cli/src/mcp/tools.rs")).expect("tools.rs");
    let table = src.split("pub const TOOLS").nth(1).expect("TOOLS table");
    let table = table.split("];").next().expect("table end");
    table.matches("tool!(").count()
}

fn booklet_count() -> usize {
    std::fs::read_dir(repo_root().join("docs/reference/methodology"))
        .expect("methodology dir")
        .filter(|e| {
            e.as_ref()
                .expect("entry")
                .path()
                .extension()
                .is_some_and(|x| x == "md")
        })
        .count()
}

#[test]
fn the_reconciliation_note_counts_match_the_code() {
    let (mcp_en, mcp_zh) = words(mcp_tool_count());
    let (bk_en, bk_zh) = words(booklet_count());
    let cases = [
        (
            "site/how/index.html",
            format!("<b>{mcp_en}</b> read-only MCP tools"),
            format!("<b>{bk_en}</b> booklets"),
        ),
        (
            "site/zh/how/index.html",
            format!("只读 MCP <b>{mcp_zh}</b> 个工具"),
            format!("下面 <b>{bk_zh}</b> 册"),
        ),
    ];
    for (page, mcp, booklets) in cases {
        let html = std::fs::read_to_string(repo_root().join(page)).expect(page);
        for needle in [&mcp, &booklets] {
            assert!(
                html.contains(needle.as_str()),
                "{page}: the note no longer says {needle:?} — the count moved and the prose did not"
            );
        }
    }
}
