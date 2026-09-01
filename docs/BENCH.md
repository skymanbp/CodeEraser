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
> release joins it: every row shares one measured date, and rows carrying
> different dates are not comparable.
>
> A release joins only when there is something new to measure. One that ships
> the same `cli/src` and `core/app` as its predecessor gets no row of its own:
> replaying the whole series to add a duplicate measurement would publish that
> drift under a new version number. So every surface printing these numbers
> beside a version names the version MEASURED — never "the latest" — and says
> so when the shipped release is a different one. The backfill driver applies
> this rule itself and names every tag it turns away, so the table and the
> rule cannot drift apart again.

The current release, v1.4.1, has no row of its own.

## Latency series (self repository)

| version | metric | p50 ms | p95 ms | n | host | measured |
|---|---|---|---|---|---|---|
| 0.1.0 | check_warm | 2264 | 2338 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | deadcode_warm | 741 | 762 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | dedup_cold | 2809 | 3735 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | dedup_warm | 362 | 366 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | docdup_warm | 962 | 999 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | hook_probe | 42 | 64 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | scan | 528 | 2473 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | check_warm | 1388 | 1440 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | deadcode_warm | 505 | 542 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | dedup_cold | 2500 | 2542 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | dedup_warm | 446 | 475 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | docdup_warm | 584 | 587 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | hook_probe | 38 | 63 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | scan | 519 | 3050 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | check_warm | 1375 | 1391 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | deadcode_warm | 519 | 529 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | dedup_cold | 2424 | 2515 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | dedup_warm | 449 | 474 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | docdup_warm | 662 | 662 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | hook_probe | 40 | 66 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | scan | 468 | 2915 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | check_warm | 1292 | 1315 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | deadcode_warm | 473 | 475 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | dedup_cold | 3367 | 3720 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | dedup_warm | 446 | 480 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | docdup_warm | 544 | 545 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | hook_probe | 32 | 56 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | scan | 790 | 2893 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | check_warm | 2267 | 2299 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | deadcode_warm | 607 | 836 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | dedup_cold | 2394 | 2404 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | dedup_warm | 653 | 760 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | docdup_warm | 646 | 684 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | hook_probe | 68 | 95 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | scan | 446 | 2823 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | check_warm | 1374 | 1541 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | deadcode_warm | 538 | 726 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | dedup_cold | 2428 | 2481 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | dedup_warm | 419 | 458 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | docdup_warm | 709 | 752 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | hook_probe | 36 | 38 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | scan | 436 | 2750 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | check_warm | 2091 | 2402 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | deadcode_warm | 516 | 520 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | dedup_cold | 2610 | 2622 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | dedup_warm | 429 | 435 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | docdup_warm | 827 | 907 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | hook_probe | 37 | 71 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | scan | 470 | 2945 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | check_warm | 1294 | 1322 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | deadcode_warm | 478 | 480 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | dedup_cold | 2474 | 2659 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | dedup_warm | 414 | 435 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | docdup_warm | 551 | 553 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | hook_probe | 36 | 58 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | scan | 434 | 2329 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | check_warm | 1387 | 1578 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | deadcode_warm | 931 | 941 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | dedup_cold | 3009 | 3786 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | dedup_warm | 720 | 770 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | docdup_warm | 1095 | 1110 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | hook_probe | 36 | 38 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | scan | 480 | 2371 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | check_warm | 1139 | 1141 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | deadcode_warm | 601 | 624 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | dedup_cold | 3857 | 5052 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | dedup_warm | 480 | 537 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | docdup_warm | 747 | 764 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | hook_probe | 35 | 56 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | scan | 960 | 2990 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | check_warm | 1474 | 2137 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | deadcode_warm | 663 | 685 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | dedup_cold | 3041 | 3062 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | dedup_warm | 517 | 539 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | docdup_warm | 821 | 852 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | hook_probe | 46 | 67 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | scan | 553 | 2978 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | check_warm | 1123 | 1149 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | deadcode_warm | 427 | 435 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | dedup_cold | 3421 | 3572 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | dedup_warm | 271 | 276 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | docdup_warm | 604 | 604 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | hook_probe | 39 | 58 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | scan | 786 | 4980 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | check_warm | 1471 | 1536 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | deadcode_warm | 1293 | 2996 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | dedup_cold | 7340 | 7582 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | dedup_warm | 568 | 625 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | docdup_warm | 1197 | 1270 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | hook_probe | 61 | 88 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | scan | 1007 | 5066 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | check_warm | 2396 | 2501 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | deadcode_warm | 1717 | 2543 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | dedup_cold | 4448 | 4505 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | dedup_warm | 411 | 425 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | docdup_warm | 910 | 1214 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | hook_probe | 55 | 80 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | scan | 588 | 2513 | 5 | windows/x86_64/16cpu | 2026-09-01 |

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

