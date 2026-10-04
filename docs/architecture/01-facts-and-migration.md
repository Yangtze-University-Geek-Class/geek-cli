# 01 · 事实基座与旧实现处置

> 状态：`accepted` · 更新：2026-10-02 · 适用范围：geek-cli 重构（对接 `yangtzeu.work` 核心服务）。
> 事实来源：[`docs/api-forum-console.md`](../api-forum-console.md)（接口唯一真相；由 geek-cli 侧按 admin 侧 `API.md` 同步维护）；本章只记结论与处置，不复制字段细节（规则单源）。
> 约束与验证：实现 PR 的每条命令映射必须能指回手册章节；不一致时以手册为准，停手报告，不按旧代码猜。

## 1. 唯一真相与协作约束

- 接口路径、请求/响应字段、错误码、限流、编号格式，一律以 API 手册为准；本目录不重述。
- API 手册由 **geek-cli 侧维护**（2026-10-03 交接）：按 admin 侧 `docs/architecture/API.md` 同步，冲突以 admin 为准；要改接口行为仍改后端。CLI 侧发现手册与实测不符 → 记录证据并同步手册（交叉验证）。
- 旧代码、旧 README、旧 SKILL.md 中与手册冲突的表述一律视为**过期**，在当前重构中直接替换。

## 2. 服务面概览（按域）

| 域 | 前缀 | 鉴权 | 用途（CLI 视角） |
|---|---|---|---|
| 会话 | `/auth/*` | 匿名 + `sid` | `login/logout` 探测与登出；`/auth/me` 判活 |
| 我的组织 | `/api/me/orgs` | `sid` | 「先选组织，再调 admin 域」 |
| 公共 | `/api/public/*`、`/api/join/*` | 匿名 | 配置、PoW 难度、邀请链接（info/redeem） |
| 控制台 | `/api/console/*` | `sid` + 能力（capability） | 概览、称号/部门/任命、投递、意见箱、审计 |
| GitHub 网关 | `/api/admin/:org/*` | `sid` + 组织角色（member/admin） | 旧 `org/member/invite/repo/activity` 的替代通路 |
| 论坛 | `/api/forum/*` | 游客可读；成员写（`sid`） | 帖子、搜索、通知、资料/头像 |
| 探活 | `/healthz` | 匿名 | 连通性自检（`geek status`） |

## 3. 环境与 base（手册 §1.1）

| 环境 | base | 说明 |
|---|---|---|
| 正式 | `https://yangtzeu.work` | 默认 |
| 预发布 | `https://prev.yangtzeu.work` | 独立数据 |
| 本机 | `http://127.0.0.1:3000` | 直连核心 server；3456/5186 是前端 dev 端口，CLI 不用 |

- 三个环境的会话相互独立（cookie host-only）→ **按 base 分别登录、分别存储**。
- 解析顺序：`--base` > `GEEK_BASE` > 配置文件 > 默认正式；`--env prod|prev|local` 是常用三环境的语法糖。

## 4. 旧实现过期判定（模块级）

| 文件 | 旧做法 | 判定 | 处置 |
|---|---|---|---|
| `src/auth.rs` | GitHub OAuth Device flow 换 PAT（gh CLI client id）| **过期**（手册 §1.2/§2：无 PAT、无 Device flow、无论坛独立登录） | 删除；按 [02](02-session-and-transport.md) 实现 `sid` 会话 |
| `src/gh.rs` | PAT 直连 `api.github.com`，自造分页/错误映射 | **过期**（组织操作收归网关，服务端白名单+角色+审计+缓存） | 删除；新 `api/admin.rs` 走 `/api/admin/:org/*` |
| `src/forum.rs` | Bearer PAT + 旧路径（`/api/forum/categories`、`/threads`、`/posts` 自造形状） | **过期**（路径/编号/`changes` 增量/写策略全变） | 重写为 `/api/forum/*` 手册形状 |
| `src/config.rs` | `token.json` 单凭据（0600） | **过期**（多环境、无 token 概念） | 改 `session.json`（按 base）+ `config.json`（偏好） |
| `src/cli.rs` | 单文件 822 行、命令面部分对应旧后台 | **部分过期** | 按 [03](03-command-surface.md) 重建；拆分为 `commands/*` |
| `src/output.rs` | `json/pretty/table` | **保留** | 扩展：结构化错误、退出码、`NO_COLOR`、北京时间投影 |
| `install.sh` / `install.ps1` / `npm/**` / `.github/workflows/release.yml` | 分发 | **保留** | 与自更新设计对齐（[04](04-update-and-distribution.md)） |

