# Benchmarks — replayed, never hand-filled

> Generated from [contracts/bench/bench.json](../contracts/bench/bench.json) by
> `cli/tests/it/bench_render.rs` (`CE_BLESS=1` to regenerate). Every series row was
> measured by `cli/tests/it/bench.rs` (`bench_append` for a checkout, `bench_backfill`
> per release tag — each tag's OWN binaries, release builds only, fresh index per
> cold run). Frozen points carry their sealed-ledger source; points that cannot
> honestly become a per-version series say why in their epoch clause.
>
> The whole series is ONE machine — the host column repeats because it never
> varies. That is a feature for the only comparison this table makes
> (version-over-version on constant hardware) and a warning about the one it
> cannot: none of these milliseconds transfer to other hardware, and no CI
> runner replays them (PERF-BUDGET.md opens with why a shared runner cannot
> host a latency budget).

## Latency series (self repository)

| version | metric | p50 ms | p95 ms | n | host | measured |
|---|---|---|---|---|---|---|
| 0.1.0 | check_warm | 1098 | 1140 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.1.0 | deadcode_warm | 426 | 446 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.1.0 | dedup_cold | 2363 | 2420 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.1.0 | dedup_warm | 315 | 339 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.1.0 | docdup_warm | 533 | 636 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.1.0 | hook_probe | 40 | 52 | 30 | windows/x86_64/16cpu | 2026-08-21 |
| 0.1.0 | scan | 456 | 2291 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.2.0 | check_warm | 1549 | 1590 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.2.0 | deadcode_warm | 543 | 558 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.2.0 | dedup_cold | 3136 | 3219 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.2.0 | dedup_warm | 505 | 608 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.2.0 | docdup_warm | 636 | 663 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.2.0 | hook_probe | 39 | 55 | 30 | windows/x86_64/16cpu | 2026-08-21 |
| 0.2.0 | scan | 544 | 2572 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.3.0 | check_warm | 1343 | 1365 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.3.0 | deadcode_warm | 495 | 526 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.3.0 | dedup_cold | 2824 | 2889 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.3.0 | dedup_warm | 440 | 461 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.3.0 | docdup_warm | 576 | 598 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.3.0 | hook_probe | 33 | 35 | 30 | windows/x86_64/16cpu | 2026-08-21 |
| 0.3.0 | scan | 484 | 2276 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.4.0 | check_warm | 1364 | 1400 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.4.0 | deadcode_warm | 555 | 573 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.4.0 | dedup_cold | 3122 | 3166 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.4.0 | dedup_warm | 450 | 481 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.4.0 | docdup_warm | 587 | 744 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.4.0 | hook_probe | 34 | 36 | 30 | windows/x86_64/16cpu | 2026-08-21 |
| 0.4.0 | scan | 512 | 2358 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.5.0 | check_warm | 1440 | 1522 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.5.0 | deadcode_warm | 490 | 513 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.5.0 | dedup_cold | 2909 | 3021 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.5.0 | dedup_warm | 434 | 464 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.5.0 | docdup_warm | 542 | 548 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.5.0 | hook_probe | 41 | 64 | 30 | windows/x86_64/16cpu | 2026-08-21 |
| 0.5.0 | scan | 480 | 2416 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.6.0 | check_warm | 1410 | 1449 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.6.0 | deadcode_warm | 702 | 865 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.6.0 | dedup_cold | 3759 | 3840 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.6.0 | dedup_warm | 565 | 579 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.6.0 | docdup_warm | 833 | 879 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.6.0 | hook_probe | 43 | 56 | 30 | windows/x86_64/16cpu | 2026-08-21 |
| 0.6.0 | scan | 530 | 2896 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.0 | check_warm | 1404 | 1514 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.0 | deadcode_warm | 518 | 541 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.0 | dedup_cold | 3746 | 3913 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.0 | dedup_warm | 481 | 542 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.0 | docdup_warm | 603 | 653 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.0 | hook_probe | 41 | 57 | 30 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.0 | scan | 536 | 2526 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.1 | check_warm | 1917 | 2100 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.1 | deadcode_warm | 553 | 575 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.1 | dedup_cold | 2718 | 2798 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.1 | dedup_warm | 487 | 551 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.1 | docdup_warm | 691 | 698 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.1 | hook_probe | 59 | 73 | 30 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.1 | scan | 587 | 2098 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.2 | check_warm | 1571 | 1650 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.2 | deadcode_warm | 532 | 565 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.2 | dedup_cold | 2977 | 3017 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.2 | dedup_warm | 503 | 555 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.2 | docdup_warm | 635 | 652 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.2 | hook_probe | 58 | 128 | 30 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.2 | scan | 573 | 1994 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.3 | check_warm | 2065 | 2113 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.3 | deadcode_warm | 602 | 613 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.3 | dedup_cold | 2752 | 2775 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.3 | dedup_warm | 496 | 543 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.3 | docdup_warm | 657 | 724 | 3 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.3 | hook_probe | 54 | 79 | 30 | windows/x86_64/16cpu | 2026-08-21 |
| 0.7.3 | scan | 516 | 2598 | 5 | windows/x86_64/16cpu | 2026-08-21 |
| 1.0.0 | check_warm | 1449 | 1469 | 3 | windows/x86_64/16cpu | 2026-08-22 |
| 1.0.0 | deadcode_warm | 844 | 847 | 3 | windows/x86_64/16cpu | 2026-08-22 |
| 1.0.0 | dedup_cold | 4456 | 4739 | 3 | windows/x86_64/16cpu | 2026-08-22 |
| 1.0.0 | dedup_warm | 687 | 723 | 5 | windows/x86_64/16cpu | 2026-08-22 |
| 1.0.0 | docdup_warm | 893 | 1079 | 3 | windows/x86_64/16cpu | 2026-08-22 |
| 1.0.0 | hook_probe | 41 | 50 | 30 | windows/x86_64/16cpu | 2026-08-22 |
| 1.0.0 | scan | 771 | 960 | 5 | windows/x86_64/16cpu | 2026-08-22 |
| 1.0.1 | check_warm | 1616 | 1645 | 3 | windows/x86_64/16cpu | 2026-08-23 |
| 1.0.1 | deadcode_warm | 856 | 877 | 3 | windows/x86_64/16cpu | 2026-08-23 |
| 1.0.1 | dedup_cold | 3252 | 3374 | 3 | windows/x86_64/16cpu | 2026-08-23 |
| 1.0.1 | dedup_warm | 665 | 679 | 5 | windows/x86_64/16cpu | 2026-08-23 |
| 1.0.1 | docdup_warm | 1073 | 1085 | 3 | windows/x86_64/16cpu | 2026-08-23 |
| 1.0.1 | hook_probe | 68 | 93 | 30 | windows/x86_64/16cpu | 2026-08-23 |
| 1.0.1 | scan | 785 | 2995 | 5 | windows/x86_64/16cpu | 2026-08-23 |
| 1.1.0 | check_warm | 1402 | 1439 | 3 | windows/x86_64/16cpu | 2026-08-24 |
| 1.1.0 | deadcode_warm | 699 | 769 | 3 | windows/x86_64/16cpu | 2026-08-24 |
| 1.1.0 | dedup_cold | 3675 | 3709 | 3 | windows/x86_64/16cpu | 2026-08-24 |
| 1.1.0 | dedup_warm | 554 | 567 | 5 | windows/x86_64/16cpu | 2026-08-24 |
| 1.1.0 | docdup_warm | 871 | 899 | 3 | windows/x86_64/16cpu | 2026-08-24 |
| 1.1.0 | hook_probe | 43 | 55 | 30 | windows/x86_64/16cpu | 2026-08-24 |
| 1.1.0 | scan | 728 | 2869 | 5 | windows/x86_64/16cpu | 2026-08-24 |
| 1.2.0 | check_warm | 1111 | 1131 | 3 | windows/x86_64/16cpu | 2026-08-26 |
| 1.2.0 | deadcode_warm | 450 | 462 | 3 | windows/x86_64/16cpu | 2026-08-26 |
| 1.2.0 | dedup_cold | 2958 | 2979 | 3 | windows/x86_64/16cpu | 2026-08-26 |
| 1.2.0 | dedup_warm | 267 | 275 | 5 | windows/x86_64/16cpu | 2026-08-26 |
| 1.2.0 | docdup_warm | 621 | 659 | 3 | windows/x86_64/16cpu | 2026-08-26 |
| 1.2.0 | hook_probe | 34 | 37 | 30 | windows/x86_64/16cpu | 2026-08-26 |
| 1.2.0 | scan | 586 | 609 | 5 | windows/x86_64/16cpu | 2026-08-26 |

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

