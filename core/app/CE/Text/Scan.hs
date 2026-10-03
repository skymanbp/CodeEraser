-- | The `ce scan` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/scan/report.rs (`print_console`) and the
-- named-failure suffix (cli/src/report.rs `fail_suffix`), both deleted
-- once the console read these lines. The `FAIL` /
-- `warn` tags and ` -> FAIL` print in English under both languages
-- (the exit-code vocabulary).
module CE.Text.Scan (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "finding\t{} {}:{} {} = {} (limit {}) [{}]\t{} {}:{} {} = {}（上限 {}）[{}]\n\
    \summary\tscanned {} files / {} functions — {} warn, {} fail{}\t已扫描 {} 文件 / {} 函数 — {} warn，{} fail{}\n\
    \fail\t -> FAIL{}\n\
    \failed\t (failed: {})\t（失败条件：{}）"
