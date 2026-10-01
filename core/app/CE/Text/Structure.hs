-- | The `ce structure` console sentences (plan v2.32 step 5),
-- transcribed byte for byte from cli/src/structure/report.rs.
module CE.Text.Structure (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "head\tstructure score {}/{} | entropy {} | axes {} | {} dirs\t结构分数 {}/{} | 熵 {} | 判轴 {} | {} 目录\n\
    \divergence\tlayout divergence {}‰ over {} declared dirs\t布局偏离 {}‰（{} 个已声明目录）\n\
    \divergence_undefined\tlayout divergence undefined: mass outside the {} declared dirs\t布局偏离未定义：质量落在 {} 个已声明目录之外\n\
    \undeclared\tundeclared territory\t未声明领地\n\
    \declared_empty\tdeclared but empty\t已声明但为空\n\
    \deviation\tdeviation {}  {}\t偏离 {}  {}\n\
    \finding\tfinding {}  axis {}\t发现 {}  判轴 {}\n\
    \split\tsplit {}: seam after line {} ({}) — ROI {} (recover {} vs cost {})\t拆分 {}：缝在 {} 行后（{}）— ROI {}（回收 {} 对成本 {}）\n\
    \roi\t{}x\n\
    \no_seam\tno seam at all (single unit)\t根本无缝（单一单元）\n\
    \cohesive\tcohesive: best seam under ROI 1\t内聚：最优缝 ROI 不足 1\n\
    \exempt\tsize-exempt {}: {} (recover {} vs cost {})\t尺寸豁免 {}：{}（回收 {} 对成本 {}）"
