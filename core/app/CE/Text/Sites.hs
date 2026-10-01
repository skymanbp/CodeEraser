-- | The `ce graph --sites` console sentences (plan v2.32 step 5),
-- transcribed byte for byte from cli/src/graph/mod.rs `print_counts`;
-- the count rows are data, the same under both languages.
module CE.Text.Sites (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "total\tgraph sites: {} across {} files\t图引用站点：{} 个，分布于 {} 个文件\n\
    \count\t  {} {} {}"
