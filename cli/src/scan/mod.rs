//! `ce scan` orchestration: walk → parse → measure → judge → emit.
//! Since ADR-008 P3 the LEVEL judgment is the core's (scan/1, the
//! graded verdict table); measurement and report rendering stay
//! here, and the local evaluate() binding survives as the pinned
//! mirror the whole-report ensure proves equal on every judged run
//! — CLI gate, MCP tool and GUI face alike since batch-7 slice 8.

pub mod ast;
pub mod classes;
pub mod functions;
pub mod globs;
pub mod lang;
pub mod metrics;
pub mod report;
pub mod spec;
pub mod spec_hs;
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

pub fn analyze_judged(root: &Path, core: &str) -> Result<Judged> {
    let (config, files) = measure(root)?;
    let rows = report::rows_of(&files);
    let grades = wire::grade_rows(&config.thresholds)?;
    // The facts road (2.30.0, ADR-008 slice 14): the fn-naming
    // verdict never crosses — every code-6 row carries 0, and its
    // naming facts ride the aligned table (one row per function, in
    // the same files×functions order rows_of walks the code-6 rows).
    let wire_rows: Vec<[u64; 2]> = rows
        .iter()
        .map(|r| [r.code, if r.code == 6 { 0 } else { r.value as u64 }])
        .collect();
    let naming: Vec<[i64; 5]> = files
        .iter()
        .flat_map(|f| &f.functions)
        .map(|f| f.naming)
        .collect();
    // The rulepack channel (3.2.0): each row's class, assigned here
    // where its path is still known, beside the per-class overrides;
    // an unclassed repo sends neither and its bytes never move.
    let classes = classes::Classes::compile(root, &config.rules).map_err(anyhow::Error::msg)?;
    let row_classes: Vec<u64> = rows.iter().map(|r| classes.class_of(&r.file)).collect();
    let overrides = wire::class_grade_rows(&config.rules, &config.thresholds);
    // the fence (6.4.0, O33): the scan judges under the config the
    // committed baseline was established with, or names the drift —
    // a `[thresholds]` edit used to move the scan gate in silence
    let fence = crate::score::baseline::fence_status(root, &config)?;
    let req = wire::ScanRequest {
        rows: &wire_rows,
        grades: &grades,
        naming: &naming,
        row_classes: classes.declared().then_some(row_classes.as_slice()),
        overrides: &overrides,
        fence: fence.wire(),
    };
    let (levels, fail, failed) = wire::judge(core, &req)?;
    let findings = report::findings_from(&rows, &levels, &grades, (&row_classes, &overrides));
    let mirror: Vec<report::Finding> = files
        .iter()
        .flat_map(|f| report::evaluate(f, &classes.thresholds_for(&config, &f.path)))
        .collect();
    anyhow::ensure!(
        findings == mirror,
        "core scan verdicts disagree with the pinned mirror — formula drift (Scan/Cost.hs vs report.rs)"
    );
    let summary = report::summarize(&files, &findings);
    Ok((files, findings, summary, fail, failed))
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
    out.functions = measure_functions(tree.root_node(), &src, sp, language);
    Ok(out)
}

fn measure_functions(
    root: tree_sitter::Node<'_>,
    src: &[u8],
    sp: &spec::LangSpec,
    language: Lang,
) -> Vec<FnMetrics> {
    functions::extract(root, src, sp)
        .into_iter()
        .map(|unit| {
            let cog = metrics::cognitive::measure(unit.node, src, sp);
            let naming = metrics::naming::facts(language, sp.name_style, &unit.name);
            FnMetrics {
                name_ok: metrics::naming::conforms(naming),
                naming,
                name: unit.name,
                start_line: unit.start_line,
                end_line: unit.end_line,
                lines: unit.end_line - unit.start_line + 1,
                params: unit.params,
                cyclomatic: metrics::cyclo::measure(unit.node, src, sp),
                cognitive: cog.score,
                max_nesting: cog.max_nesting,
            }
        })
        .collect()
}
