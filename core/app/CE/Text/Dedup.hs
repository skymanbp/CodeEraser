-- | The `ce dedup` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/dedup/report.rs (`print_console`) and the
-- `--check` ratchet lines (cli/src/dedup/budget.rs `check`).
module CE.Text.Dedup (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "dup\tdup {}:{}-{} <-> {}:{}-{} ({} tokens)\t重复 {}:{}-{} <-> {}:{}-{}（{} tokens）\n\
    \summary\tindexed {} files ({} refreshed, {} removed) — {} clone blocks in {} groups (min {} tokens, distinct >= {}), {} low-diversity suppressed, {} hot chained, {} stale skipped\t已索引 {} 个文件（刷新 {}，移除 {}）— {} 个克隆块 / {} 组（最少 {} tokens，多样性 >= {}），抑制低多样性 {}，热链 {}，跳过陈旧 {}\n\
    \over\tdedup ratchet: {} clone blocks > budget {} — new duplication must not land\t去冗棘轮：{} 个克隆块 > 预算 {} — 新增重复不得落地\n\
    \under\tdedup ratchet: {} clone blocks < budget {} — ratchet the budget down\t去冗棘轮：{} 个克隆块 < 预算 {} — 请把预算下调咬合"
