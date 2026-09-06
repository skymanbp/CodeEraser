# 发版 runbook（两段式，ADR-007 分发链路）

> 蒸馏自 M7 发布轨设计册（原 docs/reviews/，2026-08-20 回填后清理；
> 全文在 git 历史）。鸡蛋序不可倒：**先产工件与校验和 → pin 提交 →
> 再打正式 tag**——清单永不引用未产出的 hash；tag 段不重建只验
> pin（Rust/GHC 构建非位可复现，重建即假 pin）。

## 0. 前置门（全绿才允许起步）

- 两套<!--ce:count:gates#word-->六<!--/ce-->腿狗粮门：主树 `ce scan` / `ce dedup --check` / <!--ce:gate:floor.main#digits-->`ce check --fail-under 939`<!--/ce-->
  / `ce deadcode --check` / `ce docdup --check` / `ce erase --check`，加 `ce doctor`；
  `cli/tests` 子仓同六门（`ce <gate> tests`，<!--ce:gate:floor.tests#digits-->`--fail-under 979`<!--/ce-->，子仓自带 ce.toml 与基线）。
- `cargo test --release` 全绿（含 `CE_CORE_BIN` 指向当前 core；`cli/tests` submodule 已 `update --init`）+
  clippy 零告警 + `bootstrap_e2e.sh` 全态 PASS + GUI lens 不变量。
- 版本六处一致：`cli/Cargo.toml`（唯一源，release.yml 的 dispatch
  输入也对它校验）、`core/ce-core.cabal`、`plugin/.claude-plugin/
  plugin.json`、`gui/src-tauri/tauri.conf.json`、`gui/src-tauri/
  Cargo.toml`、`npm/package.json`（版本镜像门在测试电池里，drift 即红；
  两个 Cargo.lock 由 --locked 兜住）；握手 golden `contracts/fixtures/handshake/
  hello-ok.ndjson` 的 version 回显同批重钉。
- 守卫档位有变 → CHANGELOG 按既有先例格式记 FPR 依据。
- **判决语义有变（轴语义/阈值/量纲）→ release notes 必须声明分数迁移**
  （先例：v0.5.0 的轴 3 目录计数修正案——同一仓库结构分会变）。

## 1. 第一段：draft（workflow_dispatch）

1. GitHub Actions → `release` workflow → Run workflow，输入裸版本号
   （如 `0.5.0`，不带 v）。版本输入与 crate 不符会在首步拒绝。
2. 五目标并行构建 `ce` + `ce-core` + GUI 实包（NSIS/AppImage/dmg；花名册 =
   `update::version::TARGETS`：x86_64-windows / x86_64-linux / aarch64-macos /
   x86_64-macos / aarch64-linux，v1.7.0 起五个，此前三个；每个目标走同一份可复用
   workflow `build-target.yml`，ci.yml 每周的 `release-rehearsal` 也跑它、不上传），
   十五工件 + `SHA256SUMS` 共十六资产上传为 **draft** Release。
   **铁则（用户令 2026-08-28）**：任何渠道分发的二进制——Release 资产、
   plugin manifest 所 pin 的下载物——只能来自本 workflow 的矩阵产物；
   本地构建的二进制永不上传、永不 pin、永不作为「补位」放行。
3. 本地抽验：下载任一平台二进制 `sha256sum -c` 对 SHA256SUMS。draft 以
   `--target <构建提交>` 记下产物出自哪个提交——tag 腿据此对拍源码树（§2.3）。

## 2. 第二段：pin → tag → publish

