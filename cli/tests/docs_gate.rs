//! G2 generated-docs drift gate (M8; the notice_gate / regen_tables
//! stance). The GENERATED FILE INVENTORY is exactly
//! docs/reference/cli.md and docs/reference/ce-toml.md — only this
//! generator writes them (CE_BLESS=1); a plain run byte-compares, so
//! a hand edit, or a surface change without a regen, is a red CI leg.
//!
//! No second authority anywhere: the CLI page is the real binary's
//! own `--help` output with the subcommand set parsed from the root
//! help (clap enumerates itself); the config page's key population
//! is harvested from serde's deny_unknown_fields refusal (probe,
//! don't guess — the hs_boot lesson), defaults from the ONE Debug
//! rendering of Config::default(), and only the per-key prose is
//! authored here — as a single TSV literal (the main_lang shape; a
//! tuple table would chain against HS_LICENSES under T2).

mod common;

use std::collections::BTreeSet;
use std::path::PathBuf;

/// Per-key prose, the only authored content. Key set is asserted
/// equal to the schema both ways: an undocumented field and a dead
/// key are each a red gate naming the offender.
const KEY_DOC_TSV: &str = "\
exclude\tExtra exclude globs, added on top of the built-in defaults (plan §4.1)
thresholds.file_lines_warn\tWarn once a source file passes this line count
thresholds.file_lines_fail\tHard file budget: scan fails past it, and the guard refuses writes that would land past it
thresholds.fn_lines_warn\tWarn once a function passes this line count
thresholds.fn_lines_fail\tFail once a function passes this line count
thresholds.params_warn\tWarn once a function takes more parameters than this
thresholds.cyclomatic_warn\tWarn past this cyclomatic complexity
thresholds.cognitive_warn\tWarn past this cognitive complexity
thresholds.nesting_warn\tWarn past this block-nesting depth
guard.mode\tExplicit hook tier for every rule class: observe / warn / ask / deny; unset = per-class route defaults (deny for the two FPR-promoted classes, observe otherwise). Any other value is a typo, not a tier: it resolves to observe and the SessionStart line, `ce doctor` and the observe feed all name it, so a mistyped mode can never look armed
guard.zone_tiers\tArm the graded-zone tier map (plan v2.7): a write landing <25% into (softLine, hard budget] stays observe, 25-75% warns, >75% asks. Default OFF - the zone is feed-only until a repo opts in, and the observe feed records the mapped tier when armed
dedup.budget\tOnly-shrink clone-block budget; `ce dedup --check` fails when the repo exceeds it
graph.entry_globs\tExtra liveness roots for the deadcode judgment, beyond the mechanical entry conventions
score.weights\tPer-axis weight numerators by axis name (size / complexity / clone / docdup / deadcode / churn / cycle); unlisted axes keep the equal default
score.size_penalty_max\tSoft-zone curve: the size-axis penalty of a file AT the hard line (plan v2.6; default 10)
score.soft_line_k\tRelative soft line: the multiplicative-MAD exponent k in S = clamp(median*r^k, [200,500]) (default 2)
score.dead_indeg_ceil\tDeadcode axis: a file at or below this in-degree (and unreachable) counts as orphaned
score.rewrite_num\tChurn axis rewrite-share threshold, ratio numerator (cross-multiplied)
score.rewrite_den\tChurn axis rewrite-share threshold, ratio denominator
score.cochange_floor\tCo-change count at which a pair counts as entangled
score.viol_cost\tPer-mille cost of one weighted violation in the score fold
score.default_weight\tWeight of any axis the weights table does not name
score.score_scale\tThe score's full scale (per-mille by default)
score.tol_num\tRatchet ratio leg numerator: a ceiling may grow to ceiling*tol_num/tol_den in one edit
score.tol_den\tRatchet ratio leg denominator
score.tol_abs\tRatchet absolute leg: or by this many lines, whichever is larger
structure.layout\tDeclared directory weights the divergence axis judges against; keys are root-relative directories, \".\" the catch-all bin
trend.min_points\tHistory points required before a slope is judged
trend.decline_floor_micro\tDecline floor in micro-per-mille per day; declaring it arms the trend fail bit
rules.class	Path classes with their own size and complexity lines (plan v2.13): an array of tables, each with a local `name`, its `globs` (the exclude list's dialect; the first declared match owns a path, an unmatched path keeps the global table) and `knobs` — `file_lines_warn`, `file_lines_fail`, `cognitive_warn`, `fn_lines_warn`, `fn_lines_fail`; an absent knob inherits the global line. The score reads the first three, the scan ladder all five. At most 64 classes, each class's ladder must climb, and only a class's index and knobs ever cross the wire. Enabling classes changes what the score judges a file against — scores are not comparable across that switch
";

const GEN_BANNER: &str = "<!-- GENERATED — do not edit. \
Regenerate: CE_BLESS=1 cargo test --test docs_gate. \
CI reddens when this file drifts from its regeneration. \
Length rides the CLI surface (a machine-generated projection, the \
hs_boot stance), so the scan's file-lines warn on the CLI page is an \
accounted standing warn, not maintained prose over budget. -->";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .to_path_buf()
}

