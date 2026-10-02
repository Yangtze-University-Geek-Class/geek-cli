# 03 · 命令面设计（全量命令表）

> 状态：`accepted` · 更新：2026-10-02 · 适用范围：CLI 对外命令契约（实现与 SKILL.md/README 以此为准）。
> 事实来源：API 手册（章节逐条标注）；旧命令映射见 [01](01-facts-and-migration.md) §5。
> 约束与验证：每条命令必须标注接口出处与阶段；实现后 README「命令一览」与 `SKILL.md` 同 PR 同步（`docs/10` §4）。

## 1. 组织原则

- 按 API 域分组：`org/member/team/invite/link/repo`（admin 网关）、`console`（控制台）、`forum`（论坛）、`join`（匿名邀请链接）、`self`（升级）、`login/logout/whoami/status/dashboard`（会话与入口）。
- `<org>`、`<org>/<repo>` 保持旧位置参数写法（迁移成本最低）；编号直接用 API 形态（`t73`、`p10001`、`m123`、`n9`、UUID、数字 id）。
- 同一能力只有一个命令名（不留别名）；表里「阶段」= P0 底座 / P1 治理 / P2 论坛与邀请 / P3 打磨（见 06 §4）。
- 输出默认 JSON 且与手册同形状（02 §7）；下表「输出要点」只记形状差异，不重复手册字段。

## 2. 全局参数

| 参数 | 作用 |
|---|---|
| `--format json\|pretty\|table` | 输出模式（默认 json；02 §7） |
| `--env prod\|prev\|local` / `--base <url>` | 目标环境（02 §5） |
| `--yes` | 破坏性/有外部副作用的操作显式确认（非 TTY 必填） |
| `--dry-run` | 只打印将发送的请求，不发送 |
| `--quiet` / `--verbose` | 静默 / HTTP 调试（脱敏） |
| `--no-update-check` / `--no-color` | 关闭版本检查 / 颜色（04、02） |

## 3. A 域 · 会话与入口

| 命令 | 接口 | 说明 | 阶段 |
|---|---|---|---|
| `geek login [--sid-stdin\|--sid]` | `GET /auth/me` | 导入 `sid` 并校验（02 §2） | P0 |
| `geek logout` | `POST /auth/signout` | 尽力登出 + 清本地 | P0 |
| `geek whoami` | `/auth/me` + `/api/console/me` | 聚合身份（附 `sources`） | P0 |
| `geek status` | `/healthz` + `/auth/me` + 可选 `/api/console/me` | 连通性/会话/能力自检；`--json` 输出分项 | P0 |
| `geek dashboard [<org>]` | — | 浏览器打开控制台 | P1 |

## 4. B 域 · GitHub 组织（`/api/admin/:org/*`，手册 §5）