1. `node scripts/pin_release.js <版本>`：从 draft 下载 SHA256SUMS、核对十五工件花名册
   （`scripts/roster.js` 从目标键派生资产名，`it/release_roster.rs` 守它与 Rust 常量、
   release.yml、ci.yml、清单键集同一份），把十五 pin（五目标 × ce / ce-core / GUI 安装包）
   与 `CE_MANIFEST_VERSION`、`CE_BASE_URL` 写进 `plugin/bin/manifest.env`，再按**终态**校验
   （键集恰好是花名册、每枚 pin 是 64 位十六进制且与 draft 报的那枚相等、两个版本行等于本次
   tag；判据只读终态，故原样重跑、或补跑 `--bless`，都不再被拒），再跑 `scripts/packaging.js` 重生成 `packaging/`
   下的 Homebrew 公式与 winget 清单（它们是清单的投影，随 pin 同提交）——不再手抄
   （tag 腿两者都断言：前者 == 去 v 的版本号（tag <!--ce:ver:ce#v-->`v1.6.0`<!--/ce--> ⇒ <!--ce:ver:ce#v-->`1.6.0`<!--/ce-->），后者须以
   `/download/<tag>` 结尾，忘翻即拒绝 publish、不再静默 404——
   release.yml verify-publish 腿）。**同一个提交里还有 docs-facts 一行**：
   `contracts/docs-facts.json` 的 `ver:pin#v` 是从 `CE_MANIFEST_VERSION` 派生的
   事实，pin 一动它就旧了，`facts_projection` 当场红——跑
   `CE_BLESS=1 cargo test --manifest-path cli/Cargo.toml --test it -- facts_`
   （或 `pin_release.js … --bless` 代跑）与清单同批提交。只推清单不推
   docs-facts 已在 v1.3.0 与 v1.4.1 两次把 pin 提交打红，而 tag 腿要等的正是这个提交的
   全部 check。清单、docs-facts、`packaging/` 齐动，提交并推 main，CI 绿。**tag 之前只跑离线
   三检**：`node scripts/packaging.js --check`（投影与生成器逐字节，`it/packaging.rs` 也跑它）、
   把公式拷进 `brew tap-new --no-git ci/codeeraser` 再 `brew style` 与 `brew audit --strict`、
   `pipx run check-jsonschema` 按三份 winget 清单自报的 schema 校验（后两条的逐字命令抄自
   ci.yml 的 `packaging-live`，需要 Homebrew 与 pipx 的机器）。`packaging-live` 本身**装不了
   还没发布的 draft**——公式里的 url 是公开发布 URL，此刻 draft 的资产还是 404；它是**公开渠道**
   验收：每周一随周程，publish 之后再手动 dispatch 一次（§3）——没有那一跑，两个 README 与官网
   两首页的 `brew install` / `winget install` 就是没有任何读者验过的承诺。
2. Release notes **先于 tag** 写到 draft 上：`gh release edit vX.Y.Z --notes-file <notes.md>`
   ——功能面 + 分数迁移声明（如适用）+ 未签名明示（代码签名/公证裁定不做——
   2026-08-19，SHA256 链为永久信任锚；ADR-007/R1 立场）。tag 腿拒发仍带占位句
   「Draft build phase」或没有说明的 release（此前的版本都带占位句发出、事后再改）。
3. `git tag vX.Y.Z && git push origin vX.Y.Z`——tag 腿**不重建**：先等**这个 commit 上全部
   check 完成，且 `build (ubuntu-latest)` / `build (windows-latest)` / `build-macos` 三条按名
   到齐且 success**（空的 check 表里既无 pending 也无失败，旧判据读成「全绿」= 零证据即发布；tag 会另起一次 CI，`build-macos` 自步 9 起每推都跑；排队多久由
   GitHub 说了算——v1.3.1 首打排队 30.6 min 未开工，耗尽当时 30 min 预算而拒发，预算已
   放宽到 2 h，超时按名列出未完成的腿，届时重跑该 job 即可，无须挪 tag），再核**来源**
   （draft 的 `--target` 构建提交与 tag 提交在**十二条**路径上树哈希逐一相等：`cli/src`、
   `cli/Cargo.{toml,lock}`、`core/app`、`core/ce-core.cabal`、`core/cabal.project{,.freeze}`、
   `gui/src-tauri`、`gui/ui`，外加同样决定字节的三个输入——`contracts/bench/bench.json`
   （`include_str!` 编进 GUI）、`rust-toolchain.toml`（两个 workspace 的编译器）、
   `.github/workflows/build-target.yml`（draft 跑的那份配方）——pin 提交只该动清单、
   docs-facts 与 `packaging/` 三处）与**说明**，最后
   `verify-publish` 按花名册复核十六资产（十五工件对拍 SHA256SUMS，再逐一对拍 manifest pin）
   后 publish，再把 `release` 分支快进到 tag 提交（marketplace 条目注册的就是这个分支，装机据此
   跟发布）。tag 腿等 check 时只赦免**按名列出**的 skipped 腿（`SKIPPED_OK`：本 workflow 的
   dispatch 段 `build` / `draft` + ci.yml 只在周程 / 手动跑的 `starter-https` / `setup-wiring` /
   `release-rehearsal` / `packaging-live`；`it/release_roster.rs` 守这张名单等于 ci.yml 的 `if:`
   行——漏一个名字，下一次 publish 就被它的 skipped 拒掉）。publish 之后三条**可选**腿各看
   一个仓库 secret，缺则具名跳过、走 §3 手动：`publish-crate`（`CARGO_REGISTRY_TOKEN`）、
   `homebrew-tap`（`HOMEBREW_TAP_TOKEN`）、`winget-pr`（`WINGET_TOKEN`）。

