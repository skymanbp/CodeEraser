# Tombstone residue — the erased-name conjunction

[index](../methodology.md) · [← 13 Unmentioned-declaration advisory — the mention veto](13-unmentioned-declaration-advisory.md)

Every family before this one reads a tree. This one reads a **change**: the pairs
`(before, after)` one edit, one session or one commit wrote, and asks whether a name the
change removed survives in the same change as its own absence — a heading
`Tomato and Egg (no Dongpo Pork)`, an identifier `cook_without_dongpo`, a sentence
`braise_dongpo_pork is no longer needed`. The database tombstone — the marker a deleted
record leaves in its place — gives the class its name
([mod.rs:1-7](../../../cli/src/tombstone/mod.rs#L1)). The plan calls it the third rule
class, the v2.26 T track, and admits it on the same terms as the other two: no default
tier above observe until the FPR ledger argues one
([DEVELOPMENT_PLAN.md:103](../../DEVELOPMENT_PLAN.md#L103),
[DEVELOPMENT_PLAN.md:125](../../DEVELOPMENT_PLAN.md#L125)). The split is ADR-008's, fifth
instalment: Rust measures three integers per candidate surface — its kind, the
retrospective marks in it, the erased names it binds — and Haskell decides which rows are
sites and whether the changeset is over its declared budget
([Cost.hs:1-7](../../../core/app/CE/Tombstone/Cost.hs#L1)). Names and paths never cross
the wire, row index is identity, and every number lands in the observe feed beside the
judgment it received ([mod.rs:16-28](../../../cli/src/tombstone/mod.rs#L16)).

### 1. The changeset — what one leg hands the measurement

A changeset is a list of pairs `(rel, before, after, lang)`; an absent side is the empty
text ([mod.rs:51-58](../../../cli/src/tombstone/mod.rs#L51)). Three legs build one:

- **PreToolUse** — this Write/Edit's on-disk pair, the applied text the budget rule already
  computes, for a judged language inside the config's walk
  ([guard/tombstone.rs:21-38](../../../cli/src/guard/tombstone.rs#L21)). It is the only leg
  that sees one file at a time, so it carries the session with it (§7).
- **Stop** — the working tree against `HEAD`, plus the audit's untracked files: a name that
  moved into a brand-new file is alive, and a new CHANGELOG is a changelog
  ([audit/tombstone.rs:1-7](../../../cli/src/audit/tombstone.rs#L1),
  [audit/tombstone.rs:116-123](../../../cli/src/audit/tombstone.rs#L116)).
- **precommit / commitmsg** — the index against `HEAD`, what the commit will hold; `ce
  commitmsg` adds the message as one more Markdown after-surface named `COMMIT_EDITMSG`, its
  comment lines blanked in place so a site's line is the file's own
  ([audit/tombstone.rs:25-27](../../../cli/src/audit/tombstone.rs#L25),
  [audit/tombstone.rs:134-140](../../../cli/src/audit/tombstone.rs#L134),
  [commitmsg.rs:62-67](../../../cli/src/audit/commitmsg.rs#L62)).

Every git side comes through ONE `cat-file --batch` process and the working tree through one
bounded read: a blob that is missing, binary or past `READ_CAP` drops its pair, pairs past
`PAIR_CAP` are never read, and both counts return to the caller
([texts.rs:1-8](../../../cli/src/tombstone/texts.rs#L1),
[texts.rs:14-23](../../../cli/src/tombstone/texts.rs#L14)). The FPR replay walks a whole
history through this same door (§9).

Two surfaces of the after side are read, and only what the change ADDED — both take the
added-line set off the four-classification's own line diff, so a paragraph that already
existed is never re-judged ([surfaces.rs:1-7](../../../cli/src/tombstone/surfaces.rs#L1),
[surfaces.rs:47-55](../../../cli/src/tombstone/surfaces.rs#L47)):

- **S⁺, the naming surface** — headings this change added (Markdown), unit names the after
  side declares and the before side did not (code), the stem of a brand-new file
  ([surfaces.rs:57-77](../../../cli/src/tombstone/surfaces.rs#L57)).
- **P⁺, the prose surface** — the added lines of every comment, docstring and paragraph
  segment docdup extracts, read raw (no mask: the name a paragraph mentions sits in
  backticks, and masking the span would hide exactly the mention the conjunction needs) and
  with no admission floor — a one-line `X is no longer needed` is a whole tombstone. Fenced
  and indented code never become segments, so an example's `(no X)` stays an example
  ([surfaces.rs:9-12](../../../cli/src/tombstone/surfaces.rs#L9),
  [surfaces.rs:111-122](../../../cli/src/tombstone/surfaces.rs#L111)).

### 2. R — the names a change erased

A NAME is a spelling a **structural position** of a text declares — a code line's identifier
outside every comment segment and every literal, a declared unit's name, a Markdown heading
as rendered, a list item's lead. An inline code span only MENTIONS: it keeps a name alive and
declares none — the third self-replay round said why, when a 5,000-character narrative line
rewritten in place dropped its own spans and re-mentioned them, and nothing had been removed
([marked.rs:1-9](../../../cli/src/tombstone/marked.rs#L1),
[marked.rs:37-43](../../../cli/src/tombstone/marked.rs#L37),
[marked.rs:119-124](../../../cli/src/tombstone/marked.rs#L119)). Literals are blanked to
spaces before the identifiers are read: the fifth round had bound `independent` out of a
caveat message and `linux` out of a cfg string
([marked.rs:70-84](../../../cli/src/tombstone/marked.rs#L70)).

Each marked text offers every **window** of 1..=`JOIN_MAX` adjacent words — the word cut
lower-cases ASCII, splits at `_`, `-`, any non-alphanumeric and a camel rise, and keeps a
non-ASCII run whole — plus each run wider than a window, so `braise_dongpo_pork` names
`dongpo_pork` too, which is what `(no Dongpo Pork)` binds
([frames.rs:12-15](../../../cli/src/tombstone/frames.rs#L12),
[frames.rs:128-134](../../../cli/src/tombstone/frames.rs#L128),
[names.rs:53-67](../../../cli/src/tombstone/names.rs#L53)). A spelling is keyed by its
canonical form — words `_`-joined — through the dedup family's fnv1a, so `DongpoPork`,
`dongpo_pork` and `Dongpo Pork` are one name, and keys are all the hub compares and all the
feed stores ([names.rs:42-51](../../../cli/src/tombstone/names.rs#L42)).

The **name floor** admits a spelling that has a letter, is at least `MIN_ASCII_NAME`
characters (`MIN_WIDE_NAME` for a CJK one — `ab` is a preposition, not a name), has no word
of the instrument's own vocabulary among its words (a frame, an absence word, a function
word, a word of a retrospective mark) nor of the repository's own `[tombstone] terms`, and is
not made of reserved words alone — `user_data` is a name, `data` is not
([names.rs:82-95](../../../cli/src/tombstone/names.rs#L82),
[vocab.rs:14-19](../../../cli/src/tombstone/vocab.rs#L14),
[vocab.rs:79-90](../../../cli/src/tombstone/vocab.rs#L79)). A text that is an absence word
WHOLE (`NotFound`, `no_std`) spells no name at all: its `found` half must not enter R just
because the word is compound ([names.rs:119-123](../../../cli/src/tombstone/names.rs#L119),
[vocab.rs:31-35](../../../cli/src/tombstone/vocab.rs#L31)).

ERASED means: a name of some before side that SURVIVES on no after side. A name survives in
every marked text this change did not add — a code span included — and in a structural one
it did add, outside the slots an absence frame binds there; a mention this change wrote into
prose or a code span is not survival, because that is where residue is written
([names.rs:11-16](../../../cli/src/tombstone/names.rs#L11),
[names.rs:142-154](../../../cli/src/tombstone/names.rs#L142)). So a name moved to another
changed file is alive, and a name that only recurs inside `(no X)` is erased
([names.rs:178-200](../../../cli/src/tombstone/names.rs#L178)):

    R = ⋃ᵢ names(beforeᵢ)  \  ⋃ᵢ alive(afterᵢ, addedᵢ)          (compared as keys)

### 3. The label rule — absence frames

A label fires when an absence frame in it binds an erased name. The frames are four tables —
English prefixes (`no`, `not`, `non`, `without`, `sans`, `minus`), English suffixes (`free`,
`less`, `removed`, `dropped`, `gone`), and the Chinese forms, read inside one run
(`无东坡肉`) or as a word before an ASCII name (`无cache`)
([vocab.rs:50-58](../../../cli/src/tombstone/vocab.rs#L50)). A prefix binds the
1..=`JOIN_MAX` words after it (`no more` counts as one prefix), a suffix the words before it,
a Chinese form the rest of its own run; the candidates are spellings, the hub keys them and
asks the erased set — a candidate that names nothing erased is nothing
([frames.rs:170-201](../../../cli/src/tombstone/frames.rs#L170),
[frames.rs:212-222](../../../cli/src/tombstone/frames.rs#L212)). A frame inside a bracket
pair is kind 0, `bracketed`; a bare one is kind 1 — the two label kinds the wire carries and
the feed names ([frames.rs:203-210](../../../cli/src/tombstone/frames.rs#L203),
[mod.rs:60-67](../../../cli/src/tombstone/mod.rs#L60)). The label row's `names` is how many
bound candidates are known — erased by this changeset, or by an earlier edit of the session
([mod.rs:175-207](../../../cli/src/tombstone/mod.rs#L175)).

The frame reading is **symmetric**: the name side and the survival side read a text through
the same door, so a window an absence frame binds is neither erased nor alive —
`fn no_return_chars` declares nothing named `return_chars` on either side of a change. The
second self-replay round found the asymmetric form: a moved `def test_header_no_return_chars`
read as "removed `return_chars` and wrote it back", the whole source of the requests corpus's
two false sites ([names.rs:97-117](../../../cli/src/tombstone/names.rs#L97),
[FPR-TOMBSTONE.md:26](../../FPR-TOMBSTONE.md#L26)).

### 4. The prose conjunction — per sentence

A prose surface is cut into sentences — after `.`, `!`, `?`, `;` when whitespace or the end
follows (so `ce.toml` and `a.rs` stay whole) and after any full-width `。！？；` — and the
conjunction is read per sentence: the fourth self-replay round had bound a name mentioned
3,000 characters away from its mark on one 5,000-character line
([frames.rs:224-248](../../../cli/src/tombstone/frames.rs#L224)). Each sentence yields one
row when it carries a retrospective mark or an erased name at all: `marks` counts every
English phrase of the mark table at word boundaries (`previously` must not match inside
`previously_seen`) and every Chinese one by substring, overlapping phrases both counting — the
number is the floor's input, not a tally; `names` counts the known names it spells, every
window once per key, plus every wide erased name it contains as a substring, since a Chinese
sentence is one word ([frames.rs:250-277](../../../cli/src/tombstone/frames.rs#L250),
[vocab.rs:60-66](../../../cli/src/tombstone/vocab.rs#L60),
[names.rs:69-80](../../../cli/src/tombstone/names.rs#L69),
[names.rs:167-176](../../../cli/src/tombstone/names.rs#L167),
[mod.rs:208-227](../../../cli/src/tombstone/mod.rs#L208)). Which of those rows is a site is
not decided here: a mark alone is a sentence about something else, a name alone is a mention
or a migration guide, and the conjunction that makes a tombstone is the core's (§6).

### 5. Exemptions — the changelog role, three witnesses and a declaration

A document whose JOB is to narrate change — a changelog, release notes, a migration guide, a
decision record — is exempt as a whole, and every exemption is counted where the reader can
see it (the docdup ledger discipline: never silent). Only Markdown can hold the role: a code
file narrates nothing by job ([role.rs:1-16](../../../cli/src/tombstone/role.rs#L1)). Four
witnesses, the first that answers deciding
([mod.rs:143-173](../../../cli/src/tombstone/mod.rs#L143)):

| witness | reads | exempts | feed `why` |
|---|---|---|---|
| **declared** | `[tombstone] ledger`, the exclude list's dialect, compiled once per run | the file, whatever its language | `declared` ([policy.rs:1-6](../../../cli/src/tombstone/policy.rs#L1), [policy.rs:36-40](../../../cli/src/tombstone/policy.rs#L36)) |
| **path** | the stem (`changelog`, `news`, `history`, `migration`, `upgrading`, …, `adr-*`) or a directory on the path (`adr`, `decisions`, `releases`, …) | the file | `path` ([role.rs:47-102](../../../cli/src/tombstone/role.rs#L47)) |
| **ledger** | the SHAPE: at least three headings, at least half of the level-2/3 ones version-indexed — a `d.d` semver, an ISO date, `Unreleased` / `未发布` | the file | `ledger` ([role.rs:104-137](../../../cli/src/tombstone/role.rs#L104)) |
| **segment** | the segment around a candidate row — the `>` quote run when the line is quoted (a banner is one run), else the section body from the nearest heading at or above down to the next heading of any level — and its distinct ledger tokens | that segment only, counted once per segment | `segment` ([role.rs:173-191](../../../cli/src/tombstone/role.rs#L173), [mod.rs:242-273](../../../cli/src/tombstone/mod.rs#L242)) |

The third witness exists because a file-level answer had to get one of two things wrong: the
plan book's banner is a ledger and its §4 is a norm. Its threshold is `SEGMENT_TOKENS` = 3
distinct ledger tokens — a three-part semver or a `v`-prefixed two-part one (the `v` dropped
from the key, so one version spelled both ways is one token), an ISO date, a 7–40 hex commit
carrying both a digit and a letter; `§4.2`, `0.57`, a `file:102-103` span and a run number are
none of these. Three is where a list becomes a ledger — the floor `ledger_shape` already
applies to headings — and the replay that set it left a window of [1, 33]: every true
positive's segment carried 0 tokens and the three in-between sites' segments 33, 75 and 77
(§9; [role.rs:162-169](../../../cli/src/tombstone/role.rs#L162),
[role.rs:234-256](../../../cli/src/tombstone/role.rs#L234)). `[tombstone] ledger` is the
backstop the 2026-09-04 ruling gave the witness — ledger-like files no witness reads, named
by the repository itself — and `[tombstone] terms` is the same table's vocabulary key: words
that never spell a name, whole or as a word of a compound
([config/tombstone.rs:1-18](../../../cli/src/config/tombstone.rs#L1),
[ce-toml.md:85-92](../ce-toml.md#L85)). An exemption enters the feed only when the changeset
erased something (a file by role or declaration) or when it suppressed a row (a segment): an
exemption that suppressed nothing is nothing to see
([mod.rs:113-134](../../../cli/src/tombstone/mod.rs#L113)).

### 6. The wire — `tombstone/1`, and what Haskell judges

Rust sends one row per candidate surface, `[kind, marks, erasedNames]`, in measurement order,
and the budget as knob `0` when one is declared; row index is identity on the wire, and this
side re-labels the indices it gets back into `file:line kind` places
([wire.rs:1-7](../../../cli/src/tombstone/wire.rs#L1),
[wire.rs:31-45](../../../cli/src/tombstone/wire.rs#L31)). A core that does not offer the
capability — every core before 6.6.0 — is healthy and answers nothing here, a named
non-judgment like a degraded reply or a site index beyond the rows sent: no failure is ever
read as "no sites" ([wire.rs:47-81](../../../cli/src/tombstone/wire.rs#L47),
[corelink.rs:36](../../../cli/src/corelink.rs#L36)). The core registers the family as the
eleventh ([Protocol.hs:102](../../../core/app/CE/Protocol.hs#L102)) and answers it through
the knobbed-table cascade `trend/2` minted: rows and knob rows count against one cap
together, the first malformed row in request order is the offence, else the first malformed
knob ([Wire.hs:112-137](../../../core/app/CE/Wire.hs#L112),
[Tombstone.hs:29-53](../../../core/app/CE/Tombstone.hs#L29)). The judgment is three lines
([Cost.hs:47-58](../../../core/app/CE/Tombstone/Cost.hs#L47),
[Tombstone.hs:55-61](../../../core/app/CE/Tombstone.hs#L55)):

    site(kind, marks, names)  ⇔  names ≥ minName  ∧  (kind ≠ kindProse  ∨  marks ≥ minMarks)
    label = |{sites : kind ≠ kindProse}|          prose = |{sites : kind = kindProse}|
    over  ⇔  budget declared  ∧  |sites| > budget            (strict; no budget, no condition)

with `minName` = 1, `minMarks` = 1, `kindProse` = 2 and `tombstoneRowCap` = 65536 — far above
any honest changeset; over the cap the core answers a complete degraded reply with an empty
site table, the condition unevaluated and the reason `tombstone_too_large`: a changeset the
core refused to judge is neither convicted nor cleared
([Cost.hs:18-45](../../../core/app/CE/Tombstone/Cost.hs#L18),
[Tombstone.hs:63-89](../../../core/app/CE/Tombstone.hs#L63)). The reply carries the site
indices in request order, the `rows` / `label` / `prose` counts, `over`, the effective knob
table echoed (empty = no budget declared) and `degraded`. Loosening either floor re-opens the
replay rounds that measured the conjunction's precision (§9): the floors are the contract,
not tuning ([Cost.hs:29-35](../../../core/app/CE/Tombstone/Cost.hs#L29)).

### 7. Three legs, one tier, one feed

The class speaks at its OWN tier, `[tombstone] tier`: `[guard] mode` does not reach it — a
class with a key of its own decides at that key, the graded zone's precedent — and it ships
at `observe` until the FPR ledger argues a promotion, §4.2's route discipline spelled as the
default. A tier outside the four is refused by name at load; every key of the table is a knob
of the canonical form, so spelled at its default it is silence and spelled elsewhere it moves
`knobs_digest` ([config/tombstone.rs:1-29](../../../cli/src/config/tombstone.rs#L1),
[config/tombstone.rs:41-59](../../../cli/src/config/tombstone.rs#L41)).

- **PreToolUse** measures the pair against R ∪ the keys every earlier `tombstone` line of the
  same session recorded — the observe feed IS the accumulator, as it is for the warn
  suppression — so an X deleted three edits ago still binds the heading written now. A
  `tombstone` line lands only when there is something to judge (this edit erased a name, or a
  surface bound a name the session erased); the judgment travels over the daemon's core link
  as rows and budget only; the hook speaks only when its tier is not `observe`, a budget is
  declared and the core said `over`
  ([guard/tombstone.rs:1-12](../../../cli/src/guard/tombstone.rs#L1),
  [guard/tombstone.rs:53-68](../../../cli/src/guard/tombstone.rs#L53),
  [guard/tombstone.rs:98-112](../../../cli/src/guard/tombstone.rs#L98),
  [hookio.rs:237-253](../../../cli/src/hookio.rs#L237),
  [proto.rs:60-67](../../../cli/src/daemon/proto.rs#L60),
  [say.rs:68-79](../../../cli/src/guard/say.rs#L68)).
- **Stop / precommit / commitmsg** measure the whole changeset with an empty session (the
  Stop sees the session's diff at once), judge over the audit's own core link, and block only
  when two bits agree — the tier is `deny` AND the core's `over`; no core is a degraded object,
  never a block and never a silent pass. The reason names who judged what, the count, the
  budget it passed, the first sites, and what to do instead — drop the label, or say what
  replaced it; the terminal faces print one line for the person: the sites when there are
  any, the degradation when there is no verdict, nothing when the changeset is clean
  ([audit/tombstone.rs:56-61](../../../cli/src/audit/tombstone.rs#L56),
  [audit/tombstone.rs:143-161](../../../cli/src/audit/tombstone.rs#L143),
  [audit/tombstone.rs:185-212](../../../cli/src/audit/tombstone.rs#L185),
  [precommit.rs:22-61](../../../cli/src/audit/precommit.rs#L22)). `ce commitmsg` exits 2
  when it cannot read the file it was handed — a gate that cannot see its input must say so —
  1 on a block, 0 otherwise ([commitmsg.rs:14-35](../../../cli/src/audit/commitmsg.rs#L14)).

Every producer writes ONE feed shape, the `tombstone` object of `ce.observe/0.9.0`: `rev`
(the vocabulary revision — a reader of the ledger must know which tables produced a row),
`erased`, `rows`, every exemption with its witness (`line` for a segment), and `judged` — the
first `SITE_CAP` sites as `file:line kind`, the label / prose split and `over` — or
`judged.degraded` naming why there is none. The per-edit leg adds the erased keys, capped at
`HASH_CAP`, and the session union's size; the git-hook faces write it with `session_id` null.
No name text is ever written ([feed.rs:1-4](../../../cli/src/tombstone/feed.rs#L1),
[feed.rs:8-59](../../../cli/src/tombstone/feed.rs#L8),
[hookio.rs:23-33](../../../cli/src/hookio.rs#L23),
[hookio.rs:69](../../../cli/src/hookio.rs#L69)). The feed is the FPR ledger's raw material
and the evaluation set's; its shape is pinned by the observe golden (§9).

### 8. Residual risks, stated

- **Single-word names count.** `legacy`, `truncate`, `nul`, `mentioned` are all above the
  three-character floor and in no table; four of the six true positives the replay found bound
  a single word. The 2026-09-04 ruling keeps them and keeps the floor — a false site on a
  common word is a `[tombstone] terms` entry away, by the repository's own word
  ([FPR-TOMBSTONE.md:94-100](../../FPR-TOMBSTONE.md#L94)).
- **CJK is measured at word boundaries only.** A Chinese phrase is one token of the word cut —
  the segmentation limit the spec states rather than solves — so a wide name is spelled only
  where a run is a heading, a list lead or an identifier by itself, survives only as a whole
  word, and is bound in prose by substring; a Chinese frame is read inside one run or before
  an ASCII name ([frames.rs:12-15](../../../cli/src/tombstone/frames.rs#L12),
  [names.rs:37-39](../../../cli/src/tombstone/names.rs#L37),
  [DEVELOPMENT_PLAN.md:125](../../DEVELOPMENT_PLAN.md#L125)).
- **The session union never forgets within a session.** A name erased in one edit and
  re-declared in a later one stays in the union until the session ends, so a still-later
  `(no X)` binds it at PreToolUse; the Stop leg, which reads the whole session's diff at once
  with an empty union, sees the move as survival and seats no site
  ([guard/tombstone.rs:98-112](../../../cli/src/guard/tombstone.rs#L98),
  [audit/tombstone.rs:134-140](../../../cli/src/audit/tombstone.rs#L134)).
- **Only judged languages are measured.** A pair whose after path is not a judged language is
  dropped before any text is read, and the prose surface is whatever docdup extracts segments
  for; a scan-only file can hold a tombstone this class never sees
  ([texts.rs:52-60](../../../cli/src/tombstone/texts.rs#L52),
  [guard/tombstone.rs:31-35](../../../cli/src/guard/tombstone.rs#L31)).
- **Bounded reads under-count, never over-count.** Pairs past `PAIR_CAP` and blobs past
  `READ_CAP` are counted back and not measured; a name erased in an unread pair cannot bind
  ([texts.rs:14-23](../../../cli/src/tombstone/texts.rs#L14)).
- **Intent is not read.** A frame is a frame: `no_std` is an absence word whole and spells
  nothing, but a genuinely new `(no cache)` heading written in the same change that removed a
  `cache` module is a site by construction, and the way out is the reason's — say what
  replaced it. `ce:allow(tombstone)` is deliberately unwired: a residue class that a pragma in
  the residue itself could wave through would measure nothing
  ([tombstone_guard.rs:224-236](../../../cli/tests/it/tombstone_guard.rs#L224)).
- **The message is Markdown by fiat**, and its comment prefix is the repository's
  `core.commentChar` / `core.commentString`, the last one set winning; `auto` reads as `#`,
  and the lines git would have picked another character for are measured as the prose they
  look like ([commitmsg.rs:37-57](../../../cli/src/audit/commitmsg.rs#L37)).

### 9. Acceptance

**The FPR replay is the class's admission ticket** ([FPR-TOMBSTONE.md](../../FPR-TOMBSTONE.md)).
`tombstone_replay` walks a git first-parent history as an edit stream — every commit one
changeset, parent blob before and child blob after, the same `measure` the hooks run, the same
`tombstone/1` judgment over one core link — and prints every seated site with the name it bound
and an excerpt, for arbitration
([tombstone_replay.rs:1-12](../../../cli/tests/it/tombstone_replay.rs#L1)). Two corpora: the
last 400 commits of `requests` and this repository's whole history. Seven rounds, each fixing
one class of DEFINITION defect and re-running in full — never a threshold: the prose surface
read only added lines (round 1: 123 → 68 hit commits on self), the framed window admitted on
neither side (round 2, which also emptied requests: 1 → 0 hit commits), inline code spans
keeping alive but declaring nothing (round 3: 64), the conjunction per sentence (round 4: 11),
literals declaring nothing (round 5: 7), arbitration (round 6: 7 commits / 9 sites = 6 true
positives, 3 in-between sites all on the plan book's version banner, 0 false), and the segment
witness (round 7: 4 / 6). The gate is ≤ 1 % of events: requests 0 / 400 from round 3 on; self
3 / 530 = 0.57 % at round 6 under the ruling's reading (provenance narrative counts as
residue, the in-between sites are counted as false), 7 / 530 = 1.32 % if the true positives
are counted against it too — stated in the ledger as such; round 7: 0 / 531 strict, 4 / 531 =
0.75 % conservative, both under the line ([FPR-TOMBSTONE.md:23-31](../../FPR-TOMBSTONE.md#L23),
[FPR-TOMBSTONE.md:44-54](../../FPR-TOMBSTONE.md#L44),
[FPR-TOMBSTONE.md:81-87](../../FPR-TOMBSTONE.md#L81)). The K = 3 derivation is the ledger's
own table (§5). The replay stays a standing `#[ignore]` leg under the EVAL-SET retirement
rule; its command line is the ledger's last section.

**The core's contract** is six probes: the conjunction's truth table; a mixed request naming
its sites, splitting them and judging the budget; no budget, no condition; the budget boundary
strict; every refusal naming its offender (`row 0: kind outside 0..2`, `knob 0: unknown knob
code`, …); an over-cap request degrading to an empty table with the condition unevaluated
([TombstoneProps.hs:31-36](../../../core/test/TombstoneProps.hs#L31)).

**The measurement's contract** is pinned per module in the tests submodule: the vocabulary
(every table lower-cased, unique, disjoint from the reserved words, the floors nesting —
[unit/tombstone/vocab.rs](../../../cli/tests/unit/tombstone/vocab.rs)), the word cut and the
frames ([unit/tombstone/frames.rs](../../../cli/tests/unit/tombstone/frames.rs)), the name
floor and survival — a move is not an erasure, a name that recurs only inside a frame is, a
framed window is no name on either side, a wide name survives only as a whole word
([unit/tombstone/names.rs](../../../cli/tests/unit/tombstone/names.rs)) — the surfaces
([unit/tombstone/surfaces.rs](../../../cli/tests/unit/tombstone/surfaces.rs)), the four
witnesses ([unit/tombstone/role.rs](../../../cli/tests/unit/tombstone/role.rs)), the text
loader ([unit/tombstone/texts.rs](../../../cli/tests/unit/tombstone/texts.rs)), the wire rows
and every named non-judgment ([unit/tombstone/wire.rs](../../../cli/tests/unit/tombstone/wire.rs)),
and the hub's rows, exemptions and feed object
([unit/tombstone.rs](../../../cli/tests/unit/tombstone.rs)). End to end, each leg has its own
battery — the PreToolUse leg seating a bracketed, a bare and a prose site, a name erased edits
ago still binding, nothing erased meaning no line at all, the declared tier and budget deciding
while `observe` stays silent ([tombstone_guard.rs](../../../cli/tests/it/tombstone_guard.rs));
the Stop and precommit legs judging one site across two files, exempting and counting a
changelog, a ledger segment and a declared ledger, keeping a moved name alive, blocking at
`deny` over budget and saying `over` in the feed, and erasing nothing on an unborn HEAD
([tombstone_audit.rs](../../../cli/tests/it/tombstone_audit.rs)); the commit-msg face
measuring the message and not its comment lines, refusing at `deny` with the message's line
named, reading the repository's own comment prefix, and exiting 2 on an unreadable file
([tombstone_commitmsg.rs](../../../cli/tests/it/tombstone_commitmsg.rs)). The feed shape is
the observe golden's, `tombstone`, `precommit` and `commitmsg` entries included
([feed.golden.json](../../../contracts/fixtures/observe-feed/feed.golden.json)); the Stop audit
and the git-hook faces are one row of the three-face parity table
([face_parity.rs](../../../cli/tests/it/face_parity.rs)).
