//! Python rungs (design §4 row 2). R1 leading-dot relative imports —
//! n dots climb n−1 package levels, then the dotted remainder walks
//! down; R2 absolute dotted paths tried against every source root
//! {repo root, src/, pyproject-declared dirs} — two roots producing
//! DIFFERENT files is ambiguous_root, never a pick; R3 the
//! `a/b/__init__.py`-exists-but-`a/b/c.py`-does-not degradation: a
//! file-level edge to the longest prefix package's __init__.py, the
//! symbol deliberately left to the ledger (degrading beats
//! guessing); R4 stdlib names (machine-generated CPython 3.13 list)
//! or pyproject-declared dependencies ⇒ External. Within one root
//! the package/module order is normative (CPython's FileFinder
//! checks directories before same-named modules), so a double hit
//! inside a root is not ambiguity — only cross-root disagreement is.
//! `importlib(var)` / `__import__` dynamism never reaches this
//! ladder: the site detector only opens import statements, so the
//! design's R5 dynamic bucket stays structurally empty here.

use super::{Outcome, Reason, Scope};
use crate::graph::roots;
use std::collections::BTreeSet;

pub fn resolve(from: &str, spec: &str, scope: &Scope) -> Outcome {
    if spec.starts_with('.') {
        return relative(from, spec, scope.files);
    }
    let roots_list = source_roots(scope);
    if let Some(outcome) = absolute(spec, &roots_list, scope.files, module_at) {
        return outcome; // R2 (or its cross-root ambiguity)
    }
    if let Some(outcome) = absolute(spec, &roots_list, scope.files, init_prefix) {
        return outcome.with_rung(3); // R3 __init__ degradation
    }
    stdlib_or_deps(spec, scope)
}

/// R1: dots climb, remainder walks down, package order normative.
fn relative(from: &str, spec: &str, files: &BTreeSet<String>) -> Outcome {
    let dots = spec.chars().take_while(|c| *c == '.').count();
    let rest = &spec[dots..];
    let mut dir = from.rfind('/').map_or("", |i| &from[..i]).to_string();
    for _ in 1..dots {
        if dir.is_empty() {
            return Outcome::Unresolved(Reason::OutOfScope);
        }
        dir = dir
            .rfind('/')
            .map_or(String::new(), |i| dir[..i].to_string());
    }
    if let Some(path) = module_at(&dir, rest, files) {
        return Outcome::Resolved { path, rung: 1 };
    }
    match init_prefix(&dir, rest, files) {
        Some(path) => Outcome::Resolved { path, rung: 3 },
        None => Outcome::Unresolved(Reason::OutOfScope),
    }
}

/// R2/R3 shared frame: try one candidate rule against every source
/// root; distinct files from different roots ⇒ ambiguous_root.
fn absolute(
    spec: &str,
    roots_list: &[String],
    files: &BTreeSet<String>,
    rule: fn(&str, &str, &BTreeSet<String>) -> Option<String>,
) -> Option<Outcome> {
    let hits: BTreeSet<String> = roots_list
        .iter()
        .filter_map(|root| rule(root, spec, files))
        .collect();
    match hits.len() {
        0 => None,
        1 => Some(Outcome::Resolved {
            path: hits.into_iter().next().expect("len checked"),
            rung: 2,
        }),
        _ => Some(Outcome::Unresolved(Reason::AmbiguousRoot)),
    }
}

/// One dotted module under one root: package before module
/// (CPython finder order — normative, not ambiguous).
fn module_at(root: &str, dotted: &str, files: &BTreeSet<String>) -> Option<String> {
    let base = if dotted.is_empty() {
        root.to_string()
    } else {
        roots::join_rel(root, &dotted.replace('.', "/"))?
    };
    let package = if base.is_empty() {
        "__init__.py".to_string()
    } else {
        format!("{base}/__init__.py")
    };
    if files.contains(&package) {
        return Some(package);
    }
    let module = format!("{base}.py");
    files.contains(&module).then_some(module)
}

/// Longest strict prefix of the dotted path whose package
/// __init__.py is in scope (R3: file-level edge, symbol stays open).
fn init_prefix(root: &str, dotted: &str, files: &BTreeSet<String>) -> Option<String> {
    let segments: Vec<&str> = dotted.split('.').filter(|s| !s.is_empty()).collect();
    for take in (1..segments.len()).rev() {
        let prefix = segments[..take].join(".");
        if let Some(hit) = module_at(root, &prefix, files)
            && hit.ends_with("/__init__.py")
        {
            return Some(hit);
        }
    }
    None
}

/// R4: stdlib table or pyproject-declared dependency ⇒ External.
/// `__future__` is a real stdlib module (PEP 236; listed by
/// `sys.stdlib_module_names`) that the public-names table below
/// filters out with every other underscore name, so its site (spec.rs
/// `Literal`) is answered here by name.
fn stdlib_or_deps(spec: &str, scope: &Scope) -> Outcome {
    let top = spec.split('.').next().unwrap_or(spec);
    let declared = roots::pyproject(scope.root).is_some_and(|p| p.deps.iter().any(|d| d == top));
    if declared || top == "__future__" || STDLIB.split_whitespace().any(|m| m == top) {
        return Outcome::External { rung: 4 };
    }
    Outcome::Unresolved(Reason::OutOfScope)
}

/// Import roots: repo root and src/ always (design §4), plus the
/// pyproject-declared package dirs.
fn source_roots(scope: &Scope) -> Vec<String> {
    let mut out = vec![String::new(), "src".to_string()];
    if let Some(py) = roots::pyproject(scope.root) {
        out.extend(py.source_dirs);
    }
    out.dedup();
    out
}

/// CPython 3.13 public top-level stdlib modules — machine-generated
/// from `sys.stdlib_module_names` (2026-08-12), never hand-typed.
/// A missing name degrades to Unresolved (precision-safe), visible
/// in the ledger, and is repaid by regenerating this list (the
/// one-shot regen_tables drift check retired to git history).
const STDLIB: &str = "\
abc antigravity argparse array ast asyncio atexit base64 bdb binascii
bisect builtins bz2 cProfile calendar cmath cmd code codecs codeop collections
colorsys compileall concurrent configparser contextlib contextvars copy
copyreg csv ctypes curses dataclasses datetime dbm decimal difflib dis
doctest email encodings ensurepip enum errno faulthandler fcntl filecmp
fileinput fnmatch fractions ftplib functools gc genericpath getopt getpass
gettext glob graphlib grp gzip hashlib heapq hmac html http idlelib imaplib
importlib inspect io ipaddress itertools json keyword linecache locale
logging lzma mailbox marshal math mimetypes mmap modulefinder msvcrt multiprocessing
netrc nt ntpath nturl2path numbers opcode operator optparse os pathlib
pdb pickle pickletools pkgutil platform plistlib poplib posix posixpath
pprint profile pstats pty pwd py_compile pyclbr pydoc pydoc_data pyexpat
queue quopri random re readline reprlib resource rlcompleter runpy sched
secrets select selectors shelve shlex shutil signal site smtplib socket
socketserver sqlite3 sre_compile sre_constants sre_parse ssl stat statistics
string stringprep struct subprocess symtable sys sysconfig syslog tabnanny
tarfile tempfile termios textwrap this threading time timeit tkinter token
tokenize tomllib trace traceback tracemalloc tty turtle turtledemo types
typing unicodedata unittest urllib uuid venv warnings wave weakref webbrowser
winreg winsound wsgiref xml xmlrpc zipapp zipfile zipimport zlib zoneinfo";
