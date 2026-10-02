# 开发协作规则手册（通用版）

> 从 geek_main（yzgc-admin）的项目规范中提炼出的通用工程协作规则：分支模型、issue/PR 生命周期、提交信息、
> 追踪记录与执行记录、代码审查、tag 发版与人工验收、构建与验证、文档与模块对齐。
> 已去掉项目专属内容（域名、镜像名、服务名、人员、仓库名），可直接搬到新项目使用。
> 提炼日期：2026-10-02。
>
> 本目录随 geek-cli 仓库维护：**[10-geek-cli.md](10-geek-cli.md) 是该项目的现行落地规则**（工具链、CLI 契约、文档同步、发版流程、未做项）；与 01–09 冲突时以 10 为准。

## 核心思想（先读这 5 条）

1. **一件事 = 一个 issue = 一个 `task/<issue>/<slug>` 分支 = 一个 worktree = 一个 PR**，生命周期跟着 issue 走。
2. **两条长期分支**：`main`（正式）与 `stage`（集成/预发布）；其余分支短生命周期，合并即删。
3. **发版靠 tag，不靠分支**：push 分支只跑 CI；`vX.Y.Z-rc.N` → 预发布，人工验收通过后在同一提交打 `vX.Y.Z` → 正式。
4. **不同证据不互相替代**：类型检查 ≠ 构建 ≠ mock 预览 ≠ 浏览器验证 ≠ 线上人工验收；机器 PASS 不构成发布授权。
5. **规则单源**：同一条规则只在一处定义，其余地方链接过去；历史/提议文档不得当作现行规范执行。

## 目录

| 文件 | 内容 |
|---|---|
| [01-git-workflow.md](01-git-workflow.md) | 分支模型、不变量、命名、worktree、PR 合并方式、hotfix |
| [02-commit-message.md](02-commit-message.md) | Conventional Commits 结构、原子性、提交 ≠ 发版 |
| [03-issue-and-pr.md](03-issue-and-pr.md) | issue 正文契约、优先级、环境信息、PR 九段正文契约 |
| [04-tracking-records.md](04-tracking-records.md) | issue/PR 评论的追踪记录格式、生命周期、巡检规则 |
| [05-execution-notes.md](05-execution-notes.md) | `notes/` 执行记录：目录、格式、阶段词表、门禁 |
| [06-code-review.md](06-code-review.md) | 审查触发时机、逐项清单、严重度、结论格式、记录位置 |
| [07-release-and-acceptance.md](07-release-and-acceptance.md) | tag 发版、rc 编号、人工验收、环境隔离、回滚、tag 不可变 |
| [08-build-and-test.md](08-build-and-test.md) | 工具链固定、构建/验证命令分层、测试隔离、CI 结构 |
| [09-docs-and-modules.md](09-docs-and-modules.md) | 文档状态词表、文档跟着模块改、模块边界、agent 入口门禁 |
| [10-geek-cli.md](10-geek-cli.md) | **本项目专属（现行）**：模块地图、工具链与验收入口、CLI 契约、文档同步对照、发版流程、落地状态 |
| [api-forum-console.md](api-forum-console.md) | **只读**：极客班论坛与控制台接口手册（geek-cli 对接的唯一真相，不可修改） |
| [architecture/](architecture/README.md) | **geek-cli v2 架构（重写设计）**：会话/传输/命令面/自更新/安全/模块与路线图 |
| [templates/](templates/) | 可直接复制的模板：bug issue、feature issue、PR、追踪评论、执行记录 |

## 新项目最小落地清单

- [ ] 建 `main` + `stage` 两条长期分支；把「main 只接受 stage 快进」写进分支保护/钩子/CI。
- [ ] 定义分支命名正则（`/` 分层、段内 `_`、禁 `-`），写进 pre-push 钩子和 CI。
- [ ] 仓库模板化：issue 表单（bug/feature）、PR 模板（九段契约）。
- [ ] 写清「先开 issue → 建 task 分支 → PR 回 stage → 合并即清理」的工作流文档。
- [ ] 建两个记录流：issue/PR 评论（给所有人看进展）+ `notes/` 执行记录（按人按时间追溯）。
- [ ] 审查清单与结论格式落进规范；PR 必须带审查结论。
- [ ] 发版流程：tag 触发部署；`rc → 人工验收 → 正式`；tag 不可移动/删除；部署开关默认关闭。
- [ ] 固定工具链（Node/pnpm 版本、锁文件 frozen install），提供单一验收入口（如 `pnpm verify`）。
- [ ] 「文档跟着模块改」对照表 + 检查脚本；生成式索引不手工维护。
- [ ] CI 汇总成**一个 required check**，任何上游失败即失败；不得用 `continue-on-error` 掩盖。

## 使用建议

- 01–09 是与具体项目无关的通用方法，示例命令（`scripts/task.mjs`、`pnpm verify` 等）以原项目为准；本仓库（geek-cli）的现行参数与替代机制见 [10-geek-cli.md](10-geek-cli.md)。
- 规则之间的「为什么」都保留了——只抄形状不抄理由，遇到边界情况就会走偏。
