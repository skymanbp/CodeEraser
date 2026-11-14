# codeeraser plugin

English | **[中文](README.zh.md)**

A passive guard of <!--ce:count:hooks#word-->four<!--/ce--> hooks (all fail-open: an internal failure always lets the write through) + <!--ce:count:skills#word-->one<!--/ce--> active skill + <!--ce:count:commands#word-->one<!--/ce--> command:

| hook | command | behaviour |
|---|---|---|
| SessionStart | `ce health --hook` | the health line (version / guard tier / index / daemon) + daemon warm-up; when a newer release exists, a second line says so (the check is cached for a day, `CE_UPDATE_CHECK=0` turns it off, no network means no line) |
| PreToolUse (Write\|Edit) | `ce probe --hook` | two classes on the content about to be written: the T1/T2 duplicate probe and the hard budget (the write leaves the file past its hard line and longer than before; a write that does not lengthen it is recorded, not refused), decided by `ce.toml [guard] mode`; a write that stays under the hard line but lands between the soft line and the hard line is recorded by the graded-zone observer, and speaks by where it lands only once `[guard] zone_tiers` is on; the tombstone class is judged in the core by its own `[tombstone] tier` and budget and speaks only when the core answers over; the intra-function dead-code class speaks by its own `[flow] tier` (observe as shipped) only when this write brings a new judged finding |
| PostToolUse (Write\|Edit) | `ce settle --hook` | records, never speaks: when this session's PreToolUse answered `ask` for the same tool call (the same `tool_use_id`) and the tool did run, it appends one `settled` line to the observe ledger — the person let the write through; a refusal fires no hook, so the absence of that line is the record of a refusal. When a daemon is running it also asks it to re-measure the one file just written in the shared index, so the next write's duplicate probe sees the code this session just wrote; only fingerprinted languages, in scope, this project's own files, only when the index exists and is of the current generation, never building an index, never starting a daemon; a failure only marks `reindex.degraded` on the `settled` line |
| Stop | `ce audit --hook` | net LOC + duplicate blocks touching the change, stopping the turn only at the deny tier; the tombstone leg is judged in the core by `[tombstone] tier` and budget the same way and also stops only at deny. The four-class summary (cross-file moves / stacking suspicions) is recorded, never judged, its ledger in [docs/FPR-L2.md](../docs/FPR-L2.md); same-role advisor rows (when a unit this session added has a top-1 carrying the role bit) go only into the observe ledger and never stop the turn |

