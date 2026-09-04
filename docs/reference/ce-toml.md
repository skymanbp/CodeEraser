<!-- GENERATED — do not edit. Regenerate: CE_BLESS=1 cargo test --test it docs_gate::. CI reddens when this file drifts from its regeneration. Length rides the CLI surface (a machine-generated projection, the hs_boot stance), so the scan's file-lines warn on the CLI page is an accounted standing warn, not maintained prose over budget. -->

# `ce.toml` reference

Declarative-only by design (plan §5.9): no executable fields, ever. An unknown key is a parse error (`deny_unknown_fields`) — the guard then degrades to observe and NAMES the error instead of silently dropping a mistyped policy. An *(absent)* judgment knob means the Haskell core's own default judges; every report echoes its effective values under `knobs`.

## Top level

| key | default | meaning |
|---|---|---|
| `exclude` | *(absent)* | Extra exclude globs, added on top of the built-in defaults (plan §4.1); gitignore syntax, `/` separators only — a `\` is an escape, a leading `!` or `#` is refused by name, `dir/` names a directory. A committed ratchet row whose file an exclude (or an ignore file) hides is `dropped`, not removed: `ce check` fails by name (`rows_dropped`, 6.4.0) until `CE_ACCEPT_FENCE=1 ce baseline` owns the exclusion |

## [thresholds]

| key | default | meaning |
|---|---|---|
| `file_lines_warn` | `300` | Warn once a source file passes this line count |
| `file_lines_fail` | `750` | Hard file budget: scan fails past it, and the guard refuses writes that would land past it |
| `fn_lines_warn` | `50` | Warn once a function passes this line count |
| `fn_lines_fail` | `75` | Fail once a function passes this line count |
| `params_warn` | `5` | Warn once a function takes more parameters than this |
| `cyclomatic_warn` | `15` | Warn past this cyclomatic complexity |
| `cognitive_warn` | `15` | Warn past this cognitive complexity |
| `cognitive_fail` | `0` | Fail past this cognitive complexity. `0` — the default — is the published "no hard line", so the complexity axis ships with no absolute limit at all: the plan's own §4.1 evidence row records that cognitive complexity has no support on the correctness axis, and a metric this project declines to over-claim gets no wall by default. Declaring one arms `ce scan`'s fail tier (and `[[rules.class]]`) and nothing else — the score's complexity axis still charges against `cognitive_warn` alone, so turning this on never moves a score, and the PreToolUse hook never reads it (§4.2 keeps write-time checks AST-free) |
| `nesting_warn` | `4` | Warn past this block-nesting depth |

## [guard]

| key | default | meaning |
|---|---|---|
| `mode` | *(absent)* | Explicit hook tier for every rule class: observe / warn / ask / deny; unset = per-class route defaults (deny for the two FPR-promoted classes, observe otherwise). Any other value is a typo, not a tier: it resolves to observe and the SessionStart line, `ce doctor` and the observe feed all name it, so a mistyped mode can never look armed |
| `zone_tiers` | `false` | Arm the graded-zone tier map (plan v2.7): a write landing <25% into (softLine, hard budget] stays observe, 25-75% warns, >75% asks. Default OFF - the zone is feed-only until a repo opts in, and the observe feed records the mapped tier when armed |

## [dedup]

| key | default | meaning |
|---|---|---|
| `budget` | *(absent)* | Only-shrink clone-block budget; `ce dedup --check` fails when the repo exceeds it |

## [graph]

| key | default | meaning |
|---|---|---|
| `entry_globs` | *(absent)* | Extra liveness roots for the deadcode judgment, beyond the mechanical entry conventions; the exclude list's dialect (`dir/` selects the directory's files; `src/**/*.ts` and every other pattern read as written) |
| `crate_roots` | *(absent)* | Rust crate roots of a tree whose manifest lives elsewhere (the test-suite submodule is a slice of the `cli` package): root-relative exact paths. A declared root mounts its `mod` children and anchors `crate::` paths like a manifest target, and is one for the deadcode entry role; a declared path that is not a walked Rust file is refused by name |
| `scc_floor` | *(absent)* | The cycle floor: the smallest strongly connected component the graph's cycle table and the score's cycle axis count as a cycle — one knob, two faces (graph/1 `sccFloor`, verdict/1 threshold `cycleFloor`). Absent = 2, a lone file is never a cycle; 1 counts a file exactly when it imports itself; 0 is refused by name |

## [score]

