# 07 · 待定、待确认与暂缓事项（登记表）

> 状态：`current`（登记表本身是现行约定的一部分）· 更新：2026-10-02 · 适用范围：geek-cli v2 全部待定事项。
> 事实来源：issue #2 评审（@Crosery）、GitHub API 实读（ruleset #24362647、仓库设置）、本目录 01–06。
> 约定：**任何未定论、待确认、暂缓的事项一律先登记到本文件**——不散落在对话、issue 评论或记忆里。与 `docs/10` §7 的分工：§7 只列**已确定要做**的落地缺口；本文件收录**尚未确定 / 待所有者确认 / 暂缓**的项。

## A. 待所有者确认（阻塞「框架冻结」）

| # | 事项 | 现状与证据 | 待谁确认 / 触发条件 |
|---|---|---|---|
| A1 | ruleset 的绕过（bypass）配置与评审陈述不一致 | 评审 §0 称「所有者以 `pull_request` 模式绕过」；API 实读 ruleset `main: PR gate`（#24362647）`bypass_actors = null`、`current_user_can_bypass = never`——**当前无人可绕过** | @Crosery 确认或调整 ruleset；不影响我方（一律走 PR、不越权） |
| A2 | 合并方式开关（squash/rebase 仍开启） | 项目规则＝只用 merge commit（`docs/01`）；仓库设置与 ruleset 目前允许 merge / squash / rebase 三种 | @Crosery 决定是否收紧（`docs/10` §7-9） |
| A3 | `ci.yml` 纳入 required check | P0 交付 `.github/workflows/ci.yml`；ruleset 暂未包含 required status checks | @Crosery 在 `ci.yml` 落地后加入（`docs/10` §7-2） |

## B. 暂缓（等日后成熟再处理）

| # | 事项 | 定夺（来源） | 重启条件 |
|---|---|---|---|
| B1 | CLI 登录通道（loopback / CLI token） | issue #2 §3.1：本期不做；`GEEK_SID` / `--sid-stdin` 已覆盖 CI 与 agent | 出现明确的首次登录体验需求 → geek_main 开 issue |
| B2 | 服务端推送通道（SSE / Webhook） | issue #2 §3.5：不做；通知靠轮询（05 §A.7） | 后端规划推送能力时另立设计 |
| B3 | 发布元数据接口 | issue #2 §3.2：不做；自更新继续走 GitHub Releases（04） | 需要渠道 / 灰度控制时 |
| B4 | 缓存新鲜度旁路（`?fresh=1` 之类） | issue #2 §3.4：不做；120s 窗口已写入 02 §5 | 自动化被缓存窗口实际伤到时 |
| B5 | 机器人专用限额 / 批量接口 | issue #2 §3.3：暂无机器人额度，额度对所有人一致 | 自动化任务频繁撞 429 时 |
| B6 | API 手册 8 处差异修正 | issue #2 §4：由 geek_main 侧开 issue 修；CLI 先按服务端行为（01 §8） | 手册更新后，01 §8 收敛为指针 |
| B7 | npm 发布失败的自动补发 | 现为手动 re-run（10 §7-7） | 发版流程自动化时 |

## C. 已定夺、明确不做（留痕，避免反复讨论）

- 旧命令 shim、PAT / Device flow 兼容：不做（01 §6）。
- 为 CLI 新增服务端接口：不做（issue #2 §3.2）。
- 未收录接口的 CLI 命令（官网投递、公开意见墙、dev 快照等）：不做（03 §11）。

## D. 指向（避免双份清单）

- 工程欠账与已确定缺口（PR CI、notes / worktree 脚本、测试缺位、fmt/clippy 历史欠账、README forum 命令、`stage` 分支等）：见 `docs/10` §7。
- fork 删除（我方操作项）：需 `gh auth refresh -h github.com -s delete_repo` 后执行 `gh repo delete LYsnowQ/geek-cli --yes`。

## E. 维护约定（规则的一部分）

- **新增**：任何未定事项先加一条（编号 + 现状/证据 + 待确认人/触发条件 + 来源），再讨论。
- **确认后**：标记「已定夺（日期）+ 结论 + 落地 issue/PR 链接」，**不删行**（留痕）；随后按 `docs/10` 流程开 issue 实施。
- **框架冻结**：issue #2 关闭后，本目录（`docs/architecture/`）视为冻结版；后续对框架的修改必须新开 issue，不在本目录直接改。
- 本文件随架构修订同 PR 更新（`docs/10` §4 的同步义务同样适用）。
