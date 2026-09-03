# Benchmarks — replayed, never hand-filled

> Generated from [contracts/bench/bench.json](../contracts/bench/bench.json) by
> `cli/tests/it/bench_render.rs` (`CE_BLESS=1` to regenerate). Every series row was
> measured by `cli/tests/it/bench.rs` (`bench_append`, for a checkout) or by
> `cli/tests/it/bench_backfill.rs` (`bench_backfill`, per release tag — the tag’s
> submodules seated, its OWN binaries, release builds only, fresh index per cold
> run). Frozen points carry their sealed-ledger source; points that cannot
> honestly become a per-version series say why in their epoch clause.
>
> Six of the seven metrics run against this repository. `hook_probe` does
> not: it times `ce probe --hook` against a seeded two-file fixture rebuilt
> identically for every tag, so the write-time probe stays comparable across
> versions instead of tracking the tree's growth.
>
> The whole series is ONE machine — the host column repeats because it never
> varies. That is a feature for the only comparison this table makes
> (version-over-version on constant hardware) and a warning about the one it
> cannot: none of these milliseconds transfer to other hardware, and no CI
> runner replays them (PERF-BUDGET.md opens with why a shared runner cannot
> host a latency budget).
>
> One machine is not one machine-state. v1.2.0's row was first taken on
> 2026-08-26; replaying that same tag four days later — its own tree, its own
> binaries — moved every one of its seven metrics, from 11 % faster to 12 %
> slower, which is wider than most deltas a reader would try to read out of
> this table. So the series is replayed WHOLE, in one sitting, whenever a
> release joins it, and a tag whose minutes were disturbed is measured again
> alone on that same day: every row shares one measured date (a test below
> holds that line), and rows carrying different dates are not comparable.
>
> A release joins only when there is something new to measure. One that ships
> the same `cli/src` and `core/app` as its predecessor gets no row of its own:
> replaying the whole series to add a duplicate measurement would publish that
> drift under a new version number. So every surface printing these numbers
> beside a version names the version MEASURED — never "the latest" — and says
> so when the shipped release is a different one. Both row writers apply the
> rule: the backfill names every tag it turns away, and a checkout that
> brings nothing new is refused rather than measured a second time. So the
> table cannot gain a row the rule forbids, and a release that earns one
> says which of the two reasons it has none yet.

The newest row, v1.5.1, is the release this build is.

## Latency series (self repository)