## 5. 旧命令 → 新命令映射（逐条）

> 处置四类：**保留**（语义与通路等价）、**改造**（换通路/换参数/换输出形状）、**删除**（新 API 无此能力）、**新增**（见 03）。

| 旧命令 | 处置 | 新命令 / 接口 | 备注 |
|---|---|---|---|
| `login` | 改造 | `geek login`（粘贴/stdin/`GEEK_SID`） | 不再有 device code；见 02 §2 |
| `logout` | 保留 | `geek logout`（`POST /auth/signout` + 本地清除） | 未登录调用也成功 |
| `whoami` | 改造 | `geek whoami`（`/auth/me` + `/api/console/me` + `/api/me/orgs`） | 聚合形状 `{health?, auth, console?, orgs?}`（与 `status` 同源，见 02 §7） |
| — | 新增 | `geek status`（`/healthz` + 会话 + 能力摘要） | 排障入口 |
| `org list` | 保留 | `GET /api/me/orgs` | 输出仍在 `orgs[]` |
| `org show <org>` | 改造 | `GET /api/admin/:org/overview` | 响应含 `role/org/counts` |
| `org members <org>` | 保留 | `GET /api/admin/:org/members` | 与 `member list` 同源 |
| `member list/show` | 保留 | 同上；`show` = 客户端过滤单条 | 无单成员接口 |
| `member remove` | 保留 | `DELETE .../members/:login` | admin；要求 `--yes` |
| `member set-role` | 保留 | `PATCH .../members/:login/role` | admin；`--yes` |
| `invite list` | 改造 | `GET .../invitations`（`pending`+`history`） | 输出形状变化 |
| `invite user` / `invite email` | **删除** | 邀请改走**邀请链接**：`geek link create` + 被邀请人 `geek join redeem` | 手册 §3.3/§5：API 没有「直接按用户名邀请」接口 |
| `invite cancel <id>` | 保留 | `DELETE .../invitations/:id` | id 为数字 |
| `repo list/show` | 保留 | `GET .../repos` / `GET .../repos/:repo` | `show` 含分支/协作者/钩子 |
| `repo branches` | 删除（并入） | `repo show` 的分支列表 | 无独立接口 |
| `repo tree` | 保留 | `GET .../tree?ref=&path=` | 参数名对齐手册 |
| `repo cat` | 改造 | `geek repo file` | 目录/超 1MB 的返回形状不同（`too_large`） |
| `repo commits` | 保留 | `GET .../commits?sha=&per_page=&page=` | 新增 `repo commit <sha>`（含 diff） |
| `repo issues` / `repo prs` | 保留 | `GET .../issues|pulls?state=` | 输出形状变化（含更多字段） |
| `repo create` | 改造 | `POST .../create-repo` | `visibility` 增 `internal` |
| `repo delete` | 保留 | `DELETE .../repos/:repo` | `--yes` |
| `repo collab/uncollab` | 保留 | `PUT/DELETE .../collaborators/:login` | admin |
| `activity events` | 保留 | `GET .../activity` | ≤100 条 |
| `forum me` | 删除 | 身份看 `forum state` 的 `viewer` / `console me` | 新 API 无独立 me |
| `forum stats` | 改造 | `forum state` 的 `counters`（`--summary` 投影） | |
| `forum categories` | 改造 | `state.categories` | |
| `forum threads/thread` | 改造 | `state.topics` / `GET /api/forum/topics/:id/posts` | 编号 `t73`；正文单独取 |
| `forum new/reply` | 改造 | `POST /topics` / `POST /posts` | `--body` 明文最长 20000；游客回复走 PoW |
| `forum like/delete-thread/delete-post` | 改造 | `POST /posts/:id/like`、`DELETE /posts/:id` | 话题不可删；首帖不可删；toggle 语义 |
| `forum pin-thread` | 改造 | `POST /topics/:id/pin`（body `{pinned}`） | 授权改为 `forum.topic.pin` 能力 |
| `forum user <name>` | 删除 | 用 `forum search <q>` 或 `state.users` | API 无按名查用户 |
| `forum groups / users / set-role / new-teacher / teacher-overview` | **删除** | — | 新 API 无这些概念（角色体系=称号+能力，见手册 §4/§8） |
| `dashboard` | 保留 | 打开 `<base>/console` | 浏览器打开控制台；命令名与 03 一致 |
| — | 新增 | `console *`（含 `console application *` / `console feedback *`）、`join *`、`self *` | 见 03 全量表；家族名以 `console` 为前缀，不存在顶层 `application` / `feedback` 命令 |

