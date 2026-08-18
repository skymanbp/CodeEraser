<!-- GENERATED — do not edit. Regenerate: CE_BLESS=1 cargo test --test docs_gate. CI reddens when this file drifts from its regeneration. Length rides the CLI surface (a machine-generated projection, the hs_boot stance), so the scan's file-lines warn on the CLI page is an accounted standing warn, not maintained prose over budget. -->

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
| `mode` | *(absent)* | Explicit hook tier for every rule class: observe / warn / ask / deny; unset = per-class route defaults (deny for the two FPR-promoted classes, observe otherwise) |

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

