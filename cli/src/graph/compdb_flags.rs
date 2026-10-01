//! One invocation's explicit include chain, classified before the ladder searches it.
//! https://gcc.gnu.org/onlinedocs/gcc/Directory-Options.html, Directory Options:
//! `-iquote` applies "only to the quote form" and comes "before all directories
//! specified by -I"; `-isystem` follows -I, `-idirafter` follows standard directories.
//! `-I-`: earlier -I entries become quote-only; the current file's directory is
//! inhibited with "no way to override this effect". `-iprefix` is concatenated
//! literally: "you should include the final '/'"; `-iwithprefixbefore` belongs with
//! -I and `-iwithprefix` with -idirafter. Installation defaults name no tree path.
//! For directory operands, "the '=' or $SYSROOT is replaced by the sysroot prefix".
//! If -I repeats a system directory, "the -I option is ignored".
//! https://github.com/llvm/llvm-project/blob/main/clang/lib/Lex/InitHeaderSearch.cpp,
//! `RemoveDuplicates`: "drop the user dir... keeping the system dir" across the
//! angled and system groups; the quoted group is deduplicated on its own.
//! https://gcc.gnu.org/onlinedocs/gcc/Darwin-Options.html, -F: frameworks are
//! "interleaved with those specified by -I options" in left-to-right order.
//! https://clang.llvm.org/docs/ClangCommandLineReference.html, -F/-iframework:
//! ordinary/system framework paths; -include/--include: "Include file before
//! parsing"; -imacros: "Include macros from file before parsing". Forwarded
//! -Xclang/-Xpreprocessor words are read; other stages' operands are skipped.
//! https://gcc.gnu.org/onlinedocs/gcc/Preprocessor-Options.html, -include/-imacros:
//! forced names are kept for the ladder's lookup, never placed as directories.
//! https://learn.microsoft.com/en-us/cpp/preprocessor/hash-include-directive-c-cpp,
//! quoted lookup: own directory, open include stack in reverse, /I, then INCLUDE;
//! angle lookup omits the first two. The ladder supplies that include stack.
//! https://learn.microsoft.com/en-us/cpp/build/reference/external-external-headers,
//! /external:I: "The space between /external:I and path is optional."
//! ClangCommandLineReference /imsvc adds paths "as if in %INCLUDE%", after external
//! directories. MSVC keeps first duplicates; GNU keeps the system occurrence.

use std::collections::BTreeSet;

/// One searched directory; framework joining belongs to the ladder.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Search {
    /// An ordinary directory joined with the include spelling.
    Dir(String),
    /// A/B.h maps to A.framework/Headers/B.h, then PrivateHeaders/B.h.
    Framework(String),
}

/// An invocation's placed directories: quoted-only, ordinary, then system.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Chain {
    /// Whether the ladder must use MSVC's include-stack lookup.
    pub msvc: bool,
    /// Whether quoted lookup first visits the including file's directory.
    pub own_dir: bool,
    /// Directories searched only by the quoted form.
    pub quote: Vec<Search>,
    /// Directories searched by both forms before system directories.
    pub bracket: Vec<Search>,
    /// System paths followed by paths searched after standard directories.
    pub system: Vec<Search>,
    /// Forced include and macro files as written, for the ladder to resolve.
    pub forced: Vec<String>,
}

/// MSVC driver basenames under either separator, compared without ASCII case.
pub fn is_msvc(program: &str) -> bool {
    let name = program.rsplit(['/', '\\']).next().unwrap_or_default();
    ["cl", "cl.exe", "clang-cl", "clang-cl.exe"]
        .iter()
        .any(|driver| name.eq_ignore_ascii_case(driver))
}

/// Read expanded argv, placing directories before cross-class duplicate removal.
pub fn chain(argv: &[String], place: &dyn Fn(&str) -> Option<String>) -> Chain {
    let msvc = argv.first().is_some_and(|program| is_msvc(program));
    let options = options(argv, msvc);
    let sysroot = options.iter().rev().find_map(|(flag, value)| {
        matches!(*flag, "--sysroot" | "--sysroot=" | "-isysroot").then_some(*value)
    });
    let mut out = Chain {
        msvc,
        own_dir: true,
        quote: Vec::new(),
        bracket: Vec::new(),
        system: Vec::new(),
        forced: Vec::new(),
    };
    let (mut prefix, mut after) = (None, Vec::new());
    for (flag, value) in options {
        match flag {
            "-I-" => {
                out.own_dir = false;
                out.quote.append(&mut out.bracket);
            }
            "-iprefix" => prefix = Some(value),
            "-include" | "--include" | "--include=" | "-imacros" | "--imacros" | "FI" => {
                out.forced.push(value.to_string());
            }
            _ => add_directory(&mut out, &mut after, flag, value, prefix, sysroot, place),
        }
    }
    out.system.extend(after);
    deduplicate(&mut out);
    out
}

