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
> so when the shipped release is a different one. Both row writers apply the
> rule: the backfill names every tag it turns away, and a checkout that
> brings nothing new is refused rather than measured a second time. So the
> table cannot gain a row the rule forbids, and a release that earns one
> says which of the two reasons it has none yet.

The current release, v1.5.1, earns a row and does not have one yet: the whole series is replayed in one sitting after the tag.

## Latency series (self repository)

| version | metric | p50 ms | p95 ms | n | host | measured |
|---|---|---|---|---|---|---|
| 0.1.0 | check_warm | 1160 | 1202 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | deadcode_warm | 410 | 427 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | dedup_cold | 2396 | 2458 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | dedup_warm | 333 | 354 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | docdup_warm | 464 | 469 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | hook_probe | 44 | 55 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | scan | 462 | 2292 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | check_warm | 1539 | 1577 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | deadcode_warm | 572 | 600 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | dedup_cold | 2854 | 2897 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | dedup_warm | 498 | 530 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | docdup_warm | 716 | 767 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | hook_probe | 42 | 60 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | scan | 542 | 2562 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | check_warm | 2241 | 2888 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | deadcode_warm | 601 | 626 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | dedup_cold | 3826 | 5200 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | dedup_warm | 602 | 795 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | docdup_warm | 685 | 698 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | hook_probe | 112 | 194 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | scan | 842 | 2745 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | check_warm | 1524 | 1538 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | deadcode_warm | 762 | 858 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | dedup_cold | 3114 | 3380 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | dedup_warm | 564 | 630 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | docdup_warm | 669 | 697 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | hook_probe | 36 | 40 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | scan | 714 | 3055 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | check_warm | 1757 | 1877 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | deadcode_warm | 483 | 493 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | dedup_cold | 2446 | 2457 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | dedup_warm | 422 | 427 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | docdup_warm | 566 | 586 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | hook_probe | 58 | 63 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | scan | 455 | 2406 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | check_warm | 1360 | 1676 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | deadcode_warm | 489 | 494 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | dedup_cold | 2714 | 2876 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | dedup_warm | 442 | 475 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | docdup_warm | 563 | 566 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | hook_probe | 36 | 41 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.6.0 | scan | 506 | 2509 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | check_warm | 2105 | 2230 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | deadcode_warm | 726 | 796 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | dedup_cold | 3219 | 3352 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | dedup_warm | 642 | 683 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | docdup_warm | 860 | 919 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | hook_probe | 71 | 99 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.0 | scan | 695 | 3199 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | check_warm | 1355 | 1370 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | deadcode_warm | 491 | 500 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | dedup_cold | 2173 | 2202 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | dedup_warm | 457 | 484 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | docdup_warm | 557 | 559 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | hook_probe | 39 | 43 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.2 | scan | 491 | 2271 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | check_warm | 1671 | 1689 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | deadcode_warm | 591 | 718 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | dedup_cold | 2341 | 2618 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | dedup_warm | 511 | 582 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | docdup_warm | 718 | 727 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | hook_probe | 50 | 73 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.7.3 | scan | 572 | 2456 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | check_warm | 1262 | 1273 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | deadcode_warm | 680 | 701 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | dedup_cold | 3163 | 3198 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | dedup_warm | 552 | 598 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | docdup_warm | 856 | 933 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | hook_probe | 42 | 47 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 1.0.0 | scan | 722 | 2799 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | check_warm | 1288 | 1289 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | deadcode_warm | 697 | 702 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | dedup_cold | 2797 | 2947 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | dedup_warm | 564 | 574 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | docdup_warm | 867 | 886 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | hook_probe | 42 | 46 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 1.1.0 | scan | 640 | 2625 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | check_warm | 1042 | 1049 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | deadcode_warm | 436 | 442 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | dedup_cold | 2941 | 3039 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | dedup_warm | 278 | 290 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | docdup_warm | 610 | 613 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | hook_probe | 38 | 45 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 1.2.0 | scan | 601 | 2542 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | check_warm | 1164 | 1181 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | deadcode_warm | 1092 | 2201 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | dedup_cold | 4323 | 4556 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | dedup_warm | 400 | 407 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | docdup_warm | 865 | 905 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | hook_probe | 47 | 52 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 1.3.0 | scan | 544 | 2432 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | check_warm | 1270 | 1318 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | deadcode_warm | 959 | 2133 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | dedup_cold | 4571 | 4612 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | dedup_warm | 391 | 411 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | docdup_warm | 824 | 858 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | hook_probe | 47 | 173 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.0 | scan | 600 | 2539 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.1 | check_warm | 1323 | 1348 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.1 | deadcode_warm | 1024 | 2311 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.1 | dedup_cold | 4654 | 4770 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.1 | dedup_warm | 405 | 417 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.1 | docdup_warm | 850 | 855 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.1 | hook_probe | 51 | 57 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 1.4.1 | scan | 606 | 2633 | 5 | windows/x86_64/16cpu | 2026-09-01 |

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