## 3. 发布后渠道

- **crates.io**：tag 腿 `publish-crate` 每次都先 `cargo publish --dry-run --locked`
  （证明包能建），仓库 secret `CARGO_REGISTRY_TOKEN` 在座才 `cargo publish`（crates.io
  已有该版本即具名跳过，不算失败）；不在座则手动 `cd cli && cargo publish`（token 由用户
  本机配置，永不入库/入对话）。子仓 `cli/tests` 必须在座：包里带 `tests/unit/**`
  （src 每个 `#[cfg(test)]` 的 `#[path]` 挂载目标，步 #13），缺了它下载者的
  `cargo test` 编译即错——`it/unit_mounts.rs` 钉「声明 = 磁盘 = 打包」三集合，
  CI 另解包跑 `cargo check --tests`。
- **Homebrew tap**（O71）：`packaging/homebrew/Formula/codeeraser.rb` 由 `node scripts/packaging.js`
  从清单派生（pin 后自动重生成；`it/packaging.rs` 守字节并把 url / sha256 反读回清单对拍；
  ci.yml 的 `packaging-live`——每周一随周程、**publish 之后手动 dispatch 一次**——在 macOS 与
  ubuntu 上 `brew style` / `brew audit --strict` / `brew install --formula` 装真 pin 并
  `ce --version`；它装的是公开发布 URL，故发布前跑不得）。tag 腿 `homebrew-tap` 在 secret
  `HOMEBREW_TAP_TOKEN` 在座时经 contents API 把公式写进 `skymanbp/homebrew-codeeraser`
  （该仓须先建好，空仓即可；不存在按名拒绝）；不在座则手动把该文件复制进 tap 仓。用户侧
  `brew install skymanbp/codeeraser/codeeraser`。
- **winget**（O71）：`packaging/winget/manifests/s/skymanbp/CodeEraser/<版本>/` 三份清单同源派生
  （schema 1.10.0；`packaging-live` 经 pipx 调 `check-jsonschema` 对 Microsoft 的 schema 校验——CI 的
  校验工具，不是仓内实现语言；清单字节纯 ASCII，生成器拒绝其他）。tag 腿
  `winget-pr` 在 `WINGET_TOKEN` 在座时同步用户的 fork、切分支、经 contents API 放文件、对
  microsoft/winget-pkgs 开 PR（首次标题 New package，其后 New version）；不在座则手动从该目录
  开 PR。合并后 `winget install skymanbp.CodeEraser`。
- **裁定点**：三条腿首次启用是对外动作（tap 仓、fork、PR 都挂在用户账户下），发版前由用户裁
  是否放 secret；裁不放则 README 双语与官网两首页的 Homebrew · winget 行须在发版前撤下。
  deb / rpm / AUR **不做**（2026-09-06 裁：deb / rpm 与 AppImage 同一份二进制只换外壳、各加
  四枚 pin 而无人要；AUR 要维护者账户与第三方仓里的 PKGBUILD；Linuxbrew 已覆盖 Linux 包管理）。
- **npm 指针**：`npm/`（入库：package.json + README，只转发 Releases、无
  二进制；版本随六处一致门走）——`cd npm && npm publish`。账户 2FA 需用户在
  交互终端完成 passkey/OTP，非交互 shell 里 publish 必 EOTP。
