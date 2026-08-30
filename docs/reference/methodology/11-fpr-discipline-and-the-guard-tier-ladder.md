# FPR discipline and the guard tier ladder

[index](../methodology.md) · [← 10 Score trajectory — the trend slope verdict](10-score-trajectory-the-trend-slope-verdict.md) · [→ 12 Deterministic erase — the safety predicate](12-deterministic-erase-the-safety-predicate.md)

### The thesis: a deterministic gate over a nondeterministic writer

The guard sits on `PreToolUse` for `Write|Edit` ([guard.rs:26-35](../../../cli/src/guard.rs#L26)) and answers one question per pending write with exact arithmetic, never with a model. Two rule classes fire there: a T1/T2 duplicate-write probe against the fingerprint index, and a hard-budget breach computed locally ([guard.rs:80-99](../../../cli/src/guard.rs#L80)).

Since the K-round root fix the duplicate-write rule judges **novel** duplication only: the probe's matches are subtracted by the matches the REPLACED content already carried — Write replaces the on-disk file, Edit replaces `old_string` — so a full rewrite of a file carrying budgeted blocks, or an edit inside one, is carried duplication and stays silent, while an introduction still denies ([guard.rs:169-199](../../../cli/src/guard.rs#L169)). A split to a NEW file has no replaced content and still denies; its reason teaches the ordering that passes — trim the source first, because candidate verification reads the current tree ([guard.rs:250-251](../../../cli/src/guard.rs#L250)). The baseline probe runs only when the first probe matched, and a degraded baseline subtracts nothing: allowance never rides an unanswered question.

Determinism is bought by *replaying the write* rather than estimating it. `resulting_lines` computes the exact post-write line count — `Write` is its own payload (`content.lines().count()`, [budget.rs:129-132](../../../cli/src/guard/budget.rs#L129)); `Edit` is the payload applied to the on-disk file under Edit's own semantics: CRLF-normalized, and a non-`replace_all` edit requires a **unique** match, matching the real tool's rejection of ambiguity ([budget.rs:134-149](../../../cli/src/guard/budget.rs#L134)). Any case where the tool call would fail on its own — missing file, empty or absent `old_string`, ambiguous match — returns `None` and the rule stays silent ([budget.rs:129-149](../../../cli/src/guard/budget.rs#L129)). The gate never judges a write that will not land.

The same discipline is applied to the *policy* value, not only the measurement. `Guard::tier` is total by construction: an unrecognized `[guard] mode` resolves to an `observe (ce.toml ERROR: unknown guard mode …)` string rather than being passed through verbatim ([tier.rs:56-65](../../../cli/src/config/tier.rs#L56)), because a pass-through typo previously disarmed every enforcement path while `SessionStart` still printed the mode as armed ([tier.rs:47-55](../../../cli/src/config/tier.rs#L47)). A load failure renders through the same single throat, `tier_of` ([tier.rs:71-79](../../../cli/src/config/tier.rs#L71)), so a broken config can never print byte-identically to a deliberate `observe`.

Two honesty boundaries bound the thesis, and both are written into the plan:

- `PreToolUse` is a **behavior-shaping layer, not a security boundary** — an agent can bypass it with `Bash: echo >>` or `sed -i`; the backstop is the `Stop` audit over `git diff` (write-tool agnostic) plus the CI gate ([DEVELOPMENT_PLAN.md:92-94](../../DEVELOPMENT_PLAN.md#L92)).
- The hook is **fail-open**: any internal failure allows the edit, and the degraded run lands in the observe feed ([guard.rs:5-6](../../../cli/src/guard.rs#L5)); a probe that cannot reach the daemon returns `None`, which is logged as `degraded` rather than treated as "no duplicates" ([guard.rs:213-231](../../../cli/src/guard.rs#L213), [guard.rs:297](../../../cli/src/guard.rs#L297)).

### The tier ladder

Four tiers, and nothing else is a tier: `TIERS = ["observe", "warn", "ask", "deny"]` ([tier.rs:40](../../../cli/src/config/tier.rs#L40)). They map onto Claude Code's decision JSON at one emission point ([guard.rs:257-274](../../../cli/src/guard.rs#L257)):

| tier | `permissionDecision` | effect |
|---|---|---|
| `observe` | *(none — return before printing)* | feed line only, no injected text |
| `warn` | `allow` | edit proceeds, reason surfaces as a visible warning |
| `ask` | `ask` | user is prompted |
| `deny` | `deny` | write is refused, reason points at the existing `file:line` |

A mistyped mode falls through the same `match` to no output ([guard.rs:264](../../../cli/src/guard.rs#L264)) — it can never enforce — while the degraded string still rides the observe feed's `mode` field ([guard.rs:282](../../../cli/src/guard.rs#L282), [guard.rs:288-302](../../../cli/src/guard.rs#L288)) and the health line.

Tier resolution is per rule class, not global: an explicit `[guard] mode` overrides every class; otherwise the §4.2 route default for that class applies ([tier.rs:42-48](../../../cli/src/config/tier.rs#L42)). The route default for the two promoted `PreToolUse` classes is a single constant read by both `guard.rs` and `health.rs`, so the enforced tier and the reported tier cannot drift:

```
PROMOTED_DEFAULT = "deny"     // tier.rs:36
```

([tier.rs:36](../../../cli/src/config/tier.rs#L36); consumed at [guard.rs:56](../../../cli/src/guard.rs#L56) and [health.rs:59](../../../cli/src/health.rs#L59)). Everything else routes to `observe` — explicitly because it has no FPR record of its own, which is the entry requirement below ([DEVELOPMENT_PLAN.md:101-103](../../DEVELOPMENT_PLAN.md#L101)).

When several rules fire on one write, the decision line is emitted at the **strongest** tier among them, ranked by index in `TIERS` (unknown values rank 0 = `observe`): class rules carry the class mode, the zone rule carries its own mapped tier, so a zone warn never rides a deny-class escalator ([guard.rs:123-140](../../../cli/src/guard.rs#L123)). A broken `ce.toml` overrides the computed tier down to a visible `warn` that names the parse error ([guard.rs:149-153](../../../cli/src/guard.rs#L149)).

### The FPR gate that promotes a class

The ladder is a **route**, written into the plan so the default can neither stay at `warn` forever nor start at `deny` ([DEVELOPMENT_PLAN.md:96-103](../../DEVELOPMENT_PLAN.md#L96)):

1. 0.x (M3–M4): default `warn`; `deny` exists as an opt-in capability.
2. After the M4 FPR gate passes: **T1/T2 exact duplicate write** and **hard-budget breach (file > 750 lines)** promote to `ask`.
3. 1.0 (M7): those two classes promote to `deny`; every other rule stays `observe` for want of its own FPR record. Each default change records its FPR evidence in the CHANGELOG.

The gate itself is quantitative and appears in two places with two shapes. The M3 acceptance criterion is **≤ 1 mis-block in 500 real normal edits**, with `N=1` demonstrations explicitly disallowed ([DEVELOPMENT_PLAN.md:282](../../DEVELOPMENT_PLAN.md#L282)); the M4 main gate is **FPR ≤ 1% over 500 real normal edits**, on a pre-registered evaluation set frozen before implementation, ≥ 200 edit samples, ≥ 50% from real agent transcripts ([DEVELOPMENT_PLAN.md:283](../../DEVELOPMENT_PLAN.md#L283)). Risk register R4 states the admission rule flatly: **deny admission = the M4 FPR gate (≤ 1%)** ([DEVELOPMENT_PLAN.md:318](../../DEVELOPMENT_PLAN.md#L318)).

Sample purity is part of the gate, not an afterthought: only observe-mode sessions and pre-M3 no-guard sessions may be sampled, edits the guard already intervened in are excluded and the exclusion ratio reported — otherwise FPR is biased downward by the guard's own shaping and the deny admission becomes self-certifying ([DEVELOPMENT_PLAN.md:283](../../DEVELOPMENT_PLAN.md#L283)). This is why every probed event is appended to `<root>/.ce/observe.ndjson` in **all** modes ([guard.rs:7-8](../../../cli/src/guard.rs#L7)) and why the feed carries `session_id`: the evaluation set is partitioned by session, and neither the dogfood-session count nor the purity rule is answerable without it ([envelope.rs:16](../../../cli/src/guard/envelope.rs#L16), [hookio.rs:44-51](../../../cli/src/hookio.rs#L44)).

#### The recorded replay

The M3 replay treats git linear history as a real edit stream: each changed code file in each commit is one "write the child version's content into the parent state" event, materialized incrementally in a shadow directory, **probe first, then apply**; guard ran at default knobs `t = 50 / min_distinct = 7` ([FPR-REPLAY.md:8-13](../../FPR-REPLAY.md#L8)). Those two knobs are the shipped defaults: `t = window + kgram - 1 = 26 + 25 - 1 = 50` tokens, aligned to the jscpd `min-tokens` default ([mod.rs:283-310](../../../cli/src/dedup/mod.rs#L283)), and the diversity floor `7`, calibrated because arbitrated data-row false positives measured `distinct <= 6` while arbitrated true clones measured `>= 7`. Since proto 2.19.0 (batch-7 slice 1) the floor's AUTHORITY is the core's ([CE/Dedup/Cost.hs](../../../core/app/CE/Dedup/Cost.hs)): `ce dedup --check` ships the pre-filter distinct counts, the core re-derives the admitted count and judges the budget from its own number, and Rust's `DEFAULT_MIN_DISTINCT` survives as a declared mirror ([pairs.rs:114-118](../../../cli/src/dedup/pairs.rs#L114)) held equal by three legs — the knob-face pin in core_wire, the mirror declaration, and the per-run `dedupBlocks` ensure in budget.rs. On the `--check` road an override may only tighten those two knobs (`--min-tokens` > 50, `--min-distinct` > 7 or `0` are refused by name — [budget.rs:21](../../../cli/src/dedup/budget.rs#L21)), so the operating point the replay measured is the one the gate judges at.

Result ledger ([FPR-REPLAY.md:17-21](../../FPR-REPLAY.md#L17)):

| corpus | events | blocks | false blocks after arbitration | per-500 |
|---|---|---|---|---|
| requests, 400-commit history tail (pinned at `1f6589ec`) | 487 | 0 | 0 | 0.00 |
| CodeEraser self-repo, full history (real agent edit stream) | 143 | 35 | 0 | 0.00 |
| total | 630 | 35 | **0** | **0.00 ≤ 1** |

The 35 self-repo blocks arbitrated as **true positives**, itemized: 2 were zod locale fixtures that are T2 clones by design, and 33 were `rust_fn(seed)` / `tmp()` / git helpers copy-pasted across 8 test files by the author — remediated in four batches, 251 → 211 → 209 → 205 → 202 clone blocks with the dedup budget stepped down in lockstep ([FPR-REPLAY.md:23-37](../../FPR-REPLAY.md#L23)). The replay also caught a product defect: a candidate file disappearing between index and probe (delete/rename race, triggered by a real directory-rename commit in the requests history) used to error the whole probe, and now degrades to a skip ([FPR-REPLAY.md:39-44](../../FPR-REPLAY.md#L39)).

The K-round re-run (2026-08-26, model arbitration under user delegation) extended this ledger once and forced the semantics fix above: requests replayed 487 events / 0 intercepts again verbatim; the self-repo's full 2274-event history raised 505 intercepts under the factory whole-content semantics — decomposed into 90 genuine introductions (child-state re-verified pair by pair with today's detector), 32 split/fold mid-states (the write-first ordering of exactly the extract-to-a-leaf refactors this repo's own 300-line discipline produces), and 383 re-fires on files carrying budgeted blocks. Counted as mis-blocks the 32 read 1.41% > the M4 gate, and the ruling was to fix the semantics rather than demote the tier; under the fixed semantics the raw count falls to 139, and the gate arithmetic is recorded in both honest framings — 7.03/500 under the replay's whole-write model, 0.00/500 under the live flow (fragment-probed Edits plus the taught safe ordering; the production feed shows 719 probes, 9 fires, none of this class) ([FPR-REPLAY.md:54-90](../../FPR-REPLAY.md#L54)).

Stated boundary: the replay instrument `cli/tests/fpr_replay.rs` was retired with the M7.5 slimming pass and re-retired after the K-round re-run; this ledger is the record, and re-verification resurrects the instrument from git history with the two recorded same-generation shims ([FPR-REPLAY.md:3-6](../../FPR-REPLAY.md#L3), [FPR-REPLAY.md:98-154](../../FPR-REPLAY.md#L98)).

### The hard-budget rule

Scope is decided **before** any disk read — language arm, then exclude walk — so the hook never pays an unbounded read for a file it is about to declare out of scope ([budget.rs:45-59](../../../cli/src/guard/budget.rs#L45)). The rule then fires iff:

```
cap = lines_for(file).file_lines_fail     // the file's class table; 750 for class 0
breach  ⇔  cap != 0 ∧ lines > cap
```

([budget.rs:72-78](../../../cli/src/guard/budget.rs#L72); default `file_lines_fail = 750`, [ce-toml.md:18](../ce-toml.md#L18)). Since the rulepack's hook slice (plan v2.13 ① P4) the line is the **file's own**: `lines_for` compiles the declared `[[rules.class]]` set once per hook run and takes the class's effective table, the global one for class 0 — the same reading the scan gate and the score take, so the hook denies at exactly the line the CI wall would fail at ([budget.rs:19-25](../../../cli/src/guard/budget.rs#L19)); the graded zone reads its H and its warn fallback from the same table. The `cap == 0` guard is load-bearing: it encodes "no hard line exists" per the P3 grade-table contract, where a naive comparison read `0` as "every write breaches" ([budget.rs:72-74](../../../cli/src/guard/budget.rs#L72)). Since 6.4.0 the table the hook reads is the FENCED one: with a committed baseline whose digest differs from the declared config's — or one that cannot be read — `thresholds`, `exclude` and the classes are taken at their shipped values and the deny reason names the fence, because a drifted config is an unverified document and a hook that kept any part of it would let the edit that produced the drift choose its own budget; `[guard]` mode stays as declared, a mode being a visible act rather than a budget ([budget.rs:98-120](../../../cli/src/guard/budget.rs#L98)). Every firing gets its own `budget` feed line in **every** tier, because the step-3 decision at 1.0 needs per-rule records ([budget.rs:153-166](../../../cli/src/guard/budget.rs#L153)) — the feed's `budget` event was added in schema `0.4.0` for exactly that reason ([hookio.rs:41-42](../../../cli/src/hookio.rs#L41)).

### `zone_tiers`: the opt-in position-to-tier map

Below the hard line, a write landing in the graded zone `(S, H]` is scored by position. Position is integer per-mille:

```
S = committed baseline softLine, else the file's file_lines_warn    // its class's, else the global 300
H = the file's file_lines_fail                                      // its class's, else the global 750
permille = (lines - S) * 1000 / (H - S)
```

([budget.rs:188-193](../../../cli/src/guard/budget.rs#L188); `S` is read off the committed `ce-baseline.json` via `committed_soft`, [budget.rs:259-264](../../../cli/src/guard/budget.rs#L259); `file_lines_warn = 300`, [ce-toml.md:17](../ce-toml.md#L17)). A degenerate zone — `H == 0` (no hard line), `H <= S`, or `lines <= S` — logs nothing rather than fabricating a position ([budget.rs:190-192](../../../cli/src/guard/budget.rs#L190)).

The map is a three-way partition on that per-mille ([budget.rs:232-241](../../../cli/src/guard/budget.rs#L232)):

```
0   ..= 249  →  observe
250 ..= 750  →  warn
751 ..       →  ask
```

At the default `S = 300`, `H = 750` (so `H - S = 450`), integer division puts the boundaries at `lines >= 413` for `warn` and `lines >= 638` for `ask` — derived from the constants above, not stated in the source.

The switch is **off by default** and changes one repo only: `[guard] zone_tiers`, default `false` ([tier.rs:24-28](../../../cli/src/config/tier.rs#L24), [ce-toml.md:31](../ce-toml.md#L31)). Disarmed, the zone is feed-only — a `zone` line is written in every tier and `zone_assess` returns `None`, injecting nothing ([budget.rs:210-216](../../../cli/src/guard/budget.rs#L210)). Armed, the resolved tier is additionally stamped as `zone_tier` on the feed line, so the record says what the rule *did*, not only where the write landed ([budget.rs:210-212](../../../cli/src/guard/budget.rs#L210), [hookio.rs:31-34](../../../cli/src/hookio.rs#L31)).

The default stays `observe` for one reason, written at the declaration site: §4.2's FPR discipline gates any default flip on the zone feed's own record ([tier.rs:24-28](../../../cli/src/config/tier.rs#L24)), and the `zone` event exists precisely as "the per-rule record any future zone→tier promotion must argue its FPR case from" ([hookio.rs:36-39](../../../cli/src/hookio.rs#L36)). The rule class has no FPR ledger yet, therefore it does not enforce by default — the same admission test the two promoted classes had to pass.

### Injection budget: warns are rate-limited, enforcement is not

An anti-bloat tool must not itself become a context entropy source (§4.4 B4). Two mechanisms:

- **Per-(rule, file, session) suppression.** A warn fires once per session per file per rule; the observe feed *is* the accumulator, consulted **before** the current event lands in it so a warn cannot read its own fresh line as "already warned" ([guard.rs:63-70](../../../cli/src/guard.rs#L63), [budget.rs:197-200](../../../cli/src/guard/budget.rs#L197), [hookio.rs:208-224](../../../cli/src/hookio.rs#L208)). Suppression is skipped entirely when `mode ∈ {deny, ask}` — enforcement is not context bloat and repeats every time it holds ([guard.rs:65-67](../../../cli/src/guard.rs#L65)); an armed zone `ask` likewise repeats, only `warn` is budgeted ([budget.rs:200](../../../cli/src/guard/budget.rs#L200)). Counting is conservative in the reporting direction: a clean probe line (`matches == 0`) and an unarmed zone line (`zone_tier` absent or `observe`) never count as "already warned", and any read or parse failure returns "not warned" — fail open toward *reporting*, the opposite bias from enforcement's fail-open ([hookio.rs:220-231](../../../cli/src/hookio.rs#L220)).
- **Token clip at the single emission throat.** `WARN_BUDGET_TOKENS = 200`, `STOP_BUDGET_TOKENS = 400`, converted to chars by one declared measurement constant `CHARS_PER_TOKEN = 4` ([hookio.rs:159-161](../../../cli/src/hookio.rs#L159)); the clip is char-boundary safe and appends `… (clipped; full report in .ce/observe.ndjson)`, pointing at the full on-disk record ([hookio.rs:163-176](../../../cli/src/hookio.rs#L163), applied at [guard.rs:158-163](../../../cli/src/guard.rs#L158)).

The feed schema is a versioned contract pinned by a golden fixture, bumped on any shape change — and on the one recorded SEMANTIC break: <!--ce:report:observe#schemaver-->`ce.observe/0.7.0`<!--/ce--> ([hookio.rs:52](../../../cli/src/hookio.rs#L52)) re-defines `matches` on probe events as the NOVEL count (the subtraction above), because pre/post counts are not comparable as FPR raw material; the `zone_tier` key arrived at `0.6.0`, the `zone` event at `0.5.0`, the `budget` event at `0.4.0`, and `session_id` at `0.2.0` ([hookio.rs:18-51](../../../cli/src/hookio.rs#L18)). That versioning is what makes the FPR ledger replayable across releases rather than a claim about a build nobody can reconstruct.
