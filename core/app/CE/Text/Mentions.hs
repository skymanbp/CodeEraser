-- | The `ce graph --mentions` console sentences (plan v2.32 step 5),
-- transcribed byte for byte from cli/src/mention/face.rs (`console`,
-- `rates_console`). The rescan suffix prints in English under both
-- languages.
module CE.Text.Mentions (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "universe\tmention universe: {} files, {} mention sources, {} rows, {} files at the per-file cap (rev {})\t提及语料宇宙：{} 个文件，{} 个提及源文件，{} 行，{} 个文件触及单文件上限（rev {}）\n\
    \skipped\t  skipped: {} over 4 MiB, {} binary, {} signed, {} walk errors\t  跳过：{} 超 4 MiB，{} 二进制，{} 产品签名，{} walk 错误\n\
    \run\t  this run: {} refreshed, {} removed, {} rows clipped, {} files starved by the table cap{}\t  本次：刷新 {}，移除 {}，裁剪 {} 行，{} 个文件被表上限饿住{}\n\
    \rescan\t (rev changed: full rescan)\n\
    \outside\t  judged files outside the universe: {} over cap, {} binary, {} in nested repositories, {} ignore skew\t  判决文件不在宇宙内：{} 超限，{} 二进制，{} 在嵌套仓，{} 忽略语义差\n\
    \dist\t  dist/*.js bundler-suffixed runs (name$N): {}\t  dist/*.js 打包器去重后缀 run（name$N）：{}\n\
    \rates\t  {}: {} declared ({} exported) — {} unmentioned ({} exported); vetoed by another file {} (of which {} only by a same-name declaration), by fold {}, by the file's own exceptions {}\t  {}：声明 {}（导出 {}）——未提及 {}（导出 {}）；他文件否决 {}（其中 {} 仅因同名声明得救），折叠否决 {}，自文件例外否决 {}"
