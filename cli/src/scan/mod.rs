//! `ce scan` orchestration: walk → parse → measure → judge → emit.
//! Since ADR-008 P3 the LEVEL judgment is the core's (scan/1, the
//! graded verdict table); measurement and report rendering stay
//! here, and the local evaluate() binding survives as the pinned
//! mirror the whole-report ensure proves equal on every gate run.

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
    let (config, files) = measured(root)?;
    let rows = report::rows_of(&files);
    let grades = wire::grade_rows(&config.thresholds)?;
    let wire_rows: Vec<[u64; 2]> = rows.iter().map(|r| [r.code, r.value as u64]).collect();
    let (levels, fail) = wire::judge(core, &wire_rows, &grades)?;
    let findings = report::findings_from(&rows, &levels, &grades);
    // ADR-008 P3 drift ensure: the report is built from the CORE's
    // levels; the pinned mirror must reproduce it whole or the run
    // dies loudly — formula drift named, never a silently forked
    // verdict (the mcp/score surfaces read the mirror, so this is
    // what keeps them equal to the gate by proof)
    let mirror: Vec<report::Finding> = files
        .iter()
        .flat_map(|f| report::evaluate(f, &config.thresholds))
        .collect();
    anyhow::ensure!(
        findings == mirror,
        "core scan verdicts disagree with the pinned mirror — formula drift (Scan/Cost.hs vs report.rs)"
    );
    let summary = report::summarize(&files, &findings);
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

/// Measurement alone — the walk every scan surface shares.
fn measured(root: &Path) -> Result<(crate::config::Config, Vec<FileMetrics>)> {
    walk::each_surviving(root, |path, language, src| {
        measure_file(src, path, root, language)
    })
}

/// Library entry shared by the MCP server and the score family's
/// measurement reuse: measure and evaluate through the MIRROR
/// binding without printing anything (no core link on these
/// auxiliary surfaces; the gate's ensure proves the mirror equal).
pub fn analyze(root: &Path) -> Result<(Vec<FileMetrics>, Vec<report::Finding>, report::Summary)> {
    let (config, files) = measured(root)?;
    let findings: Vec<_> = files
        .iter()
        .flat_map(|f| report::evaluate(f, &config.thresholds))
        .collect();
    let summary = report::summarize(&files, &findings);
    Ok((files, findings, summary))
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
