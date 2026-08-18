# CodeEraser

> An eraser against LLM-induced code & document entropy.

LLMs drift toward stacking and patching over long-lived work: the same
function implemented twice, the same fact written in three places,
updates that arrive as appends, files that only ever grow. CodeEraser
fights that drift at the moment of writing — a Rust CLI + Tauri GUI in
front of a Haskell judgment core, shipped as a Claude Code plugin with
PreToolUse/Stop interception, and reachable from any agent through a
read-only MCP report surface, pre-commit, and CI exit codes.

## Status

🚧 **0.x preview. M0–M6 shipped; M7 (release track) in progress.**

The locked plan is the contract: [docs/DEVELOPMENT_PLAN.md](docs/DEVELOPMENT_PLAN.md).
This repository gates itself with its own scanner, clone ratchet,
baseline and deadcode/docdup checks on every push.

## Install (from source)

Prerequisites: the pinned Rust toolchain (`cli/rust-toolchain.toml`)
and GHC 9.14.1 + cabal for the judgment core.

```sh
# the judgment core (ce-core)
cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)

# the CLI (binary name: ce)
cargo install --path cli
```

Judgment subcommands take the core via `--core "$CE_CORE_BIN"` (or the
`CE_CORE_BIN` environment variable through the daemon/MCP paths).

### Binaries — unsigned, verify checksums

Release artifacts (three platforms + GUI installers) are built by the
[release workflow](.github/workflows/release.yml) with a `SHA256SUMS`
manifest. **They are not code-signed or notarized yet** (deferred past
1.0 by plan amendment v2.1): Windows SmartScreen and macOS Gatekeeper
will warn or refuse until you allow the app explicitly. The trust
anchor is the checksum chain — after downloading:

```sh
sha256sum -c --ignore-missing SHA256SUMS
```

The Claude Code plugin's starter (`plugin/bin/ce.sh`) enforces the
same pins automatically and refuses a mismatching download out loud.

## Commands

| Command | What it reports / judges |
|---|---|
| `ce scan` | size / complexity / readability metrics, core-graded |
| `ce dedup` | T1/T2 clone blocks (winnowing index); `--check` gates the budget |
| `ce clone` | T3 near-miss clones (tree edit distance) |
| `ce docdup` | documentation duplication (paragraphs, comments, docstrings) |
| `ce graph --sites` / `ce deadcode` | reference sites; liveness verdicts |
| `ce churn` / `ce join` | git-window churn; the three-signal join |
| `ce structure` | tree-scale structure judgment (seven axes) |
| `ce check` / `ce baseline` | ADR-006 ratchet + score floor against `ce-baseline.json` |
| `ce mcp` | read-only MCP server: every report above as a tool |
| `ce doctor` / `ce eject` | health line; full per-project uninstall (dry-run default) |

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

- [DEVELOPMENT_PLAN](docs/DEVELOPMENT_PLAN.md) — the locked plan; every milestone answers to it
- [EVAL-SET](docs/EVAL-SET.md) — frozen evaluation universes, samples, audits and their gates
- [PERF-BUDGET](docs/PERF-BUDGET.md) · [FPR-REPLAY](docs/FPR-REPLAY.md) · [T1-INTERCEPT](docs/T1-INTERCEPT.md) — measured budgets and replay ledgers
- [contracts/VERSIONING.md](contracts/VERSIONING.md) — the wire contract and its SemVer rules
- [docs/reviews/](docs/reviews/) — attack/design review records, one file per round

## Architecture

| Layer | Language | Owns |
|---|---|---|
| Core (`core/`) | Haskell | judgment: rules, verdicts, scoring ratchet, graph liveness, TSED, structure entropy |
| Frontend (`cli/`) | Rust | parsing (tree-sitter), winnowing index, CLI, daemon, GUI backend, hooks, MCP |
| GUI (`gui/`) | Rust + vanilla JS | Tauri shell over the same report schema the CLI emits |
| Plugin (`plugin/`) | manifest + hooks + sh starter | marketplace layout, pinned-binary bootstrap, interception |

## License

Apache-2.0 — see [LICENSE](LICENSE). Third-party inventory: [NOTICE](NOTICE)
(regenerated and gated byte-exact in CI by `cli/tests/notice_gate.rs`).
