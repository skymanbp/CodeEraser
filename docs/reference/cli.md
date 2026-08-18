<!-- GENERATED — do not edit. Regenerate: CE_BLESS=1 cargo test --test docs_gate. CI reddens when this file drifts from its regeneration. Length rides the CLI surface (a machine-generated projection, the hs_boot stance), so the scan's file-lines warn on the CLI page is an accounted standing warn, not maintained prose over budget. -->

# `ce` command reference

Every block below is the binary's own `--help` output (English face; `--lang zh` or `CE_LANG=zh` switches the console at runtime).

## ce

```text
CodeEraser — erase LLM-induced code & document entropy

Usage: ce [OPTIONS] <COMMAND>

Commands:
  doctor     Environment + project health: ce-core handshake, project status line, degradation counter (never starts the daemon)
  scan       Measure size / complexity / readability metrics (M1 modules; levels graded by the core since ADR-008 P3)
  churn      Time-dimension metrics: append vs rewrite, windowed churn, co-change pairs (M4; report-only, feeds the M5 join)
  graph      Dependency-graph subsystem: --sites lists reference sites (resolution-free); liveness lives under `ce deadcode`
  deadcode   Judge liveness over the cached reference graph (M5-2h): the ladder's edges, the Haskell core's four-way verdicts
  clone      T3 near-miss clone judgment (M5-3): TED via the core's clone/1; --units lists the cached unit universe instead
  docdup     Documentation-duplication judgment (M5-3g): exact Jaccard via the core's docdup/1 over the cached live segments
  join       Three-signal join (M5-3h): similarity + graph position + per-unit churn, file and unit tiers (report-only until 3i)
  structure  Tree-scale structure judgment (M6): entropy, axes and findings via the core's structure/1 (report-only in S2)
  trend      Score trajectory over mainline history (M7-P4): per-commit absolute check score, cached in the index, rebuildable
  check      ADR-006 gate (M5-3i): judge the repo against ce-baseline.json — ratchet OR --fail-under floor, either alone fails
  baseline   Persist the core's newBaseline as ce-baseline.json (the violation set only shrinks without CE_ACCEPT_BASELINE=1)
  dedup      Detect T1/T2 clones via the winnowing fingerprint index (M2)
  daemon     Run the per-project daemon in the foreground (ADR-003); normally lazy-started by `ce ping` / hook probes
  ping       Round-trip a ping through the project daemon (lazy-starts it)
  probe      PreToolUse cheap gate: read the hook envelope on stdin, probe the daemon, emit a permission decision per ce.toml [guard]
  audit      Stop audit v1: net LOC + duplicate blocks touching changed files (blocks the stop only in deny mode)
  health     SessionStart health line + daemon warm-up
  precommit  pre-commit gate: staged net LOC + touched duplicates (exit 1 in deny mode when duplicates are touched)
  mcp        MCP server over stdio: the read-only report face of every judgment family (M7-P2)
  eject      Uninstall project state: .ce/, baseline, pins (dry-run default)
  help       Print this message or the help of the given subcommand(s)

Options:
      --lang <LANG>  Console language (wins over CE_LANG) [possible values: en, zh]
  -h, --help         Print help
  -V, --version      Print version
```

## ce doctor

```text
Environment + project health: ce-core handshake, project status line, degradation counter (never starts the daemon)

Usage: ce doctor [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Project root to report on (default: current directory)

Options:
      --core <CORE>  Path to the ce-core executable (default: CE_CORE_BIN, a ce-core beside this binary, then PATH) [default: ce-core]
      --lang <LANG>  Console language (wins over CE_LANG) [possible values: en, zh]
  -h, --help         Print help
```

## ce scan

```text
Measure size / complexity / readability metrics (M1 modules; levels graded by the core since ADR-008 P3)

Usage: ce scan [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to scan (default: current directory)

Options:
      --format <FORMAT>  [default: console] [possible values: console, json]
      --lang <LANG>      Console language (wins over CE_LANG) [possible values: en, zh]
      --core <CORE>      Path to the ce-core executable (default: CE_CORE_BIN, a ce-core beside this binary, then PATH) [default: ce-core]
  -h, --help             Print help
```

## ce churn

```text
Time-dimension metrics: append vs rewrite, windowed churn, co-change pairs (M4; report-only, feeds the M5 join)

Usage: ce churn [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Repository root (default: current directory)

Options:
      --days <DAYS>      History window in days [default: 14]
      --lang <LANG>      Console language (wins over CE_LANG) [possible values: en, zh]
      --format <FORMAT>  [default: console] [possible values: console, json]
  -h, --help             Print help
```

## ce graph

```text
Dependency-graph subsystem: --sites lists reference sites (resolution-free); liveness lives under `ce deadcode`

Usage: ce graph [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Directory to analyze (default: current directory)

Options:
      --lang <LANG>      Console language (wins over CE_LANG) [possible values: en, zh]
      --sites            List reference sites
      --format <FORMAT>  [default: console] [possible values: console, json]
  -h, --help             Print help
```

