# Field test — the full suite on real repositories (M9 batch 6)

> Plan v2.8 batch 6: run every report family, the gates and the
> dry-run eraser on at least two REAL repositories that are not this
> one, ledger what happens, and feed the findings back as fixes. The
> two subjects are active local projects of the maintainer — a
> Python Claude Code memory plugin (70 scannable files, 1027
> functions; "repo A") and a Python enforcement plugin (86 files,
> 1001 functions; "repo B"). Neither had ever run CodeEraser. All
> state created by the test (`.ce/`, `ce-baseline.json`) was removed
> afterwards with `ce eject`; both repos are identified here by shape
> rather than name on purpose.

## What was run

Per repo, with the HEAD binaries (release build, own core):
`scan`, `dedup`, `dedup --check`, `docdup`, `deadcode`,
`churn --days 14`, `structure`, `structure --split-candidates`,
`erase` (dry-run), `baseline`, `check`, `trend --commits 8`,
`doctor`, `eject`.

## The headline finding: score saturation (fixed, proto 2.17.0)

Both repos measured **0/1000** on first contact. Repo A's size axis
alone charged 10176‰ (one 4802-line file priced like 95 hard-line
files under the quadratic extrapolation past H); repo B's charged
4325‰. Any real repository past the clamp scored identically zero —
one giant file and a repo-wide disaster were indistinguishable,
which is the dead-field failure this project was founded against,
one representation later.

Root cause was structural, not parametric: unbounded violation mass
mapped linearly onto a bounded scale and clamped. The fix replays
the design from first principles — the zone curve continues C¹
LINEARLY past H (still monotone, still charging, never quadratic
outside its contracted domain), and every axis charges the bounded
density `floor(scale·v/(v+n))` of its violation mass over its
opportunity count. Full math and citations:
[methodology 05](reference/methodology/05-scoring-and-the-adr-006-ratchet.md),
wire ledger [VERSIONING.md](../contracts/VERSIONING.md) (2.17.0),
design amendment [size-advisory.md](reference/size-advisory.md) §A.

Scores after the fix — discrimination restored, self unchanged in
ordering, CI floor re-anchored 800 → 950 at the same bite:

| repo | before | after | reading |
|---|---|---|---|
| self | 802 → 953 (re-anchored floor 950) | 953 | clean, tightly gated |
| repo B | 0 | **821** | 8 past-H giants, else tidy |
| repo A | 0 | **741** | 20 past-H giants, 43 dead files, heavier cloning |

## Secondary findings

- **Plugin-shaped repos look deader than they are.** Repo A drew 43
  dead-file verdicts out of 161 nodes: hook entry points are declared
  in JSON manifests, not imported, so the reference graph cannot see
  them. The verdicts are honest under the declared model (each one
  says "no kept in-edge and no entry flag"), and `[graph]
  entry_globs` in ce.toml is the remedy. Shipped in K step 9: when at
  least two files are dead and the dead share reaches half the tier —
  repo A's exact shape — `ce deadcode` now prints the entry_globs
  hint itself (cli/src/graph/deadcode/report.rs:78-88). A large but
  minority dead set stays hint-free on purpose: hinting an exemption
  knob at genuinely dead files would teach masking over deleting.
- **`erase` stayed safe and honest.** On both repos the dry-run plan
  licensed at most one row and demoted everything else to advisory
  with named reasons. `language_unresolved` on a plain `.py` file
  reads like a detection bug but is the conservative predicate doing
  its job — the file's LANGUAGE had hundreds of unresolved reference
  sites, so liveness cannot be trusted. Shipped in plan v2.25: the
  advisory line now carries that language's unresolved-site count
  (`reason_detail` in cli/src/erase/render.rs) — display-only, the
  wire reason bits are frozen.
- **Refusals refuse loudly.** `dedup --check` without a declared
  budget exits 2 with a one-line reason — an unjudgeable gate never
  passes.
- **Split advisory on giants.** Under the linear arm the advisor
  prices real seams on the giants it used to call cohesive (repo B's
  largest doc: recover 28560 vs cost 6950 milli). The raw
  milli-penalty units read awkwardly at that magnitude — ledgered as
  a batch-9 polish candidate, and shipped in batch 9 P15: the
  advisory now leads with the ROI the core's verdict is made on and
  the ‰ glyph came off the absolute operands
  (cli/src/structure/report.rs:168-181).
- **Latency held.** Cold `scan` 0.6s / `dedup` 2.0s on repo A;
  `trend --commits 8` 28s cold; every command answered without a
  window flash or a hang.
