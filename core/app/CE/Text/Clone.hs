-- | The `ce clone` console sentences (plan v2.32 step 5; moved here
-- byte for byte from the measuring side at 59d17286): the `clones: `
-- prefix prints in English under both languages, and the `--units`
-- listing's unit row is English only.
module CE.Text.Clone (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "hit\tclone {} <-> {}  ted {} (nodes {}/{})\t克隆 {} <-> {}  ted {}（节点 {}/{}）\n\
    \summary\tclones: {} near-miss clone pair(s) over {} unit(s) — {} judged, {} replayed from the verdict cache, {} provably below threshold{}\tclones: {} 对近似克隆 / {} 个单元 — 判决 {}，判决缓存回放 {}，可证低于阈值预滤 {}{}\n\
    \unit\t{}  {}#{}  {} nodes\n\
    \units\tclone units: {}\t克隆单元：{}"
