# AGENTS.md

本仓库（geek-cli）的人类与 agent 协作入口。

**进入仓库第一件事**：`git branch --show-current` 确认当前分支，然后完整读完：

1. [docs/README.md](docs/README.md) — 通用协作规范（01–09）
2. [docs/10-geek-cli.md](docs/10-geek-cli.md) — 本项目现行规则（**两者冲突时以它为准**）

读完之前：不改文件、不装依赖、不跑脚本、不动数据、不做任何 Git 写操作。上下文被压缩或换会话后重新确认分支并补读；文档缺失或规则冲突时停下报告，不凭记忆继续。

## 硬门禁（摘要，细则全在 docs/）

- 改动前先有 issue；分支用 `task/<issue>/<slug>`，禁止直接向 `main` 提交（过渡期约定见 10 §6）。
- 提交前本地四步全过：`cargo fmt --all -- --check` → `cargo clippy --all-targets --locked -- -D warnings` → `cargo test --locked` → `cargo build --locked --release`（10 §2）。
- CLI 面或行为变更必须同 PR 同步 `README.md` 与 `SKILL.md`（10 §4 对照表）。
- 创建 / 推送 / 移动任何发布 tag 需所有者对具体版本号授权——agent 不得自行打 tag（10 §5）。
- 禁止把 token、密钥、真实数据写进代码、文档、issue、PR、执行记录或日志。
- 缺凭据、规则冲突、文档缺失时报告阻塞；不伪造验证结果，不删断言换绿色。
