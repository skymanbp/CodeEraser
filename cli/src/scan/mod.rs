//! `ce scan` orchestration: walk → parse → measure → judge → emit.
//! Since ADR-008 P3 the LEVEL judgment is the core's (scan/1, the
//! graded verdict table); measurement and report rendering stay
//! here, and the local evaluate() binding survives as the pinned
//! mirror the whole-report ensure proves equal on every judged run
//! — CLI gate, MCP tool and GUI face alike since batch-7 slice 8.

pub mod ast;
pub mod functions;
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
}

pub fn run(root: &Path, format: Format, core: &str) -> Result<ExitCode> {
    let (files, findings, summary, fail) = analyze_judged(root, core)?;
    match format {
        Format::Console => report::print_console(&findings, &summary),
        Format::Json => println!("{}", report_string(&files, &findings, summary)?),
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
/// never a silently forked verdict.
type Judged = (
    Vec<FileMetrics>,
    Vec<report::Finding>,
    report::Summary,
    bool,
);

pub fn analyze_judged(root: &Path, core: &str) -> Result<Judged> {
    let (config, files) = measure(root)?;
    let rows = report::rows_of(&files);
    let grades = wire::grade_rows(&config.thresholds)?;
    let wire_rows: Vec<[u64; 2]> = rows.iter().map(|r| [r.code, r.value as u64]).collect();
    let (levels, fail) = wire::judge(core, &wire_rows, &grades)?;
    let findings = report::findings_from(&rows, &levels, &grades);
    let mirror: Vec<report::Finding> = files
        .iter()
        .flat_map(|f| report::evaluate(f, &config.thresholds))
        .collect();
    anyhow::ensure!(
        findings == mirror,
        "core scan verdicts disagree with the pinned mirror — formula drift (Scan/Cost.hs vs report.rs)"
    );
    let summary = report::summarize(&files, &findings);
    Ok((files, findings, summary, fail))
}

/// The scan report as its canonical JSON string (schema §7.1).
pub fn report_string(
    files: &[FileMetrics],
    findings: &[report::Finding],
    summary: report::Summary,
) -> Result<String> {
    let rep = report::Report {
        schema: report::SCHEMA,
        files,
        findings,
        summary,
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
    out.functions = measure_functions(tree.root_node(), &src, sp);
    Ok(out)
}

fn measure_functions(
    root: tree_sitter::Node<'_>,
    src: &[u8],
    sp: &spec::LangSpec,
) -> Vec<FnMetrics> {
    functions::extract(root, src, sp)
        .into_iter()
        .map(|unit| {
            let cog = metrics::cognitive::measure(unit.node, src, sp);
            FnMetrics {
                name_ok: metrics::naming::conforms(sp.name_style, &unit.name),
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
