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

The newest row, v1.5.1, is the release this build is.

## Latency series (self repository)

| version | metric | p50 ms | p95 ms | n | host | measured |
|---|---|---|---|---|---|---|
| 0.1.0 | check_warm | 1134 | 1223 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | deadcode_warm | 370 | 413 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | dedup_cold | 2485 | 2535 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | dedup_warm | 314 | 396 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | docdup_warm | 446 | 458 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | hook_probe | 37 | 47 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.1.0 | scan | 458 | 2506 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | check_warm | 1391 | 1425 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | deadcode_warm | 511 | 544 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | dedup_cold | 2689 | 2692 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | dedup_warm | 452 | 491 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | docdup_warm | 598 | 610 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | hook_probe | 36 | 53 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.2.0 | scan | 458 | 2444 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | check_warm | 1384 | 1409 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | deadcode_warm | 508 | 521 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | dedup_cold | 2746 | 2825 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | dedup_warm | 440 | 445 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | docdup_warm | 582 | 587 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | hook_probe | 34 | 43 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.3.0 | scan | 470 | 2447 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | check_warm | 1456 | 1463 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | deadcode_warm | 509 | 571 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | dedup_cold | 2756 | 2767 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | dedup_warm | 454 | 495 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | docdup_warm | 609 | 624 | 3 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | hook_probe | 34 | 41 | 30 | windows/x86_64/16cpu | 2026-09-01 |
| 0.4.0 | scan | 485 | 2522 | 5 | windows/x86_64/16cpu | 2026-09-01 |
| 0.5.0 | check_warm | 1267 | 1270 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.5.0 | deadcode_warm | 466 | 478 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.5.0 | dedup_cold | 2553 | 2613 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.5.0 | dedup_warm | 409 | 432 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 0.5.0 | docdup_warm | 524 | 557 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.5.0 | hook_probe | 36 | 53 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 0.5.0 | scan | 488 | 2501 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 0.6.0 | check_warm | 1375 | 1380 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.6.0 | deadcode_warm | 485 | 495 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.6.0 | dedup_cold | 2554 | 2604 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.6.0 | dedup_warm | 444 | 503 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 0.6.0 | docdup_warm | 584 | 602 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.6.0 | hook_probe | 35 | 56 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 0.6.0 | scan | 451 | 2384 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.0 | check_warm | 1455 | 1501 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.0 | deadcode_warm | 521 | 572 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.0 | dedup_cold | 2599 | 2673 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.0 | dedup_warm | 447 | 480 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.0 | docdup_warm | 626 | 645 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.0 | hook_probe | 36 | 40 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.0 | scan | 505 | 2545 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.2 | check_warm | 1322 | 1333 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.2 | deadcode_warm | 471 | 472 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.2 | dedup_cold | 2582 | 2593 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.2 | dedup_warm | 423 | 453 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.2 | docdup_warm | 551 | 563 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.2 | hook_probe | 38 | 40 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.2 | scan | 451 | 2347 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.3 | check_warm | 1357 | 1444 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.3 | deadcode_warm | 506 | 533 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.3 | dedup_cold | 2568 | 2605 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.3 | dedup_warm | 427 | 483 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.3 | docdup_warm | 597 | 599 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.3 | hook_probe | 36 | 40 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 0.7.3 | scan | 493 | 2423 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.0.0 | check_warm | 1173 | 1178 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.0.0 | deadcode_warm | 644 | 646 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.0.0 | dedup_cold | 3257 | 3487 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.0.0 | dedup_warm | 515 | 526 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.0.0 | docdup_warm | 765 | 768 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.0.0 | hook_probe | 36 | 41 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 1.0.0 | scan | 568 | 2552 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.1.0 | check_warm | 1240 | 1286 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.1.0 | deadcode_warm | 689 | 692 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.1.0 | dedup_cold | 3362 | 3436 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.1.0 | dedup_warm | 511 | 525 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.1.0 | docdup_warm | 823 | 882 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.1.0 | hook_probe | 36 | 43 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 1.1.0 | scan | 576 | 2545 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.2.0 | check_warm | 1104 | 1106 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.2.0 | deadcode_warm | 436 | 450 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.2.0 | dedup_cold | 3615 | 3627 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.2.0 | dedup_warm | 277 | 312 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.2.0 | docdup_warm | 618 | 635 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.2.0 | hook_probe | 37 | 44 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 1.2.0 | scan | 624 | 2607 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.3.0 | check_warm | 1105 | 1129 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.3.0 | deadcode_warm | 989 | 2158 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.3.0 | dedup_cold | 4748 | 5121 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.3.0 | dedup_warm | 392 | 442 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.3.0 | docdup_warm | 830 | 845 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.3.0 | hook_probe | 46 | 58 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 1.3.0 | scan | 576 | 2462 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.0 | check_warm | 1367 | 1384 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.0 | deadcode_warm | 1042 | 2385 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.0 | dedup_cold | 4444 | 4479 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.0 | dedup_warm | 413 | 438 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.0 | docdup_warm | 872 | 915 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.0 | hook_probe | 44 | 51 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.0 | scan | 613 | 2843 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.1 | check_warm | 1329 | 1412 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.1 | deadcode_warm | 1109 | 2336 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.1 | dedup_cold | 5118 | 5188 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.1 | dedup_warm | 513 | 991 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.1 | docdup_warm | 837 | 940 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.1 | hook_probe | 45 | 70 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 1.4.1 | scan | 729 | 2802 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.0 | check_warm | 1366 | 1447 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.0 | deadcode_warm | 1199 | 2351 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.0 | dedup_cold | 4271 | 4290 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.0 | dedup_warm | 410 | 431 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.0 | docdup_warm | 867 | 868 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.0 | hook_probe | 46 | 62 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.0 | scan | 616 | 2946 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.1 | check_warm | 1353 | 1395 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.1 | deadcode_warm | 1049 | 2234 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.1 | dedup_cold | 4247 | 4290 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.1 | dedup_warm | 400 | 435 | 5 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.1 | docdup_warm | 879 | 944 | 3 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.1 | hook_probe | 44 | 48 | 30 | windows/x86_64/16cpu | 2026-09-02 |
| 1.5.1 | scan | 612 | 2460 | 5 | windows/x86_64/16cpu | 2026-09-02 |

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

