//! Bound-name extraction per language, measured on the site nodes the
//! detector actually emits at (not on hand-picked subtrees) — the
//! shapes come from the four adversarially-reviewed specs, including
//! every construct a reviewer used to refute a first draft.
//!
//! Two tables rather than a test per language: a per-language
//! assertion body is a T2 clone chain by this repo's own gate, which
//! said so on the first draft (the EraseProps precedent).

use super::*;
use crate::graph::spec::{Specifier, sites as site_table};
use crate::scan::ast;
use crate::scan::ast::children;

/// A source sample and the (local, target) pairs its imports must
/// offer, under one language.
type Case<'a> = (Lang, &'a str, &'a [(&'a str, &'a str)]);

/// Bindings of every site in `src`, sorted — walks with the same
/// table and the same emit-at rule sites.rs uses, so the node handed
/// to `of_site` is the node production hands it.
fn bindings(src: &str, lang: Lang) -> Vec<(String, String)> {
    let tree = ast::parse_lang(src, lang).expect("parses");
    let bytes = src.as_bytes();
    let table = site_table(lang);
    let mut out = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        if let Some(kind) = table.iter().find(|k| k.node == node.kind()) {
            // EachImportTarget emits at the CHILD; every other
            // specifier emits at the matched node itself
            let emitted: Vec<Node<'_>> = match kind.via {
                Specifier::EachImportTarget => children(node),
                _ => vec![node],
            };
            for site in emitted {
                out.extend(
                    of_site(site, bytes, lang)
                        .into_iter()
                        .map(|b| (b.local, b.target)),
                );
            }
        }
        for child in children(node).into_iter().rev() {
            stack.push(child);
        }
    }
    out.sort_unstable();
    out
}

/// What each language's import syntax actually names. Rust's row is
/// the load-bearing one: modules and items look ALIKE here on
/// purpose — separating them is the symbols lookup's job, not this
/// module's, and an extractor that tried would be minting R6's edges.
#[test]
fn each_language_offers_the_names_its_imports_bind() {
    let cases: &[Case] = &[
        (
            Lang::Python,
            "from .structures import CaseInsensitiveDict\n\
             from .exceptions import JSONDecodeError as RequestsJSONDecodeError\n\
             from typing import (\n    Any,\n    cast,\n)\n",
            &[
                ("Any", "Any"),
                ("CaseInsensitiveDict", "CaseInsensitiveDict"),
                ("RequestsJSONDecodeError", "JSONDecodeError"),
                ("cast", "cast"),
            ],
        ),
        (
            Lang::TypeScript,
            "import { z } from \"zod/v3\";\n\
             import { z as z3 } from \"zod\";\n\
             export { a as b } from \"./m\";\n",
            &[("b", "a"), ("z", "z"), ("z3", "z")],
        ),
        (
            Lang::Rust,
            "use crate::config::Config;\n\
             use crate::{churn, dedup};\n\
             use crate::scan::{ast, spec as scan_spec};\n\
             use std::io::Read as _;\n",
            &[
                ("Config", "Config"),
                ("ast", "ast"),
                ("churn", "churn"),
                ("dedup", "dedup"),
                ("scan_spec", "spec"),
            ],
        ),
        (
            Lang::Haskell,
            "module M where\nimport CE.Wire (RowsReq (..), knoblessRows)\n",
            &[("RowsReq", "RowsReq"), ("knoblessRows", "knoblessRows")],
        ),
    ];
    for (lang, src, want) in cases {
        let expect: Vec<(String, String)> = want
            .iter()
            .map(|(l, t)| ((*l).to_string(), (*t).to_string()))
            .collect();
        assert_eq!(bindings(src, *lang), expect, "{lang:?}");
    }
}

/// The forms that name nothing a target DECLARES, every one drawn
/// from a refuter's counter-example: a module-object binding, a
/// wildcard or glob, a default or namespace import, an `as _`, and
/// Haskell's `hiding` (which names the COMPLEMENT of a list) or a
/// bare import (which names everything the target exports). None is
/// enumerable from this file's syntax, so each offers nothing rather
/// than guessing.
#[test]
fn forms_that_name_no_declaration_offer_no_candidate() {
    let cases: &[(Lang, &str)] = &[
        (Lang::Python, "import sys\n"),
        (Lang::Python, "import encodings.idna\n"),
        (Lang::Python, "import simplejson as json\n"),
        (Lang::Python, "from m import *\n"),
        (Lang::TypeScript, "import Image from \"next/image\";\n"),
        (Lang::TypeScript, "import * as fs from \"fs\";\n"),
        (Lang::Rust, "use crate::config::*;\n"),
        (
            Lang::Haskell,
            "module M where\nimport Data.List hiding (sort)\n",
        ),
        (Lang::Haskell, "module M where\nimport Data.List\n"),
    ];
    for (lang, src) in cases {
        assert!(
            bindings(src, *lang).is_empty(),
            "{lang:?} {src:?} must offer no target-side name"
        );
    }
}
