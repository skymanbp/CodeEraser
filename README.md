# CodeEraser

[![ci](https://github.com/skymanbp/CodeEraser/actions/workflows/ci.yml/badge.svg)](https://github.com/skymanbp/CodeEraser/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/codeeraser)](https://crates.io/crates/codeeraser) [![npm](https://img.shields.io/npm/v/codeeraser)](https://www.npmjs.com/package/codeeraser) [![license](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE) [![site](https://img.shields.io/badge/site-codeeraser.dev-e4574a)](https://codeeraser.dev)

English | **[中文](README.zh.md)**

> An eraser against LLM-induced code & document entropy.

<img src="docs/assets/gui-structure.png" alt="The GUI's structure treemap and score, judging this repository" width="740">

LLMs drift toward stacking and patching over long-lived work: the same function implemented twice,
the same fact written in three places, updates that arrive as appends, files that only ever grow.
CodeEraser fights that drift at the moment of writing — a Rust CLI + Tauri GUI in front of a Haskell
judgment core, shipped as a Claude Code plugin with PreToolUse/Stop interception, a read-only MCP report surface, pre-commit, and CI exit codes.

## Status

🏁 **v1.0.0 — complete.** Every milestone of the locked plan is delivered and the final sweep is clean:
the two-lane audit's 113 findings reconciled (81 fixed, 29 dispositioned in writing, 3 refutations standing),
716 documentation claims re-verified against the tree, and every number on the site either produced by replay
or retaken from real output. Installers, [crates.io](https://crates.io/crates/codeeraser), the npm pointer and
[codeeraser.dev](https://codeeraser.dev) are live. Scores under 1.0.0 are **not comparable** with 0.7.3 —
the density-law and cycle-axis migrations are declared in the release notes, and a floor calibrated against a
pre-1.0 band needs a named `CE_ACCEPT_BASELINE=1` re-establish.

The locked plan is the contract: [docs/DEVELOPMENT_PLAN.md](docs/DEVELOPMENT_PLAN.md). This repository gates itself with its own scanner,
clone ratchet, baseline and deadcode/docdup checks on every push to `main` (plus pull requests and a weekly scheduled run).

## Install

Install surfaces are layered: the **installer** is the GUI+CLI superset,
the **plugin** the guard layer on any base, the rest CLI-only.

**Installer (recommended).** Every [release](https://github.com/skymanbp/CodeEraser/releases) ships three GUI installers
(NSIS `setup.exe` / AppImage / dmg), each bundling the GUI with `ce` **and** the `ce-core` judgment core as sidecars.
On Windows (v0.7.2+) the installer asks for elevation and puts the install dir on the machine PATH — `ce` works
from any terminal (AppImage/dmg users add the app dir to PATH themselves).

**Claude Code plugin (the guard layer).** `/plugin marketplace add skymanbp/CodeEraser`, then `/plugin install codeeraser`.
The starter resolves both binaries by pin: a matching local or PATH copy first (since v0.7.3; the installer leaves one on PATH), then a pinned download,
then an unverified PATH binary that says so out loud.

**CLI only.** Download `ce-<ver>-<platform>` and `ce-core-<ver>-<platform>` (x86_64-windows / x86_64-linux / aarch64-macos), rename them
`ce` / `ce-core`, and put them side by side on PATH; judgment commands use the sibling resolver. Or install `ce`
with `cargo install codeeraser` and place a `ce-core` beside it. `SHA256SUMS` covers every asset.

**From source.** Prerequisites: the pinned Rust toolchain (`rust-toolchain.toml` at the repository root) and GHC 9.14.1 + cabal for the core.

```sh
# the judgment core (ce-core)
cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)
cd .. && cargo install --path cli   # the CLI
```

Core resolution is one chain everywhere: `CE_CORE_BIN` → a `ce-core` sibling of the running binary → PATH; an explicit `--core <path>` always wins.

### Binaries — unsigned, verify checksums

Release artifacts are built by the [release workflow](.github/workflows/release.yml) and pinned in `SHA256SUMS`.
**They are not code-signed or notarized** (ruled out 2026-08-19); Windows SmartScreen and macOS Gatekeeper will warn.
The permanent trust anchor is the checksum chain — after downloading:

```sh
sha256sum -c --ignore-missing SHA256SUMS
```

The Claude Code plugin's starter (`plugin/bin/ce.sh`) enforces the same pins automatically and refuses a mismatching download out loud.

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
| `ce mcp` | read-only MCP server: 11 report tools; erase plans and doctor are not exposed |
| `ce doctor` / `ce eject` | health line; full per-project uninstall (dry-run default) |

Console reports and `--help` speak English by default and Chinese under `--lang zh` (or `CE_LANG=zh`; the flag wins). JSON output and the FAIL/pass vocabulary are never translated — they are the machine face. The GUI carries its own language toggle.

## Guard (Claude Code plugin)

The plugin intercepts at PreToolUse (cheap probes) and audits at Stop. Since the 1.0 tier switch, the two FPR-gated rule classes — exact T1/T2 duplicate writes and hard-budget breaches (a write leaving a file past 750 lines) — **deny by default**; everything else observes until it has its own false-positive record (ledger in [CHANGELOG.md](CHANGELOG.md)). An explicit `[guard] mode` in `ce.toml` overrides every class. The graded size zone between the soft line and the hard budget stays observe-only by default; `[guard] zone_tiers` opts a repo into the position→tier map (<25% observe / 25–75% warn / >75% ask). Honest boundary: PreToolUse shapes behavior, it is not a security wall — shell writes bypass it. The Stop audit re-judges net LOC and touched duplicates; CI carries the hard size wall and ratchet.

## Evaluation dashboard

<!-- bench:begin -->
### Latest-version latency · v1.0.0

| percentile | `check_warm` | `deadcode_warm` | `dedup_cold` | `dedup_warm` | `docdup_warm` | `hook_probe` | `scan` |
|---|---:|---:|---:|---:|---:|---:|---:|
| p50 ms | 1449 | 844 | 4456 | 687 | 893 | 41 | 771 |
| p95 ms | 1469 | 847 | 4739 | 723 | 1079 | 50 | 960 |

### Frozen evaluation points

| metric | value | source |
|---|---|---|
| `docdup_d3_precision` | 17/17 scoped (100%) | `docs/EVAL-SET-M5-3.md:81-87 + contracts/eval/docdup-precision-*-v1.json` |
| `docdup_d1_recall` | 100% | `docs/EVAL-SET-M5-3.md:81-87 + contracts/eval/docdup-precision-*-v1.json` |
| `t3_precision` | 61 answered / 0 wrong (1.000) | `docs/EVAL-SET-M5-3.md:41-47 + contracts/eval/t3-precision-*-v1.json` |
| `graph_precision` | overall gate >= 0.90 held | `docs/EVAL-SET.md:280-292 + contracts/eval/graph-precision-*-v1.json` |
| `fourclass_fpr` | 0/600 flagged (gate <= 1%) | `contracts/eval/fpr-fourclass-v1.json + docs/EVAL-SET.md:131-140` |
| `guard_fpr_per500` | 0.00 per 500 edits | `docs/FPR-REPLAY.md:16-36` |
| `l2_moved_recall` | 547/547 cross-file moved lines | `docs/EVAL-SET.md:97-129 + contracts/eval/commit-l2*-v1.json` |
| `dedup_recall_vs_jscpd` | cobra 106/109 raw -> 106/106 attributed | `contracts/fixtures/crosscheck/DEDUP-CALIBRATION.md:96-137` |
| `t3_recall_vs_similarity` | zod 0.50 / requests 0.158 / cobra 0.154 (raw) | `docs/EVAL-SET-M5-CLOSE.md:38-63` |

Every value is generated from `contracts/bench/bench.json`; the test rejects hand edits to this block. [Full replay notes and per-version series](docs/BENCH.md) · [Complete website dashboard](https://codeeraser.dev/bench/)
<!-- bench:end -->

## Documentation

- [Tech stack](https://codeeraser.dev/stack/) · [evaluation dashboard](https://codeeraser.dev/bench/) — the website's component map and complete generated record
- [CLI reference](docs/reference/cli.md) · [ce.toml reference](docs/reference/ce-toml.md) — generated from the binary and the config schema; a CI gate reddens on drift
- [DEVELOPMENT_PLAN](docs/DEVELOPMENT_PLAN.md) · [EVAL-SET](docs/EVAL-SET.md) · [FIELD-TEST](docs/FIELD-TEST.md) — locked plan, frozen evaluation design and real-repository findings
- [BENCH](docs/BENCH.md) · [PERF-BUDGET](docs/PERF-BUDGET.md) · [FPR-REPLAY](docs/FPR-REPLAY.md) · [T1-INTERCEPT](docs/T1-INTERCEPT.md) — generated series and measured replay ledgers
- [contracts/VERSIONING.md](contracts/VERSIONING.md) · [docs/RELEASE.md](docs/RELEASE.md) — wire SemVer and the two-phase release runbook
- [docs/reference/methodology.md](docs/reference/methodology.md) — every verdict's math, one booklet per family, with formula and constant citations to implementation lines
- [structure axes](docs/reference/structure-axes.md) · [size advisory](docs/reference/size-advisory.md) · [erase contract](docs/reference/erase.md) — focused behavior contracts

## Under the hood / tech stack

![Detailed stack: Rust measurement, the versioned wire, Haskell judgment, product faces and the release pin chain](docs/assets/stack.svg)

Rust owns source-facing work: tree-sitter parsing, the SQLite WAL
fingerprint index, resolver ladders, git windows, the lazy project
daemon, and fact gathering. Those facts cross one SemVer-negotiated
NDJSON wire. Haskell owns product decisions: score and ratchet
verdicts, graph liveness and cycle membership, clone/docdup decisions,
split pricing, and erase authorization. The terminal, Tauri GUI,
read-only MCP server, Claude Code hooks and CI render or enforce those
same report shapes.

- The push workflow runs the six self-hosting product gates, including the explicit score floor; this repository is the standing dogfood fixture.
- ADR-006 ceilings and violation sets live in `ce-baseline.json`; cleanup tightens them, while growth needs an explicit re-establish.
- CLI/config references are generated, and the twelve-booklet methodology has machine-checked citations, navigation and EN/ZH constants.
- A guard class moves to deny only after its own false-positive record is entered in [CHANGELOG.md](CHANGELOG.md); unqualified classes remain observe.
- `ce erase` gathers deterministic facts and lets the Haskell safety predicate authorize removals; it never asks a model to rewrite code.
- Release builds are two-phase: hashes come from draft assets, pins land in the tree, and the tag verifies those same bytes without rebuilding.

[Website stack page](https://codeeraser.dev/stack/) ·
[verdict methodology](docs/reference/methodology.md) · [wire contract](contracts/VERSIONING.md)

The boundary is concrete: Rust emits measured facts; Haskell returns the decisions.

## License

Apache-2.0 — see [LICENSE](LICENSE). Third-party inventory: [NOTICE](NOTICE)
(regenerated and gated byte-exact in CI by `cli/tests/notice_gate.rs`).

"CodeEraser"™ is a trademark of skymanbp. Per Apache-2.0 §6, the
license covers the code, not the name.
