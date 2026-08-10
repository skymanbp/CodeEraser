# M4 预注册评估集 v1（计划 §6 M4，D2-1 纯净度）

> 冻结于四分类任何实现代码之前（预注册的全部意义）。本文件 + 
> [contracts/eval/manifest-v1.json](../contracts/eval/manifest-v1.json)
> 共同构成冻结记录；样本载荷含其它私有仓库全文，按用户拍板
> （2026-08-10）**不入库**，落本地 `.ce-eval/`（.gitignore），由
> manifest 的逐样本 SHA-256 钉定、可随时重建校验。

## 构成（用户拍板 2026-08-10）

| 项 | 值 |
|---|---|
| 总量 | **600**，100% 真实 agent transcript（计划下限 ≥200、≥50%） |
| observe 档（feed 链接，机器可证未塑形） | 400（候选池 3,915） |
| 无 guard 时代（< 2026-08-07 18:20 装机，UTC 2026-08-07T10:20） | 200（候选池 8,271） |
| 标注子集 | 200（四分类 ground truth 用） |
| 视界 frozen_at | 2026-08-10T15:24:50 UTC |
| 语言分布 | py 380 / md 169 / rs 46 / ts 3 / go 2（按池比例，未人为配平） |
| 工具分布 | Edit 323 / Write 277 |

## 方法（全程无 RNG、无时钟——同输入必同输出，已双跑逐字节复验）

1. **扫描**：遍历本机 Claude Code transcripts，配对每个 Edit/Write
   `tool_use` 与其 `toolUseResult`，同一 id 只消费一次（compact/resume
   会重放历史行，实测单会话 1,242/3,282 个 id 重复至 6 次）。
2. **重建**：before = `originalFile`（Edit 必须非空；Write 空前态仅当
   `type=create`——update 缺前态不可知，按弃置计，绝不伪造）；after =
   Write 的 `content` 或对 before 施加 `structuredPatch`（上下文与删除行
   严格核对，不匹配即弃置——不复刻 Edit 匹配语义，ADR-004 教训）。
3. **视界**：`frozen_at` 在扫描收集点生效，弃置计数器同样被视界界定
   （transcripts 持续生长，双跑确定性检查曾抓到 +2 漂移）。
4. **分层抽样**：按 (项目, 语言) 分层，最大余数法配额 ∝ 层大小，层内
   SHA-256 哈希序取前 N；标注子集对 600 个 id 二次哈希序取 200。
5. **弃置全量入册**（manifest `excluded`，无静默截断）：错误/被拒结果、
   五语言外、前态不可知、超 1 MiB、历史重放、guard 时代无 feed 链接
   （58 个编辑，纯净但无机器证据，不采）、deny 测试仓（0）。

## Ground truth 溯源（标注子集 200）

`git diff -M -C` 交叉 + 启发式预标四分类（matched/novel/moved/deleted），
随后**由 agent 逐条审核全部 200 条**（用户 2026-08-10 明确委托，替代
计划原文的人工标注；预标与审核修正分列存储，锚定偏差可审计）。

## 复跑 / 校验

```
cd cli
CE_EVAL_TRANSCRIPTS=<transcripts root> CE_EVAL_FEEDS=<projects root> \
CE_EVAL_FROZEN_AT=2026-08-10T15:24:50 \
cargo test --test eval_extract -- --ignored --nocapture
# 重建 .ce-eval/ 并重写 manifest；与已提交 manifest diff 为空即完整复现
```

约束：重建依赖本机 transcripts 留存；`.ce-eval/` 丢失可重建，
transcripts 被清理后仅 manifest 哈希可证完整性（样本另行备份归用户）。
