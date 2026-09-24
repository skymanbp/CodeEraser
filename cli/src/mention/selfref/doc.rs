//! The doc-comment half of the self-mention regions (split from
//! selfref.rs when plan v2.30 step 3 gave Java and the C family their
//! doc grammars): the runs of consecutive doc comments, each comment's
//! text past its markers, and the code blocks the language's doc tool
//! renders — where a same-file mention hands the name to a second
//! interpreter (a doctest, a rendered example), never plain prose.

mod blocks;

use crate::mention::conv::text;
use crate::scan::lang::Lang;
use tree_sitter::Node;

/// The maximal runs of consecutive doc-comment nodes (§2 (c)): Rust
/// and C / C++ `///`/`//!` line comments and `/** */`/`/*! */` blocks,
/// Java `/** */` blocks and `///` Markdown comments, Haskell `haddock`
/// nodes. A run ends at the first non-doc node or row gap; its code
/// blocks (`code_blocks`) are the region.
pub(super) struct Runs {
    lang: Lang,
    lines: Vec<String>,
    end_row: Option<usize>,
}

impl Runs {
    pub(super) fn new(lang: Lang) -> Self {
        Runs {
            lang,
            lines: Vec::new(),
            end_row: None,
        }
    }

    pub(super) fn feed(&mut self, node: Node<'_>, src: &[u8], out: &mut Vec<String>) {
        let Some(body) = doc_body(self.lang, node, src) else {
            // a comment's own marker children are not a break in the
            // run; any other node is
            if !node.kind().ends_with("comment_marker") && node.kind() != "doc_comment" {
                self.flush(out);
            }
            return;
        };
        if self
            .end_row
            .is_some_and(|r| r + 1 < node.start_position().row)
        {
            self.flush(out);
        }
        // an empty doc line is still a line of the run (the blank an
        // indented block follows), though a Java or C comment node
        // stops before its newline and so has no line to split
        let empty = body.is_empty().then(String::new);
        self.lines
            .extend(body.lines().map(str::to_string).chain(empty));
        // a Rust line comment's node spans its newline: its LAST row
        // of text is the one adjacency is measured from
        let end = node.end_position();
        self.end_row = Some(end.row - usize::from(end.column == 0 && end.row > 0));
    }

    pub(super) fn flush(&mut self, out: &mut Vec<String>) {
        if !self.lines.is_empty() {
            out.extend(blocks::code_blocks(
                self.lang,
                &std::mem::take(&mut self.lines),
            ));
        }
        self.end_row = None;
    }
}

/// The comment's text with its doc markers stripped, or None for a
/// node that is not a doc comment.
fn doc_body(lang: Lang, node: Node<'_>, src: &[u8]) -> Option<String> {
    let t = text(node, src);
    match (lang, node.kind()) {
        (Lang::Rust, "line_comment") => line_doc(t, &["///", "//!"]),
        (Lang::Rust, "block_comment") => block_doc(t, &["/**", "/*!"]),
        (Lang::Java, "line_comment") => line_doc(t, &["///"]),
        (Lang::Java, "block_comment") => block_doc(t, &["/**"]),
        // one `comment` kind spells both C-family forms
        (Lang::C | Lang::Cpp, "comment") => {
            line_doc(t, &["///", "//!"]).or_else(|| block_doc(t, &["/**", "/*!"]))
        }
        (Lang::Haskell, "haddock") => Some(
            t.lines()
                .map(|l| {
                    let l = l.trim_start();
                    let l = l
                        .strip_prefix("--")
                        .or_else(|| l.strip_prefix("{-"))
                        .unwrap_or(l);
                    l.strip_suffix("-}")
                        .unwrap_or(l)
                        .trim_start_matches(['|', '^', '$', '*'])
                })
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        _ => None,
    }
}

/// A line doc comment's text past its marker; a fourth slash (`////`)
/// is a plain comment.
fn line_doc(t: &str, markers: &[&str]) -> Option<String> {
    let rest = markers.iter().find_map(|m| t.strip_prefix(m))?;
    (!rest.starts_with('/')).then(|| rest.to_string())
}

/// A block doc comment's lines, each past its leading `*`; `/**/` and a
/// `/***` banner are plain comments.
fn block_doc(t: &str, markers: &[&str]) -> Option<String> {
    let inner = markers
        .iter()
        .find_map(|m| t.strip_prefix(m))?
        .strip_suffix("*/")?;
    (!inner.starts_with('*')).then(|| {
        inner
            .lines()
            .map(|l| l.trim_start().strip_prefix('*').unwrap_or(l.trim_start()))
            .collect::<Vec<_>>()
            .join("\n")
    })
}
