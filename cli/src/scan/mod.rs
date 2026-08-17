//! `ce scan` orchestration: walk → parse → measure → evaluate → emit.

pub mod ast;
pub mod functions;
pub mod lang;
pub mod metrics;
pub mod report;
pub mod spec;
pub mod spec_hs;
pub mod walk;

use anyhow::{Context, Result};
use lang::Lang;
use metrics::{FileMetrics, FnMetrics};
use std::path::Path;
use std::process::ExitCode;

pub enum Format {
    Console,
    Json,
}

pub fn run(root: &Path, format: Format) -> Result<ExitCode> {
    let (files, findings, summary) = analyze(root)?;
    let failed = summary.fails > 0;
    match format {
        Format::Console => report::print_console(&findings, &summary),
        Format::Json => println!("{}", report_string(&files, &findings, summary)?),
    }
    Ok(if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

/// Library entry shared by the CLI and the MCP server: measure and
/// evaluate without printing anything.
pub fn analyze(root: &Path) -> Result<(Vec<FileMetrics>, Vec<report::Finding>, report::Summary)> {
    let (config, files) = walk::each_surviving(root, |path, language, src| {
        measure_file(src, path, root, language)
    })?;
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
