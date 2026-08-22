# 发版 runbook（两段式，ADR-007 分发链路）

> 蒸馏自 M7 发布轨设计册（原 docs/reviews/，2026-08-20 回填后清理；
> 全文在 git 历史）。鸡蛋序不可倒：**先产工件与校验和 → pin 提交 →
> 再打正式 tag**——清单永不引用未产出的 hash；tag 段不重建只验
> pin（Rust/GHC 构建非位可复现，重建即假 pin）。

## 0. 前置门（全绿才允许起步）

- 六腿狗粮门：`ce scan` / `ce dedup --check` / `ce check --fail-under 950`
  / `ce deadcode --check` / `ce docdup --check` / `ce erase --check`，加 `ce doctor`。
- `cargo test --release` 全绿（含 `CE_CORE_BIN` 指向当前 core）+
  clippy 零告警 + `bootstrap_e2e.sh` 全态 PASS + GUI lens 不变量。
- 版本五处一致：`cli/Cargo.toml`（唯一源，release.yml 的 dispatch
  输入也对它校验）、`core/ce-core.cabal`、`plugin/.claude-plugin/
  plugin.json`、`gui/src-tauri/tauri.conf.json`、`gui/src-tauri/
  Cargo.toml`（版本镜像门在测试电池里，drift 即红；两个 Cargo.lock
  由 --locked 兜住）；握手 golden `contracts/fixtures/handshake/
  hello-ok.ndjson` 的 version 回显同批重钉。
- 守卫档位有变 → CHANGELOG 按既有先例格式记 FPR 依据。
- **判决语义有变（轴语义/阈值/量纲）→ release notes 必须声明分数迁移**
  （先例：v0.5.0 的轴 3 目录计数修正案——同一仓库结构分会变）。

## 1. 第一段：draft（workflow_dispatch）

1. GitHub Actions → `release` workflow → Run workflow，输入裸版本号
   （如 `0.5.0`，不带 v）。版本输入与 crate 不符会在首步拒绝。
2. 三平台并行构建 `ce` + `ce-core` + GUI 实包（NSIS/AppImage/dmg），
   九工件 + `SHA256SUMS` 共十资产上传为 **draft** Release。
3. 本地抽验：下载任一平台二进制 `sha256sum -c` 对 SHA256SUMS。

## 2. 第二段：pin → tag → publish

1. 把 draft 的 SHA256SUMS 逐值写进 `plugin/bin/manifest.env`
   （九 pin：三平台 ce + 三平台 ce-core + 三平台 GUI 安装包），并同批翻 `CE_MANIFEST_VERSION`
   与 `CE_BASE_URL`（tag 腿断言前者 == tag；后者 URL 内嵌 tag，忘翻即
   下载 404）——十一行齐动，提交并推 main，CI 绿。
2. `git tag vX.Y.Z && git push origin vX.Y.Z`——tag 腿**只验 pin**
   后 publish（不重建）；`verify-publish` 复核十资产（九工件对拍
   SHA256SUMS，九工件对拍 manifest pin）。
3. Release notes：功能面 + 分数迁移声明（如适用）+ 未签名明示
   （0.x 无代码签名，SHA256 链路承重——ADR-007/R1 立场）。

## 3. 发布后渠道

- **crates.io**：`cd cli && cargo publish`（token 由用户本机配置，
  永不入库/入对话）。
- **npm 指针**：指针包（package.json + README，只转发 Releases、无
  二进制）bump version 后 `npm publish`——账户 2FA 需用户在交互终端
  完成 passkey/OTP，非交互 shell 里 publish 必 EOTP。
- **官网**：Cloudflare Pages **手动部署**（无 GitHub 集成）。`.secret`
  里是造币母 token（无 Pages 权限，验证 active 但 /accounts 为空是
  正常态）：用它 POST /user/tokens 铸 1 小时临时 token（权限组
  `Pages Write`，账户 ef6ce0a8b2c4ba8529b41aa6fd5b4f45），临时 token
  进 `CLOUDFLARE_API_TOKEN` 跑 `npx wrangler pages deploy site
  --project-name codeeraser`，完毕 DELETE /user/tokens/<id> 销毁；
  任何 token 值不落对话/不落库。
- **marketplace**：清单随 main 走，无独立发布步。
- 记账：CLAUDE.md 状态行 + ccm 发版记录。

## 4. 回归口径

- 陌生机器一条命令可用（SessionStart 下载→校验→原子落位）；
  校验失败必须响亮拒绝不落位不转 PATH（篡改样本三态回归=
  `bootstrap_e2e.sh`，CI 三平台常驻）。
- air-gapped 手动放置路保留（空 pin 回归按构造成立）。
