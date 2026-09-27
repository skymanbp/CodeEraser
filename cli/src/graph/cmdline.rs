//! Compiler command and response-file argv, read without consulting the host.
//! Clang's JSON reader chooses its syntax with the host's `_WIN32`; here the
//! command's own program token chooses it so one database reads alike everywhere.
//! https://github.com/llvm/llvm-project/blob/main/clang/lib/Tooling/JSONCompilationDatabase.cpp
//! `CommandLineArgumentParser`: only space separates; adjacent quoted and free
//! pieces concatenate, single quotes are literal, and a partial final word stays.
//! https://github.com/llvm/llvm-project/blob/main/llvm/lib/Support/CommandLine.cpp
//! `TokenizeGNUCommandLine`: "Consume runs of whitespace", "Backslash escapes
//! the next character", including inside either kind of quote. A final slash stays.
//! `TokenizeWindowsCommandLine`, `parseBackslash`: pairs of slashes before a quote
//! yield slashes; an odd remainder escapes the quote. "Consecutive double-quotes
//! inside a quoted string implies one double-quote". The leading program follows
//! CreateProcess: its backslashes are literal. These are lexical readers, not shells.

use super::compdb_flags::is_msvc;
use std::{iter::Peekable, str::Chars};

/// The unread characters shared by the three tokenizers.
type Input<'a> = Peekable<Chars<'a>>;

/// A Windows tool spelling: separator, drive, executable suffix, or MSVC basename.
pub fn windows_shaped(program: &str) -> bool {
    let drive = program
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphabetic)
        && program.as_bytes().get(1) == Some(&b':');
    program.contains('\\')
        || drive
        || [".exe", ".bat", ".cmd"]
            .iter()
            .any(|ext| program.to_ascii_lowercase().ends_with(ext))
        || is_msvc(program)
}

/// The command's first space-delimited word, or its initial double-quoted run.
fn program_of(command: &str) -> &str {
    let command = command.trim_start_matches(' ');
    match command.strip_prefix('"') {
        Some(rest) => rest.split('"').next().unwrap_or_default(),
        None => command.split(' ').next().unwrap_or_default(),
    }
}

/// A database command in the syntax its program spelling selects on every host.
pub fn split_command(command: &str) -> Vec<String> {
    if windows_shaped(program_of(command)) {
        split_windows(command, true)
    } else {
        split_gnu_json(command)
    }
}

/// Clang's JSON POSIX command reader: only space separates, single quotes are literal.
pub fn split_gnu_json(command: &str) -> Vec<String> {
    tokenize(
        command,
        |c| c == ' ',
        |input, out| unix_word(input, out, true),
    )
}

/// LLVM's GNU response reader: ASCII whitespace separates, both quotes allow escapes.
pub fn split_gnu(text: &str) -> Vec<String> {
    tokenize(text, whitespace, |input, out| unix_word(input, out, false))
}

/// LLVM's Windows response reader, optionally with CreateProcess program handling.
pub fn split_windows(text: &str, mut leading_program: bool) -> Vec<String> {
    tokenize(text, windows_space, |input, out| {
        if leading_program {
            program(input, out);
            leading_program = false;
        } else {
            windows_word(input, out);
        }
    })
}

/// The common word loop, preserving empty quoted words and partial final words.
fn tokenize(
    text: &str,
    separator: fn(char) -> bool,
    mut word: impl FnMut(&mut Input<'_>, &mut String),
) -> Vec<String> {
    let mut input = text.chars().peekable();
    let mut words = Vec::new();
    loop {
        while input.next_if(|c| separator(*c)).is_some() {}
        if input.peek().is_none() {
            return words;
        }
        let mut out = String::new();
        word(&mut input, &mut out);
        words.push(out);
    }
}

/// LLVM's four whitespace characters, deliberately excluding other Unicode spaces.
fn whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}

/// Windows also accepts NUL between arguments.
fn windows_space(c: char) -> bool {
    whitespace(c) || c == '\0'
}

/// Adjacent free and quoted pieces of either GNU reader.
fn unix_word(input: &mut Input<'_>, out: &mut String, json: bool) {
    let separator: fn(char) -> bool = if json { |c| c == ' ' } else { whitespace };
    while let Some(c) = input.next_if(|c| !separator(*c)) {
        match c {
            '"' | '\'' => quoted(input, out, c, json),
            '\\' => escaped(input, out, json),
            _ => out.push(c),
        }
    }
}

/// One GNU quoted piece, with JSON single quotes disabling backslash escapes.
fn quoted(input: &mut Input<'_>, out: &mut String, quote: char, json: bool) {
    while let Some(c) = input.next() {
        if c == quote {
            return;
        }
        if c == '\\' && (!json || quote == '"') {
            escaped(input, out, json);
        } else {
            out.push(c);
        }
    }
}

/// An escape's following character; only the response reader keeps a terminal slash.
fn escaped(input: &mut Input<'_>, out: &mut String, json: bool) {
    match input.next() {
        Some(c) => out.push(c),
        None if !json => out.push('\\'),
        None => {}
    }
}

/// A leading Windows program ends at whitespace or its opening quote's mate.
fn program(input: &mut Input<'_>, out: &mut String) {
    let quoted = input.next_if_eq(&'"').is_some();
    while let Some(c) = input.next_if(|c| quoted || !windows_space(*c)) {
        if quoted && c == '"' {
            return;
        }
        out.push(c);
    }
}

/// A Windows argument, whose quoted runs may contain doubled literal quotes.
fn windows_word(input: &mut Input<'_>, out: &mut String) {
    let mut quoted = false;
    while let Some(c) = input.next_if(|c| quoted || !windows_space(*c)) {
        match c {
            '\\' => backslashes(input, out),
            '"' if quoted && input.next_if_eq(&'"').is_some() => out.push('"'),
            '"' => quoted = !quoted,
            _ => out.push(c),
        }
    }
}

/// A slash run with its first slash consumed; only a following quote changes it.
fn backslashes(input: &mut Input<'_>, out: &mut String) {
    let mut count = 1;
    while input.next_if_eq(&'\\').is_some() {
        count += 1;
    }
    if input.peek() != Some(&'"') {
        out.extend(std::iter::repeat_n('\\', count));
        return;
    }
    out.extend(std::iter::repeat_n('\\', count / 2));
    if count % 2 == 1 {
        input.next();
        out.push('"');
    }
}

/// Mounted command-reader unit tests.
#[cfg(test)]
#[path = "../../tests/unit/graph/cmdline.rs"]
mod tests;
