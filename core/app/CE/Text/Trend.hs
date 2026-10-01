-- | The `ce trend` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/trend/report.rs (`print_console`,
-- `print_verdict_tail`, `print_shape_facts`), the verdict words
-- (cli/src/trend/judge.rs `verdict_str`) and the exit veto's two
-- sentences (cli/src/main_judge.rs `trend_cmd`, behind the `trend
-- check: ` prefix of `emit_checked`, English under both languages).
-- ` -> FAIL` prints in English under both (the exit-code vocabulary).
module CE.Text.Trend (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "point\ttrend {} {} score {}/{} | axes {}\t趋势 {} {} 分数 {}/{} | 判轴 {}\n\
    \failed\ttrend {} FAILED: {}\t趋势 {} 失败：{}\n\
    \verdict\ttrend verdict: {}{}{}\t趋势判决：{}{}{}\n\
    \improving\timproving\t上行\n\
    \flat\tflat\t持平\n\
    \degrading\tdegrading\t恶化\n\
    \unjudged\tunjudged (below minPoints)\t未判（低于最小点数）\n\
    \slope\t (slope {}‰/day)\t（斜率 {}‰/日）\n\
    \cliff\ttrend cliff: -{}‰ into {}\t趋势断崖：-{}‰ 落在 {}\n\
    \decline\ttrend decline run: {} commits from {}\t趋势持续下行：{} 个提交，起于 {}\n\
    \window\ttrend window: {} commits, {} measured, {} pending\t趋势窗口：{} 个提交，已测 {}，待测 {}\n\
    \check\ttrend check: {}\n\
    \declined\tscore falls {}‰/day, past the declared {}‰/day decline floor\t分数每日下跌 {}‰，越过声明的每日 {}‰ 下行地板\n\
    \refused\t{} of {} window commits refused to measure: {} — {}\t窗口内 {} / {} 个提交拒绝测量：{} —— {}"
