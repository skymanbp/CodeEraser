//! Haskell legs: the top-level guard (H2/H4), the header-less rule and
//! the export-list lexer (H5). Every list-shaped fixture below compiles
//! under GHC 9.10.3 and its named binding is consumed by an importer —
//! the GHC verdict, not this crate's reading, is the oracle (plan v2.17
//! L round, K32).

use super::hs_lex::{ASC_SYMBOL, entries, is_symbol, strip_comments};
use super::tests::check;
use super::*;

fn hs(case: &str) {
    check(Lang::Haskell, case);
}

/// A header WITHOUT an export list exports every top-level binding;
/// a header WITH one exports exactly it. The negative half is the
/// load-bearing one: without it the export-list reader could be
/// returning None for every module and the first case would still
/// pass.
#[test]
fn export_list_is_read_from_the_header() {
    hs("module M where\nopen :: Int\nopen = 1\nshut :: Int\nshut = 2 ⇒ open:ES shut:ES");
    hs("module M (open) where\nopen :: Int\nopen = 1\nshut :: Int\nshut = 2 ⇒ open:ES shut:-");
}

/// No header is `module Main(main) where` the way GHC 9.10.3 reads it:
/// `main` alone when a top-level `main` exists (`import Main (helper)`
/// is refused, GHC-61689), everything when none does (accepted).
#[test]
fn headerless_module_exports_main_alone_when_it_has_one() {
    hs("helper :: Int\nhelper = 1\nmain :: IO ()\nmain = print helper ⇒ helper:- main:ES");
    hs("helper :: Int\nhelper = 1\nother :: Int\nother = 2 ⇒ helper:ES other:ES");
}

/// K21-hs (H2/H4): a `where`-bound helper that borrows an exported
/// name is not exported by it, and class default bodies and instance
/// bodies are members, not module bindings — even under a list-less
/// header that exports "everything".
#[test]
fn only_top_level_bindings_are_exported() {
    hs("module M (foo) where\nfoo :: Int\nfoo = go where go = 1\n\
        bar :: Int\nbar = x where foo = 3\n              x = foo ⇒ foo:ES go:- bar:- foo:- x:-");
    hs(
        "module M where\nclass C a where\n  cm :: a -> Int\n  cm _ = 0\n\
        instance C Int where\n  cm = id\ntop :: Int\ntop = 1 ⇒ cm:- cm:- top:ES",
    );
}

/// L round step 8 (O55/O29): the six named type forms are units, and
/// the export list names them by their head — `T(..)` and `C(m)` name
/// the type and the class, a bare `S` the synonym, `type Fam` under
/// ExplicitNamespaces the family; the class method `m` inside `C(m)`
/// names no top-level binding, and a form the list omits is not
/// exported. Under a list-less header every top-level form is.
#[test]
fn type_forms_are_units_named_by_the_export_list_head() {
    const BODY: &str = "data T = A\nnewtype N = N Int\ntype S = Int\nclass C a where\n  m :: a -> Int\n\
                        type family Fam a\ndata family DF a\nm' :: Int\nm' = 1 ⇒ ";
    for (header, want) in [
        (
            "module M (T(..), C(m), S, type Fam) where\n",
            "T:ES N:- S:ES C:ES Fam:ES DF:- m'/0:-",
        ),
        (
            "module M where\n",
            "T:ES N:ES S:ES C:ES Fam:ES DF:ES m'/0:ES",
        ),
    ] {
        super::tests::check_units(Lang::Haskell, &format!("{header}{BODY}{want}"));
    }
}

/// The step-8 review's three export shapes: a type operator's
/// `(:^:)(..)` / `type (:+:)(..)` entry names the operator (the member
/// list used to stay attached and the operator read private) as the
/// bare `((:%:))` always did; an associated family is exported with
/// its class — through `C(..)` or a listed `C(Fam)`, never by `C`
/// alone; and a `data`/`newtype instance` of a family mints no unit of
/// its own — the grammar wraps the instance's `data_type`/`newtype`
/// node in a `data_instance`, and the register used to key each as a
/// second and third declaration of the family (the review's
/// duplicate-row claim, confirmed by the producer before the fix).
#[test]
fn type_operators_associated_families_and_instances() {
    const CLASS: &str = "class C a where\n  type Fam a\n  data DF a\n  m :: a -> Int ⇒ ";
    for (header, want) in [
        ("module A (C(..)) where\n", "C:ES Fam:ES DF:ES"),
        ("module A (C(Fam)) where\n", "C:ES Fam:ES DF:-"),
        ("module A (C(type DF, m)) where\n", "C:ES Fam:- DF:ES"),
        ("module A (C) where\n", "C:ES Fam:- DF:-"),
    ] {
        super::tests::check_units(Lang::Haskell, &format!("{header}{CLASS}{want}"));
    }
    for (src, want) in [
        (
            "module R ((:^:)(..)) where\ndata (:^:) a b = N a b ⇒ ",
            "(:^:):ES",
        ),
        (
            "module P (type (:+:)(..)) where\ndata (:+:) a b = L a | R b ⇒ ",
            "(:+:):ES",
        ),
        (
            "module B ((:%:)) where\ndata (:%:) a b = N a b ⇒ ",
            "(:%:):ES",
        ),
        (
            "module N (n) where\ndata family DF a\nnewtype instance DF Bool = DFB Bool\n\
             data instance DF Int = DFI Int\nn :: Int\nn = 1 ⇒ ",
            "DF:- n/0:ES",
        ),
    ] {
        super::tests::check_units(Lang::Haskell, &format!("{src}{want}"));
    }
}

