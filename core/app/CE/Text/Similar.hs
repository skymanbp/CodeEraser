-- | The `ce similar` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/similar/face.rs (`console`,
-- `widened_note`, `degraded_note`). A candidate's row and its `-` / `?`
-- role marks print the same under both languages.
module CE.Text.Similar (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "head\tsimilar: {} — {} query term(s), {} candidate(s), {} same-role{}{}\tsimilar：{} — {} 个查询项、{} 个候选、{} 个同角色{}{}\n\
    \widened\t, {} from the associative view\t，联想视图另加 {} 个\n\
    \degraded\t — degraded: {} (measured order, no role bits)\t — 已降级：{}（按度量序、无角色位）\n\
    \candidate\t  {} {}  {}  {}{}\n\
    \same\tsame-role\t同角色\n\
    \associative\t (associative)\t（联想）"
