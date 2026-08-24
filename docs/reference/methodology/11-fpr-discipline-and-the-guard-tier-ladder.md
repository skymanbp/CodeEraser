# FPR discipline and the guard tier ladder

[index](../methodology.md) · [← 10 Score trajectory — the trend slope verdict](10-score-trajectory-the-trend-slope-verdict.md) · [→ 12 Deterministic erase — the safety predicate](12-deterministic-erase-the-safety-predicate.md)

### The thesis: a deterministic gate over a nondeterministic writer

The guard sits on `PreToolUse` for `Write|Edit` ([guard.rs:61](../../../cli/src/guard.rs#L26)) and answers one question per pending write with exact arithmetic, never with a model. Two rule classes fire there: a T1/T2 duplicate-write probe against the fingerprint index, and a hard-budget breach computed locally ([guard.rs:78-82](../../../cli/src/guard.rs#L43)).

Determinism is bought by *replaying the write* rather than estimating it. `resulting_lines` computes the exact post-write line count — `Write` is its own payload (`content.lines().count()`, [budget.rs:86-88](../../../cli/src/guard/budget.rs#L86)); `Edit` is the payload applied to the on-disk file under Edit's own semantics: CRLF-normalized, and a non-`replace_all` edit requires a **unique** match, matching the real tool's rejection of ambiguity ([budget.rs:99-104](../../../cli/src/guard/budget.rs#L99)). Any case where the tool call would fail on its own — missing file, empty or absent `old_string`, ambiguous match — returns `None` and the rule stays silent ([budget.rs:84-104](../../../cli/src/guard/budget.rs#L84)). The gate never judges a write that will not land.

The same discipline is applied to the *policy* value, not only the measurement. `Guard::tier` is total by construction: an unrecognized `[guard] mode` resolves to an `observe (ce.toml ERROR: unknown guard mode …)` string rather than being passed through verbatim ([tier.rs:56-65](../../../cli/src/config/tier.rs#L56)), because a pass-through typo previously disarmed every enforcement path while `SessionStart` still printed the mode as armed ([tier.rs:47-55](../../../cli/src/config/tier.rs#L47)). A load failure renders through the same single throat, `tier_of` ([tier.rs:71-79](../../../cli/src/config/tier.rs#L71)), so a broken config can never print byte-identically to a deliberate `observe`.

Two honesty boundaries bound the thesis, and both are written into the plan:

- `PreToolUse` is a **behavior-shaping layer, not a security boundary** — an agent can bypass it with `Bash: echo >>` or `sed -i`; the backstop is the `Stop` audit over `git diff` (write-tool agnostic) plus the CI gate ([DEVELOPMENT_PLAN.md:92-94](../../DEVELOPMENT_PLAN.md#L92)).
- The hook is **fail-open**: any internal failure allows the edit, and the degraded run lands in the observe feed ([guard.rs:4-8](../../../cli/src/guard.rs#L4)); a probe that cannot reach the daemon returns `None`, which is logged as `degraded` rather than treated as "no duplicates" ([guard.rs:197-215](../../../cli/src/guard.rs#L165), [guard.rs:279](../../../cli/src/guard.rs#L247)).

### The tier ladder

Four tiers, and nothing else is a tier: `TIERS = ["observe", "warn", "ask", "deny"]` ([tier.rs:40](../../../cli/src/config/tier.rs#L40)). They map onto Claude Code's decision JSON at one emission point ([guard.rs:241-256](../../../cli/src/guard.rs#L209)):

| tier | `permissionDecision` | effect |
|---|---|---|
| `observe` | *(none — return before printing)* | feed line only, no injected text |
| `warn` | `allow` | edit proceeds, reason surfaces as a visible warning |
| `ask` | `ask` | user is prompted |
| `deny` | `deny` | write is refused, reason points at the existing `file:line` |

A mistyped mode falls through the same `match` to no output ([guard.rs:249](../../../cli/src/guard.rs#L249)) — it can never enforce — while the degraded string still rides the observe feed's `mode` field ([guard.rs:114](../../../cli/src/guard.rs#L114), [guard.rs:271-284](../../../cli/src/guard.rs#L239)) and the health line.

Tier resolution is per rule class, not global: an explicit `[guard] mode` overrides every class; otherwise the §4.2 route default for that class applies ([tier.rs:42-48](../../../cli/src/config/tier.rs#L42)). The route default for the two promoted `PreToolUse` classes is a single constant read by both `guard.rs` and `health.rs`, so the enforced tier and the reported tier cannot drift:

```
PROMOTED_DEFAULT = "deny"     // tier.rs:36
```

([tier.rs:36](../../../cli/src/config/tier.rs#L36); consumed at [guard.rs:93](../../../cli/src/guard.rs#L93) and [health.rs:62](../../../cli/src/health.rs#L62)). Everything else routes to `observe` — explicitly because it has no FPR record of its own, which is the entry requirement below ([DEVELOPMENT_PLAN.md:101-103](../../DEVELOPMENT_PLAN.md#L101)).

When several rules fire on one write, the decision line is emitted at the **strongest** tier among them, ranked by index in `TIERS` (unknown values rank 0 = `observe`): class rules carry the class mode, the zone rule carries its own mapped tier, so a zone warn never rides a deny-class escalator ([guard.rs:159-176](../../../cli/src/guard.rs#L159)). A broken `ce.toml` overrides the computed tier down to a visible `warn` that names the parse error ([guard.rs:177-184](../../../cli/src/guard.rs#L145)).

### The FPR gate that promotes a class

The ladder is a **route**, written into the plan so the default can neither stay at `warn` forever nor start at `deny` ([DEVELOPMENT_PLAN.md:96-103](../../DEVELOPMENT_PLAN.md#L96)):

1. 0.x (M3–M4): default `warn`; `deny` exists as an opt-in capability.
2. After the M4 FPR gate passes: **T1/T2 exact duplicate write** and **hard-budget breach (file > 750 lines)** promote to `ask`.
3. 1.0 (M7): those two classes promote to `deny`; every other rule stays `observe` for want of its own FPR record. Each default change records its FPR evidence in the CHANGELOG.

The gate itself is quantitative and appears in two places with two shapes. The M3 acceptance criterion is **≤ 1 mis-block in 500 real normal edits**, with `N=1` demonstrations explicitly disallowed ([DEVELOPMENT_PLAN.md:266](../../DEVELOPMENT_PLAN.md#L266)); the M4 main gate is **FPR ≤ 1% over 500 real normal edits**, on a pre-registered evaluation set frozen before implementation, ≥ 200 edit samples, ≥ 50% from real agent transcripts ([DEVELOPMENT_PLAN.md:267](../../DEVELOPMENT_PLAN.md#L267)). Risk register R4 states the admission rule flatly: **deny admission = the M4 FPR gate (≤ 1%)** ([DEVELOPMENT_PLAN.md:301](../../DEVELOPMENT_PLAN.md#L301)).

Sample purity is part of the gate, not an afterthought: only observe-mode sessions and pre-M3 no-guard sessions may be sampled, edits the guard already intervened in are excluded and the exclusion ratio reported — otherwise FPR is biased downward by the guard's own shaping and the deny admission becomes self-certifying ([DEVELOPMENT_PLAN.md:267](../../DEVELOPMENT_PLAN.md#L267)). This is why every probed event is appended to `<root>/.ce/observe.ndjson` in **all** modes ([guard.rs:7-8](../../../cli/src/guard.rs#L7)) and why the feed carries `session_id`: the evaluation set is partitioned by session, and neither the dogfood-session count nor the purity rule is answerable without it ([envelope.rs:16](../../../cli/src/guard/envelope.rs#L16), [hookio.rs:36-43](../../../cli/src/hookio.rs#L36)).

#### The recorded replay

The M3 replay treats git linear history as a real edit stream: each changed code file in each commit is one "write the child version's content into the parent state" event, materialized incrementally in a shadow directory, **probe first, then apply**; guard ran at default knobs `t = 50 / min_distinct = 7` ([FPR-REPLAY.md:7-12](../../FPR-REPLAY.md#L7)). Those two knobs are the shipped defaults: `t = window + kgram - 1 = 26 + 25 - 1 = 50` tokens, aligned to the jscpd `min-tokens` default ([mod.rs:296-313](../../../cli/src/dedup/mod.rs#L296)), and the diversity floor `7`, calibrated because arbitrated data-row false positives measured `distinct <= 6` while arbitrated true clones measured `>= 7`. Since proto 2.19.0 (batch-7 slice 1) the floor's AUTHORITY is the core's ([CE/Dedup/Cost.hs](../../../core/app/CE/Dedup/Cost.hs)): `ce dedup --check` ships the pre-filter distinct counts, the core re-derives the admitted count and judges the budget from its own number, and Rust's `DEFAULT_MIN_DISTINCT` survives as a declared mirror ([pairs.rs:113-118](../../../cli/src/dedup/pairs.rs#L113)) held equal by three legs — the knob-face pin in core_wire, the mirror declaration, and the per-run `dedupBlocks` ensure in budget.rs.

Result ledger ([FPR-REPLAY.md:16-20](../../FPR-REPLAY.md#L16)):

| corpus | events | blocks | false blocks after arbitration | per-500 |
|---|---|---|---|---|
| requests, 400-commit history tail (pinned at `1f6589ec`) | 487 | 0 | 0 | 0.00 |
| CodeEraser self-repo, full history (real agent edit stream) | 143 | 35 | 0 | 0.00 |
| total | 630 | 35 | **0** | **0.00 ≤ 1** |

The 35 self-repo blocks arbitrated as **true positives**, itemized: 2 were zod locale fixtures that are T2 clones by design, and 33 were `rust_fn(seed)` / `tmp()` / git helpers copy-pasted across 8 test files by the author — remediated in four batches, 251 → 211 → 209 → 205 → 202 clone blocks with the dedup budget stepped down in lockstep ([FPR-REPLAY.md:22-36](../../FPR-REPLAY.md#L22)). The replay also caught a product defect: a candidate file disappearing between index and probe (delete/rename race, triggered by a real directory-rename commit in the requests history) used to error the whole probe, and now degrades to a skip ([FPR-REPLAY.md:38-43](../../FPR-REPLAY.md#L38)).

Stated boundary: the replay instrument `cli/tests/fpr_replay.rs` was retired with the M7.5 slimming pass; this ledger is the final record, and re-verification requires resurrecting the instrument from git history ([FPR-REPLAY.md:3-5](../../FPR-REPLAY.md#L3), [FPR-REPLAY.md:47-52](../../FPR-REPLAY.md#L47)).

### The hard-budget rule

Scope is decided **before** any disk read — language arm, then exclude walk — so the hook never pays an unbounded read for a file it is about to declare out of scope ([budget.rs:52-59](../../../cli/src/guard/budget.rs#L52)). The rule then fires iff:

```
cap = thresholds.file_lines_fail          // default 750
breach  ⇔  cap != 0 ∧ lines > cap
```

([budget.rs:63-69](../../../cli/src/guard/budget.rs#L63); default `file_lines_fail = 750`, [ce-toml.md:18](../ce-toml.md#L18)). Since the rulepack's hook slice (plan v2.13 ① P4) the line is the **file's own**: `lines_for` compiles the declared `[[rules.class]]` set once per hook run and takes the class's effective table, the global one for class 0 — the same reading the scan gate and the score take, so the hook denies at exactly the line the CI wall would fail at ([budget.rs:66-81](../../../cli/src/guard/budget.rs#L66)); the graded zone reads its H and its warn fallback from the same table. The `cap == 0` guard is load-bearing: it encodes "no hard line exists" per the P3 grade-table contract, where a naive comparison read `0` as "every write breaches" ([budget.rs:108-110](../../../cli/src/guard/budget.rs#L108)). Every firing gets its own `budget` feed line in **every** tier, because the step-3 decision at 1.0 needs per-rule records ([budget.rs:72-85](../../../cli/src/guard/budget.rs#L72)) — the feed's `budget` event was added in schema `0.4.0` for exactly that reason ([hookio.rs:33-34](../../../cli/src/hookio.rs#L33)).

### `zone_tiers`: the opt-in position-to-tier map

Below the hard line, a write landing in the graded zone `(S, H]` is scored by position. Position is integer per-mille:

```
S = committed baseline softLine, else thresholds.file_lines_warn   // fallback default 300
H = thresholds.file_lines_fail                                     // default 750
permille = (lines - S) * 1000 / (H - S)
```

([budget.rs:192-196](../../../cli/src/guard/budget.rs#L192); `S` is read off the committed `ce-baseline.json` via `committed_soft`, [budget.rs:155-160](../../../cli/src/guard/budget.rs#L155); `file_lines_warn = 300`, [ce-toml.md:17](../ce-toml.md#L17)). A degenerate zone — `H == 0` (no hard line), `H <= S`, or `lines <= S` — logs nothing rather than fabricating a position ([budget.rs:183-185](../../../cli/src/guard/budget.rs#L183)).

The map is a three-way partition on that per-mille ([budget.rs:146-152](../../../cli/src/guard/budget.rs#L146)):

```
0   ..= 249  →  observe
250 ..= 750  →  warn
751 ..       →  ask
```

At the default `S = 300`, `H = 750` (so `H - S = 450`), integer division puts the boundaries at `lines >= 413` for `warn` and `lines >= 638` for `ask` — derived from the constants above, not stated in the source.

The switch is **off by default** and changes one repo only: `[guard] zone_tiers`, default `false` ([tier.rs:24-28](../../../cli/src/config/tier.rs#L24), [ce-toml.md:31](../ce-toml.md#L31)). Disarmed, the zone is feed-only — a `zone` line is written in every tier and `zone_assess` returns `None`, injecting nothing ([budget.rs:165-171](../../../cli/src/guard/budget.rs#L165)). Armed, the resolved tier is additionally stamped as `zone_tier` on the feed line, so the record says what the rule *did*, not only where the write landed ([budget.rs:128-130](../../../cli/src/guard/budget.rs#L128), [hookio.rs:23-26](../../../cli/src/hookio.rs#L23)).

The default stays `observe` for one reason, written at the declaration site: §4.2's FPR discipline gates any default flip on the zone feed's own record ([tier.rs:24-28](../../../cli/src/config/tier.rs#L24)), and the `zone` event exists precisely as "the per-rule record any future zone→tier promotion must argue its FPR case from" ([hookio.rs:28-31](../../../cli/src/hookio.rs#L28)). The rule class has no FPR ledger yet, therefore it does not enforce by default — the same admission test the two promoted classes had to pass.

### Injection budget: warns are rate-limited, enforcement is not

An anti-bloat tool must not itself become a context entropy source (§4.4 B4). Two mechanisms:

- **Per-(rule, file, session) suppression.** A warn fires once per session per file per rule; the observe feed *is* the accumulator, consulted **before** the current event lands in it so a warn cannot read its own fresh line as "already warned" ([guard.rs:100-106](../../../cli/src/guard.rs#L65), [budget.rs:116-118](../../../cli/src/guard/budget.rs#L116), [hookio.rs:198-214](../../../cli/src/hookio.rs#L198)). Suppression is skipped entirely when `mode ∈ {deny, ask}` — enforcement is not context bloat and repeats every time it holds ([guard.rs:103-105](../../../cli/src/guard.rs#L103)); an armed zone `ask` likewise repeats, only `warn` is budgeted ([budget.rs:118](../../../cli/src/guard/budget.rs#L118)). Counting is conservative in the reporting direction: a clean probe line (`matches == 0`) and an unarmed zone line (`zone_tier` absent or `observe`) never count as "already warned", and any read or parse failure returns "not warned" — fail open toward *reporting*, the opposite bias from enforcement's fail-open ([hookio.rs:141-165](../../../cli/src/hookio.rs#L141)).
- **Token clip at the single emission throat.** `WARN_BUDGET_TOKENS = 200`, `STOP_BUDGET_TOKENS = 400`, converted to chars by one declared measurement constant `CHARS_PER_TOKEN = 4` ([hookio.rs:147-149](../../../cli/src/hookio.rs#L147)); the clip is char-boundary safe and appends `… (clipped; full report in .ce/observe.ndjson)`, pointing at the full on-disk record ([hookio.rs:147-158](../../../cli/src/hookio.rs#L147), applied at [guard.rs:190-192](../../../cli/src/guard.rs#L155)).

The feed schema is a versioned contract pinned by a golden fixture, bumped on any shape change: `ce.observe/0.6.0` ([hookio.rs:18](../../../cli/src/hookio.rs#L18)), with the `zone_tier` key added at `0.6.0`, the `zone` event at `0.5.0`, the `budget` event at `0.4.0`, and `session_id` at `0.2.0` ([hookio.rs:18-43](../../../cli/src/hookio.rs#L18)). That versioning is what makes the FPR ledger replayable across releases rather than a claim about a build nobody can reconstruct.
