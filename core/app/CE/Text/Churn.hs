-- | The `ce churn` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/churn/report.rs (`print_console`, deleted
-- at step 5 R0 when the face began printing these); `beyond` is v2.35
-- V1's, written here.
module CE.Text.Churn (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "window\tchurn window {}d: {} commits, appended {} / rewrote {} lines\t改动窗口 {} 天：{} 个提交，追加 {} / 重写 {} 行\n\
    \survival\twindow survival: {} of {} added lines survive at HEAD ({} churned)\t窗口存活：新增 {} / {} 行存活至 HEAD（{} 已翻改）\n\
    \beyond\tnote: {} line(s) blame gives to window commits beyond the ledger's added lines (a line end, a final newline, another diff alignment), not counted as survivors\t注：{} 行被 blame 归给窗内提交、却超出账本记下的新增行（行尾、文件末换行、diff 对齐不同），不计入存活\n\
    \pair\tco-change x{}: {} <-> {}\t共变 x{}：{} <-> {}\n\
    \more\tco-change: {} more pairs below the display cut\t共变：另有 {} 对低于显示截断\n\
    \skipped\tnote: {} commit(s) above {} files skipped for pairing\t注：{} 个提交超过 {} 文件上限，未参与配对\n\
    \submodules\tnote: no file history for declared submodule(s) {} — the ledger is the superproject's own history\t注：声明的 submodule {} 无文件历史——账本只计超仓自身的历史"
