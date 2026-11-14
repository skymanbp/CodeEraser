//! Size metrics: total lines and comment lines (whole file).
//! Function length / params live on FnUnit (functions.rs).

use crate::scan::spec::LangSpec;
use tree_sitter::Node;

/// A text's line count — the ONE rule the scan's hard line, the
/// guard's hard budget and every "how many lines has this text" reader
/// count by: the number of `\n`, plus one when the input is not empty
/// and does not end in `\n`; the value `str::lines().count()` gives on
/// the same bytes, LF or CRLF. No UTF-8 requirement: a line count is
/// not a text. Two entries: a slice, and a stream.
pub fn total_lines(src: &[u8]) -> usize {
    folded(newlines(src), src.last().copied())
}

/// The same count off a reader, streamed in fixed chunks (no size cap):
/// only a real I/O failure leaves it unanswered (REVIEW-1 Q10). The
/// guard reads a file's size before a write through it, and the replay
/// instruments count blobs through it.
pub fn count_lines_in(mut r: impl std::io::Read) -> std::io::Result<usize> {
    let mut buf = [0u8; CHUNK];
    let (mut n_lines, mut last) = (0usize, None);
    loop {
        let n = match r.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        n_lines += newlines(&buf[..n]);
        last = Some(buf[n - 1]);
    }
    Ok(folded(n_lines, last))
}

/// The stream's read size.
pub const CHUNK: usize = 64 * 1024;

fn newlines(bytes: &[u8]) -> usize {
    bytes.iter().filter(|b| **b == b'\n').count()
}

/// The fold both entries end in: the newline count and the last byte.
fn folded(newlines: usize, last: Option<u8>) -> usize {
    newlines + usize::from(last.is_some_and(|b| b != b'\n'))
}

/// Lines covered by comment nodes (a line with any comment counts once).
pub fn comment_lines(root: Node<'_>, spec: &LangSpec) -> usize {
    let mut lines = std::collections::HashSet::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if spec.comment_kinds.contains(&node.kind()) {
            for row in node.start_position().row..=node.end_position().row {
                lines.insert(row);
            }
        }
        stack.extend(crate::scan::ast::children(node).into_iter().rev());
    }
    lines.len()
}

#[cfg(test)]
#[path = "../../../tests/unit/scan/metrics/size.rs"]
mod tests;