| 命令 | 接口 | 说明 | 阶段 |
|---|---|---|---|
| `geek org list` | `GET /api/me/orgs` | 可用组织 + 角色 | P0 |
| `geek org show <org>` | `GET …/overview` | 概况 + 计数 | P0 |
| `geek org security <org>` | `GET …/security` | 安全能力状态 | P1 |
| `geek org activity <org> [--limit]` | `GET …/activity` | ≤100 条 | P0 |
| `geek org edit <org> [字段…]` | `PATCH …/org` | admin；字段=手册 §5.3 | P2 |
| `geek member list <org>` | `GET …/members` | 含 `viewer_role` | P0 |
| `geek member show <org> <login>` | `GET …/members` | 客户端过滤单条 | P0 |
| `geek member remove <org> <login> --yes` | `DELETE …/members/:login` | admin | P1 |
| `geek member set-role <org> <login> --role` | `PATCH …/members/:login/role` | admin | P1 |
| `geek team list <org>` | `GET …/teams` | | P1 |
| `geek team create <org> <name> [--description --privacy]` | `POST …/teams` | admin | P1 |
| `geek team delete <org> <slug> --yes` | `DELETE …/teams/:slug` | admin | P1 |
| `geek invite list <org>` | `GET …/invitations` | `pending` + `history` | P1 |
| `geek invite cancel <org> <id> --yes` | `DELETE …/invitations/:id` | admin | P1 |
| `geek link list <org>` | `GET …/invite-links` | admin | P2 |
| `geek link create <org> --hours --max-uses [--note --team]` | `POST …/invite-links` | admin；输出含 `url`（token 即凭据，勿入日志） | P2 |
| `geek link disable\|enable <org> <token>` | `PATCH …/invite-links/:token` | admin | P2 |
| `geek link delete <org> <token> --yes` | `DELETE …/invite-links/:token` | admin | P2 |
| `geek repo list <org>` | `GET …/repos` | | P0 |
| `geek repo show <org>/<repo>` | `GET …/repos/:repo` | 分支/协作者/钩子 | P0 |
| `geek repo tree <org>/<repo> [--ref --path]` | `GET …/tree` | | P0 |
| `geek repo file <org>/<repo> <path> [--ref]` | `GET …/file` | 目录/超 1MB 有专门形状 | P0 |
| `geek repo commits <org>/<repo> [--sha --path --per-page --page]` | `GET …/commits` | | P0 |
| `geek repo commit <org>/<repo> <sha>` | `GET …/commits/:sha` | 含 diff（patch 截断） | P1 |
| `geek repo issues <org>/<repo> [--state]` | `GET …/issues` | 不含 PR | P0 |
| `geek repo issue show <org>/<repo> <n>` | `GET …/issues/:n` | 详情 + 评论 | P1 |
| `geek repo issue comment <org>/<repo> <n> --body` | `POST …/issues/:n/comments` | | P1 |
| `geek repo issue state <org>/<repo> <n> --state open\|closed` | `PATCH …/issues/:n` | | P1 |
| `geek repo prs <org>/<repo> [--state]` | `GET …/pulls` | | P0 |
| `geek repo pr show <org>/<repo> <n>` | `GET …/pulls/:n` | 详情 + 评论 + 文件 | P1 |
| `geek repo pr merge <org>/<repo> <n> [--method merge\|squash\|rebase --sha] --yes` | `PUT …/pulls/:n/merge` | 默认 `merge`；YUGC 仓库规范要求 merge commit | P1 |
| `geek repo create <org> <name> [--visibility --description --no-init --gitignore --license]` | `POST …/create-repo` | admin；`visibility` 增 `internal` | P1 |
| `geek repo delete <org>/<repo> --yes` | `DELETE …/repos/:repo` | 不可恢复 | P1 |
| `geek repo collab <org>/<repo> <login> [--permission]` | `PUT …/collaborators/:login` | admin | P1 |
| `geek repo uncollab <org>/<repo> <login> --yes` | `DELETE …/collaborators/:login` | admin | P1 |
| （可选，P3）`geek org logs <org>`、`geek org feedback *` | `GET …/logs`、`GET …/feedback` | 旧管理端入口，admin；默认不暴露，避免与 console 域重复 | P3 |

## 5. C 域 · 控制台（`/api/console/*`，手册 §4）

| 命令 | 接口 | 说明 | 阶段 |
|---|---|---|---|
| `geek console me` | `GET /api/console/me` | 称号 + 能力 + `blocked` | P0 |
| `geek console summary` | `GET …/summary` | 权限内子集 | P1 |
| `geek console catalogue` | `GET …/catalogue` | 配置类只读数据 | P1 |
| `geek console people` | `GET …/people` | 全员 + 称号 | P1 |
| `geek console audit [--limit --offset --action]` | `GET …/audit` | 脱敏审计 | P1 |
| `geek console department list` | `GET …/departments` | | P1 |
| `geek console department create <id> --name --tag --icon --tone [--head-caps --member-caps --sort-order]` | `POST …/departments` | admin（`admiral_required`） | P2 |
| `geek console department edit <id> [字段… --archived]` | `PATCH …/departments/:id` | | P2 |
| `geek console department delete <id> --yes` | `DELETE …/departments/:id` | 连带指派一起删 | P2 |
| `geek console title set <title-id> [--label --tag --icon --tone --description --capabilities]` | `PATCH …/titles/:id` | admin/captain 限制见手册 | P2 |
| `geek console assignment list [--department --role]` | `GET …/assignments` | | P1 |
| `geek console assignment assign <login> --role <captain\|head\|member\|alumni> [--department --note]` | `POST …/assignments` | 舰长移交有感 | P1 |
| `geek console assignment revoke <id> --yes` | `DELETE …/assignments/:id` | | P1 |
| `geek console application list [--status --q --limit --offset]` | `GET …/applications` | | P1 |
| `geek console application show <uuid>` | `GET …/applications/:id` | 含审核历史 | P1 |
| `geek console application review <uuid> --status … [--expected-status --expected-review-id --note --notify --time --place --notes --message] --yes` | `PATCH …/applications/:id` | 未传 `expected-*` 时 CLI 先读一次自动填入（仍防陈旧覆盖）；发信有副作用 | P1 |
| `geek console application export [--status --out <file>]` | `GET …/applications/export.csv` | `--out` 落盘；stdout 时原样输出 CSV | P1 |
| `geek console feedback list [--status --limit]` | `GET …/feedback` | | P1 |
| `geek console feedback reply <id> [--status --reply]` | `PATCH …/feedback/:id` | | P1 |
| `geek console feedback delete <id> --yes` | `DELETE …/feedback/:id` | | P1 |

