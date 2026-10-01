-- | The `ce arch` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/arch/console.rs: English beside Chinese,
-- `{}` holes; a row the measuring side printed in English under both
-- languages carries the same template twice.
module CE.Text.Arch (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "degraded\tarch: degraded — {}\tarch：已降级——{}\n\
    \counts\t{} files, {} directories, {} file references, {} package references: {} cut(s), {} cluster(s), {} misplaced\t{} 个文件、{} 个目录、{} 条文件引用、{} 条包引用：{} 条切边、{} 个簇、{} 个错位\n\
    \level\tlevel {}: {}\t第 {} 层：{}\n\
    \no_cuts\tno cycles among directories\t目录之间没有环\n\
    \exact\texact\t精确\n\
    \greedy\tgreedy\t贪心\n\
    \cut\tcut: {} → {}  ({} refs, {})\t切边：{} → {}（{} 处引用，{}）\n\
    \cut_file\t  {} → {}  ({})\n\
    \no_misplaced\tno misplaced file\t没有错位的文件\n\
    \misplaced\tmisplaced: {} (in {}, cluster majority {})\t错位：{}（在 {}，簇多数在 {}）\n\
    \impact_head\timpact (depth):\t影响面（深度）：\n\
    \impact\t  {}  {}\n\
    \metrics_head\t{}  fanIn  fanOut  instability\n\
    \metrics_row\t{}{}  {}  {}  {}"
