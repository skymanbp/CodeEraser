# Addendum 3 to BARS.md — one more construct, and what `k1_strict` covers (2026-11-11, after all answers)

BARS.md and addenda 1-2 are not edited. Known to the main session at this moment: what addendum 2 lists, and
nothing more — no class of any row, and not the answer of the one file this ruling was raised on
(`luarocks win32/lua5.1/include/luaconf.h`); the main session has not looked it up and will not before the class
docs are committed.

## Ruling 16 — a fragment the parser could not place

A top-level `ERROR` node (top level after rule (T) of addendum 2 has opened wrappers and conditionals).

A fourth rule, stated before its use:

- (W) Witness. The class `runs-alone` rests on a witness: a statement the parser read and section 2 calls
  active. A fragment the parser could not place is no witness of anything — the classifier does not know what
  it is.

So under `k1` a top-level `ERROR` fragment is not a witness: it does not make the row `runs-alone`. The row
records ruling 16 and the fragment's line. Under `k1_strict` it is active, like every construct the inventory
reported as undecided. A `MISSING` node belongs to the parsed statement that holds it and needs no ruling. An
`ERROR` node inside a binding's right-hand side is not on rule 5's list, so that binding is active under both
classes (the letter decides).

RESULT-2A lists every row that carries ruling 16, per language, with the line.

## What `k1_strict` covers — the written definition stands

Addendum 2 defines it: "every construct the lane's inventory reported as undecided is ACTIVE / not
constant-like". That is: section 2 as the implementer read it BEFORE asking the main session anything, with
everything it could not place counted active. The implementer wrote that reading without access to any answer.

Rulings 11 (R constants), 12 (a template string with no substitution) and 13 (a binding with no right-hand
side) were placed by the implementer's draft before the question was asked; the main session only confirmed
them. They are therefore inert under `k1_strict` as well. Rulings 1-6, 8, 10 and 16 are the ones the
inventory reported: active / not constant-like under `k1_strict`. Rulings 7, 9, 14 and 15 are active under
both.

RESULT-2A still prints how many rows carry 11, 12 or 13, per language.