/// An entry qualified by the module's own name is the local binding;
/// one under a foreign qualifier re-exports an import and names no
/// local binding (GHC 9.10.3 rules both).
#[test]
fn qualified_entries_match_only_under_the_own_module_name() {
    hs("module MQ ( MQ.foo ) where\nfoo :: Int\nfoo = 1\nbar :: Int\nbar = 2 ⇒ foo:ES bar:-");
    hs("module MR ( XM.foo ) where\nimport qualified XM\nfoo :: Int\nfoo = 1 ⇒ foo:-");
}

/// K32 (H5): the export list under its own lexing. Each header is one
/// GHC-ruled fixture — hsprobe5..9, the maximal-munch four, the three
/// first-half witnesses, the layout-split and CPP-carrying lists —
/// over the same body; the `bar` mentioned inside hsprobe6's comment
/// and the never-listed `zzz` are the negative half.
#[test]
fn export_list_is_lexed_by_haskell_comment_rules() {
    const BODY: &str = "\nfoo :: Int\nfoo = 1\nbar :: Int\nbar = 2\nzzz :: Int\nzzz = 3 ⇒ ";
    for (header, want) in [
        ("module M ((-->), foo, bar) where", "foo:ES bar:ES zzz:-"),
        ("module M ( {- {- a -} -} foo ) where", "foo:ES bar:- zzz:-"),
        (
            "module M ( foo --- comment mentioning bar\n , baz ) where\nbaz = 3",
            "baz:ES foo:ES bar:- zzz:-",
        ),
        ("module M ( (--⊕) , foo ) where", "foo:ES bar:- zzz:-"),
        (
            "module M ( foo {- -- -} , bar ) where",
            "foo:ES bar:ES zzz:-",
        ),
        (
            "module M ( foo -- {- opener inside a line comment\n , bar ) where",
            "foo:ES bar:ES zzz:-",
        ),
        ("module M ( (<--) , bar ) where", "foo:- bar:ES zzz:-"),
        ("module M ( (|--) , bar ) where", "foo:- bar:ES zzz:-"),
        ("module M ( (-->--) , bar ) where", "foo:- bar:ES zzz:-"),
        (
            "module M ( T((:--)) , bar ) where\ndata T = Int :-- Int",
            "foo:- bar:ES zzz:-",
        ),
        ("module M ( (----->) , bar ) where", "foo:- bar:ES zzz:-"),
        ("module M ( (-) , bar ) where", "foo:- bar:ES zzz:-"),
        (
            "module M\n  ( foo --\n  , bar\n  ) where",
            "foo:ES bar:ES zzz:-",
        ),
        (
            "module M\n  ( foo -- ^ documented\n  , bar\n  ) where",
            "foo:ES bar:ES zzz:-",
        ),
        ("module M ( module M ) where", "foo:ES bar:ES zzz:ES"),
        (
            "module M (\n  module\n  M\n  ) where",
            "foo:ES bar:ES zzz:ES",
        ),
        ("module M ( module Other ) where", "foo:- bar:- zzz:-"),
        (
            "{-# LANGUAGE CPP #-}\nmodule M\n  ( foo\n#ifdef FLAG\n  , bar\n#endif\n  ) where",
            "foo:ES bar:ES zzz:-",
        ),
    ] {
        hs(&format!("{header}{BODY}{want}"));
    }
}

/// The two stripping orders the single pass replaces, as text: each
/// would lose `bar` — the first by eating `-} , bar` inside a line
/// comment, the second by opening a block on a `{-` that a line
/// comment already owns. A CPP directive line goes the way of a line
/// comment; a `#` that is not a directive stays.
#[test]
fn comments_come_off_in_one_pass() {
    assert_eq!(strip_comments("( foo {- -- -} , bar )"), "( foo  , bar )");
    assert_eq!(strip_comments("( foo -- {-\n , bar )"), "( foo \n , bar )");
    assert_eq!(strip_comments("( a {- x {- y -} z -} b )"), "( a  b )");
    assert_eq!(strip_comments("( (<--) , x --- y\n )"), "( (<--) , x \n )");
    assert_eq!(strip_comments("( foo {-# INLINE #-} )"), "( foo  )");
    assert_eq!(
        strip_comments("( foo\n#ifdef FLAG\n , bar\n#endif\n )"),
        "( foo\n\n , bar\n\n )"
    );
    assert_eq!(strip_comments("( foo\n  , (#) )"), "( foo\n  , (#) )");
}

/// A comma inside a subordinate list does not split the entry.
#[test]
fn entries_split_at_depth_zero_only() {
    assert_eq!(
        entries("( T(A, B), foo , (<+>) )"),
        vec!["T(A, B)", "foo", "(<+>)"]
    );
    assert_eq!(entries("()"), Vec::<String>::new());
}

/// The symbol table is the Haskell 2010 `ascSymbol` set — twenty
/// distinct characters — and the Unicode arm is by general category:
/// a math symbol and a quotation mark are symbols, a combining mark
/// and a CJK letter are not, and `special` never is.
#[test]
fn symbol_characters_follow_haskell_2010() {
    let mut distinct = ASC_SYMBOL.to_vec();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(distinct.len(), 20);
    for c in ['⊕', '«', '→', '¬'] {
        assert!(is_symbol(c), "{c}");
    }
    for c in ['a', '(', ')', '_', '"', '\'', '中', '\u{301}', ' ', '\n'] {
        assert!(!is_symbol(c), "{c:?}");
    }
}