| version | metric | p50 ms | p95 ms | n | host | measured |
|---|---|---|---|---|---|---|
| 0.1.0 | check_warm | 1012 | 1013 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.1.0 | deadcode_warm | 341 | 342 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.1.0 | dedup_cold | 1831 | 1842 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.1.0 | dedup_warm | 289 | 292 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.1.0 | docdup_warm | 413 | 414 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.1.0 | hook_probe | 31 | 34 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 0.1.0 | scan | 416 | 2249 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.2.0 | check_warm | 1243 | 1251 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.2.0 | deadcode_warm | 457 | 463 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.2.0 | dedup_cold | 2008 | 2015 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.2.0 | dedup_warm | 410 | 422 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.2.0 | docdup_warm | 527 | 531 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.2.0 | hook_probe | 30 | 33 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 0.2.0 | scan | 421 | 2332 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.3.0 | check_warm | 1245 | 1267 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.3.0 | deadcode_warm | 466 | 476 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.3.0 | dedup_cold | 2012 | 2022 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.3.0 | dedup_warm | 407 | 414 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.3.0 | docdup_warm | 531 | 531 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.3.0 | hook_probe | 30 | 31 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 0.3.0 | scan | 434 | 2334 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.4.0 | check_warm | 1250 | 1251 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.4.0 | deadcode_warm | 472 | 480 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.4.0 | dedup_cold | 2037 | 2043 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.4.0 | dedup_warm | 411 | 418 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.4.0 | docdup_warm | 532 | 551 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.4.0 | hook_probe | 31 | 35 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 0.4.0 | scan | 426 | 2266 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.5.0 | check_warm | 1227 | 1228 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.5.0 | deadcode_warm | 449 | 455 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.5.0 | dedup_cold | 1960 | 1989 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.5.0 | dedup_warm | 400 | 403 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.5.0 | docdup_warm | 519 | 522 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.5.0 | hook_probe | 33 | 36 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 0.5.0 | scan | 419 | 2684 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.6.0 | check_warm | 1253 | 1257 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.6.0 | deadcode_warm | 469 | 469 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.6.0 | dedup_cold | 2015 | 2051 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.6.0 | dedup_warm | 408 | 413 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.6.0 | docdup_warm | 528 | 538 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.6.0 | hook_probe | 33 | 37 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 0.6.0 | scan | 433 | 2674 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.0 | check_warm | 1260 | 1268 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.0 | deadcode_warm | 469 | 470 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.0 | dedup_cold | 2037 | 2053 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.0 | dedup_warm | 409 | 411 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.0 | docdup_warm | 531 | 537 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.0 | hook_probe | 33 | 36 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.0 | scan | 430 | 2695 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.2 | check_warm | 1248 | 1269 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.2 | deadcode_warm | 463 | 465 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.2 | dedup_cold | 2037 | 2038 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.2 | dedup_warm | 415 | 417 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.2 | docdup_warm | 540 | 541 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.2 | hook_probe | 33 | 35 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.2 | scan | 437 | 2678 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.3 | check_warm | 1297 | 1305 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.3 | deadcode_warm | 494 | 503 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.3 | dedup_cold | 2016 | 2033 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.3 | dedup_warm | 409 | 418 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.3 | docdup_warm | 558 | 559 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.3 | hook_probe | 33 | 36 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 0.7.3 | scan | 464 | 2353 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.0.0 | check_warm | 1130 | 1137 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.0.0 | deadcode_warm | 595 | 600 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.0.0 | dedup_cold | 2445 | 2468 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.0.0 | dedup_warm | 482 | 486 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.0.0 | docdup_warm | 739 | 745 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.0.0 | hook_probe | 34 | 39 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 1.0.0 | scan | 520 | 2817 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.1.0 | check_warm | 1171 | 1193 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.1.0 | deadcode_warm | 629 | 664 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.1.0 | dedup_cold | 2519 | 2529 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.1.0 | dedup_warm | 496 | 503 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.1.0 | docdup_warm | 796 | 798 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.1.0 | hook_probe | 34 | 36 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 1.1.0 | scan | 547 | 2826 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.2.0 | check_warm | 984 | 984 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.2.0 | deadcode_warm | 415 | 417 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.2.0 | dedup_cold | 2785 | 2809 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.2.0 | dedup_warm | 263 | 264 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.2.0 | docdup_warm | 578 | 582 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.2.0 | hook_probe | 34 | 38 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 1.2.0 | scan | 572 | 2843 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.3.0 | check_warm | 1088 | 1088 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.3.0 | deadcode_warm | 925 | 2075 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.3.0 | dedup_cold | 3899 | 3905 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.3.0 | dedup_warm | 377 | 386 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.3.0 | docdup_warm | 792 | 837 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.3.0 | hook_probe | 42 | 45 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 1.3.0 | scan | 527 | 2822 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.0 | check_warm | 1268 | 1269 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.0 | deadcode_warm | 979 | 2183 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.0 | dedup_cold | 4062 | 4080 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.0 | dedup_warm | 391 | 393 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.0 | docdup_warm | 819 | 825 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.0 | hook_probe | 42 | 45 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.0 | scan | 590 | 2450 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.1 | check_warm | 1253 | 1278 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.1 | deadcode_warm | 953 | 2202 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.1 | dedup_cold | 4031 | 4049 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.1 | dedup_warm | 389 | 391 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.1 | docdup_warm | 819 | 823 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.1 | hook_probe | 42 | 46 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 1.4.1 | scan | 588 | 2484 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.0 | check_warm | 1256 | 1288 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.0 | deadcode_warm | 953 | 2190 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.0 | dedup_cold | 4043 | 4085 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.0 | dedup_warm | 395 | 398 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.0 | docdup_warm | 820 | 835 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.0 | hook_probe | 42 | 47 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.0 | scan | 584 | 2469 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.1 | check_warm | 1269 | 1296 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.1 | deadcode_warm | 950 | 2204 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.1 | dedup_cold | 4080 | 4092 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.1 | dedup_warm | 398 | 413 | 5 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.1 | docdup_warm | 832 | 836 | 3 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.1 | hook_probe | 42 | 45 | 30 | windows/x86_64/16cpu | 2026-09-03 |
| 1.5.1 | scan | 585 | 2463 | 5 | windows/x86_64/16cpu | 2026-09-03 |

## Frozen evaluation points

| metric | value | source |
|---|---|---|
| docdup_d3_precision | 17/17 scoped (100%) | docs/EVAL-SET-M5-3.md:81-87 + contracts/eval/docdup-precision-*-v1.json |
| docdup_d1_recall | 100% | docs/EVAL-SET-M5-3.md:81-87 + contracts/eval/docdup-precision-*-v1.json |
| t3_precision | 61 answered / 0 wrong (1.000) | docs/EVAL-SET-M5-3.md:41-47 + contracts/eval/t3-precision-*-v1.json |
| graph_precision | overall gate >= 0.90 held | docs/EVAL-SET.md:280-292 + contracts/eval/graph-precision-*-v1.json |
| fourclass_fpr | 0/600 flagged (gate <= 1%) | contracts/eval/fpr-fourclass-v1.json + docs/EVAL-SET.md:131-140 |
| guard_fpr_per500 | 0.00 per 500 edits | docs/FPR-REPLAY.md:16-36 + :47-94 |
| l2_moved_recall | 547/547 cross-file moved lines | docs/EVAL-SET.md:97-129 + contracts/eval/commit-l2*-v1.json |
| dedup_recall_vs_jscpd | cobra 106/109 raw -> 106/106 attributed | contracts/fixtures/crosscheck/DEDUP-CALIBRATION.md:96-137 |
| t3_recall_vs_similarity | zod 0.50 / requests 0.158 / cobra 0.154 (raw) | docs/EVAL-SET-M5-CLOSE.md:38-63 |

Per-point detail, freeze dates and epoch clauses live in the JSON itself.