## ce deadcode

```text
Judge liveness over the cached reference graph (M5-2h): the ladder's edges, the Haskell core's four-way verdicts

Usage: ce deadcode [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Directory to judge (default: current directory)

Options:
      --db <DB>          Index database path (default: <root>/.ce/index.db)
      --lang <LANG>      Console language (wins over CE_LANG) [possible values: en, zh]
      --core <CORE>      Path to the ce-core executable (default: CE_CORE_BIN, a ce-core beside this binary, then PATH) [default: ce-core]
      --format <FORMAT>  [default: console] [possible values: console, json]
      --check            Exit 1 when any file-tier dead verdict lands (the CI dogfood gate for the M5-2 full-disposition acceptance row)
  -h, --help             Print help
```

## ce clone

```text
T3 near-miss clone judgment (M5-3): TED via the core's clone/1; --units lists the cached unit universe instead

Usage: ce clone [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Directory to analyze (default: current directory)

Options:
      --format <FORMAT>  [default: console] [possible values: console, json]
      --lang <LANG>      Console language (wins over CE_LANG) [possible values: en, zh]
      --core <CORE>      Path to the ce-core executable (default: CE_CORE_BIN, a ce-core beside this binary, then PATH) [default: ce-core]
      --db <DB>          Index database path (default: <root>/.ce/index.db)
      --units            List the unit universe instead of judging
  -h, --help             Print help
```

## ce docdup

```text
Documentation-duplication judgment (M5-3g): exact Jaccard via the core's docdup/1 over the cached live segments

Usage: ce docdup [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Directory to analyze (default: current directory)

Options:
      --format <FORMAT>  [default: console] [possible values: console, json]
      --lang <LANG>      Console language (wins over CE_LANG) [possible values: en, zh]
      --core <CORE>      Path to the ce-core executable (default: CE_CORE_BIN, a ce-core beside this binary, then PATH) [default: ce-core]
      --db <DB>          Index database path (default: <root>/.ce/index.db)
      --check            Exit 1 when any duplication is reported (the CI dogfood gate — plan §7.5's docdup clause, honored in code since M5 close)
  -h, --help             Print help
```

## ce join

```text
Three-signal join (M5-3h): similarity + graph position + per-unit churn, file and unit tiers (report-only until 3i)

Usage: ce join [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Directory to analyze (default: current directory)

Options:
      --format <FORMAT>  [default: console] [possible values: console, json]
      --lang <LANG>      Console language (wins over CE_LANG) [possible values: en, zh]
      --core <CORE>      Path to the ce-core executable (default: CE_CORE_BIN, a ce-core beside this binary, then PATH) [default: ce-core]
      --db <DB>          Index database path (default: <root>/.ce/index.db)
      --days <DAYS>      Churn window in days [default: 14]
  -h, --help             Print help
```

## ce structure

```text
Tree-scale structure judgment (M6): entropy, axes and findings via the core's structure/1 (report-only in S2)

Usage: ce structure [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Directory to analyze (default: current directory)

Options:
      --format <FORMAT>  [default: console] [possible values: console, json]
      --lang <LANG>      Console language (wins over CE_LANG) [possible values: en, zh]
      --core <CORE>      Path to the ce-core executable (default: CE_CORE_BIN, a ce-core beside this binary, then PATH) [default: ce-core]
      --db <DB>          Index database path (default: <root>/.ce/index.db)
      --deep             Also roll clone blocks and dead units up per directory and judge the S6 redundancy axis (runs the dedup census and the liveness judgment; absent = the axis is honestly unjudged)
      --days <DAYS>      Judge the S5 doc-staleness axis over this git window in days (docs whose referenced code changed after their last edit; absent = the axis is honestly unjudged)
  -h, --help             Print help
```

## ce trend

```text
Score trajectory over mainline history (M7-P4): per-commit absolute check score, cached in the index, rebuildable

Usage: ce trend [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Directory to analyze (default: current directory)

Options:
      --format <FORMAT>    [default: console] [possible values: console, json]
      --lang <LANG>        Console language (wins over CE_LANG) [possible values: en, zh]
      --core <CORE>        Path to the ce-core executable (default: CE_CORE_BIN, a ce-core beside this binary, then PATH) [default: ce-core]
      --db <DB>            Index database path (default: <root>/.ce/index.db)
      --commits <COMMITS>  Mainline window: newest N first-parent commits [default: 30]
      --batch <BATCH>      Measure at most this many uncached commits per run (absent = all of them; the GUI passes small batches for progress)
  -h, --help               Print help
```

## ce check

