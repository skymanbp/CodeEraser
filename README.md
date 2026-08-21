# CodeEraser

[![ci](https://github.com/skymanbp/CodeEraser/actions/workflows/ci.yml/badge.svg)](https://github.com/skymanbp/CodeEraser/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/codeeraser)](https://crates.io/crates/codeeraser) [![npm](https://img.shields.io/npm/v/codeeraser)](https://www.npmjs.com/package/codeeraser) [![license](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE) [![site](https://img.shields.io/badge/site-codeeraser.dev-e4574a)](https://codeeraser.dev)

English | **[中文](README.zh.md)**

> An eraser against LLM-induced code & document entropy.

<img src="docs/assets/gui-structure.png" alt="The GUI's structure treemap and score, judging this repository" width="740">

LLMs drift toward stacking and patching over long-lived work: the same
function implemented twice, the same fact written in three places,
updates that arrive as appends, files that only ever grow. CodeEraser
fights that drift at the moment of writing — a Rust CLI + Tauri GUI in
front of a Haskell judgment core, shipped as a Claude Code plugin with
PreToolUse/Stop interception, and reachable from any agent through a
read-only MCP report surface, pre-commit, and CI exit codes.

## Status

🚀 **v0.7.3 released — the size advisory closes its loop.**
Installers, [crates.io](https://crates.io/crates/codeeraser), the npm
pointer and [codeeraser.dev](https://codeeraser.dev) are live (0.7.1–
0.7.3 are patch releases: elevation up front, the CLI on the machine
PATH with its registry value kind intact, plugin reuse of pin-identical
binaries, no console flashes under the GUI, and bare `ce` answers help).
This cycle: the split advisory prices a seam's FULL cost — severed
references, cut clone blocks and crossing co-change pairs, each at a
corpus-calibrated price — the graded zone's position→tier map is
wired behind an explicit `[guard] zone_tiers` opt-in (the default
stays observe-only: no false-positive record, no promotion), and the
advisory joins the MCP tool surface and the GUI structure screen.
The soft line stays relative and re-derives at every named baseline
re-establish; `ce-baseline.json` is the one authority.

The locked plan is the contract: [docs/DEVELOPMENT_PLAN.md](docs/DEVELOPMENT_PLAN.md).
This repository gates itself with its own scanner, clone ratchet,
baseline and deadcode/docdup checks on every push to `main` (plus
pull requests and a weekly scheduled run).

## Install

Install surfaces are layered: the **installer** is the GUI+CLI
superset, the **plugin** the guard layer on any base, the rest CLI-only.

**Installer (recommended).** Every
[release](https://github.com/skymanbp/CodeEraser/releases) ships three
GUI installers (NSIS `setup.exe` / AppImage / dmg), each bundling the
GUI with `ce` **and** the `ce-core` judgment core as sidecars. On
Windows (v0.7.2+) the installer asks for elevation and puts the
install dir on the machine PATH — `ce` works from any terminal
(AppImage/dmg users add the app dir to PATH themselves).

**Claude Code plugin (the guard layer).** `/plugin marketplace add
skymanbp/CodeEraser`, then `/plugin install codeeraser`. The starter
resolves both binaries by pin: whichever copy — local or **on PATH** —
already matches the SHA256 answers first (v0.7.3; the installer leaves
one there, and a byte-identical fetch is waste), then a pinned
download, then an unverified PATH binary that says so out loud.

**CLI only.** Download `ce-<ver>-<platform>` and
`ce-core-<ver>-<platform>` (x86_64-windows / x86_64-linux /
aarch64-macos), rename them `ce` / `ce-core`, side by side on PATH —
judgment subcommands find the core through the sibling resolver leg.
Or `cargo install codeeraser` builds `ce` from source; drop a
`ce-core` beside it. `SHA256SUMS` covers every asset.

**From source.** Prerequisites: the pinned Rust toolchain
(`cli/rust-toolchain.toml`) and GHC 9.14.1 + cabal for the core.

```sh
# the judgment core (ce-core)
cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)
cargo install --path cli   # the CLI
```

Core resolution is one chain everywhere: `CE_CORE_BIN` → a `ce-core`
sibling of the running binary → PATH; an explicit `--core <path>`
always wins.

### Binaries — unsigned, verify checksums

Release artifacts are built by the
[release workflow](.github/workflows/release.yml) and pinned in
`SHA256SUMS`. **They are not code-signed or notarized** (ruled out
2026-08-19 — the cost/benefit does not hold for a free tool): Windows
SmartScreen and macOS Gatekeeper will warn until you allow the app
explicitly. The permanent trust anchor is the checksum chain — after
downloading:

```sh
sha256sum -c --ignore-missing SHA256SUMS
```

The Claude Code plugin's starter (`plugin/bin/ce.sh`) enforces the
same pins automatically and refuses a mismatching download out loud.

## Commands

| Command | What it reports / judges |
|---|---|
| `ce scan` | size / complexity / readability metrics, core-graded; the size-only arm also gates js/css/html/vue/svelte/sh/yml |
| `ce dedup` | T1/T2 clone blocks (winnowing index); `--check` gates the budget |
| `ce clone` | T3 near-miss clones (tree edit distance) |
| `ce docdup` | documentation duplication (paragraphs, comments, docstrings) |
| `ce graph --sites` / `ce deadcode` | reference sites; liveness verdicts |
| `ce churn` / `ce join` | git-window churn; the three-signal join |
| `ce structure` | tree-scale structure judgment (seven axes); `--split-candidates` prices the best seam of every file past the soft line — or writes its cohesion alibi |
| `ce trend` | score trajectory over mainline history (cache rebuilds from git) |
| `ce erase` | deterministic two-phase eraser: plans only provably-safe removals (dead files, verbatim doc twins, whole-unit T1 twins), dry-run default, `--apply` behind clean-worktree preconditions |
| `ce check` / `ce baseline` | ADR-006 ratchet + score floor against `ce-baseline.json` |
| `ce mcp` | read-only MCP server: every report above as a tool |
| `ce doctor` / `ce eject` | health line; full per-project uninstall (dry-run default) |

Console reports and `--help` speak English by default and Chinese
under `--lang zh` (or `CE_LANG=zh`; the flag wins). JSON output and
the FAIL/pass vocabulary are never translated — they are the machine
face. The GUI carries its own language toggle.

## Guard (Claude Code plugin)

The plugin intercepts at PreToolUse (cheap probes) and audits at Stop.
Since the 1.0 tier switch, the two FPR-gated rule classes — exact
T1/T2 duplicate writes and hard-budget breaches (a write leaving a
file past 750 lines) — **deny by default**; everything else observes
until it has its own false-positive record (ledger in
[CHANGELOG.md](CHANGELOG.md)). An explicit `[guard] mode` in `ce.toml`
overrides every class. The graded size zone between the soft line and
the hard budget stays observe-only by default; `[guard] zone_tiers`
opts a repo into the position→tier map (<25% observe / 25–75% warn /
>75% ask). Honest boundary: PreToolUse shapes behavior,
it is not a security wall — shell writes bypass it, and the Stop
audit + CI gates are the backstop.

## Documentation

- [CLI reference](docs/reference/cli.md) · [ce.toml reference](docs/reference/ce-toml.md) — generated from the binary and the config schema; a CI gate reddens on drift
- [DEVELOPMENT_PLAN](docs/DEVELOPMENT_PLAN.md) — the locked plan; every milestone answers to it
- [EVAL-SET](docs/EVAL-SET.md) — frozen evaluation universes, samples, audits and their gates
- [PERF-BUDGET](docs/PERF-BUDGET.md) · [FPR-REPLAY](docs/FPR-REPLAY.md) · [T1-INTERCEPT](docs/T1-INTERCEPT.md) — measured budgets and replay ledgers
- [contracts/VERSIONING.md](contracts/VERSIONING.md) — the wire contract and its SemVer rules
- [docs/RELEASE.md](docs/RELEASE.md) — the two-phase release runbook
- [docs/reference/methodology.md](docs/reference/methodology.md) — **how every verdict is computed**: the math of each judgment family, one file per family, every formula and constant cited to the line that implements it
- [docs/reference/structure-axes.md](docs/reference/structure-axes.md) — structure/1 axis semantics (S0–S6)
- [docs/reference/size-advisory.md](docs/reference/size-advisory.md) — the size soft-zone + split-ROI contract (shipped in v0.6.0; four-leg seam pricing in v0.7.0)
- [docs/reference/erase.md](docs/reference/erase.md) — the deterministic two-phase eraser contract (M9): what may be erased, what only advised, and why

## Architecture

![From repository to verdict: sources measured in Rust, judged in Haskell, rendered by five faces](docs/assets/architecture.svg)

| Layer | Language | Owns |
|---|---|---|
| Core (`core/`) | Haskell | judgment: rules, verdicts, scoring ratchet, graph liveness, TSED, structure entropy |
| Frontend (`cli/`) | Rust | parsing (tree-sitter), winnowing index, CLI, daemon, GUI backend, hooks, MCP |
| GUI (`gui/`) | Rust + vanilla JS | Tauri shell over the same report schema the CLI emits |
| Plugin (`plugin/`) | manifest + hooks + sh starter | pinned-binary bootstrap, interception (the marketplace manifest lives at the repo root `.claude-plugin/`) |

## License

Apache-2.0 — see [LICENSE](LICENSE). Third-party inventory: [NOTICE](NOTICE)
(regenerated and gated byte-exact in CI by `cli/tests/notice_gate.rs`).

"CodeEraser"™ is a trademark of skymanbp. Per Apache-2.0 §6, the
license covers the code, not the name.
