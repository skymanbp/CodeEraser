//! Transcript scanning for the M4 pre-registered evaluation set:
//! stream Claude Code session .jsonl files, pair every Edit/Write
//! tool_use with its toolUseResult, and reconstruct the full
//! before/after file states. Nothing is dropped silently — every
//! rejection lands in a named counter that the manifest reports.

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// One reconstructible edit event. `before`/`after` are the complete
/// file contents around the edit — everything L0 (git diff) through
/// L2 (AST cost model) needs, with no repo replay.
pub struct Candidate {
    pub session_id: String,
    pub project_slug: String,
    pub ts: String,
    pub tool: String,
    pub file_path: String,
    pub lang: &'static str,
    pub before: String,
    pub after: String,
}

#[derive(Default)]
pub struct DropCounts {
    pub error_result: usize,      // tool errored / was denied — no result payload
    pub unsupported_tool: usize,  // MultiEdit etc. (zero in the surveyed corpus)
    pub unsupported_lang: usize,  // outside the five M1 languages
    pub unreconstructible: usize, // before-state absent / patch mismatch
    pub oversize: usize,          // before+after > 1 MiB (documented cap)
    /// compact/resume rewrites transcript history, so the same
    /// tool_use id recurs (measured: 1242 of 3282 ids in one session,
    /// up to 6×) — each event is consumed exactly once
    pub replayed_history: usize,
}

const MAX_SAMPLE_BYTES: usize = 1_048_576;

/// The five M1 first-wave languages (plan §6 M1); tsx rides the TS grammar.
pub fn lang_of(path: &str) -> Option<&'static str> {
    let ext = Path::new(path).extension()?.to_str()?;
    match ext {
        "ts" | "tsx" => Some("ts"),
        "py" => Some("py"),
        "rs" => Some("rs"),
        "go" => Some("go"),
        "md" => Some("md"),
        _ => None,
    }
}

/// Scan every session file under one project slug directory. The
/// horizon lives HERE, not downstream: transcripts keep growing while
/// sessions run (this tool's own session included), so every counter
/// must be scoped to events before `frozen_at` or the manifest's
/// funnel numbers drift between reruns (the double-run determinism
/// check caught exactly that: +2 unreconstructible).
pub fn scan_project(dir: &Path, frozen_at: &str, out: &mut Vec<Candidate>, drops: &mut DropCounts) {
    let slug = dir.file_name().map(|s| s.to_string_lossy().into_owned());
    let Some(slug) = slug else { return };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "jsonl") {
            let session = path.file_stem().map(|s| s.to_string_lossy().into_owned());
            if let (Some(session), Ok(text)) = (session, std::fs::read_to_string(&path)) {
                let mut scanner = SessionScanner {
                    slug: &slug,
                    session: &session,
                    frozen_at,
                    pending: HashMap::new(),
                    consumed: HashSet::new(),
                };
                scanner.run(&text, out, drops);
            }
        }
    }
}

struct PendingUse {
    tool: String,
    input: Value,
    ts: String,
}

/// Per-session pairing state; methods keep every signature within the
/// dogfood param budget instead of threading six loose arguments.
struct SessionScanner<'a> {
    slug: &'a str,
    session: &'a str,
    frozen_at: &'a str,
    pending: HashMap<String, PendingUse>,
    consumed: HashSet<String>,
}

