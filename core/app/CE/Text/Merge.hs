-- | The `ce merge` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/merge/console.rs.
module CE.Text.Merge (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "degraded\tmerge: degraded — {} (no judgment)\t合并：已降级——{}（未判决）\n\
    \counts\t{} group(s) judged, {} feasible, {} hole(s), {} duplicate(s) merged; not sent: {} not isomorphic, {} without a slot table, {} unbuilt, {} over the cap\t判决 {} 组，可行 {}，洞 {} 个，同集合并 {}；未送：不同构 {}、无位置类表 {}、树未建 {}、超上限 {}\n\
    \group\tgroup {}: {} · {} members · {} params · savings {} lines · {}\t组 {}：{} · {} 个成员 · {} 个参数 · 省 {} 行 · {}\n\
    \fragment\t{} (fragment)\n\
    \feasible\tfeasible\t可行\n\
    \infeasible\tinfeasible ({})\t不可行（{}）\n\
    \member\t  {}  lines {}-{}\t  {}  行 {}-{}\n\
    \member_run\t  {}  lines {}-{} (run {}-{})\t  {}  行 {}-{}（合并段 {}-{}）\n\
    \param\t  param {}: {}\t  参数 {}：{}\n\
    \cell\tm{} \"{}\""
