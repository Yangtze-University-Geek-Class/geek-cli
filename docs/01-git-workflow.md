# Git 分支模型与工作流

## 核心模型

- **长期分支只有两条**：`main`（正式、稳定，只由 `stage` 合入）与 `stage`（集成分支，短生命周期分支的落点）。
- **其余全是短生命周期分支**，任务结束必须删除；不允许出现第三条长期分支（历史上用过的 `next`、`develop`、`release` 都不再有效）。
- **发版靠 tag，不靠分支**：push `stage`/`main` 只跑 CI，不部署任何环境。部署只由发布 tag 触发（见 07）。

## 两条不变量（必须同时成立）

1. `stage` 必须包含 `main`：`git merge-base --is-ancestor origin/main origin/stage` 为真（stage ≥ main）。
2. `main` 不得领先 `stage`：任何写入 `main` 的提交都必须已经存在于 `stage`；`main` 只能由 `stage` 快进而来。

由脚本、CI 和本地 pre-push 钩子强制。违反任一条即视为分支模型被破坏，先修复再继续；**不要用 force-push 掩盖差异**。本地自查：

```bash
node scripts/check-branch-invariants.mjs   # 只读：核对不变量 + 命名卫生
```

## 第负一步：先确认分支

任何提交、推送、切分支、改文件之前，先跑 `git branch --show-current` 确认自己在哪条分支。不在预期分支时停止操作并说明，不能靠 `git checkout .`、`reset --hard` 或清理工作区自行纠正。Agent 进入仓库的第一件事同样是确认分支 + 读完规范，不能「先执行、后补读」。

## 分支命名：只用 `/` 分层，不用 `-`

- 段内只含小写字母与数字；多词用 `_` 连接；不允许大写、`-`、首尾 `_`、连续 `__`、多余层级。
- GitHub 用户名带 `-` 的写成 `_`；大写转小写。

| 类型 | 形状 | 示例 |
|---|---|---|
| 任务分支 | `task/<issue>/<slug>` | `task/12/portal_redesign` |
| 个人分支 | `dev/<github-username>` | `dev/crosery` |

- `task/<issue>/<slug>` **只能从最新 `stage` 拉出**，一个 issue 一条，PR 合并后立即删除。
- `dev/<username>` 是个人自由开发区：可自由提交、可 force-push 自己的分支，但**不得作为进入 `stage` 的凭据**，也不部署。要送上 `stage` 的唯一合法路径是：从 `stage` 拉一条干净的 `task/<issue>/<slug>`，重新提交或 cherry-pick 经过审查的改动。
- 禁止把 `dev/**`、`task/**` 直接合并进 `main`。

## 一个 issue 一个 worktree

**一个 issue = 一个 task 分支 = 一个 git worktree = 一个 PR，四者生命周期相同。**

- **开工**：脚本从最新 `origin/stage` 建分支，同时在主工作区外的独立目录建 worktree（示例：`node scripts/task.mjs start <issue> <slug>`，worktree 在 `.claude/worktrees/task-<issue>`）。之后所有编辑、安装、构建、测试、提交都在这个 worktree 里做。
- **不碰主工作区**：主工作区（及别的 worktree）可能有别人未提交的改动、正在跑的进程；在自己的 worktree 里做，互不影响，不用切分支、不用 stash。
- **一个 issue 只有一个 worktree**：重复 start 应拒绝并提示直接进去继续做；不要在同一个 worktree 里做第二件事。
- **结束**：PR 合并后，`node scripts/task.mjs finish <issue>` 删 worktree 与本地分支（在主工作区运行，不要在要删的 worktree 里运行）。
- **忘了清理就推不上去**：pre-push 钩子检查本机是否有「PR 已合并 / issue 已关但还没 finish」的 worktree，有则拒绝任何推送并列出清理命令。钩子每台克隆启用一次：`git config core.hooksPath .githooks`；`git push --no-verify` 能跳过本地钩子，但跳过不等于收尾。
- 工作区有未提交改动、PR 还开着、issue 还开着且 PR 没合并时，脚本只报告原因不删除；放弃的 issue 先写「关闭」记录再关，之后就能清理。
- worktree 目录加进 `.gitignore`；每个 worktree 需要自己 `pnpm install --frozen-lockfile`（包管理器全局仓库会复用已下载的包）。