Every sentence of a write refusal and of the Stop / precommit / commitmsg faces is written by the core in the language of the moment (the hooks ask through the daemon's core link, the audit through its own, once); when the core cannot phrase it the decision stands and the sentence becomes one English fallback, `ce: rule … fired; the core could not phrase the reason: …` — it names the rule that fired, never the decision, one spelling at every tier (the audit faces read `ce <face>: rule … fired; the core could not phrase the verdict: …`, without the `rule … fired; ` part when the local rule does not block).

Skill: [`skills/erase/`](skills/erase/SKILL.md) walks the findings of dedup / deadcode / join
to safe deletions (read in full first, check references, check whether a command or a tool's convention uses the file by name, delete in small batches, re-run the gates to prove convergence),
and Claude invokes it when the user asks to clean up duplication or dead code. `ce erase` itself erases only byte-identical documentation spans; unreferenced files and whole-unit twins inside them
are always advisory rows (since 9.1.0 the reason is `reachable_by_name`: nothing in the tree references them, which does not mean nothing uses them), the skill's half to judge; `ce deadcode` lists them by default and blocks nothing, and blocks new unreferenced files (a dependent as much as a root) only when the project declares `[deadcode] block`.

Command: [`/codeeraser:update`](commands/update.md) runs `ce update` and relays its verdict
(exit 0 up to date / 1 an update exists / 2 unknown). The plugin's bound copy is set by the manifest pins,
so its update is always `/plugin update codeeraser` (a new manifest carries new pins, and the next session's
starter re-verifies and re-downloads); `ce update --yes` replaces only a copy placed by hand or shipped by the installer.

MCP: [`.mcp.json`](.mcp.json) registers the read-only report surface (`ce mcp`); its <!--ce:count:mcp_tools#word-->twenty-one<!--/ce--> tools arrive with the
plugin: installing the plugin installs the hooks and the reports together, with no separate `claude mcp add`. Claude Code
namespaces the tool names as `mcp__plugin_codeeraser_reports__<tool>`.
The `erase` tool goes as far as the **plan** and `erase_log` only reads its audit trail: `apply` has no face and never will, because a machine
face that can delete files on its own authority is the one thing an eraser must not ship; a plan with more rows than the core's cap (`limits.caps.erase_rows`) is sent
to the core in batches by path, with a result byte-identical to one batch; a plan never proposes deleting a whole file, and a load whose target is computed or a star import whose `__all__` is not all literal
makes the language unable to vouch for an unreferenced file (reason `language_unresolved`); `update_check` likewise only checks;
`similar_units` is the same-role advisor (the same document as `ce similar`), an advisor, never a verdict;
`query` / `rules` are the code-query family (the same documents as `ce query` / `ce rules`, the question or the rules file judged on the core's `query/1`);
here they only report, and the exit code is the CLI's (it reads the core's veto bit: `ce rules` exits 1 on a violation, 2 on a program error or when the core cannot judge); `query` with `prelude: true` asks no question
and answers the prelude and the rules pack the core renders from `ce.toml` `[rules.pack]` (`ce.query-prelude/0.1.0`, the same document as `ce query --prelude` and the
GUI query screen's "Prelude" button; `pack` is null when no pack is declared);
`merge_suggestions` is the clone merge advice (the same document as `ce merge`: how many parameters merging each clone group into one function takes, which member is kept,
how many lines it saves, whether it is feasible and why, judged on the core's `merge/1`), an advisor only;
`architecture` is the architecture analysis (the same document as `ce arch`: layers, cycle-cutting edges, clusters and misplaced files, directory metrics; `impact`
gives the files one change reaches, judged on the core's `arch/1`), an advisor only.

## Install

One step from the public marketplace (the manifest is the repository root's `.claude-plugin/marketplace.json`; the registered ref is
the `release` branch, which verify-publish fast-forwards to the tag commit after every publish, so an install follows the releases, not main):

1. `/plugin marketplace add skymanbp/CodeEraser@release`
2. `/plugin install codeeraser@codeeraser`

Since v1.7.0 **`ce setup` is these two steps**, one body on every platform (`cli/src/setup/`): find Claude Code
→ keep the marketplace if registered, register it otherwise → install and refresh → write the `claude-plugin-wired` marker only when this run registered it
→ report whether this binary's directory is on PATH; when the running account is not the logged-in user it refuses by name and wires nothing (exit 13);
`--unwire` removes only what the marker says it wired. The Windows installer runs it once at install and once at uninstall
(`gui/src-tauri/windows/hooks.nsh` only prints the exit-code legend, and every failure degrades to a hint line); AppImage / dmg /
CLI-only setups run it once themselves. A local development clone registers the **repository root** as the marketplace
(`claude plugin marketplace add <repo root>`); registering the `plugin/` subdirectory is the old place before 9f86d58,
the manifest is no longer there, and the hooks drop silently on a cache miss.

`bin/ce.sh` resolves both real binaries, `ce` and the judgment core `ce-core`, in three steps (ADR-007):
a verified local copy → a download from GitHub Releases checked against the SHA256 pins in `bin/manifest.env` (the manifest pins three per target: `ce` / `ce-core` / the GUI installer,
<!--ce:count:binaries#word-->fifteen<!--/ce--> across <!--ce:count:platforms#word-->five<!--/ce--> targets; the starter takes the first two, and a manifest before v1.7.0 pins only the first three targets) → PATH as the fallback. Verification is once per session: the verified
paths are recorded in `CLAUDE_PLUGIN_DATA/bound-<manifest version>.env`, and later hooks of the session exec them directly;
the core the stamp records is the file that is really on disk (on Windows, Git Bash's `command -v` answers the path without `.exe`; the starter adds it before binding);
a new manifest or binary re-verifies, and SessionStart's `health` verifies in every session. A copy placed by hand at
`CLAUDE_PLUGIN_DATA/ce-<manifest version>-<platform>` goes through the same pin: a match runs as verified and is stamped,
a mismatch is refused by name and not run (`CE_AIRGAPPED=1` never downloads and chooses only between the hand-placed copy and PATH).
Copies and stamps live only in this plugin's own data folder: the last segment of `CLAUDE_PLUGIN_DATA` must be the plugin's name `codeeraser`
or begin with `codeeraser-`. The plugins reference (https://code.claude.com/docs/en/plugins-reference, section "Environment variables") gives the folder as
`~/.claude/plugins/data/<id>/`, with `<id>` the plugin identifier where every character other than a letter, digit, `_` or `-` is replaced by `-`;
`claude plugin install` takes the identifier as `<plugin>@<marketplace>`, and this plugin's installed folder is `codeeraser-codeeraser`.
A command run through the Bash tool (such as `/codeeraser:update`) may carry another plugin's folder: the starter then uses exactly one folder
of this plugin's beside it (saying so in one stderr line), and treats the variable as unset when there is none or more than one; another plugin's folder is never read or written.
A source install (`cargo install codeeraser` or `--path cli`) still works.

**Windows prerequisite: `sh` must be on the Windows PATH.** The four hooks and `.mcp.json` all start with a bare `sh`
to run `bin/ce.sh`, and Claude Code is a native Windows process that looks up the first word of the command
on the Windows PATH. Git for Windows puts only `Git\cmd` on PATH by default, which holds `git.exe` but no `sh.exe`;
`sh.exe` is in `Git\bin` and `Git\usr\bin`. With no `sh` on PATH neither the hooks nor the MCP surface start,
and the failure is silent: what fails to start is the very script that would report it, and the symptoms are no health line, no `.ce/`,
and the server unreachable in `/mcp`. Appending `Git\bin` to the user PATH is enough (it holds only
`bash.exe` / `git.exe` / `sh.exe`, so it does not shadow the Windows tools of the same name such as `find` / `sort`,
which live in `Git\usr\bin`); appending rather than prepending keeps `bash` resolving to the WSL one in system32.
Restart Claude Code after changing PATH: the hook configuration is read when a session starts.

## Configuration

An optional `ce.toml` in the project:

```toml
[guard]
mode = "observe"   # global override (optional): observe | warn | ask | deny
                   # unset = the plan's §4.2 route defaults (below)

[[rules.class]]    # optional: path classes (plan v2.13) — first match in declaration order, no match = the global table
name  = "generated"
globs = ["gen/**"]
[rules.class.knobs]
file_lines_fail = 900   # this class's own hard budget, refused by PreToolUse; at most 64 classes, class names and globs never cross the wire
```

The default tiers are step 3 of the §4.2 route (the 1.0 tier switch of 2026-08-17; the record is in the root
[CHANGELOG.md](../CHANGELOG.md)): the two PreToolUse rules **T1/T2 exact duplicate writes** and **the hard budget
(a write that grows a file past its hard line: <!--ce:gate:size.file_lines_fail#digits-->750<!--/ce--> by default, or the one its `[[rules.class]]` declares; the hook reads `scan::classes::Classes::thresholds_for` through `guard::budget::lines_for`, and `ce scan` sends each row's class number (`Classes::class_of`) for the core to grade by `gradeOverrides`: both faces read the same class table)** default to `deny`; the Stop audit /
precommit are not promoted classes and stay observe by default. An explicit `mode` overrides every rule class.
A duplicate write is refused only for what it newly introduces: copies already in the replaced content (the file on disk for a Write, `old_string` for an Edit) pass; since 1.10.0 by count: the probe reports how many disjoint copies of a source run the new content holds, each copy in the replaced content excuses one, and a second copy written into a file that already carries one is refused (before, the second rode on the first's pass). The rule and its three properties are in booklet 09, the replay readings in [docs/FPR-REPLAY.md](../docs/FPR-REPLAY.md).
The graded-zone tier mapping (between the soft line and the hard line by position: < 25 % observe / 25–75 % warn / > 75 % ask) is the one switch
`[guard] zone_tiers`; the first measurement read 28/5196 = 0.5388 % on this repository and 11/448 = 2.4553 % on requests, the latter over 1 %, so variant B was taken and the default stays `false`; v2.35 re-froze it on the hook's own population, with a tier only when the write lengthens the file: this repository 184/9707 = 1.8955 %, requests 5/448 = 1.1160 %, both over 1 %, the default unchanged. The hard budget's own ledger is the same page's "hard budget ledger" section. The full ledger is in [docs/FPR-REPLAY.md](../docs/FPR-REPLAY.md), section "graded-zone tier mapping".
Observe data is in `<project>/.ce/observe.ndjson` (when `.ce/` has no `.gitignore`, ce writes one holding `*`, so the whole directory stays out of version control; an existing one is left alone);
every line carries `schema` (one source, `cli/src/hookio.rs::OBSERVE_SCHEMA`; the version is the one there and is not copied here, since the M5 closing audit caught a copy reading 0.3.0 while the code read 0.4.0), `session_id` and `ts_ms`;
`tombstone` event lines (one line only when a PreToolUse write erased names, or its headings / identifiers / prose
write a name this session erased back as "no X"; only the names' hashes are recorded), `settled` event lines (the PostToolUse leg: one line only when the tool really ran after PreToolUse answered `ask`, carrying only `file` and `tool_use_id`; every line of one tool call carries `tool_use_id`, the `probe` line also carries the hook's decision for the event, `decision`, and a tombstone line whose `applied` is null is read by whether a matching `settled` line exists), and the `tombstone` object on the Stop / precommit / commitmsg lines
(this change set's measurement: the number of erased names, the number of candidate surfaces `rows`, the changelog-position exemptions (a whole file by path, by ledger shape or by `[tombstone] ledger` declaration, or a single segment whose entry carries its start `line`) and the core's verdict `judged`: the first sites `file:line kind`, the label / prose split, `over`)
speak by the class's own `[tombstone] tier` (observe by default), and only when the core answers `over` (more sites than `[tombstone] budget`); observe records and never blocks, the FPR ledger in [docs/FPR-TOMBSTONE.md](../docs/FPR-TOMBSTONE.md);
the `fourclass` object (cross-file moves and stacking suspicions) never speaks: it belongs to no tier, and its change-set level
FPR ledger is [docs/FPR-L2.md](../docs/FPR-L2.md): 821 events with none skipped, strict 0/125, wide reading 0/820; the one copy positive `1035f6b1` was missed (recall 0/1), and this batch changes no default tier;
the `similar` object (every unit this session added, asked of the same-role advisor: `rev`, `new_units`, `queried`, and `rows` of `{unit,twin,score}` when the core's top-1 carries the role bit; only `degraded` when there is no core or the index cannot be read) never speaks either: the advisor does not judge, its evaluation ledger is [docs/EVAL-SET-SIMILAR.md](../docs/EVAL-SET-SIMILAR.md);
`flow` event lines (a PreToolUse write's two sides each lowered and judged once by the core's flow/1, one line only when the after side has findings: `before` / `after` finding counts, `novel` from the multiset difference by `(unit name, kind, variable name)`, `kinds` counted by kind, `judged` whether the language is in the judged mask; `mode` is the class's own `[flow] tier`, observe as shipped) and the `flow` object on the Stop / precommit / commitmsg lines (the files in the change set that hold units, the units, the findings, `kinds`, the judged count) appear additively since feed 0.12.0; the Stop leg never stops a turn at any tier, and the site list is in `ce flow`;
when the core's definition package cannot be read (no core, or a core whose protocol major differs from this build) the hooks measure nothing, exit 0 and never refuse a write, but every hook writes one line (PreToolUse `probe`, PostToolUse `settle`, Stop `stop_audit`, SessionStart `health`): `degraded` true, `reason` the same sentence the CLI's refusal prints, the PreToolUse / PostToolUse lines also carrying `file` and `tool_use_id`, additively since feed 0.13.0; `ce doctor`'s degradation count includes them, and the SessionStart line writes "tables: unavailable — <reason>", once per session;
additively since feed 0.14.0: the `budget` line's `before_lines` and `kind` (`crossing` / `growth`) or `carried: true` (already past the line and not lengthened: passed), with a `degraded` stamp in `before_unread` when the before line count cannot be read; the `zone` line's `before_lines` and `carried` when the file does not lengthen; the `probe` line's `unapplied` (the Edit does not apply: `absent` / `unreadable` / `no_match` / `ambiguous`, `unreadable` also carrying `apply.degraded`) and `relocated` (how many matches are moves, judged by the core's `relocate/1` from the counts at HEAD and in the tree after the write), with `relocation` naming the reason when there is no move credit; the `settled` line's `reindex` (when the re-measure failed); a hook envelope that does not parse as JSON, or one that names no working directory, is not judged and still writes one `degraded` line naming the reason (before, it passed silently);
a `session_id` of `null` means the line belongs to no session; `ce precommit` / `ce commitmsg` (the latter also treats the commit message as a surface, its sites recorded as `COMMIT_EDITMSG:line prose`) run in a terminal,
not as hooks, and are the only sources of null. Splitting by session is the prerequisite of the M4 evaluation set
(plan D2-1 sample purity / D2-2 observe-tier session counts).

> The hook configuration loads when a session starts; restart the session after installing or changing tiers
> (the measured conclusion of the 2026-08-07 hook payload capture session).
