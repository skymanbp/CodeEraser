//! The Chinese surface, enumerated (K step 8).
//!
//! i18n.rs's charter says English is the default and Chinese is a
//! lookup switch. Nothing proved the switch actually covers what a
//! reader SEES: five console strings sat in English inside otherwise
//! Chinese reports for as long as they had existed, because every
//! test ran the default surface and a static grep cannot see a line
//! built by `format!` or spread across a macro's continuation.
//!
//! So this leg reads the surface itself. Each command runs under
//! `--lang zh`; a line that carries no Chinese character AND carries
//! English prose is a leak, unless it is one of the residue shapes
//! named below — which is the point of naming them: an accepted
//! English line stays visible and re-decidable instead of merging
//! into the noise. The mirror leg guards the other direction, the
//! one the charter calls HARD: the default surface must stay free of
//! Chinese, or the byte contract every other test relies on is gone.
//!
//! SCOPE, stated rather than implied: only the commands that need no
//! ce-core run here (a core-dependent leg would pass by skipping on
//! the machine that has no core, which is the failure mode step 10
//! exists to remove). scan / check / deadcode / docdup / structure /
//! erase are therefore NOT covered; they were read by hand at the
//! same time and were clean, which is evidence with a date on it,
//! not a standing guarantee.
//!
//! And the rule is LINE-level, which is the blind spot to know about:
//! `握手：失败 — proto mismatch: core 2.27.0 vs ce 6.1.0` passes,
//! because the line carries Chinese, while the tail is an anyhow
//! chain's English. Every leak this shape hides has the same origin —
//! prose minted below the presentation layer and carried as a string
//! — and that is fixed by making the producer emit a code, not by
//! teaching this rule to read half a line.

use std::process::{Command, Output};

/// A command shape under test, named as a reader would type it.
const SHAPES: &[&[&str]] = &[
    &[],
    &["graph"],
    &["graph", "--sites"],
    &["precommit"],
    &["doctor"],
    &["probe"],
    &["audit"],
    &["health"],
    &["--help"],
];

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .to_path_buf()
}

fn run(args: &[&str], zh: bool) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_ce"));
    c.current_dir(repo_root()).env_remove("CE_LANG").args(args);
    if zh {
        c.env("CE_LANG", "zh");
    }
    c.output().expect("run ce")
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn has_chinese(s: &str) -> bool {
    s.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

/// Two lowercase words of three or more letters standing NEXT TO
/// each other — the shape of prose, as opposed to an identifier
/// (`mod_decl`), a path, a version, or a lone noun like `proto`.
///
/// Adjacency is measured over whitespace tokens, never over a
/// filtered word list: filtering first made `ce-core 1.1.0 (proto
/// 6.1.0)` read as prose, because dropping the digits stood `core`
/// and `proto` side by side when the line never did. Enclosing
/// punctuation is stripped (so `mismatch:` still counts as a word)
/// while a token holding a digit is never one.
fn has_prose(s: &str) -> bool {
    let word = |t: &str| {
        let core = t.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        core.len() >= 3
            && core.chars().all(|c| c.is_ascii_lowercase())
            && !t.chars().any(|c| c.is_ascii_digit())
    };
    s.split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|w| word(w[0]) && word(w[1]))
}

/// English lines that stay English ON PURPOSE, each with the ruling
/// that grants it. The whole residue is here: anything else the leak
/// rule flags is a defect, not a convention.
fn accepted_residue(shape: &[&str], l: &str) -> bool {
    // clap's own framework vocabulary — main_lang.rs:11-12 rules that
    // `error:`, `Usage:` and the auto-generated `help` subcommand row
    // stay English, the same stance `check` takes with FAIL/pass
    let clap_frame = l.trim_start().starts_with("help ") && l.contains("Print this message");
    // the sites table is (language, site-kind, count) straight off
    // the parser: `rust  use  1007`. Those tokens are the same
    // identifiers ce.toml and the JSON face use, not prose
    let sites_row =
        shape == ["graph", "--sites"] && l.starts_with("  ") && l.split_whitespace().count() == 3;
    clap_frame || sites_row
}

#[test]
fn the_chinese_surface_leaks_no_english_prose() {
    let mut leaks: Vec<String> = Vec::new();
    for shape in SHAPES {
        for l in text(&run(shape, true)).lines() {
            if l.trim().is_empty() || has_chinese(l) || !has_prose(l) {
                continue;
            }
            if accepted_residue(shape, l) {
                continue;
            }
            leaks.push(format!("ce {}: {}", shape.join(" "), l.trim_end()));
        }
    }
    assert!(
        leaks.is_empty(),
        "{} English prose line(s) reached a Chinese reader. Either route \
         the string through i18n::line, or add it to accepted_residue \
         WITH the ruling that grants it:\n{}",
        leaks.len(),
        leaks.join("\n")
    );
}

#[test]
fn the_default_surface_stays_free_of_chinese() {
    // i18n.rs calls this the HARD constraint: the English path returns
    // the exact literal at the call site, because every assertion and
    // every machine consumer of console lines runs under the default.
    let mut bled: Vec<String> = Vec::new();
    for shape in SHAPES {
        for l in text(&run(shape, false)).lines() {
            if has_chinese(l) {
                bled.push(format!("ce {}: {}", shape.join(" "), l.trim_end()));
            }
        }
    }
    assert!(
        bled.is_empty(),
        "Chinese reached the DEFAULT surface, which is the byte \
         contract:\n{}",
        bled.join("\n")
    );
}
