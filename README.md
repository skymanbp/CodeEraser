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

🚀 **v0.5.0 released — all planned milestones (M0–M8) shipped, plus a
hardening cycle.** Installers, [crates.io](https://crates.io/crates/codeeraser),
the npm pointer and [codeeraser.dev](https://codeeraser.dev) are live.
This cycle: the daemon's credential gate hardened end to end (a
dual-lane external review, all findings closed), the size gates
extended to common front-end/script extensions (size-only, never
judged), structure axis 3 recalibrated to count directories (a
declared score migration), and the plan-v2.6 size soft-zone +
split-ROI contract locked for v0.6.

The locked plan is the contract: [docs/DEVELOPMENT_PLAN.md](docs/DEVELOPMENT_PLAN.md).
This repository gates itself with its own scanner, clone ratchet,
baseline and deadcode/docdup checks on every push to `main` (plus
pull requests and a weekly scheduled run).

## Install

**Prebuilt (v0.2.0+, recommended).** Every
[release](https://github.com/skymanbp/CodeEraser/releases) ships ten
assets: `ce` **and** the `ce-core` judgment core for three platforms
(x86_64-windows / x86_64-linux / aarch64-macos), three GUI installers
(NSIS `setup.exe` / AppImage / dmg — each bundling the GUI with both
binaries as sidecars), and `SHA256SUMS`. For the CLI: download
`ce-<ver>-<platform>`, rename it `ce`, put it on PATH, and drop
`ce-core-<ver>-<platform>` beside it as `ce-core` — judgment
subcommands find it through the sibling leg of the resolver, no
flags, no env vars.

**Claude Code plugin.** `/plugin marketplace add skymanbp/CodeEraser`,
then `/plugin install codeeraser` — both binaries arrive SHA256-verified.

**Cargo.** `cargo install codeeraser` builds `ce`; drop a `ce-core` beside it.

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
| `ce structure` | tree-scale structure judgment (seven axes) |
| `ce trend` | score trajectory over mainline history (cache rebuilds from git) |
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
overrides every class. Honest boundary: PreToolUse shapes behavior,
it is not a security wall — shell writes bypass it, and the Stop
audit + CI gates are the backstop.

## Documentation

- [CLI reference](docs/reference/cli.md) · [ce.toml reference](docs/reference/ce-toml.md) — generated from the binary and the config schema; a CI gate reddens on drift
- [DEVELOPMENT_PLAN](docs/DEVELOPMENT_PLAN.md) — the locked plan; every milestone answers to it
- [EVAL-SET](docs/EVAL-SET.md) — frozen evaluation universes, samples, audits and their gates
- [PERF-BUDGET](docs/PERF-BUDGET.md) · [FPR-REPLAY](docs/FPR-REPLAY.md) · [T1-INTERCEPT](docs/T1-INTERCEPT.md) — measured budgets and replay ledgers
- [contracts/VERSIONING.md](contracts/VERSIONING.md) — the wire contract and its SemVer rules
- [docs/RELEASE.md](docs/RELEASE.md) — the two-phase release runbook
- [docs/reference/structure-axes.md](docs/reference/structure-axes.md) — structure/1 axis semantics (S0–S6)
- [docs/reference/size-advisory.md](docs/reference/size-advisory.md) — the plan-v2.6 size soft-zone + split-ROI contract (ships in v0.6)

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

"CodeEraser"™ is a trademark of skymanbp (registration pending). Per
Apache-2.0 §6, the license covers the code, not the name.
