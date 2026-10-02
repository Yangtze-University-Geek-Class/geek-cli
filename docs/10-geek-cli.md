# geek-cli 项目专属规则（现行）

> 状态：current ｜ 更新：2026-10-02 ｜ 适用：`Yangtze-University-Geek-Class/geek-cli` 全体贡献者与 agent（人 + 机器一视同仁）。
> 01–09 是与项目无关的通用方法；本文件把它落成 geek-cli 的具体参数，并列出不适用项与未做项。两者冲突时以本文件为准。
> 事实来源：`Cargo.toml`、`src/**`、`.github/workflows/release.yml`、`install.sh`、`install.ps1`、`npm/**` 与 git 历史（截至 2026-10-02）。
> 2026-10-02 两次修订：①按架构评审（issue #2）切 v2 口径（§2/§3/§4：edition 2024、`GEEK_BASE`/`GEEK_SID`、v2 模块文档同步表）；②按所有者定夺固化**分支维护方式**（§6/§7：`main` ruleset `main: PR gate`、CODEOWNERS 批准、agent 只推不并、`stage` 暂不适用）。标「v2」的行为在重写 PR 落地前以现状为准。

## §1 项目快照

- 形态：Rust 单二进制 `geek`（crate `geek-cli`）；面向 agent 与命令行用户，默认 JSON 输出。现状 edition 2021；**v2 重写目标 edition 2024 / MSRV 1.85**（评审通过，见 `docs/architecture/README.md` D10）。
- 模块职责（改哪块看哪块）：

| 路径 | 职责 |
|---|---|
| `src/main.rs` | 入口，只做模块装配 |
| `src/cli.rs` | clap 命令树与命令实现编排（单一改动热点） |
| `src/gh.rs` | GitHub REST 客户端：分页、状态码 → 错误映射 |
| `src/forum.rs` | 论坛 API 客户端（Bearer PAT、`GEEK_FORUM_BASE`） |
| `src/auth.rs` | GitHub OAuth Device flow（登录） |
| `src/config.rs` | token 读写（`~/.config/geek/token.json`，Unix 0600） |
| `src/output.rs` | `--format json/pretty/table` 的唯一出口 |

- 分发三渠道：Release 二进制（`install.sh` / `install.ps1`，README 主推）→ npm 包装 `@yangtzeu/geek-cli`（postinstall 下载预编译二进制）→ `cargo install --git`。
- 版本单源：`Cargo.toml` 的 `version`。npm 版本在发版时由 release workflow 从 tag 覆写，`npm/package.json` 里的值不是依据。
- 不适用章节：07 的镜像 / 环境隔离 / 回滚条款（本项目无服务端、无 Docker、无部署环境，发版按 §5）；08 的 Node/pnpm 工具链条款（`npm/` 只是分发包装，不参与开发构建）。01–06、09 全部适用，参数见下。
- 重构设计（v2 架构）见 [`docs/architecture/`](architecture/README.md)；接口唯一真相见 [`docs/api-forum-console.md`](api-forum-console.md)（**只读**，与 API 不符的旧实现一律按过期处理）。

## §2 工具链与验收入口（08 落地）

- Rust stable；`Cargo.lock` 入库；所有构建（含发布）带 `--locked`；edition 随 v2 升 2024（MSRV 1.85）。
- PR 前本地验收入口，四步必须全过（本项目暂无单一脚本入口，此顺序即入口；现状 fmt / clippy 未过，属既有缺口，见 §7）：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --locked --release
```

- 依赖约束：发布矩阵含 `x86_64-unknown-linux-musl`，二进制必须可静态分发 → 新增依赖不得要求系统库（如 OpenSSL），TLS 保持 rustls。新增依赖在 PR「变更范围」写目的 / 许可 / 体积。
- 测试：现状 `src/` 没有任何测试（§7）。行为变更（错误映射、参数解析、URL / 请求拼装、格式化）要带能区分改动前后的回归测试，优先抽纯函数用 `#[cfg(test)]`；测试默认不发真实网络请求、不读真实 token 或生产数据。未验证就写「未验证」；禁止删断言 / 跳过检查换绿色（06）。
- CI 现状：只有 tag `v*` 触发的 release workflow；**PR CI 未建**（已排入 P0，§7-2）。远端合并门禁由 ruleset `main: PR gate` 提供（PR + ≥1 批准 + CODEOWNERS 批准）；构建/测试门禁 = 本机四步（结果随 PR 填写），不得声称「CI 已拦截」。

## §3 CLI 契约（本项目最重要的对外约定）