## 日常流程

```bash
git branch --show-current                      # 1. 确认当前在哪
# 2. 先开 issue，记下编号（见 03）
node scripts/task.mjs start <issue> <slug>     # 3. 从最新 stage 建 task 分支 + 独立 worktree
# 4. 在 worktree 里开发、验证、提交；每个阶段写追踪记录与执行记录（见 04、05）
# 5. 开 PR → stage，正文按九段契约写（见 03），`Closes #<issue>`
# 6. 合并后：远端分支与 issue 自动清理；本机 task.mjs finish <issue>
```

## PR 与合并方式

- PR 只能指向 `stage`；正文必须 `Closes #<issue>` 且与分支号一致。
- **只用 merge commit**（`gh pr merge <PR> --merge`），禁止 squash 与 rebase：
  - squash 会把审查、执行记录里引用的提交 SHA 从 `stage` 历史里抹掉；
  - rebase 会把 PR 的提交逐个接到第一父链上，破坏依赖第一父链的「文档跟着模块改」时间比对；
  - merge commit 保证一个 PR 只在 `stage` 上留一个合并提交，模块与文档同时到达。
- 仓库设置里关掉 squash/rebase 按钮；关掉之前靠合并的人照规则执行。
- `main` 只通过 `git merge --ff-only <被验收的提交>` 快进，禁止 merge commit 进 main（保持 main 的每个提交都存在于 stage）。

## 合并后清理（硬性）

- PR 合并进 `stage` 后，它 `Closes` 的 issue 必须关闭，task 分支与 worktree 必须清理；「完成的 PR 管理的 issue 必须清理」。
- 自动化（`issue-lifecycle` 关 issue 并留「关闭」记录、`branch-hygiene` 删远端分支）没成功的（工作流失败、fork PR 没有写权限等），由合并的人**当场**手工关 + 手工删 + 留记录，不留到每日巡检。
- 已合并的分支不再提交；后续问题开新 issue 并 `Refs #<原 issue>`。

## 线上紧急修复（Hotfix）

正式环境发生直接影响用户的严重故障（白屏、核心流程阻断、安全事故等）时：

1. **先开 issue**（或事后 1 小时内补齐），打上 `hot-fix` 与 `P0` 标签。
2. **以最新 `origin/main` 为基线**拉修复分支，最短路径合入 `main` 上线消除影响。
3. **强制执行不变量合回**：验收通过后立即把修复合并回 `stage`：

   ```bash
   git switch stage && git merge --no-ff origin/main && git push origin stage
   ```

   再跑一次 `check-branch-invariants`，确认 `stage ≥ main`。绝不允许修复只留在 `main` 而 `stage` 遗漏（否则后续常规迭代会代码倒退或冲突）。
4. **源头根治 = 环境同构**：预发布与正式环境在配置（Nginx/反代/CSP/环境变量结构）上必须最高程度对齐，只允许域名/端口与真实密钥不同；严禁「预发布未配/未测、正式才配置」。

## 禁止事项

- 直接向 `main` 提交或推送（含 agent、脚本、网页编辑、CI 机器人）；`task/**`、`dev/**` 直接进 `main`。
- 合并后残留死分支、死 worktree，或关联 issue 仍开着。
- 在主工作区里切 task 分支开发；同一个 worktree 里做两个 issue。
- 没有 issue 就开 task 分支；`stage` 收未审查的内容。
- force-push `main`/`stage`；整分支 reset 覆盖他人提交。
- 把 `dev/**` 的提交历史、分支名或 CI 绿标当作审查凭据；用分支名当版本号或发布凭据。
