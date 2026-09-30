//! The code-query family (plan v2.31 step 2; ADR-008 seventh
//! instalment, design booklet docs/reference/analysis-track.md §4):
//! a Datalog over the index's facts, behind `ce query` (one question,
//! its answers with their derivations), `ce rules` (a file of
//! assertions turned into a gate), the MCP tools `query` / `rules`
//! and the GUI's Query screen — one document for all three faces.
//! The division: this side lexes the text (lexer.rs), assembles the
//! fact tables the program names from its own index (facts/) and
//! labels the answers back (face.rs); the grammar, the sorts, the
//! safety and stratification checks, the evaluation and every proof
//! are the core's (query/1, wire.rs). The built-in prelude below
//! goes up the same wire as the user's text, and its predicates are
//! reserved.

pub mod columns;
pub mod console;
pub mod face;
pub mod facts;
pub mod legend;
pub mod lexer;
pub mod program;
pub mod wire;

/// The built-in prelude (booklet §4.3), a `.rules` text: `ce query
/// --prelude` prints it verbatim.
pub const PRELUDE: &str = include_str!("prelude.rules");

/// The rules file's default name at the project root.
pub const RULES_FILE: &str = "ce.rules";

/// The rules file a face reads, as (label, text): the one named on
/// the command line or by the MCP argument (root-relative unless
/// absolute; it must exist), else `[rules] file` from the config (it
/// must exist), else `ce.rules` at the project root when it exists —
/// else none, and the program is the prelude alone.
pub fn rules_source(
    root: &std::path::Path,
    file: Option<&std::path::Path>,
) -> anyhow::Result<Option<(String, String)>> {
    use anyhow::Context;
    let project = crate::root::project_root(root);
    let declared = crate::config::Config::load(root)
        .map_err(anyhow::Error::msg)?
        .rules
        .file;
    let (path, required) = match (file, declared) {
        (Some(f), _) => (project.join(f), true),
        (None, Some(d)) => (project.join(d), true),
        (None, None) => (project.join(RULES_FILE), false),
    };
    let label = path.to_string_lossy().replace('\\', "/");
    if !required && !path.is_file() {
        return Ok(None);
    }
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("read rules file {label}"))?;
    Ok(Some((label, text)))
}
