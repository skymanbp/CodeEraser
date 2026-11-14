-- | The `ce docdup` console sentences (plan v2.32 step 5; moved here
-- byte for byte from the measuring side at 59d17286): the `dups: `
-- prefix prints in English under both languages, and so does the
-- `--check` veto's `docdup check: ` prefix.
module CE.Text.Docdup (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "hit\tdocdup {} <-> {}  J {}/{} verbatim {}\t文档重复 {} <-> {}  J {}/{} 逐字 {}\n\
    \seg\t{}:{}-{} {}\n\
    \summary\tdups: {} duplicate pair(s) over {} live segment(s) — {} judged, {} by Jaccard{}\tdups: {} 对文档重复 / {} 个活段 — 判决 {}，其中 Jaccard 命中 {}{}\n\
    \check\tdocdup check: {} reported duplication(s) — resolve or exempt them\tdocdup check: {} 处重复被报告 — 请解决或豁免"