- 输出分轨：默认单行 JSON 到 stdout；`--format pretty|table` 只改变人类可读形态；进度、提示、错误一律 stderr（`output::print_message`）。新命令必须经 `output::emit` 输出数据，禁止自带 `println` 把说明文字混进 JSON。
- 错误：一行、非零退出；状态码 → 文案的唯一实现处是 `gh.rs` / `forum.rs` 的 `check()`（401/403/404/422 等），新调用点必须复用，不得改写成模糊文案。权限由 GitHub / 论坛后端判定，403/404 原样上报，不做客户端绕过或盲目重试。
- 破坏性操作必须显式确认标志（先例：`repo delete --yes`）。
- 认证：统一 `config::require_token()`（`GEEK_TOKEN` 优先于本地 token 文件）；token 只落 `~/.config/geek/token.json`；任何输出（错误、日志、测试、issue / PR 评论）不得回显 token。
- 环境变量（CLI 侧唯一清单，v2）：`GEEK_BASE`、`GEEK_SID`（链路变量 `GEEK_FORMAT` / `GEEK_TIMEOUT` / `GEEK_NO_UPDATE_CHECK` / `GEEK_DEBUG` 见 `docs/architecture/05` §A.2）。旧的 `GEEK_TOKEN` / `GEEK_FORUM_BASE` 随旧实现删除。`GEEK_VERSION` / `GEEK_INSTALL_DIR` 属安装脚本，不进 CLI 代码。
- 命名与文案：子命令 kebab-case，同一能力不留第二别名；`--help` / about 英文、面向用户的错误与提示中文（现状即约定；改文案同 PR 同步 `SKILL.md` 与 `README.md`）。

## §4 文档跟着模块改（09 落地）

| 改了什么 | 同一 PR 必须同步 |
|---|---|
| `src/commands/**`（命令、参数、默认值，v2） | `README.md`「命令一览」+ `SKILL.md` 对应示例 + `docs/architecture/03` |
| `src/api/**`（端点、错误映射、重试矩阵，v2） | `docs/architecture/02` §5–§6 + `SKILL.md` Error handling |
| `src/session.rs`、`src/config.rs`（会话、环境变量，v2） | `README.md`「鉴权与权限」「环境变量」+ `SKILL.md` Auth |
| `src/output.rs`（格式行为） | `README.md`「输出格式」+ `SKILL.md` Output discipline |
| `src/update.rs`（自更新，v2） | `docs/architecture/04` + `README.md`「安装」 |
| `Cargo.toml` `version` | 发版时与 tag 一致（§5） |
| `.github/workflows/`、`install.sh`、`install.ps1`、`npm/**` | `README.md`「安装」+ `npm/README.md` |

- `SKILL.md` 是下游 agent 的 runai 技能契约：frontmatter `description` 决定技能何时触发，命令面 / 语义变化必须同 PR 更新，改 description 要在 PR 里说明理由。
- 核对方法：`cargo run -- <cmd> --help` 与 README / SKILL 的示例逐条对照；事实没变时在本 task 执行记录写 `文档核对：<路径> 不用改——<理由>`。

## §5 发版（07 落地，tag 驱动）

- **打 tag 即发布**：创建 / 推送任何发布 tag 需所有者对具体版本号的明确授权；「CI 全绿」「测试过了」不算授权；agent 不得自行打 / 推 tag。
- 版本与形状：`Cargo.toml` `version` 由普通 PR 修改；tag 形状 `vX.Y.Z` / `vX.Y.Z-rc.N`。rc 用于人工验收（GitHub `releases/latest` 不含预发布，装 rc 用 `GEEK_VERSION=vX.Y.Z-rc.N`）；正式 tag 必须与某个 rc 落在同一提交。
- 流程：
  1. 目标提交已经审查合入 `main`（经 PR + CODEOWNERS 批准，见 §6）；
  2. 打 `vX.Y.Z-rc.N` 并推送；release workflow 构建 7 个 target，上传 Release 资产 + install 脚本，并尝试 npm 发布；
  3. 人工验收：在干净机器用 `GEEK_VERSION=vX.Y.Z-rc.N` 跑 `install.sh`，验 `geek --version`、`geek whoami` 与至少一条核心命令（如 `geek org list`）；
  4. 通过后在同一提交打 `vX.Y.Z` 推送；
  5. 核对产物：Release 上 7 个二进制 + 对应 `.sha256` + `install.sh` + `install.ps1`；npm 渠道核对 `npm view @yangtzeu/geek-cli version`。
- npm 是辅助渠道且 `publish-npm` 为 `continue-on-error`：未发出（常见于缺 `NPM_TOKEN`）就在该 tag 的 run 上「Re-run failed jobs」补发；**发布 tag 不可移动、不可删除、不可重打**，打错就发下一个版本。
- 发布前人工核对 tag 与 `Cargo.toml` `version` 一致（workflow 尚未校验，§7）。
- 用户侧回滚 = 用 `GEEK_VERSION=vX.Y.<上一版>` 重装；不删除旧 Release。
- 发布与验收记录按 04 追踪记录格式留在对应 issue / PR 上。

## §6 分支、提交与执行记录（01 / 02 / 03 / 04 / 05 落地）

