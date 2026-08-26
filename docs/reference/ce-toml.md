<!-- GENERATED — do not edit. Regenerate: CE_BLESS=1 cargo test --test it docs_gate::. CI reddens when this file drifts from its regeneration. Length rides the CLI surface (a machine-generated projection, the hs_boot stance), so the scan's file-lines warn on the CLI page is an accounted standing warn, not maintained prose over budget. -->

# `ce.toml` reference

Declarative-only by design (plan §5.9): no executable fields, ever. An unknown key is a parse error (`deny_unknown_fields`) — the guard then degrades to observe and NAMES the error instead of silently dropping a mistyped policy. An *(absent)* judgment knob means the Haskell core's own default judges; every report echoes its effective values under `knobs`.

## Top level

| key | default | meaning |
|---|---|---|
| `exclude` | *(absent)* | Extra exclude globs, added on top of the built-in defaults (plan §4.1) |

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
| `entry_globs` | *(absent)* | Extra liveness roots for the deadcode judgment, beyond the mechanical entry conventions |

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
| `class` | *(absent)* | Path classes with their own size and complexity lines (plan v2.13): an array of tables, each with a local `name`, its `globs` (the exclude list's dialect; the first declared match owns a path, an unmatched path keeps the global table) and `knobs` — `file_lines_warn`, `file_lines_fail`, `cognitive_warn`, `fn_lines_warn`, `fn_lines_fail`, `ratchet_tolerance`; an absent knob inherits the global line. The score reads the size and complexity three, the scan ladder the five lines, and `ratchet_tolerance` (5.1.0) is the class's own ADR-006 allowance in lines: declared, it replaces BOTH global legs, so `0` freezes the class at its current ceilings and the global `max(+2%, +10)` never applies. At most 64 classes, each class's ladder must climb, and only a class's index and knobs ever cross the wire. Since 6.0.0 the baseline records a fingerprint of the WHOLE parsed config — every table, not just this one; editing any knob fails `ce check` by name (`knobs_digest`) until `CE_ACCEPT_BASELINE=1 ce baseline` names the new floor, so neither a glob edit nor a `[score]` edit can move every line in silence. Enabling classes changes what the score judges a file against — scores are not comparable across that switch |

