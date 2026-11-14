-- | The `ce check` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/score/report.rs (`print`,
-- `print_ratchet_tail`, `roast_line`) and the named-failure suffix
-- (cli/src/report.rs `fail_suffix`), both deleted once the console
-- read these lines. `FAIL` / `pass` print in English
-- under both languages (the exit-code vocabulary).
module CE.Text.Check (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "head\tcheck score {}/{} | findings on {} of {} lines | per 1000 lines: size {} complexity {} clone {} docdup {} unreferenced {} churn {} cycle {} | {} candidates\t检查分数 {}/{} | 有问题的行 {} / {} | 每千行：尺寸 {} 复杂度 {} 克隆 {} 文档重复 {} 无引用 {} 变动 {} 循环 {} | 候选 {}\n\
    \axis_off\t{} (off)\t{}（不计）\n\
    \dead_added\tdead_added: {} unreferenced file(s) blocked (the roots of the files in a state [deadcode] block names) — ce deadcode --check lists them\tdead_added：{} 个无引用文件被拦（处于 [deadcode] block 点名状态的文件中的根）——ce deadcode --check 会列出它们\n\
    \ratchet\tratchet: {} added, {} removed, {} over, {} tolerance drawn -> {}\t棘轮：新增 {}，移除 {}，超限 {}，动用容差 {} -> {}\n\
    \fail\tFAIL{}\n\
    \pass\tpass\n\
    \failed\t (failed: {})\t（失败条件：{}）\n\
    \note\tnote: {} blocks collapsed into existing members, {} intra-file pairs off the sim table\t注：{} 块并入既有成员，{} 个文件内对不入相似表\n\
    \degraded\tcheck degraded: {} -> FAIL (a gate that cannot judge must not pass)\t检查降级：{} -> FAIL（不能判决的门绝不放行）\n\
    \roast_clean\troast: suspiciously clean. who did you pay?\t毒舌：干净得可疑。你贿赂谁了？\n\
    \roast_slow\troast: entropy is losing, slowly. keep the eraser warm.\t毒舌：熵在缓慢败退。橡皮别放凉。\n\
    \roast_half\troast: half garden, half landfill. bring the shovel.\t毒舌：半是花园半是垃圾场。铲子带上。\n\
    \roast_exorcist\troast: this repo needs an exorcist, not an eraser.\t毒舌：这仓库需要的不是橡皮，是驱魔师。"