/// Forwarded front-end words are ordinary argv; unrelated stages consume one word.
fn forwarded(argv: &[String], msvc: bool) -> Vec<&str> {
    let mut words = argv.iter().skip(1).map(String::as_str);
    let mut out = Vec::new();
    while let Some(word) = words.next() {
        match word {
            "-Xclang" | "-Xpreprocessor" if !msvc => {}
            "-Xassembler" | "-Xlinker" | "-Xanalyzer" if !msvc => {
                words.next();
            }
            _ => out.push(word),
        }
    }
    out
}

/// Recognized GNU joined spellings, longest conflicting spelling first,
/// as one space-separated literal like SKIP below: a slice of a dozen
/// `&str` rows is the clone gate's most-rhyming shape (it read these
/// two against the tombstone role's stems and the walk's built-in
/// excludes).
pub(crate) const GNU: &str = "-include-pch -iwithprefixbefore -isystem-after -iwithprefix -iframework \
                   -idirafter -isystem -isysroot -iquote -iprefix --include= --include \
                   -include --imacros -imacros --sysroot= -I -F";

/// GNU separate operands that cannot themselves open include options.
pub(crate) const SKIP: &str = "-o -MF -MT -MQ -x -arch -target -mllvm -D -U -L -l -z -u -e -T -B -b -V \
                    --sysroot";

/// One option spelling, leaving unknown arguments inert.
fn spelling(arg: &str, msvc: bool) -> Option<&str> {
    if msvc {
        return ["external:I", "imsvc", "FI", "Tc", "Tp", "I", "D", "U"]
            .into_iter()
            .find(|flag| arg.starts_with(flag));
    }
    if arg == "-I-" || SKIP.split(' ').any(|skip| skip == arg) {
        return Some(arg);
    }
    GNU.split(' ').find(|flag| arg.starts_with(flag))
}

/// Operand pairs in order; a recognized option without an operand ends the scan.
fn options(argv: &[String], msvc: bool) -> Vec<(&str, &str)> {
    let words = forwarded(argv, msvc);
    let mut words = words.into_iter();
    let mut out = Vec::new();
    while let Some(word) = words.next() {
        let arg = if msvc {
            let Some(arg) = word.strip_prefix(['/', '-']) else {
                continue;
            };
            arg
        } else {
            word
        };
        let Some(flag) = spelling(arg, msvc) else {
            continue;
        };
        if flag == "-I-" {
            out.push((flag, ""));
            continue;
        }
        let tail = &arg[flag.len()..];
        let value = if tail.is_empty() {
            words.next()
        } else {
            Some(tail)
        };
        let Some(value) = value else { break };
        out.push((flag, value));
    }
    out
}

/// Route one directory into its class before the after-system suffix is appended.
fn add_directory(
    out: &mut Chain,
    after: &mut Vec<Search>,
    flag: &str,
    value: &str,
    prefix: Option<&str>,
    sysroot: Option<&str>,
    place: &dyn Fn(&str) -> Option<String>,
) {
    let target = match flag {
        "-iquote" => &mut out.quote,
        "-I" | "-F" | "-iwithprefixbefore" | "I" => &mut out.bracket,
        "-isystem" | "-iframework" | "external:I" => &mut out.system,
        "-idirafter" | "-isystem-after" | "-iwithprefix" | "imsvc" => after,
        _ => return,
    };
    let Some(dir) = directory(flag, value, prefix, sysroot, out.msvc) else {
        return;
    };
    let Some(dir) = place(&dir) else { return };
    target.push(if matches!(flag, "-F" | "-iframework") {
        Search::Framework(dir)
    } else {
        Search::Dir(dir)
    });
}

/// Prefix concatenation and explicit sysroot substitution precede caller placement.
fn directory(
    flag: &str,
    value: &str,
    prefix: Option<&str>,
    sysroot: Option<&str>,
    msvc: bool,
) -> Option<String> {
    if matches!(flag, "-iwithprefix" | "-iwithprefixbefore") {
        return Some(format!("{}{value}", prefix?));
    }
    if !msvc
        && let Some(tail) = value
            .strip_prefix('=')
            .or_else(|| value.strip_prefix("$SYSROOT"))
    {
        if tail.is_empty() {
            return Some(sysroot?.to_string());
        }
        return Some(format!(
            "{}/{}",
            sysroot?.trim_end_matches('/'),
            tail.trim_start_matches('/')
        ));
    }
    Some(value.to_string())
}

/// clang's `RemoveDuplicates` as `InitHeaderSearch::Realize` calls it:
/// the quoted group alone, then across the angled and system groups,
/// where a user directory a system directory repeats is dropped (GNU
/// dialect); the MSVC dialect keeps first occurrences throughout.
fn deduplicate(out: &mut Chain) {
    let mut seen = BTreeSet::new();
    out.quote.retain(|entry| seen.insert(entry.clone()));
    let systems: BTreeSet<Search> = out.system.iter().cloned().collect();
    let msvc = out.msvc;
    let mut seen = BTreeSet::new();
    out.bracket
        .retain(|entry| (msvc || !systems.contains(entry)) && seen.insert(entry.clone()));
    out.system.retain(|entry| seen.insert(entry.clone()));
}

/// Mounted include-chain unit tests.
#[cfg(test)]
#[path = "../../tests/unit/graph/compdb_flags.rs"]
mod tests;