```text
ADR-006 gate (M5-3i): judge the repo against ce-baseline.json — ratchet OR --fail-under floor, either alone fails

Usage: ce check [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Directory to analyze (default: current directory)

Options:
      --format <FORMAT>          [default: console] [possible values: console, json]
      --lang <LANG>              Console language (wins over CE_LANG) [possible values: en, zh]
      --core <CORE>              Path to the ce-core executable (default: CE_CORE_BIN, a ce-core beside this binary, then PATH) [default: ce-core]
      --db <DB>                  Index database path (default: <root>/.ce/index.db)
      --days <DAYS>              Churn window in days (omit = churn tables stay empty)
      --fail-under <FAIL_UNDER>  Fail when the score lands under this per-mille floor
  -h, --help                     Print help
```

## ce baseline

```text
Persist the core's newBaseline as ce-baseline.json (the violation set only shrinks without CE_ACCEPT_BASELINE=1)

Usage: ce baseline [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Directory to analyze (default: current directory)

Options:
      --format <FORMAT>  [default: console] [possible values: console, json]
      --lang <LANG>      Console language (wins over CE_LANG) [possible values: en, zh]
      --core <CORE>      Path to the ce-core executable (default: CE_CORE_BIN, a ce-core beside this binary, then PATH) [default: ce-core]
      --db <DB>          Index database path (default: <root>/.ce/index.db)
      --days <DAYS>      Churn window in days (omit = churn tables stay empty)
  -h, --help             Print help
```

## ce dedup

```text
Detect T1/T2 clones via the winnowing fingerprint index (M2)

Usage: ce dedup [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to index (default: current directory)

Options:
      --format <FORMAT>              [default: console] [possible values: console, json]
      --lang <LANG>                  Console language (wins over CE_LANG) [possible values: en, zh]
      --db <DB>                      Index database path (default: <path>/.ce/index.db)
      --min-tokens <MIN_TOKENS>      Report threshold in normalized tokens (default: the winnowing guarantee threshold, 50)
      --min-distinct <MIN_DISTINCT>  Diversity floor: suppress blocks with fewer unique tokens (default 7, from the M2 calibration; 0 disables)
      --check                        Only-shrink ratchet: exit 1 when clone blocks exceed the ce.toml [dedup] budget (M2 review R12; the comparison is the core's verdict since ADR-008 P2)
      --core <CORE>                  Path to the ce-core executable, consulted by --check alone (default: CE_CORE_BIN, a ce-core beside this binary, then PATH) [default: ce-core]
  -h, --help                         Print help
```

## ce daemon

```text
Run the per-project daemon in the foreground (ADR-003); normally lazy-started by `ce ping` / hook probes

Usage: ce daemon [OPTIONS] <ROOT>

Arguments:
  <ROOT>  Project root to serve

Options:
      --lang <LANG>  Console language (wins over CE_LANG) [possible values: en, zh]
  -h, --help         Print help
```

## ce ping

```text
Round-trip a ping through the project daemon (lazy-starts it)

Usage: ce ping [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Project root (default: current directory)

Options:
      --lang <LANG>  Console language (wins over CE_LANG) [possible values: en, zh]
  -h, --help         Print help
```

## ce probe

```text
PreToolUse cheap gate: read the hook envelope on stdin, probe the daemon, emit a permission decision per ce.toml [guard]

Usage: ce probe [OPTIONS]

Options:
      --hook         Hook mode (the only mode in M3)
      --lang <LANG>  Console language (wins over CE_LANG) [possible values: en, zh]
  -h, --help         Print help
```

## ce audit

```text
Stop audit v1: net LOC + duplicate blocks touching changed files (blocks the stop only in deny mode)

Usage: ce audit [OPTIONS]

Options:
      --hook         Hook mode (the only mode in M3)
      --lang <LANG>  Console language (wins over CE_LANG) [possible values: en, zh]
  -h, --help         Print help
```

## ce health

```text
SessionStart health line + daemon warm-up

Usage: ce health [OPTIONS]

Options:
      --hook         Hook mode (the only mode in M3)
      --lang <LANG>  Console language (wins over CE_LANG) [possible values: en, zh]
  -h, --help         Print help
```

## ce precommit

```text
pre-commit gate: staged net LOC + touched duplicates (exit 1 in deny mode when duplicates are touched)

Usage: ce precommit [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Repository root (default: current directory)

Options:
      --lang <LANG>  Console language (wins over CE_LANG) [possible values: en, zh]
  -h, --help         Print help
```

## ce mcp

```text
MCP server over stdio: the read-only report face of every judgment family (M7-P2)

Usage: ce mcp [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Project root the tools operate on (default: current directory)

Options:
      --lang <LANG>  Console language (wins over CE_LANG) [possible values: en, zh]
  -h, --help         Print help
```

## ce eject

```text
Uninstall project state: .ce/, baseline, pins (dry-run default)

Usage: ce eject [OPTIONS] [ROOT]

Arguments:
  [ROOT]  Project root to eject (default: current directory)

Options:
      --lang <LANG>  Console language (wins over CE_LANG) [possible values: en, zh]
      --yes          Actually remove (default: dry run naming every target)
  -h, --help         Print help
```