## 6. D 域 · 论坛（`/api/forum/*`，手册 §6）

| 命令 | 接口 | 说明 | 阶段 |
|---|---|---|---|
| `geek forum state [--summary]` | `GET /api/forum/state` | `--summary` 只投影计数/分类/置顶 | P2 |
| `geek forum topics [--category --limit]` | 同上（客户端投影） | 按 `lastActivityAt` 排序 | P2 |
| `geek forum topic <t-id> [--posts]` | `state` + `GET …/topics/:id/posts` | `--posts` 拉全部正文 | P2 |
| `geek forum search <q>` | `GET …/search?q=` | ≤50/类 | P2 |
| `geek forum new --category <id> --title <t> (--body\|--body-file) [--tags a,b]` | `POST …/topics` | 成员；输出 `topicId`/`postId` | P2 |
| `geek forum reply <t-id> (--body\|--body-file) [--reply-to <p-id>] [--guest-name <名>]` | `POST …/posts` | 无会话 + `--guest-name` → 本地 PoW；Turnstile 启用时明确报错 | P2 |
| `geek forum edit <p-id> (--body\|--body-file)` | `PATCH …/posts/:id` | 作者或版务 | P2 |
| `geek forum delete <p-id> [--yes]` | `DELETE …/posts/:id` | 软删；首帖不可删 | P2 |
| `geek forum like <p-id>` / `bookmark <p-id>` / `follow <user-id>` | `POST …/like|bookmark|users/:id/follow` | toggle；永不重试（02 §5.1） | P2 |
| `geek forum pin <t-id> [--off]` / `close <t-id> [--off]` | `POST …/topics/:id/pin|close` | 能力门；`--off` 反向 | P2 |
| `geek forum view <t-id>` | `POST …/topics/:id/view` | 显式计数；`topic` 不自动计 | P2 |
| `geek forum notifications [--limit]` | `state.notifications` | 仅自己 | P2 |
| `geek forum read <n-id>` / `read-all` | `POST …/notifications/:id/read|/read-all` | | P2 |
| `geek forum profile set [--display-name --bio --location --website --notify-reply --notify-like --notify-follow]` | `PATCH …/me/profile` | 至少一项 | P2 |
| `geek forum profile avatar <file>` / `--clear` | `PUT|DELETE …/me/avatar` | 图片原样上传，非 JSON | P2 |

## 7. E 域 · 匿名邀请与探活

| 命令 | 接口 | 说明 | 阶段 |
|---|---|---|---|
| `geek join info <token>` | `GET /api/join/:token` | 匿名；`valid`/`reason` | P2 |
| `geek join redeem <token> (--login X\|--email Y) [--note]` | `POST /api/join/:token` | 本地算 PoW；蜜罐字段不暴露；`turnstile_site_key` 非空时报错并提示走浏览器 | P2 |
| `geek health` | `GET /healthz` | 也可用 `geek status` | P0 |

## 8. F 域 · 升级与自身

| 命令 | 说明 | 阶段 |
|---|---|---|
| `geek self info` | 版本/目标平台/安装渠道/配置路径 | P3 |
| `geek self update [--check] [--version <tag>] [--force]` | 显式自更新（04 §3） | P3 |
| `geek self rollback` | 回滚到备份版本 | P3 |

## 9. 危险操作与确认（必须 `--yes`，非 TTY 缺省即失败）

删除类：`repo delete`、`repo uncollab`、`member remove`、`team delete`、`invite cancel`、`link delete`、`department delete`、`feedback delete`、`assignment revoke`、`forum delete`；
有外部副作用/难以撤销类：`pr merge`、`application review`（可能发信）、`member set-role`（降级）、`application export`（审计 + 限流 5 次/分钟，可不加 `--yes` 但会提示）。
所有写操作支持 `--dry-run`（02 §7）。

## 10. 人类用法示例 / agent 用法示例

```bash
# 人类（表格 + 北京时间）
geek repo list Yangtze-University-Geek-Class --format table
geek console application list --status interview --format table
geek forum topics --category agent-mcp

# agent / 脚本（JSON 透传 + jq，退出码判定）
geek console me | jq '.capabilities'
geek repo pr show Yangtze-University-Geek-Class/admin 42 | jq '.files[].filename'
geek console application review "$APP_ID" --status interview --time "周六 14:00" \
  --place "行政楼 302" --notes "准备 3 分钟自我介绍" --yes
# 失败时按退出码分支：3=重新登录 4=权限 7=限流 8=参数
```

## 11. 明确不做（与 API 手册 §9 对齐）

官网上投递/公开意见墙/门户文档接口、dev 快照接口、直连 `/forum` 页面前缀、部署运维接口——CLI 不提供命令。
