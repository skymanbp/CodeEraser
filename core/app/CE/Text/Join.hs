-- | The `ce join` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/join/report.rs. The verdict tail, the
-- graph position and the tokens metric print in English under both
-- languages (`verdict_str`, `pos_str`, `sim_str`'s t1t2 arm).
module CE.Text.Join (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "file\tjoin {} <-> {}: {} blocks / {} tokens / {} near-miss | graph {} | {} | churn +{}/~{} | +{}/~{} | cochange {} | {}\t联判 {} <-> {}：{} 块 / {} tokens / {} 近似对 | 图 {} | {} | 改动 +{}/~{} | +{}/~{} | 共变 {} | {}\n\
    \verdict\t{} (sev {}, conf {})\n\
    \self_pair\tself-pair (off the sim table)\n\
    \unjudged\tunjudged\n\
    \pos\tin{} out{} scc{}x{} reach{}\n\
    \pos_null\tnull\n\
    \unit\tunit {}#{}~{} <-> {}#{}~{}: {} | churn +{}/~{} | +{}/~{} | graph null (R6 locked)\t单元 {}#{}~{} <-> {}#{}~{}：{} | 改动 +{}/~{} | +{}/~{} | 图 null（R6 锁定）\n\
    \tokens\t{} tokens\n\
    \t3\tt3 ted {} (nodes {}/{})\tt3 ted {}（节点 {}/{}）\n\
    \degraded\tjoin graph leg degraded: {}\t联判图信号腿已降级：{}\n\
    \summary\tjoin {}d window: {} file pairs, {} unit rows, {} commits (verdicts by the check lattice; exit stays report-only)\t联判 {} 天窗口：{} 文件对，{} 单元行，{} 提交（判决出自 check 判决格；退出码仍仅报告）"