impl SessionScanner<'_> {
    fn run(&mut self, text: &str, out: &mut Vec<Candidate>, drops: &mut DropCounts) {
        for line in text.lines() {
            // cheap pre-filter: full parse only for lines that can matter
            if line.contains("\"tool_use\"") {
                self.collect_uses(line, drops);
            }
            if line.contains("\"tool_result\"") {
                self.resolve_results(line, out, drops);
            }
        }
    }

    fn collect_uses(&mut self, line: &str, drops: &mut DropCounts) {
        let Ok(obj) = serde_json::from_str::<Value>(line) else {
            return;
        };
        let ts = obj["timestamp"].as_str().unwrap_or_default().to_string();
        // out of horizon = out of the frozen universe entirely — not a
        // drop, and never a counter (counters must reproduce on re-runs)
        if ts.is_empty() || ts.as_str() >= self.frozen_at {
            return;
        }
        let Some(blocks) = obj["message"]["content"].as_array() else {
            return;
        };
        for blk in blocks {
            if blk["type"].as_str() != Some("tool_use") {
                continue;
            }
            match blk["name"].as_str() {
                Some(tool @ ("Edit" | "Write")) => {
                    let Some(id) = blk["id"].as_str() else {
                        continue;
                    };
                    if self.consumed.contains(id) {
                        drops.replayed_history += 1;
                        continue;
                    }
                    self.pending.insert(
                        id.to_string(),
                        PendingUse {
                            tool: tool.to_string(),
                            input: blk["input"].clone(),
                            ts: ts.clone(),
                        },
                    );
                }
                Some("MultiEdit") => drops.unsupported_tool += 1,
                _ => {}
            }
        }
    }

    fn resolve_results(&mut self, line: &str, out: &mut Vec<Candidate>, drops: &mut DropCounts) {
        let Ok(obj) = serde_json::from_str::<Value>(line) else {
            return;
        };
        let Some(blocks) = obj["message"]["content"].as_array() else {
            return;
        };
        for blk in blocks {
            if blk["type"].as_str() != Some("tool_result") {
                continue;
            }
            let Some(id) = blk["tool_use_id"].as_str() else {
                continue;
            };
            let Some(usage) = self.pending.remove(id) else {
                continue;
            };
            self.consumed.insert(id.to_string());
            if blk["is_error"].as_bool() == Some(true) {
                drops.error_result += 1;
                continue;
            }
            self.build_candidate(&usage, &obj["toolUseResult"], out, drops);
        }
    }

    /// Reconstruct the before-state honestly or not at all. Transcripts
    /// carry a full `originalFile` only sporadically (measured: 551 of
    /// 4678 Edit results in one session), and a Write result's empty
    /// original means "new file" ONLY when its type says `create` — an
    /// `update` with a missing original is unknowable, and treating it
    /// as a create would freeze a fabricated sample.
    fn build_candidate(
        &self,
        usage: &PendingUse,
        result: &Value,
        out: &mut Vec<Candidate>,
        drops: &mut DropCounts,
    ) {
        let file_path = usage.input["file_path"].as_str().unwrap_or_default();
        let Some(lang) = lang_of(file_path) else {
            drops.unsupported_lang += 1;
            return;
        };
        let original = result["originalFile"].as_str().unwrap_or_default();
        let reconstructed = match usage.tool.as_str() {
            "Write" if !original.is_empty() || result["type"].as_str() == Some("create") => usage
                .input["content"]
                .as_str()
                .map(|after| (original.to_string(), after.to_string())),
            "Edit" if !original.is_empty() => apply_patch(original, &result["structuredPatch"])
                .map(|after| (original.to_string(), after)),
            _ => None, // before-state unknowable
        };
        let Some((before, after)) = reconstructed else {
            drops.unreconstructible += 1;
            return;
        };
        if before.len() + after.len() > MAX_SAMPLE_BYTES {
            drops.oversize += 1;
            return;
        }
        out.push(Candidate {
            session_id: self.session.to_string(),
            project_slug: self.slug.to_string(),
            ts: usage.ts.clone(),
            tool: usage.tool.clone(),
            file_path: file_path.to_string(),
            lang,
            before,
            after,
        });
    }
}

/// Apply a structuredPatch (jsdiff hunk shape) to the original text.
/// Context and deletion lines are verified against the original (CR
/// stripped for the comparison only); any mismatch rejects the event
/// rather than guessing — replicating Edit's own matching semantics
/// is exactly the hidden subsystem ADR-004 refused to build.
fn apply_patch(original: &str, patch: &Value) -> Option<String> {
    let hunks = patch.as_array()?;
    let old_lines: Vec<&str> = original.split('\n').collect();
    let mut new_lines: Vec<String> = Vec::with_capacity(old_lines.len());
    let mut cursor = 0usize; // index into old_lines
    for hunk in hunks {
        let start = hunk["oldStart"].as_u64()? as usize;
        let body = hunk["lines"].as_array()?;
        let hunk_at = start.saturating_sub(1);
        if hunk_at < cursor || hunk_at > old_lines.len() {
            return None;
        }
        new_lines.extend(old_lines[cursor..hunk_at].iter().map(|s| s.to_string()));
        cursor = apply_hunk(&old_lines, hunk_at, body, &mut new_lines)?;
    }
    new_lines.extend(old_lines[cursor..].iter().map(|s| s.to_string()));
    Some(new_lines.join("\n"))
}

/// One hunk body over the original lines; returns the advanced cursor.
fn apply_hunk(
    old_lines: &[&str],
    mut cursor: usize,
    body: &[Value],
    new_lines: &mut Vec<String>,
) -> Option<usize> {
    for entry in body {
        let entry = entry.as_str()?;
        let (op, text) = entry.split_at(1);
        match op {
            " " | "-" => {
                let have = old_lines.get(cursor)?.trim_end_matches('\r');
                if have != text.trim_end_matches('\r') {
                    return None;
                }
                if op == " " {
                    new_lines.push(old_lines[cursor].to_string());
                }
                cursor += 1;
            }
            "+" => new_lines.push(text.to_string()),
            _ => return None,
        }
    }
    Some(cursor)
}
