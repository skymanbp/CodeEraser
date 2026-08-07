# crosscheck fixtures — 来源与抽样方法（M1 验收，plan §6 M1）

> 防作弊要求：fixtures 来自钉死 commit 的真实仓库 + 确定性抽样（不可挑选）。
> 文件名中 `__` = 原仓库路径的 `/`。

## 钉定仓库（2026-08-07 浅克隆）

| 语言 | 仓库 | commit | license |
|---|---|---|---|
| python | psf/requests | 1f6589ec3a1ee910f9a65cc3ceac60b26677bc0e | Apache-2.0 |
| typescript | colinhacks/zod | 912f0f51b0ced654d0069741e7160834dca742ee | MIT |
| go | spf13/cobra | adbc8813901bba65827259daa8e22ff94ec1f30e | Apache-2.0 |
| rust | BurntSushi/ripgrep | 3fce3b5bb0236da2df6d99672afb8a719642eca7 | MIT OR Unlicense |

## 抽样规则（确定性，复现命令见下）

1. `git ls-files '*.<ext>'`；
2. 排除路径匹配 `test|_test\.|\.d\.ts$|testdata`；
3. 每个路径取 `SHA1(路径字符串)`，按其十六进制字典序**升序**；
4. 取前 5 个。

抽样者无自由度：给定 commit，样本集唯一确定。zod 抽中多个 locale 文件
（函数密度低）是真实抽样的自然结果，不做人工替换；若对拍需要更多函数
样本，扩大 N 而非换文件。

复现（PowerShell）：
```powershell
git ls-files '*.py' | ?{ $_ -notmatch 'test|_test\.|\.d\.ts$|testdata' } |
  Sort-Object { [BitConverter]::ToString([Security.Cryptography.SHA1]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes($_))) } |
  Select-Object -First 5
```

## 对照工具版本（本机安装记录）

- lizard 1.23.0（CC：python/typescript/rust 兜底）
- gocyclo v0.6.0（CC：go；complexity.go 明示 "ignore default case"——
  ce 同步不计 default_case）
- gocognit（CoC：go，唯一外部 CoC 对照）
- rust-code-analysis-cli 0.0.25，`cargo install --locked`（CC：rust；
  不带 --locked 会因新依赖编译失败——安装须知）

## CoC 规范原文钉定（cli/tests/sonar_whitepaper.rs 判分依据）

- SonarSource《Cognitive Complexity》白皮书 **v1.7（2023-08-29）**，
  <https://www.sonarsource.com/docs/CognitiveComplexity.pdf>，
  SHA-256 `d1bbd47a0c48500bfeafa5dfada42ecddfe1c3e75b7adebe11cb7a037fa4cb77`
  （2026-08-07 下载核对；版权归 SonarSource，不入库，凭 hash 复验）。

上游文件版权归各自项目所有，按其 license 复制于此仅作度量对拍测试。
