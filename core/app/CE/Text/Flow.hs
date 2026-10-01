-- | The `ce flow` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/flow_report/console.rs.
module CE.Text.Flow (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "degraded\tflow: degraded — {} (no judgment)\t函数内死代码：已降级——{}（未判决）\n\
    \finding\t{}:{}  {}  {}  {}{}\n\
    \advisory\t  (advisory)\t  （顾问）\n\
    \unjudged\tunjudged {} {}: {}\t未判 {} {}：{}\n\
    \counts\t{} unit(s): {} finding(s), {} judged, {} shown; {} dynamic unit(s) skipped, {} unjudged\t{} 个单元：{} 条发现，{} 条判决，显示 {} 条；{} 个动态单元整单元跳过，{} 个未判"