- **分支维护方式（所有者定夺，2026-10-02；与 `docs/01` 冲突时以本节为准）**：
  - 长期分支只有 `main`（默认分支）。`docs/01` 的 `stage` 双分支模型**暂不适用**：不建立、不由任何人自行增设；未来启用须由所有者宣布并先修订本节。
  - `main` 受 ruleset **`main: PR gate`**（active）保护：必须开 PR、**≥1 个批准**、**CODEOWNERS（`@Crosery`）批准**、禁 force-push、禁删除分支。合并由所有者执行；**agent 只推分支、开 PR，不合并、不改 ruleset / CODEOWNERS / 分支模型**。
  - ruleset 开启 `require_extra_approval_for_unattributed_changes`：无法归属到 GitHub 账号的提交需要额外批准——agent / 脚本提交必须带可归属身份（本机 `git config user.name` / `user.email` 已配置，勿改成匿名机器人）。
  - 远端已开「合并后自动删除分支」；本地分支 / worktree 由发起人按本节收尾清理。
  - 合并方式：项目规则沿用 `docs/01`（**只用 merge commit**）；仓库层（ruleset 与仓库设置）目前允许 merge / squash / rebase 三种，是否收紧由所有者定夺。
- 工作分支（现行）：`task/<issue>/<slug>` 从最新 `main` 拉出，PR 指向 `main`；命名与 worktree 规则见下。
- 命名：用 `/` 分层，段内只含小写字母、数字与 `_`（多词用 `_` 连接），不用 `-`、大写；`task/<issue>/<slug>`、`dev/<github_username>`（GitHub 用户名里的 `-` 写成 `_`）。
- 一个 issue = 一个 task 分支 = 一个 worktree = 一个 PR；worktree 用 `git worktree add` 手工建（无脚本，§7），开发、构建、提交都在 worktree 里做，不碰主工作区。
- 提交信息（02）：`<type>(<scope>): <中文简述>` + 正文三段（为什么改 / 改变了什么 / 用什么命令验证）。scope 从本仓库模块选：`cli`、`gh`、`forum`、`auth`、`config`、`output`、`docs`、`release`、`install`、`deps`。历史提交是 `[type][user]` 风格，自本规则起新提交按 02，**不追溯改写历史**。
- issue 先行（03）：任何 task 分支先有 issue；标题、正文契约与优先级按 03（模板 `docs/templates/issue-bug.md`、`issue-feature.md`）。
- PR 九段契约（03）：模板 `docs/templates/pull-request.md`；审查、返工、验收按追踪记录格式写成评论（04，模板 `docs/templates/track-comment.md`）；不改写已发出的评论。
- 执行记录（05）：手工按 `docs/templates/note-chain.md` 写入 `notes/<日期>/<GitHub用户名>/task_<issue>_<slug>.md`（日期用 +08:00，分支名 `/`、`-` 换成 `_`），只追加、不改写；PR 链路缺「开工 / 提交 / PR / 审查」记录不得合并，由审查人核对（无脚本与 `INDEX.md` 生成，§7）。
- 禁止：直接向 `main` 提交 / 强推；**自行合并 PR 或修改所有者侧设置（ruleset / CODEOWNERS / 分支模型 / 仓库开关）**；没有 issue 就开 task 分支；合并后残留分支 / worktree / 未关 issue；把 `dev/**` 当审查凭据。

## §7 落地状态（截至 2026-10-02）

> 以下规则已生效，但机械化环节尚未实现（未做）。实现前由人工按 §2–§6 执行；实现后更新本表。

| # | 未做 | 影响 | 待办 |
|---|---|---|---|
| 1 | `stage` 分支 | `docs/01` 双分支模型暂不适用（所有者定夺，§6） | 不建立、不自行增设；启用须由所有者宣布并先修 §6 |
| 2 | PR CI（fmt / clippy / test / build） | 构建/测试门禁只在本地；远端合并门禁已由 ruleset 提供 | **已排入 P0**（`docs/architecture/06` §4）：新增 `.github/workflows/ci.yml`；建成后由所有者加入 ruleset 的 required check |
| 3 | 发布前 tag ↔ `Cargo.toml` 版本校验 | 只能人工核对 | 加到 release workflow |
| 4 | 测试 | `src/` 无测试，回归靠人工 | 随行为变更逐个补（§2） |
| 5 | task / worktree / notes 脚本与 `notes/INDEX.md` 生成 | 手工执行、人工核对 | 按需引入，输出格式必须与 01 / 05 一致 |
| 6 | `README.md`「命令一览」缺 `forum` 子命令 | 用户文档不完整（`SKILL.md` 已有） | 下一个触及 README 的 PR 补齐 |
| 7 | npm 发布失败后的自动核对 / 重试 | 依赖发版人手动 re-run | 可改为发布 job 失败即让汇总失败 |
| 8 | fmt / clippy 未过（既有代码未 rustfmt 化；`gh.rs` `GhClient::patch` 死代码，2026-10-02 基线实测） | 四步门禁暂不可全绿 | 首个 `style` PR 跑 `cargo fmt --all`；`patch` 要么用起来要么删除 |
| 9 | 合并方式开关（ruleset 与仓库设置仍允许 squash/rebase） | 与 `docs/01`「只用 merge commit」不一致，靠合并人执行 | 是否收紧由所有者定夺（§6） |