## 6. 版本与兼容策略

- 本次为**破坏性重构**：目标版本 `0.2.0`；**不提供旧命令 shim**（旧机制在后端已不存在，shim 只能报错）。
- Release notes 附本映射表；npm/install 脚本照旧，加自更新说明。
- `SKILL.md` 与 `README.md` 命令面必须同 PR 重写（协作规则 `docs/10` §4）。
- 命令名稳定性从 `0.2.0` 起承诺：改名 = 破坏性变更 + release notes 说明。

## 7. 验证方法

- 实现 PR 附「命令 → API 手册章节」对照（本文件 §5 表中可回溯）。
- 对读命令：本机对正式环境跑一遍，输出与手册字段逐一核对（人工）。
- 对写命令：预发布环境（`prev.yangtzeu.work`）实测 + `--dry-run` 快照；不拿正式环境做试验。

## 8. 服务端行为备忘（2026-10-03 已同步进手册）

> 来源：issue #2 评审 + @Crosery 2026-10-03 交接（admin 候选 `task/190/api_doc_drift@e01ed5a`，实现基线 stage `df4db703`）。以下条目**已同步进 `docs/api-forum-console.md`**；本表保留为 CLI 实现核对清单。

| # | 点 | 服务端行为（手册已同步） |
|---|---|---|
| 1 | 关注关系可见性 | `follows` 全站公开（游客也拿到全部）；只有 `bookmarks` / `notifications` 按 viewer 过滤；「我关注的」按 `followerId === viewer.userId` 筛 |
| 2 | 429 | 路由级限流统一 `{ error: "rate_limited", message: "操作太频繁，请稍后再试", request_id }`（admin#191 / PR #197 已并入 stage）；业务上限 `guest_replies_paused`、`apply_limited` 保留；旧的 `request_error` 例外**已移除** |
| 3 | `request_id` | 只在统一错误处理器输出中有；直接 `send` 的错误没有。join 的 Schema 失败 = 400 `validation_error` + Ajv 英文 message + `request_id`；路由自判的 400/404/503 = `{ error: <中文句子> }` 无 `request_id`；Fastify 自身拒绝（空 JSON body）= Fastify 错误码 + 英文 |
| 4 | join 冲突 | 另有 409 `invite_unavailable`（禁用/过期/次数用完）与 `invite_pending_review`（结果未定，不自动重试）；体 `{ error, message, request_id }`，message 与 error 相同 |
| 5 | 摘要长度 | 帖子 `excerpt`：>200 UTF-16 码元截前 200 + 「…」，最长 **201**；控制台 `strengths_excerpt` 同理，最长 **121** |
| 6 | 长度数法 | 双层：Schema `maxLength` 按 Unicode 码点、路由 `String.length` 按 UTF-16，两层都判的都要满足（游客回复正文/昵称、成员昵称、标签名、搜索词按 UTF-16） |
| 7 | issues / pulls | `octokit.paginate` 每页 50 取完、不分页；`per_page`（超 999 → 400）与 `page` 在这两个接口**不起作用**；分页参数只对 `commits` 生效 |
| 8 | 旧路径与响应头 | 未注册旧路径（`/api/forum/*`、`/auth/forum/*`、`/forum/u/*`、`DELETE /api/forum/state`）→ 410 `legacy_forum_retired`（无 `request_id`）；`HEAD /api/forum/state` → 405；每个响应带 `nosniff` / `X-Frame-Options: DENY` / `Referrer-Policy: strict-origin-when-cross-origin` |
| 9 | security 端点 | 付费组织取 Dependabot 告警失败仍 200，`alerts` 空、`dependabot.error` 为 Octokit message 原文（不脱敏）；`merge_failed` 沿用上游状态码与 message 的说明原本已对，未变 |

- `org_not_whitelisted` 在生产/预发布**不会触发**（两环境 `ALLOWED_ORGS` 为空）——不要把它当常态探测手段。
- admin 侧遗留：文档门禁 `admin#198`、forum contracts 长度注释 `admin#199`（等所有者决定，见 [`07`](07-open-questions.md) §B）。
