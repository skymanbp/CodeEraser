# Addendum 2 to BARS.md — rulings on constructs section 2 does not place (written 2026-11-11, AFTER all answers)

BARS.md (sha256 2934ab1c7f33f3373065a8fbc37b349a35b816481a882a4e94b7c3145a29fa65) is not edited.

## What was known when this was written

All 21 first-reading and all 4 second-reading answer files existed. The main session had seen: the unweighted
totals (392 in use / 28 unused / 4 unsure of 424), the mechanical tag totals (278 named / 114 generic), the
agreement figure (94 of 98 twice-read rows on the same side), the seven rows whose two answers differ, and the
evidence text of a handful of rows printed while debugging the format check. It had NOT seen the K1 class of
any row: no class doc exists, and the lane's inventory printed only node kinds, counts and one example each.
The lane had not opened any answer file.

So these rulings are made by someone who knows the answers but not the classes. To keep that from steering
anything, three things hold:

1. Each ruling follows from one of three rules stated here first, not from its effect on a table.
2. Where the letter of section 2 decides, the letter stands, even where a wider reading would be defensible
   (a regular-expression literal, an operator over literals only). The list of rule 5 is not widened.
3. The class of every row is frozen twice: `k1` (section 2 with the rulings below) and `k1_strict` (section 2
   read with no ruling: every construct the lane's inventory reported as undecided is ACTIVE / not
   constant-like). Every table of section 4 is printed under both. An outcome of section 4 ships without a
   question only when it is the same under `k1` and `k1_strict` (and, per addendum 1, under readings A and B).
   If they differ, nothing ships for that class from this audit and both sets of numbers go to the owner.

## The three rules

- (T) Transparent. A construct that only groups or selects other top-level statements, and computes nothing
  itself, is not a statement of its own: the statements inside it are judged by rules 1-6 as if they stood at
  the top level. For a compile-time conditional every branch is judged.
- (N) Nothing. A construct that executes nothing at run time is inert.
- (L) Letter. A right-hand side is constant-like only when it is on rule 5's list. It is read through syntax
  that changes nothing at run time (parentheses, type-only wrappers); the list itself gains no entry.

## Rulings (numbers are the lane's, from its phase 2A question)

1. C / C++ preprocessor conditionals (`#if`, `#ifdef`, `#ifndef`, `#elif`, `#else`, the include guard among
   them) and `extern "C" { … }` linkage blocks: (T). `#define` is a macro declaration (rule 3). Every other
   preprocessor directive (`#undef`, `#pragma`, `#error`, `#line`): (N). A `namespace` keeps rule 3 as
   written (inert as a whole).
2. A bare block — Lua `do … end`, and `{ … }` used as a statement in any language: (T).
3. The empty statement `;`: (N).
4. A hashbang line: a comment (rule 1).
5. A declaration that binds no value — Lua 5.5 `global x`, `global <const> *`: inert (rule 5 with no
   right-hand side; see 13).
6. A Lua label `::name::`: (N). A `goto` is a control statement and stays active.
7. A regular-expression literal: NOT constant-like (L: rule 5 lists number / string / boolean / null), so the
   binding is active.
8. Parentheses, TS `as` / `satisfies` / `<T>x` / the non-null `!`: read through (L); what is inside is judged.
9. An operator over literals only (`2+3`, R `3:20`, `"a" + "b"`): NOT constant-like (L: the only operator rule
   5 admits is the minus of a negative number). Active.
10. Dotted name, per language: a chain of member accesses by a fixed name, rooted at a name. In R that is
    `a$b`, `a@b`, `pkg::name`, `pkg:::name` when both sides are names. All constant-like. `f()$x`, `a[[1]]`,
    `a["b"]`, `df[…]` are not.
    Under ruling 8, `process.env.GITHUB_TOKEN!` is a dotted name and constant-like; if your implementation
    reads that line differently, say why in RESULT-2A.
11. R `TRUE` / `FALSE` / `NA` and its typed forms / `NULL` / `Inf` / `NaN`: literals of the R grammar, so
    boolean / null / number literals of rule 5.
12. A template string with no substitution: a string literal. With a substitution: active, as section 2
    already says.
13. A binding with no right-hand side (`int y;`, `local x`, `x: int`): inert — "every right-hand side is
    constant-like" holds when there is none.
14. Python augmented assignment (`x += […]`): active. It is not one of rule 5's binding forms, and it is an
    operator over the non-literal `x`.
15. Agreed as you have them, by the closed list: a subscript, an object spread, a JSX element, a
    comprehension, a conditional expression, R `[` / `[[`, and a bare name as a statement are active.

A construct that turns up later and is in neither section 2 nor this list: stop and ask again; do not place it.

## What the class docs carry (answer to the lane's design question)

Agreed: CI checks everything it can without the clones, and one `#[ignore]` leg re-derives both classes from
the clones; that leg is run in this task and its output kept. In addition:

- every row carries `k1`, `k1_strict`, the witness of each (kind and line of the first active statement, or
  null), the list of undecided kinds the row contains (the ruling numbers above), and the sha256 of the bytes
  that were classified;
- the CI leg also checks what follows from the definitions alone: a class is `runs-alone` exactly when its
  witness is non-null; `document` exactly when `doc_class` is non-null, and then both classes are `document`;
  a row that is `runs-alone` under `k1` is `runs-alone` under `k1_strict` (strict only turns inert into
  active); a row with no undecided kind has `k1 == k1_strict`;
- RESULT-2A prints, per language, how many rows differ between `k1` and `k1_strict` and by which ruling.

The order stays as registered: the class docs are committed before the lane opens any answer file.
