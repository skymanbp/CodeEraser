# The CodeEraser GUI — nine screens over one document set (reference)

> Status: shipped in stages — three screens at v0.7.3, eight at
> v1.0.0 (the graph screen closed M9), nine since the K round added
> doctor. The implementation answers to this file;
> divergence is a defect in one of the two. The screens live in
> `gui/ui/*.js`, the Tauri backend in `gui/src-tauri/src/commands.rs`
> — a separate cargo workspace from the CLI, with its own CI legs.

## The one rule

Every screen renders a **report document** — the same JSON object the
CLI prints under `--format json` and the MCP server returns — through
`codeeraser::faces`, the single adapter layer both machine surfaces
consume. The GUI performs **no second measurement and no judgment of
its own**: every verdict on screen was judged in the Haskell core,
every number measured in the `codeeraser` crate, and what remains
here is layout geometry, color, and words. There is no framework and
no build step in the webview (user ruling: judgment zero-leak into
JS); the one deliberate exception to the faces road is the erase
pair, which needs the typed `Plan` value itself (below).

Two consequences worth naming:

- **The GUI cannot disagree with the CLI.** A count on a screen and
  the same count in a terminal report came off one document. When
  they differ, one of the two renderers has a defect — the number
  itself has one home.
- **A schema is the contract.** Screens read documents by their
  `ce.*-report` schema ids; report JSON is never translated
  (`cli/src/i18n.rs` charter), so the language toggle swaps this
  face's own word tables (`gui/ui/i18n.js`, en/zh key-for-key, gated
  by `gui/tests/i18n_gate.js`) and never touches the data.

## The nine screens

| tab | document | what it shows |
|---|---|---|
| **Structure** | `ce.structure-report` | the tree-scale judgment as a treemap and a tree lens (two views over one report); per-directory findings ride the alarm ramp, and the split advisory is opt-in per scan |
| **Score** | `ce.check-report` | hero score over the effective scale, the seven verdict axes, the ratchet's four registers — the FAIL/pass vocabulary stays machine-English, exactly like the CLI's exit-code face |
| **Erase** | `ce.erase-plan` + a rendered diff | the deterministic two-phase eraser's face: preview **is** the plan (eraseable rows with provenance, advisories with named reasons, the unified diff rendered from the hashed plan); Apply stays hidden until a preview exists |
| **Trend** | `ce.trend-report` | score points over mainline history plus the core's trend judgment; "measure more" batches uncached commits so a cold cache fills at the reader's pace |
| **Candidates** | `ce.join-report` + `ce.dedup-report` | the deletion-candidate browser: three-signal file pairs, unit pairs and clone blocks in document order — no ranking is derived here, and the per-row bar is geometry over a printed number |
| **Graph** | `ce.graph-canvas` | the reference graph as a drawn map, file tier only; the layout is deterministic (the same document draws the same picture), dead files and cycle members ride the alarm ramp, node radius is degree |
| **Reports** | the remaining families | the diagnostics hub: each family's document rendered generically — counts as chips, row arrays as tables — adding zero interpretation |
| **Bench** | `contracts/bench/bench.json` | the compiled-in benchmark series, pivoted to one row per metric with a column per version; frozen points carry value **and** source |
| **Doctor** | `ce.doctor-report` | this machine's state: ce-core handshake, guard tier, index freshness, daemon, degraded-run counter — probed without starting the daemon or rebuilding the index, so the diagnostic reports a state it did not create |

## How a screen runs

Every judgment button drives one Tauri command, and every command is
the same shape: anchor the root (`codeeraser::root::project_root` —
the GUI re-roots to the enclosing project and says so), resolve the
core once (the CLI's own chain: `CE_CORE_BIN` → a `ce-core` sibling
of the executable → PATH; the installers stage ce-core as exactly
that sibling), run the library closure off the async runtime, and
bracket it with `ce-task` events that the status bar renders. A
missing core fails loudly by name, same as the CLI.

The erase pair is the one road past `faces`, by design: the preview
needs the typed `Plan` (to hash targets and render the diff from the
same bytes the plan pinned), and Apply is **the same library entry
the CLI's `--apply` drives** — one implementation, two faces, with
the contract preconditions checked in order (git repo whose toplevel
equals the root, clean worktree, unchanged targets) and every refusal
surfacing by name. The full contract is [erase.md](erase.md).

## What the GUI will not do

- **No LLM anywhere.** Nothing on any screen asks a model anything.
- **No baseline writes.** There is no accept-baseline button; the
  ratchet is moved at the CLI (`CE_ACCEPT_BASELINE=1 ce baseline`)
  where the named re-establish discipline lives.
- **No config writes.** `ce.toml` is edited by its owner, not by a
  settings panel.
- **One write verb total.** Erase Apply is the single command that
  touches user files, and it acts only on the plan the reader just
  previewed, behind the preconditions above.

## Getting it

The three release installers (NSIS `setup.exe` / AppImage / dmg)
bundle the GUI with `ce` and `ce-core` as sidecars — one install is
the whole product. The Windows installer also detects Claude Code
and wires the plugin. Building from source: `cargo build` in
`gui/src-tauri/` (its own workspace — building the CLI does not
build it, which is why CI carries dedicated GUI legs: build, clippy,
fmt, and the two webview gates `lens_invariant.js` / `i18n_gate.js`).
