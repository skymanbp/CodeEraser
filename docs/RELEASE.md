# 发版 runbook（两段式，ADR-007 分发链路）

> 蒸馏自 M7 发布轨设计册（原 docs/reviews/，2026-08-20 回填后清理；
> 全文在 git 历史）。鸡蛋序不可倒：**先产工件与校验和 → pin 提交 →
> 再打正式 tag**——清单永不引用未产出的 hash；tag 段不重建只验
> pin（Rust/GHC 构建非位可复现，重建即假 pin）。

## 0. 前置门（全绿才允许起步）

- 两套<!--ce:count:gates#word-->六<!--/ce-->腿狗粮门：主树 `ce scan` / `ce dedup --check` / <!--ce:gate:floor.main#digits-->`ce check --fail-under 946`<!--/ce-->
  / `ce deadcode --check` / `ce docdup --check` / `ce erase --check`，加 `ce doctor`；
  `cli/tests` 子仓同六门（`ce <gate> tests`，<!--ce:gate:floor.tests#digits-->`--fail-under 983`<!--/ce-->，子仓自带 ce.toml 与基线）。
- `cargo test --release` 全绿（含 `CE_CORE_BIN` 指向当前 core；`cli/tests` submodule 已 `update --init`）+
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
   **铁则（用户令 2026-08-28）**：任何渠道分发的二进制——Release 资产、
   plugin manifest 所 pin 的下载物——只能来自本 workflow 的矩阵产物；
   本地构建的二进制永不上传、永不 pin、永不作为「补位」放行。
3. 本地抽验：下载任一平台二进制 `sha256sum -c` 对 SHA256SUMS。

## 2. 第二段：pin → tag → publish

1. 把 draft 的 SHA256SUMS 逐值写进 `plugin/bin/manifest.env`
   （九 pin：三平台 ce + 三平台 ce-core + 三平台 GUI 安装包），并同批翻 `CE_MANIFEST_VERSION`
   与 `CE_BASE_URL`（tag 腿两者都断言：前者 == 去 v 的版本号（tag <!--ce:ver:ce#v-->`v1.3.0`<!--/ce--> ⇒ <!--ce:ver:ce#v-->`1.3.0`<!--/ce-->），后者须以
   `/download/<tag>` 结尾，忘翻即拒绝 publish、不再静默 404——
   release.yml verify-publish 腿）——十一行齐动，提交并推 main，CI 绿。
2. `git tag vX.Y.Z && git push origin vX.Y.Z`——tag 腿**只验 pin**
   后 publish（不重建）；`verify-publish` 复核十资产（九工件对拍
   SHA256SUMS，九工件对拍 manifest pin）。
3. Release notes：功能面 + 分数迁移声明（如适用）+ 未签名明示
   （代码签名/公证裁定不做——2026-08-19，SHA256 链为永久信任锚；
   ADR-007/R1 立场）。

## 3. 发布后渠道

- **crates.io**：`cd cli && cargo publish`（token 由用户本机配置，
  永不入库/入对话）。子仓 `cli/tests` 必须在座：包里带 `tests/unit/**`
  （src 每个 `#[cfg(test)]` 的 `#[path]` 挂载目标，步 #13），缺了它下载者的
  `cargo test` 编译即错——`it/unit_mounts.rs` 钉「声明 = 磁盘 = 打包」三集合，
  CI 另解包跑 `cargo check --tests`。
- **npm 指针**：指针包（package.json + README，只转发 Releases、无
  二进制）bump version 后 `npm publish`——账户 2FA 需用户在交互终端
  完成 passkey/OTP，非交互 shell 里 publish 必 EOTP。
- **官网**：Cloudflare Pages **手动部署**（无 GitHub 集成——推 main
  不上线）。`node scripts/deploy_site.js` 一次跑完整链：`.secret`
  里是造币母 token（无 Pages 权限，验证 active 但 /accounts 为空是
  正常态），用它 POST /user/tokens 铸 1 小时临时 token（权限组
  `Pages Write`，账户 ef6ce0a8b2c4ba8529b41aa6fd5b4f45），临时 token
  进 `CLOUDFLARE_API_TOKEN` 跑 `npx wrangler pages deploy site
  --project-name codeeraser`，finally 里 DELETE /user/tokens/<id>
  销毁；任何 token 值不落对话/不落库/不打印。部署后按页 sha256
  对拍本地 `site/` 才算上线。
- **marketplace**：清单随 main 走，无独立发布步。
- **`ce update`**（v1.3.0 起）：装机自检读 `releases/latest` 的 tag 与**该 tag 上**的
  `plugin/bin/manifest.env`——本 runbook「pin 提交先于 tag」的既有序正是它的信任锚，
  无需另发任何东西。publish 后在任一旧版装机上 `ce update` 应退 1 并报新版本；
  `ce update --yes` 落位后 `ce doctor` 握手同版；插件装机走 `/plugin update codeeraser`。
- 记账：CLAUDE.md 状态行（本地项目卡，2026-08-23 起不入库）+ ccm 发版记录。

## 4. 回归口径

- 陌生机器一条命令可用（SessionStart 下载→校验→原子落位）；
  校验失败必须响亮拒绝不落位不转 PATH（篡改样本三态回归=
  `bootstrap_e2e.sh`，CI 三平台常驻）。
- air-gapped 手动放置路保留（空 pin 回归按构造成立）。