- **官网**：Cloudflare Pages **手动部署**（无 GitHub 集成——推 main
  不上线）。`node scripts/deploy_site.js` 一次跑完整链：`.secret`
  里是造币母 token（无 Pages 权限，验证 active 但 /accounts 为空是
  正常态），用它 POST /user/tokens 铸 1 小时临时 token（权限组
  `Pages Write`，账户 ef6ce0a8b2c4ba8529b41aa6fd5b4f45），临时 token
  进 `CLOUDFLARE_API_TOKEN` 跑 `npx wrangler pages deploy site
  --project-name codeeraser`，finally 里 DELETE /user/tokens/<id>
  销毁；任何 token 值不落对话/不落库/不打印。wrangler 退 0 后它自己串跑
  `scripts/verify_site.js`（边缘节点最多等 8 × 15 s），8/8 才退 0——上线以此为准；
  该读者也可单独跑：它把八页逐个与 `git show HEAD:<page>`
  的 blob 对拍。判据**不是**「剥完摘要相同」——那样会连第二处真差异一起吞掉——
  而是「至多一处 Cloudflare 边缘注入的 beacon，剥掉它之后与 blob 逐字节相等」；
  beacon 带属性且连它所在那行的换行一起注入，两点都在脚本头注里写明。
- **官网截图**：首页三张 GUI 图是**生成物**，不是手摆的窗口——
  `node scripts/shoot_gui.js --out site/assets` 用无头 Edge（即应用自己的
  WebView2 引擎）跑真 `gui/ui`，喂的是 CLI 出的三份报告文档。`gui/ui`
  一动就得重拍：`it/site_screenshots.rs` 四腿——按 git 祖先关系拒绝比界面旧的图、
  整窗尺寸、alt 文本不得手抄数字、`site/assets/` 每个文件都得有门认领；
  `it/site_shots_receipt.rs` 收据腿——`contracts/gui-shots.json` 里的 schema 必须等于
  代码当下的三个 `SCHEMA_ID`（界面不动而报告形状动了的那条路），`ui` 摘要必须等于
  当前 `gui/ui` 树（改了界面没重拍——哪怕还没提交——当场红；祖先关系那腿只读提交，
  看不见未提交的改动）；`it/site_shoot_motion.rs` 第六腿守住
  「取景器声明 `prefers-reduced-motion` × 应用应答它」这对耦合，少一半图就不再
  可复现。重拍只在拍进 `site/assets` 时重写收据（`--out` 指仓外则不动它）。
- **marketplace**：安装包与 `ce setup` 注册的是 `skymanbp/CodeEraser@release`，verify-publish
  在 publish 之后把 `release` 分支快进到 tag 提交（非快进即拒绝并点名；手动补救
  `git push origin vX.Y.Z^{commit}:refs/heads/release`）——main 上的清单改动要等下一次发布才到装机。
- **`ce update`**（v1.3.0 起）：装机自检读 `releases/latest` 的 tag 与**该 tag 上**的
  `plugin/bin/manifest.env`——本 runbook「pin 提交先于 tag」的既有序正是它的信任锚，
  无需另发任何东西。publish 后在任一旧版装机上 `ce update` 应退 1 并报新版本；
  `ce update --yes` 落位后 `ce doctor` 握手同版；插件装机走 `/plugin update codeeraser`。
- 记账：CLAUDE.md 状态行（本地项目卡，2026-08-23 起不入库）+ ccm 发版记录。

## 4. 回归口径

- 陌生机器一条命令可用（SessionStart 下载→校验→原子落位）；
  校验失败必须响亮拒绝不落位不转 PATH（篡改样本三态回归=
  `bootstrap_e2e.sh`，CI 三平台常驻）。
- air-gapped 手动放置路保留（空 pin 回归按构造成立）；非空 pin 的手放：
  `CLAUDE_PLUGIN_DATA/ce-<版本>-<平台>` 与 pin 相符即按已验证执行并落戳，不相符
  具名拒绝、不执行、不落戳（`bootstrap_e2e.sh` 状态 15 / 16）。