| key | default | meaning |
|---|---|---|
| `weights` | *(absent)* | Per-axis weight numerators by axis name (size / complexity / clone / docdup / deadcode / churn / cycle); unlisted axes keep the equal default |
| `size_penalty_max` | *(absent)* | Soft-zone curve: the size-axis penalty of a file AT the hard line (plan v2.6; default 10) |
| `soft_line_k` | *(absent)* | Relative soft line: the multiplicative-MAD exponent k in S = clamp(median*r^k, [200,500]) (default 2) |
| `dead_indeg_ceil` | *(absent)* | Deadcode axis: a file at or below this in-degree (and unreachable) counts as orphaned |
| `rewrite_num` | *(absent)* | Churn axis rewrite-share threshold, ratio numerator (cross-multiplied) |
| `rewrite_den` | *(absent)* | Churn axis rewrite-share threshold, ratio denominator |
| `cochange_floor` | *(absent)* | Co-change count at which a pair counts as entangled |
| `viol_cost` | *(absent)* | Per-mille cost of one weighted violation in the score fold |
| `default_weight` | *(absent)* | Weight of any axis the weights table does not name |
| `score_scale` | *(absent)* | The score's full scale (per-mille by default) |
| `tol_num` | *(absent)* | Ratchet ratio leg numerator: a ceiling may grow to ceiling*tol_num/tol_den in one edit |
| `tol_den` | *(absent)* | Ratchet ratio leg denominator |
| `tol_abs` | *(absent)* | Ratchet absolute leg: or by this many lines, whichever is larger |

## [structure]

| key | default | meaning |
|---|---|---|
| `layout` | *(absent)* | Declared directory weights the divergence axis judges against; keys are root-relative directories, "." the catch-all bin |

## [trend]

| key | default | meaning |
|---|---|---|
| `min_points` | *(absent)* | History points required before a slope is judged |
| `decline_floor_micro` | *(absent)* | Decline floor in micro-per-mille per day; declaring it arms the trend fail bit |

## [rules]

| key | default | meaning |
|---|---|---|
| `class` | *(absent)* | Path classes with their own size and complexity lines (plan v2.13): an array of tables, each with a local `name`, its `globs` (the exclude list's dialect through the exclude list's own parser — gitignore syntax, `dir/` the directory's files; the first declared match owns a path, an unmatched path keeps the global table) and `knobs` — `file_lines_warn`, `file_lines_fail`, `cognitive_warn`, `cognitive_fail`, `fn_lines_warn`, `fn_lines_fail`, `ratchet_tolerance`, `cognitive_ratchet_tolerance`; an absent knob inherits the global line. The score reads the size and complexity three, the scan ladder the six lines — `cognitive_fail` (v2.24) among them, so a class may carry a complexity wall the rest of the tree does not — and `ratchet_tolerance` (5.1.0) is the class's own ADR-006 allowance in lines: declared, it replaces BOTH global legs, so `0` freezes the class at its current ceilings and the global `max(+2%, +10)` never applies; `cognitive_ratchet_tolerance` (6.4.0) is the same allowance for the fn-complexity rows alone — declared, it replaces `ratchet_tolerance` for those rows, so a class may freeze its lines and still allow complexity growth, or the reverse. At most 64 classes, each class's ladder must climb, and only a class's index and knobs ever cross the wire. Since 6.0.0 the baseline records a fingerprint of the WHOLE parsed config — every table, not just this one — and since O39 that fingerprint is the CANONICAL effective knob set: the values that differ from the shipped defaults, so comments, key order, a knob spelled at its default and an optional knob nobody declared leave it alone, and a class's `name` is a label outside it (a rename is silence); editing any effective knob fails `ce check` by name (`knobs_digest`) until `CE_ACCEPT_FENCE=1 ce baseline` re-pins the same ceilings under the new config (refused when anything else held) or `CE_ACCEPT_BASELINE=1` names a new floor, so neither a glob edit nor a `[score]` edit can move every line in silence. Since 6.4.0 `ce scan` judges the same fence (`knobs_digest` among its named conditions, exit 1), and the PreToolUse hook judges budgets with the SHIPPED thresholds, exclude list and classes while the config drifts, naming the fence in its deny reason — a drifted config is an unverified document, and a hook that kept any part of it would let the edit that produced the drift choose its own budget. Enabling classes changes what the score judges a file against — scores are not comparable across that switch |

## [tombstone]

| key | default | meaning |
|---|---|---|
| `tier` | *(absent)* | The tombstone class's own hook tier: observe / warn / ask / deny; unset = observe. `[guard] mode` does not reach this class — a class with a key of its own decides at that key — and the class ships at observe until docs/FPR-TOMBSTONE.md argues a promotion (plan §4.2). Any other value is refused at load by name, so a mistyped tier can never look armed |
| `budget` | *(absent)* | Sites one changeset may carry before the class's condition holds (`sites > budget`, judged by the core over tombstone/1); absent = no condition is evaluated (`over` never holds) while the core still seats the sites; the judgment reaches the observe feed either way |
| `ledger` | *(absent)* | Files declared to hold the changelog role, in the exclude list's dialect (`dir/` the directory's files): exempt whole and counted `declared` in the feed — the backstop for a ledger neither the path, the shape nor the segment witness reads |
| `terms` | *(absent)* | The repository's own vocabulary: words that never spell a name, whole or as a word of a compound (`pork` declared keeps `braise_pork` out and leaves `braise` in); matched case-insensitively |

