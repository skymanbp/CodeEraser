# Benchmarks — replayed, never hand-filled

> Generated from [contracts/bench/bench.json](../contracts/bench/bench.json) by
> `cli/tests/it/bench_render.rs` (`CE_BLESS=1` to regenerate). Every series row was
> measured by `cli/tests/it/bench.rs` (`bench_append` for a checkout, `bench_backfill`
> per release tag — the tag's submodules seated, its OWN binaries, release builds
> only, fresh index per cold run). Frozen points carry their sealed-ledger source;
> points that cannot
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
> so when the shipped release is a different one.

The current release, v1.3.2, has no row of its own.

## Latency series (self repository)

| version | metric | p50 ms | p95 ms | n | host | measured |
|---|---|---|---|---|---|---|
| 0.1.0 | check_warm | 1086 | 1093 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.1.0 | deadcode_warm | 350 | 358 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.1.0 | dedup_cold | 1874 | 1877 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.1.0 | dedup_warm | 301 | 309 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.1.0 | docdup_warm | 444 | 451 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.1.0 | hook_probe | 32 | 38 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 0.1.0 | scan | 417 | 2588 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.2.0 | check_warm | 1332 | 1395 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.2.0 | deadcode_warm | 473 | 490 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.2.0 | dedup_cold | 2168 | 2241 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.2.0 | dedup_warm | 428 | 435 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.2.0 | docdup_warm | 554 | 566 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.2.0 | hook_probe | 33 | 36 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 0.2.0 | scan | 454 | 2857 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.3.0 | check_warm | 1370 | 1374 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.3.0 | deadcode_warm | 529 | 561 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.3.0 | dedup_cold | 2198 | 2204 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.3.0 | dedup_warm | 441 | 446 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.3.0 | docdup_warm | 609 | 618 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.3.0 | hook_probe | 42 | 45 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 0.3.0 | scan | 455 | 2989 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.4.0 | check_warm | 1308 | 1325 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.4.0 | deadcode_warm | 473 | 488 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.4.0 | dedup_cold | 2155 | 2184 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.4.0 | dedup_warm | 423 | 441 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.4.0 | docdup_warm | 554 | 563 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.4.0 | hook_probe | 34 | 45 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 0.4.0 | scan | 440 | 2832 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.5.0 | check_warm | 1304 | 1331 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.5.0 | deadcode_warm | 460 | 469 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.5.0 | dedup_cold | 2034 | 2042 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.5.0 | dedup_warm | 415 | 431 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.5.0 | docdup_warm | 534 | 632 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.5.0 | hook_probe | 36 | 46 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 0.5.0 | scan | 429 | 2836 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.6.0 | check_warm | 1364 | 1394 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.6.0 | deadcode_warm | 482 | 509 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.6.0 | dedup_cold | 2201 | 2271 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.6.0 | dedup_warm | 429 | 449 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.6.0 | docdup_warm | 550 | 584 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.6.0 | hook_probe | 37 | 49 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 0.6.0 | scan | 478 | 2884 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.0 | check_warm | 1363 | 1381 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.0 | deadcode_warm | 500 | 530 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.0 | dedup_cold | 2250 | 2259 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.0 | dedup_warm | 419 | 480 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.0 | docdup_warm | 563 | 583 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.0 | hook_probe | 37 | 46 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.0 | scan | 457 | 2981 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.1 | check_warm | 1327 | 1327 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.1 | deadcode_warm | 472 | 487 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.1 | dedup_cold | 2123 | 2142 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.1 | dedup_warm | 428 | 449 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.1 | docdup_warm | 546 | 562 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.1 | hook_probe | 34 | 42 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.1 | scan | 433 | 2794 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.2 | check_warm | 1315 | 1316 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.2 | deadcode_warm | 480 | 493 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.2 | dedup_cold | 2121 | 2138 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.2 | dedup_warm | 426 | 433 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.2 | docdup_warm | 580 | 592 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.2 | hook_probe | 34 | 42 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.2 | scan | 449 | 2384 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.3 | check_warm | 1338 | 1348 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.3 | deadcode_warm | 489 | 497 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.3 | dedup_cold | 2123 | 2158 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.3 | dedup_warm | 424 | 425 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.3 | docdup_warm | 574 | 578 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.3 | hook_probe | 35 | 37 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 0.7.3 | scan | 461 | 2314 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.0 | check_warm | 1151 | 1169 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.0 | deadcode_warm | 614 | 619 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.0 | dedup_cold | 2495 | 2497 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.0 | dedup_warm | 489 | 502 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.0 | docdup_warm | 777 | 789 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.0 | hook_probe | 34 | 36 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.0 | scan | 537 | 2875 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.1 | check_warm | 1135 | 1168 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.1 | deadcode_warm | 597 | 612 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.1 | dedup_cold | 2961 | 2974 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.1 | dedup_warm | 486 | 498 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.1 | docdup_warm | 751 | 754 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.1 | hook_probe | 35 | 40 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 1.0.1 | scan | 526 | 2378 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 1.1.0 | check_warm | 1167 | 1207 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.1.0 | deadcode_warm | 633 | 641 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.1.0 | dedup_cold | 3024 | 3033 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.1.0 | dedup_warm | 494 | 511 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 1.1.0 | docdup_warm | 787 | 801 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.1.0 | hook_probe | 35 | 48 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 1.1.0 | scan | 551 | 2838 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 1.2.0 | check_warm | 988 | 995 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.2.0 | deadcode_warm | 413 | 416 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.2.0 | dedup_cold | 3300 | 3317 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.2.0 | dedup_warm | 261 | 268 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 1.2.0 | docdup_warm | 579 | 588 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.2.0 | hook_probe | 35 | 42 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 1.2.0 | scan | 573 | 2869 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 1.3.0 | check_warm | 1078 | 1082 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.3.0 | deadcode_warm | 923 | 2093 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.3.0 | dedup_cold | 4738 | 4743 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.3.0 | dedup_warm | 381 | 384 | 5 | windows/x86_64/16cpu | 2026-08-30 |
| 1.3.0 | docdup_warm | 801 | 809 | 3 | windows/x86_64/16cpu | 2026-08-30 |
| 1.3.0 | hook_probe | 41 | 43 | 30 | windows/x86_64/16cpu | 2026-08-30 |
| 1.3.0 | scan | 518 | 2428 | 5 | windows/x86_64/16cpu | 2026-08-30 |

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