/// One `--help` face off the real binary, language pinned to en.
fn help_of(sub: Option<&str>) -> String {
    let mut c = std::process::Command::new(env!("CARGO_BIN_EXE_ce"));
    if let Some(s) = sub {
        c.arg(s);
    }
    let out = c
        .arg("--help")
        .env_remove("CE_LANG")
        .output()
        .expect("run ce");
    assert!(out.status.success(), "`ce {sub:?} --help` exited nonzero");
    String::from_utf8(out.stdout)
        .expect("utf8")
        .replace("\r\n", "\n")
}

/// Subcommand names off the root help's `Commands:` section — clap's
/// own enumeration, so a new subcommand reaches the page with no
/// second list to forget. Name lines sit at exactly two spaces;
/// wrapped about-text is indented deeper.
fn subcommands(root_help: &str) -> Vec<String> {
    root_help
        .split("Commands:")
        .nth(1)
        .expect("root help has a Commands: section")
        .lines()
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| l.starts_with("  "))
        .filter(|l| !l.starts_with("    "))
        .filter_map(|l| l.split_whitespace().next())
        .filter(|n| *n != "help")
        .map(str::to_string)
        .collect()
}

fn cli_page() -> String {
    let root = help_of(None);
    let mut s = format!(
        "{GEN_BANNER}\n\n# `ce` command reference\n\nEvery block below is \
         the binary's own `--help` output (English face; `--lang zh` or \
         `CE_LANG=zh` switches the console at runtime).\n\n## ce\n\n```text\n{root}```\n"
    );
    for sub in subcommands(&root) {
        let h = help_of(Some(&sub));
        // a nested subcommand would silently under-document — red
        // here forces the generator to grow with the surface
        assert!(!h.contains("Commands:"), "nested commands under `ce {sub}`");
        s += &format!("\n## ce {sub}\n\n```text\n{h}```\n");
    }
    s
}

