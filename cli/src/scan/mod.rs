//! `ce scan` orchestration: walk → parse → measure → judge → emit.
//! Since ADR-008 P3 the LEVEL judgment is the core's (scan/1, the
//! graded verdict table); measurement and report rendering stay
//! here, and the local evaluate() binding survives as the pinned
//! mirror the whole-report ensure proves equal on every judged run
//! — CLI gate, MCP tool and GUI face alike since batch-7 slice 8.

pub mod ast;
pub mod binding;
pub mod callees;
pub mod calls;
pub mod chunk;
pub mod classes;
pub mod complexity;
pub mod declarator;
pub mod functions;
pub mod globs;
pub mod html;
pub mod lang;
pub mod metrics;
pub mod opaque;
pub mod outputs;
pub mod report;
pub mod spec;
pub mod walk;
pub mod wire;

use anyhow::{Context, Result};
use lang::Lang;
use metrics::{FileMetrics, FnMetrics};
use std::path::Path;
use std::process::ExitCode;

pub enum Format {
    Console,
    Json,
    Sarif,
}

pub fn run(root: &Path, format: Format, core: &str) -> Result<ExitCode> {
    let (files, findings, summary, fail, failed) = analyze_judged(root, core)?;
    match format {
        Format::Console => report::print_console(&findings, &summary, &failed),
        Format::Json => println!("{}", report_string(&files, &findings, summary, &failed)?),
        Format::Sarif => println!("{}", report::sarif_string(&findings)?),
    }
    Ok(if fail {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

/// Measurement alone — the walk every scan surface shares; the score
/// and structure families reuse it for their LOC/tree facts without
/// ever touching a verdict.
pub fn measure(root: &Path) -> Result<(crate::config::Config, Vec<FileMetrics>)> {
    walk::each_surviving(root, |path, language, src| {
        measure_file(src, path, root, language)
    })
}

/// The one judged scan entry every verdict-bearing surface shares —
/// the CLI gate, the MCP tool and the GUI face alike (batch-7 slice
/// 8: the retired mirror-only analyze() made the pinned mirror the
/// SOLE authority on the auxiliary surfaces, guarded only when the
/// gate happened to run). Levels come from the core (scan/1); the
/// ADR-008 P3 drift ensure then proves the pinned mirror equal on
/// EVERY surface, or the run dies loudly — formula drift named,
/// never a silently forked verdict. The last two are the fail bit and
/// the conditions it is the disjunction of (6.4.0: `hard_line`, the
/// config fence `knobs_digest`, `degraded`).
type Judged = (
    Vec<FileMetrics>,
    Vec<report::Finding>,
    report::Summary,
    bool,
    Vec<String>,
);

/// A measured tree with the core's verdict on it, and the ONE road a
/// complexity value takes: the three numbers are derived from each
/// unit's structural events and the recursion increment is charged
/// here and nowhere else (plan v2.30 step 7b ③), so `rows` below
/// already carry the numbers the core judged with. `measure` alone
/// answers no complexity at all — its units read 0 there — which
/// is exactly what the structure family, the one reader that never
/// looks at complexity, should keep getting.
pub struct Settled {
    pub config: crate::config::Config,
    pub files: Vec<FileMetrics>,
    pub classes: classes::Classes,
    pub rows: Vec<report::Row>,
    pub grades: Vec<[u64; 3]>,
    pub row_classes: Vec<u64>,
    pub overrides: Vec<[u64; 4]>,
    pub levels: Vec<u8>,
    pub fail: bool,
    pub failed: Vec<String>,
}

pub fn settle(root: &Path, core: &str) -> Result<Settled> {
    let (config, mut files) = measure(root)?;
    let blocks = report::blocks_of(&files);
    // the call table (6.5.0): the arcs this side proved inside one
    // parse unit, on row indices — the core finds the cycles; the
    // events table (7.2.0): each unit's classified structure, keyed
    // by its cognitive row — the core folds the three numbers
    let calls = complexity::arcs(&files, &blocks);
    let events = complexity::events(&files, &blocks);
    let rows = report::rows_of(&files);
    let sent = wire_rows(&rows);
    let t = tables(root, &config, &files, &rows)?;
    let req = wire::ScanRequest {
        rows: &sent,
        grades: &t.grades,
        naming: &t.naming,
        row_classes: t.classes.declared().then_some(t.row_classes.as_slice()),
        overrides: &t.overrides,
        fence: t.fence,
        blocks: &blocks,
        calls: &calls,
        events: &events,
    };
    let j = wire::judge(core, &req)?;
    complexity::apply(&mut files, &blocks, &j.derived, &j.bumped)?;
    // rebuilt AFTER the derivation: these are the values that were
    // graded, so the report, the mirror and the core read one number
    let rows = report::rows_of(&files);
    Ok(Settled {
        config,
        files,
        classes: t.classes,
        rows,
        grades: t.grades,
        row_classes: t.row_classes,
        overrides: t.overrides,
        levels: j.levels,
        fail: j.fail,
        failed: j.failed,
    })
}

/// The rows as they cross — the facts roads: the fn-naming verdict
/// never does (2.30.0, ADR-008 slice 14) — every code-6 row carries 0
/// and its naming facts ride the aligned table, one row per function
/// in the files×functions order rows_of walks the code-6 rows — and
/// neither do the three complexity numbers (7.2.0): codes 3..5 carry
/// 0 and the core refuses a pre-judged value there by name.
fn wire_rows(rows: &[report::Row]) -> Vec<[u64; 2]> {
    rows.iter()
        .map(|r| {
            [
                r.code,
                if (3..=6).contains(&r.code) {
                    0
                } else {
                    r.value as u64
                },
            ]
        })
        .collect()
}

/// The tables a judged scan sends beside its rows, each built where
/// its facts are still known: the grade table ce.toml speaks, the
/// naming facts aligned to the code-6 rows (2.30.0), the rulepack
/// channel (3.2.0) — each row's class beside the per-class overrides,
/// neither riding on an unclassed repo — and the fence (6.4.0, O33)
/// the scan judges under: the config the committed baseline was
/// established with, or the drift by name (a `[thresholds]` edit used
/// to move the scan gate in silence).
struct Tables {
    classes: classes::Classes,
    grades: Vec<[u64; 3]>,
    naming: Vec<[i64; 5]>,
    row_classes: Vec<u64>,
    overrides: Vec<[u64; 4]>,
    fence: serde_json::Value,
}

fn tables(
    root: &Path,
    config: &crate::config::Config,
    files: &[FileMetrics],
    rows: &[report::Row],
) -> Result<Tables> {
    let classes = classes::Classes::compile(root, &config.rules).map_err(anyhow::Error::msg)?;
    Ok(Tables {
        grades: wire::grade_rows(&config.thresholds)?,
        naming: files
            .iter()
            .flat_map(|f| &f.functions)
            .map(|f| f.naming)
            .collect(),
        row_classes: rows.iter().map(|r| classes.class_of(&r.file)).collect(),
        overrides: wire::class_grade_rows(&config.rules, &config.thresholds),
        fence: crate::score::baseline::fence_status(root, config)?.wire(),
        classes,
    })
}

pub fn analyze_judged(root: &Path, core: &str) -> Result<Judged> {
    let s = settle(root, core)?;
    let findings = report::findings_from(
        &s.rows,
        &s.levels,
        &s.grades,
        (&s.row_classes, &s.overrides),
    );
    let mirror: Vec<report::Finding> = s
        .files
        .iter()
        .flat_map(|f| report::evaluate(f, &s.classes.thresholds_for(&s.config, &f.path)))
        .collect();
    anyhow::ensure!(
        findings == mirror,
        "core scan verdicts disagree with the pinned mirror — formula drift (Scan/Cost.hs vs report.rs)"
    );
    let summary = report::summarize(&s.files, &findings);
    Ok((s.files, findings, summary, s.fail, s.failed))
}

/// The scan report as its canonical JSON string (schema §7.1).
pub fn report_string(
    files: &[FileMetrics],
    findings: &[report::Finding],
    summary: report::Summary,
    failed: &[String],
) -> Result<String> {
    let rep = report::Report {
        schema: report::SCHEMA,
        files,
        findings,
        summary,
        failed,
    };
    Ok(serde_json::to_string_pretty(&rep)?)
}

fn measure_file(src: Vec<u8>, path: &Path, root: &Path, language: Lang) -> Result<FileMetrics> {
    let mut out = FileMetrics {
        path: walk::rel_str(root, path),
        lang: language.name(),
        total_lines: metrics::size::total_lines(&src),
        comment_lines: 0,
        functions: Vec::new(),
        calls: Vec::new(),
    };
    let Some(grammar) = language.grammar() else {
        return Ok(out); // Markdown: size-only per plan §6 M1
    };
    let sp = spec::spec(language);
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&grammar)
        .with_context(|| format!("grammar {}", language.name()))?;
    let tree = parser
        .parse(&src, None)
        .with_context(|| format!("parse {}", path.display()))?;
    out.comment_lines = metrics::size::comment_lines(tree.root_node(), sp);
    let units = functions::extract(tree.root_node(), &src, sp);
    out.calls = calls::edges(&units, &src, sp)
        .into_iter()
        .map(|(from, to)| (from as u32, to as u32))
        .collect();
    out.functions = measure_functions(units, &src, sp, language);
    Ok(out)
}

fn measure_functions(
    units: Vec<functions::FnUnit<'_>>,
    src: &[u8],
    sp: &spec::LangSpec,
    language: Lang,
) -> Vec<FnMetrics> {
    units
        .into_iter()
        .map(|unit| {
            let naming = metrics::naming::facts(language, sp.name_style, &unit.name);
            FnMetrics {
                name_ok: metrics::naming::conforms(naming),
                naming,
                events: metrics::events::emit(unit.node, src, sp),
                name: unit.name,
                start_line: unit.start_line,
                end_line: unit.end_line,
                lines: unit.end_line - unit.start_line + 1,
                params: unit.params,
                cyclomatic: 0,
                cognitive: 0,
                max_nesting: 0,
            }
        })
        .collect()
}
