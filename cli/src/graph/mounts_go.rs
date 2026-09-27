//! The Go arm of the package-privacy bit — mounts.rs's child, split at
//! the 300 line when plan v2.30 step 5b taught the clause reader a
//! block comment sharing the clause's line: `package main` is never
//! importable, and an `internal/` directory is importable only from
//! its parent tree; `_test.go` is a test file and out of this fact
//! (its word carries TEST).

use std::path::Path;

pub(crate) fn go_private(root: &Path, path: &str) -> bool {
    if path.ends_with("_test.go") {
        return false;
    }
    let internal = path.split('/').rev().skip(1).any(|seg| seg == "internal");
    internal || go_package(root, path).as_deref() == Some("main")
}

/// The package clause: the first line OUTSIDE comments that opens with
/// `package `, its next word. Block comments carry state across lines
/// because both misreadings are wrong answers, not safe ones — a
/// gofmt-indented `package main` example inside a doc comment must not
/// win (bit 1 raises the row to code 1), and a comment naming another
/// package must not hide a real `package main`. A clause sharing its
/// line with a block comment — after one that closes, between two — is
/// read from the code the line keeps (plan v2.30 step 5b).
fn go_package(root: &Path, path: &str) -> Option<String> {
    let text = std::fs::read_to_string(root.join(path)).ok()?;
    let mut in_block = false;
    for raw in text.lines() {
        let Some(line) = outside_block(raw, &mut in_block) else {
            continue;
        };
        if let Some(rest) = line.strip_prefix("package ") {
            return Some(
                rest.split([' ', '\t', '/'])
                    .next()
                    .unwrap_or("")
                    .to_string(),
            );
        }
    }
    None
}

/// The code of one line under the block-comment state: the text
/// outside every `/* … */` region, spliced — a comment may open, close
/// or both on the line — with a `//` outside a block cutting the rest;
/// None when nothing but comment remains. Go comments do not nest, so
/// one flag is the whole state.
fn outside_block(line: &str, in_block: &mut bool) -> Option<String> {
    let (mut code, mut rest) = (String::new(), line);
    loop {
        if *in_block {
            let Some((_, after)) = rest.split_once("*/") else {
                break;
            };
            *in_block = false;
            rest = after;
        }
        let line_comment = rest.find("//");
        match rest.find("/*") {
            Some(open) if line_comment.is_none_or(|cut| open < cut) => {
                code.push_str(&rest[..open]);
                *in_block = true;
                rest = &rest[open + 2..];
            }
            _ => {
                code.push_str(&rest[..line_comment.unwrap_or(rest.len())]);
                break;
            }
        }
    }
    let code = code.trim();
    (!code.is_empty()).then(|| code.to_string())
}