/// serde's own field list for one ce.toml table, harvested from the
/// deny_unknown_fields refusal a canary key provokes: every
/// backticked name after "expected".
fn probed_keys(section: Option<&str>) -> BTreeSet<String> {
    let doc = match section {
        None => "__canary__ = 1".to_string(),
        Some(s) => format!("[{s}]\n__canary__ = 1"),
    };
    let err = toml::from_str::<codeeraser::config::Config>(&doc)
        .expect_err("canary key must be refused")
        .to_string();
    let tail = err
        .split("expected")
        .nth(1)
        .unwrap_or_else(|| panic!("no expected-fields list in: {err}"));
    tail.split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// (field, value-literal) pairs at one brace depth of a Debug
/// rendering — Config::default() is the single machine source for
/// every default shown on the page.
fn debug_pairs(dbg: &str) -> Vec<(String, String)> {
    let inner = dbg
        .split_once('{')
        .and_then(|(_, r)| r.rsplit_once('}'))
        .expect("struct Debug shape")
        .0;
    let (mut out, mut depth, mut start) = (Vec::new(), 0usize, 0);
    for (i, b) in inner.bytes().enumerate() {
        match b {
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth -= 1,
            b',' if depth == 0 => {
                push_pair(&mut out, &inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    push_pair(&mut out, &inner[start..]);
    out
}

fn push_pair(out: &mut Vec<(String, String)>, field: &str) {
    if let Some((k, v)) = field.split_once(':') {
        out.push((k.trim().to_string(), v.trim().to_string()));
    }
}

/// Empty containers and None all mean the same thing on this page.
fn default_cell(v: &str) -> String {
    match v {
        "None" | "[]" | "{}" => "*(absent)*".into(),
        _ => format!("`{v}`"),
    }
}

/// One markdown table for one group of keys, defaults joined from
/// the Debug pairs, prose from the TSV; `used` collects coverage for
/// the final set-equality check.
fn key_table(prefix: &str, fields: &[(String, String)], used: &mut BTreeSet<String>) -> String {
    let doc: std::collections::HashMap<&str, &str> = KEY_DOC_TSV
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .collect();
    let mut s = String::from("| key | default | meaning |\n|---|---|---|\n");
    for (k, v) in fields {
        let full = if prefix.is_empty() {
            k.clone()
        } else {
            format!("{prefix}.{k}")
        };
        let text = doc
            .get(full.as_str())
            .unwrap_or_else(|| panic!("no TSV doc line for `{full}`"));
        s += &format!("| `{k}` | {} | {text} |\n", default_cell(v));
        used.insert(full);
    }
    s
}

fn config_page() -> String {
    let dbg = format!("{:?}", codeeraser::config::Config::default());
    let top = debug_pairs(&dbg);
    let mut used = BTreeSet::new();
    let mut s = format!(
        "{GEN_BANNER}\n\n# `ce.toml` reference\n\nDeclarative-only by design \
         (plan §5.9): no executable fields, ever. An unknown key is a parse \
         error (`deny_unknown_fields`) — the guard then degrades to observe \
         and NAMES the error instead of silently dropping a mistyped policy. \
         An *(absent)* judgment knob means the Haskell core's own default \
         judges; every report echoes its effective values under `knobs`.\n"
    );
    let leaves: Vec<_> = top
        .iter()
        .filter(|(_, v)| !is_section(v))
        .cloned()
        .collect();
    s += &format!("\n## Top level\n\n{}", key_table("", &leaves, &mut used));
    for (name, v) in top.iter().filter(|(_, v)| is_section(v)) {
        let fields = debug_pairs(v);
        let probed = probed_keys(Some(name));
        let named: BTreeSet<String> = fields.iter().map(|(k, _)| k.clone()).collect();
        // serde-facing names vs struct fields: a future rename would
        // split the two sources — red, never a silently wrong page
        assert_eq!(probed, named, "[{name}] serde/Debug field sets differ");
        s += &format!("\n## [{name}]\n\n{}", key_table(name, &fields, &mut used));
    }
    let tsv: BTreeSet<String> = KEY_DOC_TSV
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(k, _)| k.to_string())
        .collect();
    assert_eq!(
        tsv, used,
        "TSV keys != schema keys (dead or missing doc line)"
    );
    s
}

fn is_section(v: &str) -> bool {
    v.split_once(" {")
        .is_some_and(|(t, _)| t.chars().all(|c| c.is_ascii_alphanumeric()))
}

#[test]
fn cli_reference_matches_its_regeneration() {
    common::assert_matches_golden(&cli_page(), &repo_root().join("docs/reference/cli.md"));
}

#[test]
fn config_reference_matches_its_regeneration() {
    // the top-level probe must see every section the page renders —
    // ties the section walk itself to serde's own key list
    let probed = probed_keys(None);
    let dbg = format!("{:?}", codeeraser::config::Config::default());
    let named: BTreeSet<String> = debug_pairs(&dbg).iter().map(|(k, _)| k.clone()).collect();
    assert_eq!(probed, named, "top-level serde/Debug field sets differ");
    common::assert_matches_golden(
        &config_page(),
        &repo_root().join("docs/reference/ce-toml.md"),
    );
}
