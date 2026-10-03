# 极客班论坛与控制台 · 接口使用手册（geek-cli 对接用）

> 状态：`current` · 整理日期：2026-10-02 · **更新：2026-10-03**（按 admin 候选文档 `task/190/api_doc_drift@e01ed5a` 与 #191 / PR #197 的 429 修复同步 §1.3 / §1.4 / §3.3 / §5.3 / §6.2 / §6.3；实现基线 stage `df4db703`）。
> 事实来源：`geek_main` 仓库现行代码
> （`app/server/src/routes/console/**`、`app/server/src/routes/forum-api/**`、`app/server/src/routes/admin/**`、
> `app/server/src/routes/portal/{org,join}.ts`、`app/server/src/middleware/**`、`app/server/src/lib/roles.ts`、
> `app/server/src/lib/forum-rules.ts`、`docs/architecture/API.md`）。
> 维护：geek-cli 侧按 admin 侧 `docs/architecture/API.md` 同步，冲突以 admin 为准（issue #2 交接，2026-10-03）。
> 收录范围：登录与会话（§2）、公共接口：配置 / PoW / 邀请链接（§3）、控制台（§4）、GitHub 组织接口（§5）、论坛（§6）、
> 探活与只读文本（§7）；取舍说明见 [§9](#9-取舍与未收录)。

## 目录

- [1. 通用约定](#1-通用约定)
- [2. 登录与会话 `/auth/*`](#2-登录与会话-auth)
- [3. 公共接口：配置、PoW 与邀请链接](#3-公共接口配置pow-与邀请链接)
- [4. 控制台 `/api/console/*`](#4-控制台-apiconsole)
- [5. GitHub 组织接口 `/api/admin/:org/*`](#5-github-组织接口-apiadminorg)
- [6. 论坛 `/api/forum/*`](#6-论坛-apiforum)
- [7. 探活与只读文本](#7-探活与只读文本)
- [8. 枚举与常量附录](#8-枚举与常量附录)
- [9. 取舍与未收录](#9-取舍与未收录)

---

## 1. 通用约定

### 1.1 基地址与环境

| 环境 | 地址 | 说明 |
|---|---|---|
| 正式 | `https://yangtzeu.work` | 官网、论坛（`/forum/`）、控制台（`/console`）同域 |
| 预发布 | `https://prev.yangtzeu.work` | 同上，数据独立 |
| 本地开发 | `http://127.0.0.1:3000` | 核心服务（server）直连；论坛前端在 3456、控制台在 5186 时各自代理 `/api` 到 3000 |

所有接口都是**同域相对路径**：`/api/console/*`、`/api/forum/*`。CLI 只需一个 base（默认正式域名），无需拼接 `/console`、`/forum` 页面前缀。

### 1.2 认证：统一登录 `sid` cookie

- 登录入口是核心服务的 GitHub OAuth：`GET /auth/github?return_to=...` → GitHub → `GET /auth/callback`，成功后**签发 `sid` cookie**（HttpOnly、`SameSite=Lax`、`Path=/`、有效期 7 天、host-only 不写 Domain；HTTPS 环境带 `Secure`）。
- 官网、论坛、控制台共用这一个 `sid`；**论坛没有自己的登录接口，也没有 PAT / Device flow / API token 机制**。
- 携带方式：请求头 `Cookie: sid=<值>`。
- 只有 `CONSOLE_ORG`（默认 `Yangtze-University-Geek-Class`）的 **active 组织成员**能登录成功；登录失败时回跳地址带 `signin=not_member|invite_pending|cancelled|failed`。
- 论坛：**没有 `sid`（或会话失效、登录后被移出组织）= 游客**。游客能看帖、能按 PoW 规则回复；其余写操作要成员。
- 控制台：所有接口先要求登录，再按**能力（capability）**判定，见 §4。
- 会话失效：带 `sid` 的请求若 GitHub 侧拒绝了该会话的令牌，服务端结束会话并回 `401 session_expired`（之后按未登录处理）。
- 未登录固定错误：`401 { "error": "not_signed_in" }`。

> ⚠️ 对接提醒：CLI 要拿到 `sid` 需要自己实现浏览器 OAuth 抓 cookie 或等后端支持 CLI 登录（**当前不存在**）；见 §2。

### 1.3 请求与响应格式

- 请求/响应一律 JSON（唯一例外：论坛头像上传/下载，见 §6）。
- **未知字段会被拒绝**：控制台所有 body 与 query、论坛所有 body 都开 `additionalProperties: false`（多一个字段 → `400 validation_error`）；论坛的读接口（`state`/`topics/:id/posts`/`search`）不校验多余 query。
- 长度分两层校验：Schema（Ajv 的 `minLength`/`maxLength`，`unicode` 保持默认）按 **Unicode 码点**（星平面字符算 1）；路由自判长度按 **UTF-16 码元**（`String.length`，星平面字符算 2）。两层都判的字段**两个上限都要满足**（例：标题 120 码点可含 120 个表情；游客回复正文按 UTF-16 数 2000，1001 个表情即 `content_too_long`）。按 UTF-16 数的还有：游客昵称 20 / 成员昵称 30（都先 NFKC 且 trim）、新建标签名 20（trim 后）、搜索词 100（trim 后）。
- 不带请求体的 POST（点赞、收藏、关注、浏览、已读）**不要发 `Content-Type: application/json` 的空 body**——Fastify 会按空 JSON 解析并拒绝（400）。CLI 干脆不发 body、不发 Content-Type。
- 响应头：`/api/*` 与 `/auth/*` 一律 `Cache-Control: no-store`；唯一例外是论坛头像 `GET /api/forum/avatars/<hash>.webp` 的 200（长期缓存）；server 的每个响应（含错误、404 与静态文件）另带 `X-Content-Type-Options: nosniff`、`X-Frame-Options: DENY`、`Referrer-Policy: strict-origin-when-cross-origin`（只描述 server 本身，不声称已验证部署代理链）。
- 资源编号：`/api/console/*` 里名叫 `:id` 的参数是数字（如意见箱 id、指派 id）；投递 id 叫 `:application_id` 且是 UUID；论坛编号是字符串（见 §1.6）。

### 1.4 错误模型

错误体统一形状：

```json
{ "error": "<机器码>", "message": "<中文，可直接展示>", "request_id": "<请求 id>" }
```

> 形状有例外：`request_id` 只在经统一错误处理器输出的错误里出现（路由抛错、Schema 校验失败、限流、Fastify 自身拒绝、GitHub 上游错误、`session_expired`）；**直接 `send` 的错误没有**——如 401 `not_signed_in`（只有 `error`）、403 `invalid_origin` / `missing_capability` / `org_not_whitelisted` / `not_a_member_of_org` / `requires_org_admin`、410 `legacy_forum_retired`、413 `avatar_too_large`，以及 `/api/join/:token` 路由自判的 400/404/503（`{ error: <中文句子> }`，`error` 不是机器码）。join 的 Schema 校验失败是另一形状：400 `{ error: "validation_error", message: <Ajv 英文>, request_id }`；Fastify 自己拒绝的请求（如空 JSON body）`error` 是 Fastify 错误码（如 `FST_ERR_CTP_EMPTY_JSON_BODY`）、`message` 英文。**不要假设 `error` 恒为机器码、`message` 恒为中文。**

| HTTP | 含义 | 典型机器码 |
|---|---|---|
| 400 | 参数/校验失败 | `validation_error`（提示写明哪个字段）；业务码如 `invalid_status`、`letter_required`、`empty_content` |
| 401 | 未登录/会话失效 | `not_signed_in`、`session_expired`、`signin_required`（论坛成员操作） |
| 403 | 没有权限或来源不符 | `missing_capability`（控制台，附带 `capability`、`any_of`、可选 `reason`）、`invalid_origin`、`forbidden`、`admiral_required`、`out_of_department_scope` |
| 404 | 资源不存在 | `not_found` |
| 410 | 旧接口已停用 | `legacy_forum_retired`（无 `request_id`；见 §6.3 注） |
| 409 | 冲突 | `status_changed`、`post_deleted`、`assignment_exists`、`department_exists`、`captain_transfer_required` |
| 413 / 415 | 请求体超限 / Content-Type 不支持 | `avatar_too_large`、`unsupported_media_type` |
| 429 | 限流 | 路由级统一 `rate_limited`（见下）；业务上限 `guest_replies_paused`（论坛游客全站） |
| 5xx | 服务异常 | `internal_error`（脱敏，不暴露上游原文）；GitHub 上游 4xx 映射为 `upstream_rejected` |

- GitHub 上游错误统一映射，不回显上游原文；上游 5xx/无状态码一律脱敏为 5xx `internal_error`。
- **429 约定**：路由级限流统一为 `{ error: "rate_limited", message: "操作太频繁，请稍后再试", request_id }`（admin#191/#197 起；join / export.csv / assignments / feedback 等一致）；业务上限另用专属机器码（本手册涉及 `guest_replies_paused`；`apply_limited` 属未收录的官网投递）。
- 两处按设计回上游原文：`PUT …/pulls/:n/merge` 失败的 `merge_failed`（沿用上游状态码）与 `GET …/security` 的 `dependabot.error`（Octokit message）。
- 控制台缺能力时 `403 missing_capability` 体里：`capability` 是 `any_of[0]`，`reason` 只在被 GitHub 上限挡掉时出现（`github_admin_required` / `github_membership_required`），`message` 形如「需要「查看投递」权限」。

### 1.5 写请求的来源校验（CLI 重点）

`middleware/http-policy` 对**非 GET/HEAD/OPTIONS**请求做来源检查：

- 有 `Origin` 头且 ≠ 本环境 `PUBLIC_ORIGIN` → `403 invalid_origin`；
- 没有 `Origin` 但 `Sec-Fetch-Site: cross-site` → `403 invalid_origin`。

**不带 `Origin`、不带 Sec-Fetch-Site 的命令行客户端可以正常写**（"bearer-only clients remain usable"）。所以 CLI 不要带 `Origin` 头。

### 1.6 论坛编号格式（写进路径参数）

| 实体 | 形状 | 正则（与代码一致） | 备注 |
|---|---|---|---|
| 话题 | `t73` | `^t[1-9][0-9]{0,8}$` | 新话题从 `t1001` 起 |
| 帖子 | `p10001`、旧帖首帖 `body-73` | `^(?:p[1-9][0-9]{0,9}\|body-[1-9][0-9]{0,8})$` | 新帖子从 `p10001` 起 |
| 用户 | 成员 `m<github_user_id>`、游客 `g<n>`、官方账号 `u-<slug>` | `^(?:m[1-9][0-9]{0,14}\|g[1-9][0-9]{0,9}\|u-[a-z0-9-]{1,32})$` | 格式不对 → 400 |
| 通知 | `n9` | `^n[1-9][0-9]{0,9}$` | |
| 头像文件 | 64 位小写十六进制 + `.webp` | `^[0-9a-f]{64}\.webp$` | |

### 1.7 会话与身份的返回示例

- 控制台：`GET /api/console/me` 返回登录名、称号、能力清单（见 §4.2）。
- 论坛：`GET /api/forum/state` 的 `state.viewer` = `{ userId, kind: "guest"|"member", capabilities }`；写接口的回答里也带同一个 `viewer`。
- 论坛成员能力只含 `forum.*` 前缀的能力；能力来源与控制台同一条 `computeAccess` 路径（控制台改了权限包，论坛下一次请求就生效）。

---

## 2. 登录与会话 `/auth/*`

> 控制台的所有接口、论坛的所有写操作，都以本节登录得到的 `sid` cookie 为前提。CLI 必须先解决登录。论坛与控制台共用这一套会话。

### 2.1 登录流程（浏览器 OAuth）

```text
GET /auth/github?return_to=<本环境地址>  ──302──▶  GitHub 授权页
GitHub ──▶ GET /auth/callback?code=…&state=…  ──核对 state cookie、查组织成员──▶
  成功：写 sid cookie，302 回 return_to
  失败：不建会话，302 回 return_to 并带 ?signin=<原因>
```

- 只有 `CONSOLE_ORG` 的 active 成员能登录成功；不是成员（含邀请未接受）不建会话。
- `return_to` 白名单：**必须与本环境 `PUBLIC_ORIGIN` 同源**（缺省或不合规时回退到 `<PUBLIC_ORIGIN>/console`）；拒绝 `//` 开头、含反斜杠/空白/控制字符、带用户名密码的地址；长度 ≤2048。
- 会话 cookie：`sid`（HttpOnly、`SameSite=Lax`、`Path=/`、有效期 7 天、host-only 不写 Domain；HTTPS 环境带 `Secure`）。

### 2.2 接口

| 方法与路径 | 参数 | 成功 | 错误与说明 |
|---|---|---|---|
| `GET /auth/github` | `return_to?` | 302 到 GitHub 授权页；同时写签名的 `oauth_state` cookie | 参数不合规时静默回退到 `<PUBLIC_ORIGIN>/console`，不报错 |
| `GET /auth/callback` | `code`、`state`、`error?` | 一律 302 回 `return_to`：成功带 `sid`；未成功带 `signin=not_member\|invite_pending\|cancelled\|failed` | 400 `missing_params`（缺 `state`，或 code 与 error 都没有）、400 `invalid_state`（state 签名/有效期/cookie 不符）；410 `legacy_forum_retired`（state 以 `forum-` 开头）。这三种返回 JSON，不跳转 |
| `POST /auth/signout` | 无（可选 `sid`） | 200 `{ "ok": true }`，清 `sid` 与旧 `forum_sid` cookie | 未登录调用也返回 ok |
| `GET /auth/me` | 无（可选 `sid`） | 200 `{ "signed_in": false }` 或 `{ "signed_in": true, "login", "user_id", "avatar_url", "console_link" }` | GitHub 拒绝会话里的令牌时：清会话，返回 `{ "signed_in": false, "session_expired": true }`；其余情况不因 GitHub 出错而失败 |

- `signin` 原因（回跳地址上的查询参数）：`not_member`（不在组织）、`invite_pending`（组织邀请还没接受）、`cancelled`（用户在 GitHub 取消）、`failed`（换 token/取用户/查成员出错）。
- `console_link`：登录者持有 `console.access`、`github.org.read`、`feedback.read` **之外**的任意能力时为 `true`（管理者）；普通舰员/领航员为 `false`；查询 GitHub 出错按 `false` 处理。它只决定界面显不显示入口，控制台准入仍按能力判定。
- `/auth/me` 在**没有** `sid` 时也返回 200（`signed_in: false`），不会 401。

### 2.3 CLI 拿 `sid` 的可行路径（当前均未由后端提供）

1. **浏览器登录 + 抓 cookie**：用户在浏览器完成 `/auth/github` 流程后，把 `sid` 交给 CLI（手工粘贴或读取浏览器 cookie 存储）。这是现状下唯一无需改后端的方式。
2. **本地回环**：CLI 起本地回调地址——但 `return_to` 只接受与 `PUBLIC_ORIGIN` 同源的地址，回环地址不在白名单，**直接走不通**，需要后端放开或新增 CLI 登录通道。
3. **后端加 CLI 登录（device flow / CLI token）**：根治办法，当前不存在。

> 配置筛查：CLI 不需要自带 OAuth 客户端配置，也不接触服务端密钥；`sid` 按凭据保存（等同密码），不要写进日志或明文配置。

### 2.4 `GET /api/me/orgs`（会话可用的组织）

- **作用**：列出当前会话（token）能用的组织与本人角色；CLI 用它「先选组织，再调 `/api/admin/:org/*`」。
- **鉴权**：`sid`；**错误**：401 `not_signed_in` / `session_expired`。
- **参数**：无。
- **响应 200**：

```json
{
  "orgs": [ { "login": "Yangtze-University-Geek-Class", "name": "…", "avatar_url": "…", "role": "admin" | "member", "state": "active", "html_url": "https://github.com/…" } ],
  "allowed_orgs": ["yangtze-university-geek-class"]
}
```

- 数据来自 GitHub `GET /user/memberships/orgs`（`state=active`）；部署配置了 `ALLOWED_ORGS` 时只返回白名单内的组织（大小写不敏感），`allowed_orgs` 是这份白名单本身（未配置时为空数组）。
- 结果按登录名缓存 **120 秒**；刚改过组织成员关系的场景注意这一点。

## 3. 公共接口：配置、PoW 与邀请链接

### 3.1 公共配置 `/api/public/*`

两个接口都**匿名**可调，不需要 `sid`：

| 方法与路径 | 参数 | 响应 200 |
|---|---|---|
| `GET /api/public/config` | 无 | `{ "turnstile_site_key": string \| null, "pow_difficulty": number }` |
| `GET /api/public/org` | 无 | `{ "tones": {…}, "titles": [ … ], "departments": [ … ] }` |

- `turnstile_site_key`：`null` = 该环境没启用 Turnstile；非空时匿名写接口要带人机验证 token。
- `pow_difficulty`：PoW 需要的前导十六进制 `0` 个数（默认 3）。论坛 `state.guestPolicy.powDifficulty` 给的是同一值。
- `GET /api/public/org`：`titles` 按 `admin、captain、head、member、alumni、guest` 排，每项 `{ id, label, tag, icon, tone, description, rank }`；`departments` 只含未归档部门 `{ id, name, tag, icon, tone, description }`；**不含权限包、不含任何人**。控制台改过称号/部门后立即生效，官网「组织架构」与论坛称号都用它。

### 3.2 PoW（游客回复等匿名写接口的前置）

客户端找一个 `nonce`，使：

```text
sha256(`${timestamp}:${bodyForHash}:${nonce}`) 的十六进制结果以 pow_difficulty 个 0 开头
```

- `timestamp`：毫秒整数，与服务端时钟偏差 ≤5 分钟（否则按过期处理）。
- `nonce`：1–32 字符。
- `bodyForHash`：按接口定义；**论坛游客回复**是 `` `${topicId}:${content}` ``（正文原样、不去空白）。
- 参考实现：`nonce` 用递增数字循环哈希；难度 3 期望约 4096 次，毫秒级。

### 3.3 邀请链接 `/api/join/:token`（匿名）

> 邀请链接由组织 admin 创建（创建入口见 §5 的 `POST /api/admin/:org/invite-links`）；**token 本身即凭据**，持有者可用创建者的授权发出组织邀请。

| 方法与路径 | 参数 | 成功 | 错误与说明 |
|---|---|---|---|
| `GET /api/join/:token` | 路径 `token` | 200 `{ org, note, team_slug, expires_at, remaining_uses, valid, reason }`；`reason` 无效时是「邀请链接已被禁用 / 邀请链接已过期 / 邀请链接使用次数已用完」之一 | 404 `邀请链接不存在` |
| `POST /api/join/:token` | 见下 | 200 `{ ok: true, invitation_id, message }` | 见下；限流 5 次/分钟/客户端 |

**POST 请求体**：

| 字段 | 约束 |
|---|---|
| `github_login` | 与 `email` 至少填一个；`^[a-zA-Z0-9](?:[a-zA-Z0-9]|-(?=[a-zA-Z0-9])){0,38}$` |
| `email` | 与 `github_login` 至少填一个；`^\S+@\S+\.\S+$` 形状 |
| `note` | ≤280 字（超长截断） |
| `pow` | 同 §3.2；**摘要输入是 `join:<token>:<trim 后的 github_login>:<trim 后的 email>`**（缺的字段按空串） |
| `website` / `homepage` / `url_ref` | 蜜罐，必须为空（否则按机器人处理） |
| `turnstile_token` | `/api/public/config` 的 `turnstile_site_key` 非空时必填 |

**行为与错误**：

- 邀请用链接创建者的 token 发出；创建者授权已失效 → 503「邀请链接的发起人 token 失效，请联系管理员重新生成链接」。
- **幂等**：同一链接、同一收件人（登录名或邮箱）已成功发出过 → 直接回 200 `{ ok, invitation_id, message: "邀请已发出，请查看 GitHub 通知。" }`，不重复发。
- 已知失败 → 400，文案按上游状态区分：`该用户已在组织中`、`GitHub 拒绝邀请（账号不存在或邮箱已被邀请）`、`GitHub 用户名不存在，请检查拼写`、`邀请失败，稍后重试`。
- **结果未知**（上游 5xx/超时）→ 503「邀请结果待核对，请勿重复提交并联系管理员」；这种情况 CLI 不要自动重试，让人去核对。
- **409（经错误处理器，`{ error, message, request_id }`，`message` 与 `error` 相同）**：`invite_unavailable`（链接已禁用、已过期或次数用完）、`invite_pending_review`（同一链接、同一收件人上一次的结果还没定——正在发或结果不确定、等管理员核对）；`invite_pending_review` 属**结果未定，不要自动重试**。
- 400 分两种形状：Schema 校验（`validation_error` + Ajv 英文 `message` + `request_id`；未知字段、`github_login`>39、`email`>254、`note`>280、`pow` 形状不符等）与路由自判（`{ error: <中文句子> }`，无 `request_id`：蜜罐非空、PoW 不对、两者都没填、格式不对、Turnstile 未过、已知失败）。404 `邀请链接不存在` 同为路由自判、无 `message`/`request_id`。
- 字段校验：两者都没填 → 400「需要填写 GitHub 用户名或邮箱」；格式不对 → 400「GitHub 用户名格式无效」/「邮箱格式无效」；PoW、蜜罐、人机验证失败使用公开表单的固定文案（见 §3.2）。
- 每次尝试都会写 `invitations` 历史（组织 admin 可在 `GET /api/admin/:org/invitations` 的 `history` 里看到）；审计操作者记作 `public:<链接 token>`（控制台审计展示时截前 6 位）。

## 4. 控制台 `/api/console/*`

模块：`app/server/src/routes/console/`；组织固定为 `CONSOLE_ORG`（路径里没有 `:org`）；全部接口先 `requireAuth`，再 `requireCapability(...)`（`GET /me` 只要求登录）。

### 4.1 能力模型速查

- `GET /api/console/me` 返回本人 `capabilities`（有顺序的能力 id 数组）、`titles`（称号视图）、`blocked`（被 GitHub 上限挡掉的能力）、`head_of`（本人负责的部门 id）。
- 授权只看服务端能力；`403 missing_capability` 里的 `capability` 提示缺哪项。
- 组织 owner（admin）永远拥有全部能力；`console.access` 持有任意一项能力就自动获得。
- 依赖关系（蕴含）：`github.*.manage → github.org.read`；`applications.review|export → applications.read`；`feedback.manage → feedback.read`；`roles.manage → roles.department.manage`。
- 只有提督和舰长能持有 `roles.manage`，且不能放进部门权限包（`400 captain_only_capability`）。

完整能力清单见 §8.1。

### 4.2 接口总表

| # | 方法 路径 | 作用 | 鉴权 | 限流 |
|---|---|---|---|---|
| 1 | `GET /api/console/me` | 当前身份：登录名、称号、能力、被挡能力、负责部门 | 登录 | 无 |
| 2 | `GET /api/console/summary` | 概览统计（按本人能力返回子集） | `console.access` | 无 |
| 3 | `GET /api/console/catalogue` | 称号/色调/能力清单/领域/默认权限包/图标/投递状态 | `console.access` | 无 |
| 4 | `GET /api/console/departments` | 部门列表（含负责人、人数、权限包） | `console.access` | 无 |
| 5 | `POST /api/console/departments` | 新建部门 | `roles.manage` + GitHub 组织 owner | 无 |
| 6 | `PATCH /api/console/departments/:department_id` | 修改部门（含归档） | 同上 | 无 |
| 7 | `DELETE /api/console/departments/:department_id` | 删除部门 + 其全部指派 | 同上 | 无 |
| 8 | `PATCH /api/console/titles/:title_id` | 改称号（名字/标签/图标/色调/说明/权限包） | `roles.manage` | 无 |
| 9 | `GET /api/console/assignments?department_id=&role=` | 称号指派列表 | `roles.manage` 或 `roles.department.manage` | 无 |
| 10 | `POST /api/console/assignments` | 任命（含舰长移交） | 同上 | 30 次/分钟 |
| 11 | `DELETE /api/console/assignments/:id` | 撤销指派 | 同上（按目标行判断） | 无 |
| 12 | `GET /api/console/people` | 成员全名单（含称号，按层级排序） | `roles.manage` 或 `roles.department.manage` | 无 |
| 13 | `GET /api/console/applications` | 投递列表（筛选/搜索/分页/计数） | `applications.read` | 无 |
| 14 | `GET /api/console/applications/export.csv` | 导出 CSV | `applications.export` | 5 次/分钟 |
| 15 | `GET /api/console/applications/:application_id` | 投递详情（审核历史 + 信件） | `applications.read` | 无 |
| 16 | `PATCH /api/console/applications/:application_id` | 改状态 / 写备注 / 发通知信 | `applications.review` | 无 |
| 17 | `GET /api/console/feedback` | 意见箱列表 | `feedback.read` | 无 |
| 18 | `PATCH /api/console/feedback/:id` | 改意见状态 / 回复 | `feedback.manage` | 无 |
| 19 | `DELETE /api/console/feedback/:id` | 删除意见 | `feedback.manage` | 无 |
| 20 | `GET /api/console/audit` | 审计日志（org 固定，按前缀筛选） | `audit.read` | 无 |

### 4.3 接口详解

#### 1. `GET /api/console/me`

- **作用**：当前登录者在控制台的身份。CLI 用它做 `whoami` / 权限自检。
- **参数**：无。
- **响应 200**：

```json
{
  "login": "crosery",
  "avatar_url": "https://avatars.githubusercontent.com/u/...",
  "org": "Yangtze-University-Geek-Class",
  "github_role": "admin" | "member" | null,
  "title": { "id": "captain", "label": "舰长", "tag": "CAPTAIN", "icon": "star-filled", "tone": "amber", "department": null, "source": "assignment", "assignment_id": 12 },
  "titles": [ /* TitleView[]，主称号在前（rank 最小） */ ],
  "capabilities": ["console.access", "applications.read", "..."],
  "blocked": [ { "capability": "github.repos.manage", "reason": "github_admin_required" } ],
  "head_of": ["tech"]
}
```

- `TitleView`：`{ id, label, tag, icon, tone, department: { id, name, tag, icon, tone } | null, source: "assignment"|"github"|"none", assignment_id }`。
- `blocked[].reason`：`github_admin_required`（能力要组织 admin，本人只是 member）或 `github_membership_required`（已不在组织）。
- **错误**：401 `not_signed_in` / `session_expired`。

#### 2. `GET /api/console/summary`

- **作用**：概览页统计；**只返回调用者有权限看的键**（没有权限的键整块不出现）。
- **响应 200**：

```json
{
  "applications": { "total": 120, "by_status": { "received": 80, "interview": 20, "accepted": 5, "rejected": 15 }, "last_7d": 9 },
  "feedback": { "open": 3, "total": 40 },
  "people": { "assignments": 25, "departments": 4 }
}
```

- 分块权限：`applications` ← `applications.read`；`feedback` ← `feedback.read`；`people` ← `roles.manage` 或 `roles.department.manage`。

#### 3. `GET /api/console/catalogue`

- **作用**：控制台所有「配置类」只读数据；称号名字/标签/图标/色调/说明与权限包以数据库为准（控制台改过就变）。
- **响应 200**：

```json
{
  "titles": [ { "id": "admin", "label": "提督", "tag": "ADMIRAL", "icon": "user-admin", "tone": "violet", "rank": 0, "description": "…" } ],
  "crew": { "tag": "CREW", "tone": "slate", "rank": 3 },
  "tones": { "amber": "#855700", "cobalt": "#3346C8", "…": "…" },
  "capabilities": [ { "id": "applications.read", "domain": "applications", "label": "查看投递", "description": "查看投递" } ],
  "domains": [ { "id": "console", "label": "控制台" } ],
  "role_base": { "admin": ["..."], "captain": ["..."], "head": ["..."], "member": ["..."], "alumni": ["..."], "guest": [] },
  "implies": { "applications.review": ["applications.read"] },
  "captain_only": ["roles.manage"],
  "department_icons": ["star-filled", "badge", "…"],
  "application_statuses": [ { "id": "received", "label": "已收到" } ]
}
```

#### 4. `GET /api/console/departments`

- **响应 200**：`{ "departments": [ DepartmentView ] }`，已归档的排在最后。

```
DepartmentView = {
  id, name, tag, icon, tone, description,
  head_capabilities: string[], member_capabilities: string[],
  sort_order: number, archived: boolean,
  heads: string[],      // 负责人的 GitHub 登录名
  crew_count: number
}
```

#### 5. `POST /api/console/departments`

- **鉴权**：`roles.manage` **且** GitHub 组织 owner（否则 403 `admiral_required`）。
- **Body**（未知字段 400；必填：`id`、`name`、`tag`、`icon`、`tone`、`head_capabilities`）：

| 字段 | 约束 |
|---|---|
| `id` | `^[a-z][a-z0-9-]{1,31}$`，已存在 → 409 `department_exists` |
| `name` | 2–12 字 |
| `tag` | `^[A-Z][A-Z0-9-]{1,15}$` |
| `icon` | §8.4 的 18 个图标名之一 |
| `tone` | §8.3 的 8 个色调之一 |
| `description` | ≤200 字 |
| `head_capabilities` | 能力 id 数组（不能含 `roles.manage` → 400 `captain_only_capability`） |
| `member_capabilities` | 可选，同上 |
| `sort_order` | 可选，整数 0–10000 |

- **响应 201**：`{ "department": DepartmentView }`；审计 `department.create`。
- **错误**：400 `validation_error` / `captain_only_capability`；403 `missing_capability` / `admiral_required`；409 `department_exists`。

#### 6. `PATCH /api/console/departments/:department_id`

- 同上鉴权；body 是上一节（`POST /api/console/departments`）字段的任意子集（至少 1 个），另可 `"archived": true|false` 归档/取消归档。
- **响应 200**：`{ "department": DepartmentView }`；审计 `department.update`（`changed` 字段列表）。
- **错误**：404 `not_found`；403 `admiral_required`；400 同上。

#### 7. `DELETE /api/console/departments/:department_id`

- **作用**：同一事务删除部门与它的全部 head/member 指派（撤掉指派的人失去该部门称号）。
- **响应 200**：`{ "ok": true, "removed": 3 }`（`removed` = 撤掉的指派条数）；审计 `department.delete`（含被撤名单）。
- **错误**：404 `not_found`；403 `admiral_required`。
- 默认部门只在首次启动写入，删掉后重启不会补回。

#### 8. `PATCH /api/console/titles/:title_id`

- **`title_id`**：`admin` / `captain` / `head` / `member` / `alumni` / `guest`（其它值 400）。
- **鉴权**：`roles.manage`；其中 `admin`、`captain` 两个称号**只有 admin 本人**能改（否则 403 `admiral_required`）。
- **Body**（至少 1 个字段）：

| 字段 | 约束 |
|---|---|
| `label` | 1–8 字，不能只有空白 |
| `tag` | `^[A-Z][A-Z0-9-]{1,15}$` |
| `icon` | §8.4 图标名之一 |
| `tone` | §8.3 色调之一 |
| `description` | ≤200 字 |
| `capabilities` | 能力 id 数组；`admin`/`guest` 固定 → 400 `title_capabilities_fixed`；非 captain 放 `roles.manage` → 400 `captain_only_capability` |

- **响应 200**：`{ "title": { id, label, tag, icon, tone, description, capabilities: [] } }`；审计 `title.update`。

#### 9. `GET /api/console/assignments`

- **Query**：`department_id`（`^[a-z][a-z0-9-]{1,31}$`）、`role`（`captain|head|member|alumni`），都可选。
- **权限行为**：持 `roles.manage` 看全部；只持 `roles.department.manage` 时只看本人负责部门（传别的部门 → 403 `out_of_department_scope`）。
- **响应 200**：

```json
{
  "assignments": [ { "id": 12, "github_login": "crosery", "github_user_id": 123456, "role": "head", "department_id": "tech", "note": null, "granted_by": "admin-login", "created_at": 1750000000000 } ],
  "captain": { "github_login": "someone" }
}
```

- `captain` 没有现任舰长时为 `null`。

#### 10. `POST /api/console/assignments`

- **鉴权**：`roles.manage` 任意角色；`roles.department.manage` 只能给自己负责部门任命 `role=member`。
- **Body**：

| 字段 | 约束 |
|---|---|
| `github_login` | GitHub 用户名（大小写不敏感，服务端转小写） |
| `role` | `captain` / `head` / `member` / `alumni` |
| `department_id` | `head` 必填（缺 → 400 `department_required`）；`captain`/`alumni` 必须不带（带 → 400 `department_not_allowed`）；部门不存在或已归档 → 400 `unknown_department` |
| `note` | ≤200 字，可选 |

- **行为**：用调用者自己的 GitHub token 查该账号（查不到 → 400 `github_user_not_found`）；`captain` 任命会在事务里移交现任舰长（审计 `role.captain.transfer`，否则 `role.assign`）。
- **响应 201**：`{ "assignment": AssignmentRow }`。
- **错误**：400（上表）；403 `missing_capability` / `out_of_department_scope` / `captain_required`（captain 只能由提督或现任舰长指定）；409 `assignment_exists`；429。

#### 11. `DELETE /api/console/assignments/:id`

- `:id` 是数字指派 id。
- **响应 200**：`{ "ok": true }`；审计 `role.revoke`。
- **错误**：404 `not_found`；409 `captain_transfer_required`（舰长那条只有本人或 admin 能撤；其他人先移交）；403 越权时。

#### 12. `GET /api/console/people`

- **响应 200**：`{ "people": [ { "login": "x", "user_id": 123, "avatar_url": "...", "github_role": "admin"|"member"|null, "titles": [TitleView] } ] }`。
- 名单 = GitHub 组织全部成员 **+ 有称号但已不在组织里的人**（后者 `github_role: null`）。
- 排序：主称号 rank（小者在前）→ 登录名。
- 每个人的 `titles` 与本人登录后看到的一致（owner 是提督，无指派默认舰员）。

#### 13. `GET /api/console/applications`

- **Query**：`status`（`received|interview|accepted|rejected`，其它值含 `reviewing` → 400）、`q`（≤100 字，按姓名/班级/邮箱 LIKE，`%`/`_`/`\` 当字面量）、`limit`（1–200，默认 50）、`offset`（0–99999999，默认 0）。
- **响应 200**：

```json
{
  "items": [ { "id": "uuid", "name": "张三", "class_name": "25 级", "email": "a@b.c", "strengths_excerpt": "前 120 字…", "status": "received", "created_at": 1750000000000, "last_review": { "to_status": "interview", "reviewer": "crosery", "created_at": 1750000001000 } } ],
  "total": 120,
  "counts": { "received": 80, "interview": 20, "accepted": 5, "rejected": 15 }
}
```

- `strengths_excerpt`：取前 120 个 UTF-16 码元，截断时末尾加「…」，**最长 121**。

#### 14. `GET /api/console/applications/export.csv`

- **Query**：`status`（可选）。
- **响应 200**：`text/csv; charset=utf-8`，带 UTF-8 BOM，`Content-Disposition: attachment; filename="applications-YYYYMMDD.csv"`（北京日期）。
- 列：`name,class_name,email,strengths,status,created_at_beijing`（时间形如 `2026-09-27 01:05:00`，北京时间）；单元格以 `= + - @ \t \r` 开头时前面加 `'` 防公式注入。
- 审计 `application.export`；限流 5 次/分钟。

#### 15. `GET /api/console/applications/:application_id`

- `:application_id` 是 UUID（格式不符 400）。
- **响应 200**：

```json
{
  "application": { "id": "uuid", "name": "…", "class_name": "…", "email": "…", "strengths": "完整正文", "status": "received", "created_at": 0 },
  "reviews": [ { "id": 3, "from_status": "received", "to_status": "interview", "note": "…", "reviewer": "crosery", "created_at": 0, "mail": MailSummary } ],
  "received_mail": MailSummary | null,
  "mail": { "enabled": true, "recipients": "all" | "allowlist", "deliverable": true }
}
```

- `MailSummary = { status, skip_reason, attempts, subject, sent_at, updated_at }`（**不含**收件地址与正文）。
  - `status`：`pending`（排队）/ `sending`（正在发）/ `sent`（已发出）/ `failed`（发不出去，已放弃）/ `skipped`（没发）；`skip_reason`：`mail_disabled`（没配发信商）、`not_allowlisted`（白名单外）、`rate_limited`（全站这一小时上限）；历史版本还写过 `recipient_limited`、`source_limited`，现在不再产生。
- 审计 `application.view`；404 `not_found`。

#### 16. `PATCH /api/console/applications/:application_id`

- **作用**：改投递状态（可选发通知信）、写审核备注。审核历史是追加式，不修改旧记录。
- **Body**：

| 字段 | 约束 |
|---|---|
| `status` | 四个状态之一；其它 → 400 `invalid_status` |
| `expected_status` | 页面上看到的状态；和库里不同 → 409 `status_changed` |
| `expected_review_id` | 页面上最大的审核记录 id（无记录填 0）；不同 → 409 `status_changed`。**要改状态时这两项都必填**（旧页面不带给 409「这个页面是旧版本，刷新后再改」） |
| `note` | ≤2000 字，只给审核人看，**不进信、不进审计** |
| `notify` | 默认 `true`；`false` 表示这次不发信 |
| `letter` | 发信内容：`time`（≤60，待面试必填）、`place`（≤120，待面试必填）、`notes`（≤1000，待面试=面试说明 / 已录取=接下来要做的事）、`message`（≤1000，未通过=原因） |

- **行为**：状态真的变成 `interview`/`accepted`/`rejected` 且 `notify !== false` 时，同一事务写审核记录 + 入发信队列（接口不等发信结果）；改回 `received`、只写备注、`notify:false` 都不发信。信件在写库前渲染，内容不合规直接 400 且状态不变。
- **响应 200**：`{ "application": {…}, "review": { …, "mail": MailSummary } }`。
- **错误**：400 `invalid_status` / `no_change`（状态没变且无备注）/ `letter_required`（要发待面试信但缺时间/地点，带 `fields`）/ `letter_invalid`（信渲染不出来，例如没配回信地址） / `validation_error`；404；409 `status_changed`。

#### 17. `GET /api/console/feedback`

- **Query**：`status`（`open|triaged|in_progress|done|wont_do|spam`，可选）、`limit`（1–500，默认 200）。
- **响应 200**：`{ "items": [ { "id": 1, "category": "…", "content": "…", "contact": "…", "submitter_login": "…" | null, "status": "open", "reply": "…" | null, "replied_by": "…" | null, "replied_at": 0 | null, "created_at": 0 } ], "counts": { "open": 3 } }`。
- 提交者的 IP、User-Agent 等只在服务端，不下发。

#### 18. `PATCH /api/console/feedback/:id`

- **Body**（至少 1 个字段）：`status`（上面 6 个之一）、`reply`（≤5000 字，写回复时记录回复人与时间）。
- **响应 200**：`{ "ok": true }`；审计 `feedback.update`；**错误**：404 `not_found`。

#### 19. `DELETE /api/console/feedback/:id`

- **响应 200**：`{ "ok": true }`；审计 `feedback.delete`；404 `not_found`。

#### 20. `GET /api/console/audit`

- **Query**：`limit`（1–200，默认 100）、`offset`（≥0）、`action`（≤64 字，按前缀 LIKE）。
- **响应 200**：`{ "logs": [ { "id": 9, "actor": "crosery", "action": "application.review", "target": "uuid" | null, "details": {…} | null, "ip": "1.2.3.4", "created_at": 0 } ] }`。
- 只含 `org = CONSOLE_ORG` 的行；邀请链接 token 只下发前 6 位 + `…`。
- 常见写操作 `action`：`department.create|update|delete`、`title.update`、`role.assign`、`role.captain.transfer`、`role.revoke`、`application.view|review|export`、`feedback.update|delete`。

---

## 5. GitHub 组织接口 `/api/admin/:org/*`

**用途**：控制台 `/console/github/**` 页面（组织概况、成员、仓库与仓库详情、团队、活动、安全、组织资料、邀请、邀请链接、新建仓库）背后的接口。

### 5.1 通用规则

- **鉴权**：`sid` + `ALLOWED_ORGS` 白名单 + GitHub 组织角色。`:org` 是组织登录名；**与 capability 无关**（本族仍按 GitHub 组织角色控制，不经过控制台能力门）。
- **角色要求**：`member` = 该组织任意成员；`admin` = 该组织 owner。错误：
  - 401 `not_signed_in` / `session_expired`；
  - 403 `org_not_whitelisted`（组织不在部署的 `ALLOWED_ORGS` 里，体里带 `org`）、`not_a_member_of_org`、`requires_org_admin`（体里带 `your_role`）；
  - 上游 GitHub 4xx 按原状态码返回 `upstream_rejected`（不回显上游原文），5xx/无状态码脱敏为 `internal_error`。
- **数据来源**：全部用调用者自己的 GitHub token 调 GitHub REST；可以当作 GitHub API 的网关。
- **缓存**：`repos`、`repos/:repo` 缓存 60 秒，`commits`、`issues`、`pulls` 缓存 30 秒（按调用者分开）；写操作会失效相关缓存。CLI 刚改完立刻读，可能看到旧值。
- 写操作都会写审计（`org = :org`）。

### 5.2 接口总表

| 方法 路径 | 作用 | 最低角色 |
|---|---|---|
| `GET /api/admin/:org/overview` | 组织概况 + 统计 | member |
| `GET /api/admin/:org/activity` | 组织最近事件（≤100 条） | member |
| `GET /api/admin/:org/security` | 安全能力状态（2FA / Dependabot / 扫描 / 审计日志） | member |
| `GET /api/admin/:org/org` | 组织资料（GitHub 原始对象） | member |
| `PATCH /api/admin/:org/org` | 修改组织资料（字段见下） | admin |
| `GET /api/admin/:org/members` | 成员列表（含角色） | member |
| `DELETE /api/admin/:org/members/:login` | 移除成员 | admin |
| `PATCH /api/admin/:org/members/:login/role` | 改成员角色（admin/member） | admin |
| `GET /api/admin/:org/teams` | 团队列表（含成员/仓库数） | member |
| `POST /api/admin/:org/teams` | 新建团队 | admin |
| `DELETE /api/admin/:org/teams/:slug` | 删除团队 | admin |
| `GET /api/admin/:org/repos` | 仓库列表 | member |
| `GET /api/admin/:org/repos/:repo` | 仓库详情（分支/协作者/钩子） | member |
| `GET /api/admin/:org/repos/:repo/tree?ref=&path=` | 目录树 | member |
| `GET /api/admin/:org/repos/:repo/file?ref=&path=` | 文件内容 | member |
| `GET /api/admin/:org/repos/:repo/commits?sha=&path=&per_page=&page=` | 提交列表 | member |
| `GET /api/admin/:org/repos/:repo/commits/:sha` | 提交详情（含 diff） | member |
| `GET /api/admin/:org/repos/:repo/issues?state=` | Issue 列表（不含 PR） | member |
| `GET /api/admin/:org/repos/:repo/pulls?state=` | PR 列表 | member |
| `GET /api/admin/:org/repos/:repo/issues/:n` | Issue 详情 + 评论 | member |
| `GET /api/admin/:org/repos/:repo/pulls/:n` | PR 详情 + 评论 + 文件 | member |
| `PUT /api/admin/:org/repos/:repo/pulls/:n/merge` | 合并 PR | member |
| `PATCH /api/admin/:org/repos/:repo/issues/:n` | 开/关 Issue 或 PR | member |
| `POST /api/admin/:org/repos/:repo/issues/:n/comments` | 评论 Issue / PR | member |
| `POST /api/admin/:org/create-repo` | 新建仓库 | admin |
| `DELETE /api/admin/:org/repos/:repo` | 删除仓库 | admin |
| `PUT /api/admin/:org/repos/:repo/collaborators/:login` | 添加/更新协作者 | admin |
| `DELETE /api/admin/:org/repos/:repo/collaborators/:login` | 移除协作者 | admin |
| `GET /api/admin/:org/invitations` | 待接受邀请 + 本地邀请历史 | admin |
| `DELETE /api/admin/:org/invitations/:id` | 取消邀请（id 为数字） | admin |
| `GET /api/admin/:org/invite-links` | 邀请链接列表 | admin |
| `POST /api/admin/:org/invite-links` | 创建邀请链接 | admin |
| `PATCH /api/admin/:org/invite-links/:token` | 停用/启用邀请链接 | admin |
| `DELETE /api/admin/:org/invite-links/:token` | 删除邀请链接 | admin |
| `GET /api/admin/:org/logs` | 审计日志（旧管理端入口） | admin |
| `GET /api/admin/:org/feedback` | 意见箱列表（旧管理端入口） | admin |
| `PATCH /api/admin/:org/feedback/:id` | 改意见状态 / 回复（旧管理端入口） | admin |
| `DELETE /api/admin/:org/feedback/:id` | 删除意见（旧管理端入口） | admin |

### 5.3 参数与响应要点

**组织**

- `GET /overview` → `{ role, org: { login, name, description, avatar_url, html_url, plan, disk_usage_mb, public_repos, total_private_repos, created_at, billing_email, two_factor_required }, counts: { members, repos, pending_invites, invites_24h, active_invite_links } }`。
- `PATCH /org`（admin）可改字段：`name`、`description`、`company`、`email`、`location`、`blog`、`twitter_username`、`billing_email`、`default_repository_permission`（`none|read|write|admin`），以及一组 `members_can_*` 布尔开关（`create_repositories`、`create_public|private|internal_repositories`、`fork_private_repositories`、`create_pages`、`create_public|private_pages`、`invite_outside_collaborators`、`delete_repositories`、`change_repo_visibility`、`delete_issues`）；没有可改字段 → 400 `{ "error": "no editable fields" }`；响应是 GitHub 组织对象。

**成员**

- `GET /members` → `{ members: [ { login, id, avatar_url, html_url, role: "admin"|"member"|"?", state } ], viewer_role }`（查不到成员资格时 role/state 是 `"?"`）。
- `DELETE /members/:login`：不能移除自己（400 `不能移除自己`）。
- `PATCH /members/:login/role`：body `{ "role": "admin"|"member" }`，其它值 400。

**团队**

- `GET /teams` → `{ teams: [ GitHub 团队对象 + member_count + repo_count ] }`。
- `POST /teams`：body `{ "name": 必填, "description"?, "privacy"?: "secret"|"closed" }`；缺 name → 400 `name 必填`；响应 `{ ok, team }`。
- `DELETE /teams/:slug` → `{ ok: true }`。

**仓库**

- `GET /repos` → `{ repos: [ { name, full_name, visibility, default_branch, size_kb, pushed_at, updated_at, stargazers_count, forks_count, open_issues_count, archived, language, topics, html_url, description } ] }`。
- `GET /repos/:repo` → `{ info, branches: [ { name, protected } ], collaborators: [ { login, avatar_url, role, permissions } ], hooks: [ { id, name, active, events, url } ] }`。
- `GET .../tree?ref=&path=` → `{ path, entries: [ { name, path, type: "file"|"dir", size, sha } ] }`（目录在前）；GitHub 上游报错按统一映射。
- `GET .../file?ref=&path=`：缺 `path` → 400 `missing path`；path 是目录 → 400 `path is a directory`；>1MB → `{ path, name, size, too_large: true }`（不带内容）；否则 `{ path, name, size, sha, content, html_url, download_url }`。
- `GET .../commits?sha=&path=&per_page=&page=`：`per_page` 默认 30、上限 100；每条 `{ sha, short_sha, message, message_full, author, committer, actor, html_url }`。
- `GET .../commits/:sha` → `{ sha, message, author, actor, stats, files: [ { filename, status, additions, deletions, changes, patch (截断 30000 字符), previous_filename } ], html_url }`。
- `GET .../issues?state=` 与 `.../pulls?state=`（默认 `open`）：`octokit.paginate` **每页 50 条取完全部**，回答不分页（issues 去掉 PR），按登录者/仓库/`state` 缓存 30 秒；公共 querystring 校验收下的 `per_page`（1–999，超 999 → 400 `validation_error`）与 `page` **在这两个接口不起作用**——分页参数只对 `.../commits` 生效。字段含 `head/base`、`additions/deletions/changed_files`、`merged/mergeable`。
- `GET .../issues/:n` / `GET .../pulls/:n`：详情 + 评论（≤100 条）；PR 文件 patch 截断 20000 字符。
- `PUT .../pulls/:n/merge`：body `{ "merge_method"?: "merge"|"squash"|"rebase", "commit_title"?, "commit_message"?, "sha"?（40 位十六进制，防并发） }`；成功 `{ ok, merged, sha, message }`；失败**不套统一错误体**，按 GitHub 状态码返回 `{ "error": "merge_failed", "message": <上游说明> }`。
- `PATCH .../issues/:n`：body `{ "state": "open"|"closed" }` → `{ ok, state }`（Issue / PR 都可以，PR 即开或关）。
- `POST .../issues/:n/comments`：body `{ "body": 非空 }`（空 → 400 `empty_body`）→ `{ ok, id, html_url }`。
- `POST /create-repo`：body `{ name（`^[a-zA-Z0-9._-]+$`）, description?, visibility: public|private|internal, auto_init?（默认 true）, gitignore_template?, license_template? }` → `{ ok, repo: { name, full_name, html_url } }`；名字不合法 → 400 `仓库名只能含字母数字 . _ -`。
- `DELETE /repos/:repo` → `{ ok: true }`（不可恢复）。
- `PUT .../collaborators/:login`：body `{ "permission"?: "pull"|"triage"|"push"|"maintain"|"admin" }`（默认 `push`）→ `{ ok: true }`；`DELETE` 同路径移除。

**邀请与邀请链接**

- `GET /invitations` → `{ pending: [ GitHub 待接受邀请 ], history: [ { id, invite_link_token, github_login, email, note, source_ip, github_invitation_id, status, error_message, created_at } ] }`（历史最多 200 条）。
- `DELETE /invitations/:id`：`:id` 是数字邀请 id；取消 GitHub 上的待接受邀请。
- `GET /invite-links` → `{ links: [ { token, org, created_by, note, max_uses, current_uses, expires_at, team_slug, disabled, created_at } ] }`。
- `POST /invite-links`：body `{ hours: 1–8760, max_uses: 1–1000, note?: ≤280, team_slug?: string|null }` → 200 `{ ok, token, url: "<PUBLIC_ORIGIN>/join/<token>", expires_at }`。
- `PATCH /invite-links/:token`：body `{ disabled: boolean }` → `{ ok: true }`；不存在 → 404 `not_found`。
- `DELETE /invite-links/:token` → `{ ok: true }`；不存在 → 404。
- ⚠️ 邀请链接 token 是「持有即可用」的凭据：它能用创建者的授权发出组织邀请。控制台审计（§4）只显示前 6 位，旧管理端 `GET /api/admin/:org/logs` 不脱敏；CLI 不要把完整 token 打进日志。

**活动与安全**

- `GET /activity` → `{ events: [ { id, type, actor, actor_avatar, repo, created_at, payload_summary } ] }`（GitHub 组织事件，最多 100 条，`payload_summary` 是服务端压的一行说明）。
- `GET /security` → `{ plan, two_factor_required, dependabot: { supported, alerts, reason, error? }, secret_scanning: { supported, reason }, audit_log: { supported: false, reason } }`。免费组织 `dependabot.supported: false` 并带 `reason`，属正常结果；**付费组织取告警失败时接口仍回 200**：`alerts` 为空数组，`dependabot.error` 是 Octokit 异常 `message` 原文（GitHub 说明 + 文档链接，**不经脱敏**）。

**旧管理端入口（审计日志与意见箱）**

- 这两组是旧管理端页面用的接口（门槛是 GitHub 组织 admin）。控制台页面对应 `GET /api/console/audit` 与 `/api/console/feedback`（capability 门，见 §4）；两者数据同源，按需要选用。
- `GET /logs`：query `limit`（默认 100，≤500）、`offset`；→ `{ logs: [ { id, actor, action, target, details, ip, created_at } ] }`（`details` 是解析后的 JSON 或 null），按时间倒序。**与 `/api/console/audit` 的区别：这里不脱敏**——`invite_link.*` 的 `target` 与 `invite.*` 的 `public:<token>` 操作者原样下发（只有组织 admin 能读）。
- `GET /feedback`：query `status`（`open|triaged|in_progress|done|wont_do|spam`）、`limit`（默认 200，≤500）；→ `{ items: [ { id, category, content, contact, submitter_login, status, reply, replied_by, replied_at, created_at } ], counts }`；提交者 IP 与 User-Agent 只在服务端。
- `PATCH /feedback/:id`：body `{ status?, reply? }`；状态值非法 → 400「状态无效」；→ `{ ok: true }`；不存在 → 404「意见不存在」；审计 `feedback.update`。
- `DELETE /feedback/:id`：→ `{ ok: true }`；不存在 → 404「意见不存在」；审计 `feedback.delete`。

## 6. 论坛 `/api/forum/*`

模块：`app/server/src/routes/forum-api/`。身份只认核心 `sid`（见 §1.2）；存储是核心服务的 `data.db`。

### 6.1 身份与可见性

| 调用者 | 判定 | 能做什么 |
|---|---|---|
| 游客 | 无 `sid`，或会话失效，或登录后已不在 `CONSOLE_ORG` | 读全部；按 PoW + 昵称回复 |
| 成员 | 有效 `sid` 且是 `CONSOLE_ORG` active 成员 | 读全部；发帖/回复/点赞/收藏/关注/改资料/头像；按能力版务 |
| 版务能力 | `forum.topic.pin` 置顶、`forum.topic.close` 关闭、`forum.post.moderate` 编辑/删除他人帖子、在已关闭话题回复 | |

- 成员第一次请求时按 `m<GitHub user_id>` 建论坛用户，之后每次请求刷新 `role`（组织 owner 是 `admin`，其余 `member`）与 `title`（`computeAccess` 排最高的称号）；昵称/签名/头像由本人改，刷新不覆盖。
- GitHub 组织角色缓存 60 秒（移出组织后最长 60 秒生效）；GitHub 出错时成员请求失败（401 → `session_expired`；其它 4xx → `upstream_rejected`；5xx → `internal_error`），**不降级成游客**。

### 6.2 数据结构

#### `GET /api/forum/state` 的响应

```json
{
  "state": {
    "version": 1,
    "seededAt": 0,
    "counters": { "topic": 1050, "post": 12000, "notification": 30, "tag": 8 },
    "users": [ User ],
    "categories": [ Category ],
    "tags": [ Tag ],
    "topics": [ Topic ],
    "posts": [ PostSummary ],
    "notifications": [ Notification ],
    "bookmarks": [ Bookmark ],
    "follows": [ Follow ],
    "viewer": { "userId": "m123" | null, "kind": "guest" | "member", "capabilities": ["forum.topic.pin"] },
    "guestPolicy": { "powDifficulty": 4, "turnstileSiteKey": null | "0x…", "nameMax": 20, "contentMax": 2000 }
  }
}
```

| 实体 | 字段 |
|---|---|
| `User` | `id, username, displayName, bio, location, website, avatarColor, avatarUrl?, kind: "member"\|"guest"\|"official", joinedAt, role: "admin"\|"moderator"\|"member", title?: { id, department? }, notifyPrefs: { reply, like, follow }`（别人的 `notifyPrefs` 是初始值；成员总有 `avatarUrl`，游客/官方账号没有） |
| `Category` | `id, slug, name, description, color, icon` |
| `Tag` | `id, slug, name, color` |
| `Topic` | `id, slug, title, categoryId, tagIds[], authorId, createdAt, lastActivityAt, views, pinned, closed` |
| `PostSummary`（`/state` 里的帖子） | `id, topicId, authorId, createdAt, editedAt?, replyToPostId?, likeUserIds[], deleted?, excerpt`——**不带 `content`**；摘要 = Markdown 压成一行（代码块去掉、链接只留文字），超过 200 个 UTF-16 码元时取前 200 加「…」，**最长 201** |
| `Notification` | `id, recipientId, type: "reply"\|"like"\|"follow"\|"mention"\|"system", actorId, topicId?, postId?, createdAt, read` |
| `Bookmark` | `userId, postId, createdAt` |
| `Follow` | `followerId, followeeId, createdAt` |

- `notifications`、`bookmarks` 只含看的人自己的（游客两者都是空数组）；**`follows` 不按 viewer 过滤，是全站所有关注关系**（谁关注了谁对所有人公开，游客也拿到全部）——要「我关注的」自行按 `followerId === viewer.userId` 过滤。
- 正文另取：`GET /api/forum/topics/:topic_id/posts`（见下表）。

#### 写接口的统一回答（#145）

除 `GET` 外所有接口**不再返回整份 state**，而是：

```json
{
  "changes": { "users": [], "tags": [], "topics": [], "posts": [], "notifications": [], "bookmarks": [], "follows": [],
               "removed": { "bookmarks": [{ "userId": "m1", "postId": "p2" }], "follows": [{ "followerId": "m1", "followeeId": "m2" }] } },
  "viewer": { "userId": "m123", "kind": "member", "capabilities": [] },
  "guestPolicy": { "powDifficulty": 4, "turnstileSiteKey": null, "nameMax": 20, "contentMax": 2000 },
  "topicId": "t1001",   // 仅 POST /topics
  "postId": "p10001"    // POST /topics 与 POST /posts
}
```

- `changes` 只含这次写入改动的记录，每条与同一个人此刻读 `/state` 得到的那条**逐字节相同**（可见性规则一样）。
- 帖子记录两种形状：**改了正文的写入**（发帖、回复、编辑、删除）带 `content`；其余（点赞、收藏等）与 `/state` 一样只有 `excerpt`。`content` 缺失只表示「这次没发正文」，不代表帖子被删（`deleted: true` 才是）。
- 取消书签/关注放在 `changes.removed.*`，只有两端的编号。
- 成员的回答总带自己的用户记录（每次刷新角色/称号/头像）。

### 6.3 接口总表

| # | 方法 路径 | 作用 | 谁能用 | 限流 |
|---|---|---|---|---|
| 1 | `GET /api/forum/state` | 整份论坛状态（帖子只带摘要） | 所有人 | 120 次/分钟/IP |
| — | `HEAD /api/forum/state` | 明确 405（不算 state、不占额度） | — | — |
| 2 | `GET /api/forum/topics/:topic_id/posts` | 一个话题的全部帖子（含正文） | 所有人 | 120 次/分钟/IP |
| 3 | `GET /api/forum/search?q=` | 搜索话题标题/帖子正文/用户名与昵称 | 所有人 | 30 次/分钟/IP |
| 4 | `POST /api/forum/topics` | 发新话题 | 成员 | 10 次/分钟/人 |
| 5 | `POST /api/forum/posts` | 回复（成员或游客） | 成员、游客 | 成员 30 次/分钟/人；游客 5 次/分钟 + 30 次/天/IP，全站 200 次/小时 |
| 6 | `PATCH /api/forum/posts/:post_id` | 编辑帖子 | 作者或 `forum.post.moderate` | 无 |
| 7 | `DELETE /api/forum/posts/:post_id` | 软删除帖子 | 作者或 `forum.post.moderate` | 无 |
| 8 | `POST /api/forum/posts/:post_id/like` | 点赞/取消（切换） | 成员 | 无 |
| 9 | `POST /api/forum/posts/:post_id/bookmark` | 收藏/取消（切换） | 成员 | 无 |
| 10 | `POST /api/forum/users/:forum_user_id/follow` | 关注/取关（切换） | 成员 | 无 |
| 11 | `POST /api/forum/topics/:topic_id/pin` | 置顶/取消 | `forum.topic.pin` | 无 |
| 12 | `POST /api/forum/topics/:topic_id/close` | 关闭/重开 | `forum.topic.close` | 无 |
| 13 | `POST /api/forum/topics/:topic_id/view` | 浏览数 +1 | 所有人 | 60 次/分钟/IP；同 IP 同话题 1 小时只算一次 |
| 14 | `POST /api/forum/notifications/:notification_id/read` | 单条通知已读 | 成员 | 无 |
| 15 | `POST /api/forum/notifications/read-all` | 全部已读 | 成员 | 无 |
| 16 | `PATCH /api/forum/me/profile` | 改自己的资料/通知偏好 | 成员 | 无 |
| 17 | `PUT /api/forum/me/avatar` | 上传头像（原始图片） | 成员 | 10 次/小时/人 |
| 18 | `DELETE /api/forum/me/avatar` | 删除头像（回到 GitHub 头像） | 成员 | 无 |
| 19 | `GET /api/forum/avatars/:file` | 取头像 | 所有人 | 无 |

> 旧路径与方法：`/api/forum`、`/api/forum/*` 里**未注册的路径或方法**、`/auth/forum/*`、`/forum/u/*`（含 `DELETE /api/forum/state`）一律回 410 `{ error: "legacy_forum_retired", message: "旧论坛接口已停用。新论坛的接口见 /api/forum/state。" }`（HEAD 无响应体，无 `request_id`），不把旧接口映射成新接口；`HEAD /api/forum/state` 单独注册，回 405（表中已列）。

### 6.4 接口详解

#### 1. `GET /api/forum/state`

- **响应 200**：`{ "state": { … } }`（见 §6.2）。
- **错误**：429 `rate_limited`；成员请求带 sid 时 GitHub 出错按 §6.1。

#### 2. `GET /api/forum/topics/:topic_id/posts`

- **响应 200**：`{ "posts": [ Post ] }`——这个话题的全部帖子，按楼层排序，每条带 `content`（Markdown 原文；已删除的帖子 `content` 是空串）。
- **错误**：404 `not_found`（话题不存在）；400（编号格式）；429。

#### 3. `GET /api/forum/search?q=`

- **Query**：`q`（≤100 字；空词回三个空数组）。
- **响应 200**：`{ "results": { "topics": [], "posts": [], "users": [] } }`，子串匹配（`%`、`_`、`\` 当字面量），各按时间倒序、每类最多 50 条；`posts` 是带 `excerpt` 的摘要记录（`excerpt` 以命中位置为中心）。
- **错误**：400 `validation_error`（词超 100 字）；429。

#### 4. `POST /api/forum/topics`

- **鉴权**：成员（游客 → 401 `signin_required`）。
- **Body**：

| 字段 | 约束 |
|---|---|
| `title` | 1–120 字；去首尾空白后非空、不能含控制字符（否则 400 `invalid_title`） |
| `categoryId` | `^[a-z0-9][a-z0-9-]{0,39}$`；必须是已有分类（否则 400 `unknown_category`） |
| `tags` | ≤5 项，每项 ≤40 字；按 slug（英文）或名字（中文，不分大小写）找已有标签，找不到才新建 |
| `content` | 1–20000 字；去空白后非空（否则 400 `empty_content`） |

- **响应 201**：`{ …写接口统一回答, "topicId": "t1001", "postId": "p10001" }`，`changes` 含新话题、首帖和话题用到的标签。
- **错误**：401 `signin_required`；400 `validation_error` / `invalid_title` / `unknown_category` / `invalid_tag` / `empty_content`；429。

#### 5. `POST /api/forum/posts`

- **成员**：body `{ topicId, content, replyToPostId? }`；`content` ≤20000 字；30 次/分钟/人。
- **游客**：body 另需：

| 字段 | 约束 |
|---|---|
| `guest.name` | 1–20 字；允许字符集见 §8.6，不合规 400 `invalid_guest_name` |
| `pow` | `{ "timestamp": <number>, "nonce": "<≤32 字>" }`；摘要输入固定为 `` `${topicId}:${content}` ``（正文原样、不去空白），难度取 `guestPolicy.powDifficulty`；不过 → 400 `pow_invalid` |
| `website` | 蜜罐字段，**必须为空**（非空 → 400 `request_rejected`） |
| `turnstileToken` | `guestPolicy.turnstileSiteKey` 非空时必填，不过 → 400 `turnstile_failed` |

- 游客正文 ≤2000 字（超 → 400 `content_too_long`）。
- 游客额度细节：PoW 通过后**每次尝试都占一次额度**（昵称被占、人机验证没过也算）；全站上限只记真正发出的回复。
- 话题已关闭时只有持 `forum.post.moderate` 的人能回（否则 403 `forbidden`）。
- **响应 201**：`{ …统一回答, "postId": "p10001" }`；游客回复的 `changes.users` 另含新建的游客用户。
- **错误**：404 `not_found`（话题不存在）；403 `forbidden`（话题已关闭）；400 `validation_error` / `invalid_reply_target`（被回复的帖子不在这个话题）/ `empty_content` / `request_rejected` / `content_too_long` / `invalid_guest_name` / `guest_name_taken`（昵称被成员或官方账号占用）/ `pow_invalid` / `turnstile_failed`；429 `rate_limited` / `guest_replies_paused`（全站游客回复到上限，稍后再试或登录）。

#### 6. `PATCH /api/forum/posts/:post_id`

- **鉴权**：帖子作者（成员）或 `forum.post.moderate`。
- **Body**：`{ "content": "≤20000 字" }`；写 `editedAt`。
- **响应 200**：统一回答，`changes.posts` 含这条帖子（带 `content`）。
- **错误**：401；403 `forbidden`；404；409 `post_deleted`（已删除的帖子不能编辑）；400 `empty_content`。

#### 7. `DELETE /api/forum/posts/:post_id`

- **鉴权**：作者或 `forum.post.moderate`；软删除。
- **响应 200**：统一回答，`changes.posts` 含这条帖子（`deleted: true`、`content: ""`）。
- **幂等**：已删除的再删一次，直接返回它现在的样子（200）。
- **错误**：401；403；404；400 `first_post`（**话题的第一帖不能删**）。

#### 8. `POST /api/forum/posts/:post_id/like`

- **鉴权**：成员（401 `signin_required`）。
- **响应 200**：统一回答，`changes.posts` 含这条帖子（`likeUserIds` 已切换）；**错误**：401；404；409 `post_deleted`。

#### 9. `POST /api/forum/posts/:post_id/bookmark`

- **鉴权**：成员。
- **响应 200**：加上时 `changes.bookmarks` 含这条书签；取消时在 `changes.removed.bookmarks`。**错误**：401；404。

#### 10. `POST /api/forum/users/:forum_user_id/follow`

- **鉴权**：成员；`:forum_user_id` 是 `m…`/`g…`/`u-…`。
- **响应 200**：关注时 `changes.follows`；取消时 `changes.removed.follows`。**错误**：400 `cannot_follow_self`；401；404。

#### 11. `POST /api/forum/topics/:topic_id/pin`

- **鉴权**：`forum.topic.pin`；**Body**：`{ "pinned": true }`。
- **响应 200**：`changes.topics` 含这个话题；审计 `forum.topic.pin`。**错误**：401；403 `forbidden`；404。

#### 12. `POST /api/forum/topics/:topic_id/close`

- **鉴权**：`forum.topic.close`；**Body**：`{ "closed": true }`。
- **响应 200**：同上（审计 `forum.topic.close`）。**错误**：401；403；404。

#### 13. `POST /api/forum/topics/:topic_id/view`

- **响应 204**（无 body）；同一 IP（IPv6 按 /64）同一话题 1 小时只计一次。**错误**：404；429。

#### 14/15. 通知已读

- `POST /api/forum/notifications/:notification_id/read`：成员；`changes.notifications` 含这条通知；404（不存在或不是自己的）。
- `POST /api/forum/notifications/read-all`：成员；`changes.notifications` 是这次从未读变已读的那些（本来就全读过时没有这一类）。

#### 16. `PATCH /api/forum/me/profile`

- **鉴权**：成员；body 至少一项：

| 字段 | 约束 |
|---|---|
| `displayName` | NFKC 去首尾空白后 1–30 字，只收 §8.6 允许清单；不能等于官方账号昵称/用户名、别人的用户名，或控制台里有称号、登录过的别人登录名（→ 400 `display_name_taken`）；自己的登录名总能用 |
| `bio` | ≤200 字，可换行；控制字符 → 400 `invalid_bio` |
| `location` | ≤60 字；控制字符 → 400 `invalid_location` |
| `website` | 空串清空；否则必须 `https://` 开头、≤200、不带账号密码（否则 400 `invalid_website`） |
| `notifyPrefs` | `{ "reply"?: bool, "like"?: bool, "follow"?: bool }` |

- **响应 200**：统一回答，`changes.users` 只有自己的用户记录。**错误**：401；400（上表 + `validation_error` / `invalid_display_name`）。
- 昵称没变时不查重也不写；昵称规则只管新写入，库里已有的旧昵称不改。

#### 17. `PUT /api/forum/me/avatar`

- **鉴权**：成员；登录与次数在**读请求体之前**校验（游客/超限时请求体不会被读）。
- **请求**：body 是图片本身（非 JSON），`Content-Type: image/png | image/jpeg | image/webp`，≤2MB；服务端按 EXIF 摆正、居中裁方、缩到 256×256 WebP，按内容 sha256 存库；换头像后没人用的旧图删除。
- **响应 200**：统一回答，`changes.users` 只有自己的用户记录（`avatarUrl` = `/api/forum/avatars/<hash>.webp`）。
- **错误**：401；415 `unsupported_media_type`；413 `avatar_too_large`；400 `invalid_image` / `image_too_large`（像素 > 4096×4096）；429（10 次/小时，失败也计数）。

#### 18. `DELETE /api/forum/me/avatar`

- **响应 200**：统一回答，`changes.users` 只有自己的记录，`avatarUrl` 回到 GitHub 头像。**错误**：401。

#### 19. `GET /api/forum/avatars/:file`

- **文件名校验**：`^[0-9a-f]{64}\.webp$`（不符 400）。
- **响应 200**：`image/webp`，`Cache-Control: public, max-age=31536000, immutable`，`Content-Security-Policy: default-src 'none'`。**错误**：404（`no-store`）。

### 6.5 通知规则（CLI 需要理解写操作的副作用）

- 回复通知：话题作者 + 被回复的人；@提及（代码块里的不算，一条帖子最多通知 10 个被 @ 的人）；点赞（每人每帖一次）；关注（每人一次）。
- 只发给成员，不给自己；回复/点赞/关注通知看收件人的 `notifyPrefs`，@提及没有开关。
- 游客回复也会通知话题作者，`actorId` 是游客用户。

---

## 7. 探活与只读文本

| 路径 | 类型 | 说明 |
|---|---|---|
| `GET /healthz` | JSON | `{ "ok": true, "ts": <毫秒> }`，匿名；CLI 判活/连通性用 |
| `/forum/t/<id>.md` | `text/markdown; charset=utf-8` | 话题 Markdown 原文：YAML 头（`title`/`category`/`author`/`created`/`tags`/`replies`/`url`）+ 首帖 + `## 回复`（每条 `### #<楼层> <显示名> (@<用户名>) · <时间>`）；仅构建时公开的旧帖存在，其它 404 |
| `/forum/llms.txt` | `text/plain; charset=utf-8` | 全部公开话题清单（每个分类一节，每行 `- [标题](.md 地址): 首帖摘要`），置顶在前、其余按发帖时间从新到旧 |

- 这两个文件是论坛前端 `nuxt generate` 的**静态产物**（不是 API），内容停在构建时刻；之后新发的话题与回复不会出现，请求没有对应文件的地址就是 404。
- 线上前缀是 `/forum`（由 web 容器反代到论坛容器）；直连核心 server 的 `/forum` 在生产返回 503 `forum_service_not_ready`，开发态 302 到论坛前端。
- 话题页 `<head>` 在该 `.md` 存在时有 `<link rel="alternate" type="text/markdown">`。

## 8. 枚举与常量附录

### 8.1 能力清单（20 项，顺序即 `/api/console/me` 的返回顺序）

| id | 业务域 | 名称 |
|---|---|---|
| `console.access` | console | 进入控制台（持有任意能力就自动获得） |
| `github.org.read` | github | 查看 GitHub 组织 |
| `github.org.manage` | github | 修改组织资料 |
| `github.members.manage` | github | 管理组织成员 |
| `github.repos.manage` | github | 管理仓库 |
| `github.teams.manage` | github | 管理团队 |
| `github.invites.manage` | github | 管理邀请 |
| `forum.topic.pin` | forum | 置顶话题 |
| `forum.topic.close` | forum | 关闭话题 |
| `forum.post.moderate` | forum | 管理帖子（编辑/删除他人帖子；在已关闭话题回复） |
| `forum.category.manage` | forum | 管理分类（预留） |
| `forum.badge.assign` | forum | 授予论坛徽章（预留，本期没有接口） |
| `applications.read` | applications | 查看投递 |
| `applications.review` | applications | 审核投递 |
| `applications.export` | applications | 导出投递 |
| `feedback.read` | feedback | 查看意见箱 |
| `feedback.manage` | feedback | 处理意见 |
| `audit.read` | audit | 查看审计日志 |
| `roles.manage` | roles | 管理称号与部门 |
| `roles.department.manage` | roles | 任免本部门成员 |

### 8.2 称号

| id | 默认名 | 标签 | 默认图标 | 默认色调 | rank |
|---|---|---|---|---|---|
| `admin` | 提督 | ADMIRAL | user-admin | violet | 0 |
| `captain` | 舰长 | CAPTAIN | star-filled | amber | 1 |
| `head` | 队长 | LEADER | badge | cobalt | 2 |
| （`member` + 部门） | 部门 · 舰员 | CREW | 部门图标 | slate | 3 |
| `alumni` | 领航员 | NAVIGATOR | compass | jade | 4 |
| `member` | 舰员 | CREW | code | sky | 5 |
| `guest` | 乘客 | PASSENGER | user | slate | 9 |

可指派角色（`POST /assignments` 的 `role`）：`captain`、`head`、`member`、`alumni`（`admin` 由 GitHub owner 自动获得、`guest` 是没登录的人，都不能指派）。

### 8.3 色调 `tone`

`amber` `#855700`、`cobalt` `#3346C8`、`violet` `#6E44C9`、`jade` `#18694A`、`sky` `#08609A`、`coral` `#A63F16`、`rose` `#B4235A`、`slate` `#5B6475`。

### 8.4 部门图标（18 个，不带 `i-carbon-` 前缀）

`star-filled` `badge` `code` `compass` `user` `user-follow` `terminal` `forum` `application` `bullhorn` `education` `idea` `trophy` `user-favorite` `chart-network` `logo-github` `book` `user-admin`

### 8.5 状态枚举

- 投递状态：`received`（已收到）、`interview`（待面试）、`accepted`（已录取）、`rejected`（未通过）——只有这四个；列表筛 `reviewing` 会 400。
- 意见状态：`open`、`triaged`、`in_progress`、`done`、`wont_do`、`spam`。
- 论坛通知类型：`reply`、`like`、`follow`、`mention`、`system`。
- 论坛用户类型：`member`、`guest`、`official`；论坛角色：`admin`、`moderator`、`member`。

### 8.6 论坛游客昵称 / 成员昵称允许字符

NFKC（全角字母数字变半角）→ 去掉首尾空白后：只收汉字、平假名、片假名、韩文（合成音节）、拉丁字母（基本/拉丁-1/扩展 A/拼音声调）、ASCII 数字、长音符 `ー`，以及空格和 `- _ . · ・ '`。
空格不能在首尾或连着用；至少要有一个字或数字。不合规 → `invalid_guest_name` / `invalid_display_name`。

### 8.7 论坛长度与限流常量

| 项 | 值 |
|---|---|
| 话题标题 | ≤120 字 |
| 帖子正文 | 成员 ≤20000 字；游客 ≤2000 字 |
| 游客昵称 / 成员昵称 | 20 / 30 字 |
| 标签 | 每帖 ≤5 个，每个 ≤40 字 |
| 签名 / 所在地 / 网站 | 200 / 60 / 200 字 |
| 头像 | ≤2MB；源图 ≤4096×4096；输出 256×256 WebP |
| 搜索词 | ≤100 字 |
| 一条帖子最多 @通知 | 10 人 |
| 游客回复 | 每 IP 5 次/分钟、30 次/天；全站 200 次/小时 |
| 成员发话题 / 回复 | 10 / 30 次/分钟 |
| 头像上传 | 10 次/小时 |
| 读接口（state / topicPosts / search / view） | 120 / 120 / 30 / 60 次/分钟/IP |

### 8.8 计时与展示

- 控制台时间一律**北京时间（Asia/Shanghai）**展示；导出 CSV 的 `created_at_beijing` 与文件名日期也是北京时间。
- 接口时间戳都是**毫秒整数**。

---

## 9. 取舍与未收录

**已收录**（2026-10-02 补录）：登录与会话（§2，含 `GET /api/me/orgs`）、公共接口：配置 / PoW / 邀请链接（§3）、控制台（§4）、GitHub 组织接口（§5，含旧管理端 `logs` 与 `feedback`）、论坛（§6）、探活与只读文本（§7）。

**确定不收录**：

- dev 专用：`/api/local-forum/state`、`/api/local-forum/assets/:hash`、`/__geek_forum`——只在本机配置了只读快照的 dev 服务器存在，与 CLI、生产无关。
- 官网 portal 自己的功能：`GET /api/docs`、`GET /api/docs/:id`、`GET /api/feedback/categories`、`POST /api/feedback`、`GET /api/feedback/public`、`POST /api/portal/apply`——CLI 当前没有对应命令（`/api/feedback/public` 是官网公开意见墙；`/api/portal/apply` 是官网「加入我们」投递，匿名，PoW + 蜜罐 + 同一设备/IP 24 小时最多 5 份）。
- 直连 server 的 `/forum`：开发态 302 到论坛前端、生产 503；页面走 web 容器的 `/forum` 前缀，CLI 用不到。
- 部署与运维：没有对外 API（部署只由发布 tag 触发），不收录。

**待定**：无——原先列出的 6 项已全部处理：`/api/me/orgs`、`logs`、`feedback`、`/api/join/*` 已补录；`/api/portal/apply` 与直连 `/forum` 确认不需要。

---

## 附：CLI 快速自检清单

- [ ] 连通性：`GET /healthz` 返回 `{"ok":true}`；注意 base 与环境的对应（正式/预发布/本机三套数据独立）。
- [ ] 会话：`GET /auth/me` 探测；无 `sid` 时按 §2.3 的方案拿到（浏览器登录抓 cookie / 后端 CLI 登录）。
- [ ] 控制台：`GET /api/console/me` 能拿到 `capabilities` → 控制台侧可用；论坛侧再用 `GET /api/forum/state` 的 `viewer` 确认。
- [ ] 组织命令：`GET /api/me/orgs` 列出可用组织与角色（缓存 120 秒）；再用 `GET /api/admin/:org/overview` 确认角色与白名单（403 `org_not_whitelisted` = 组织不在部署白名单）。
- [ ] 写请求不带 `Origin`、不带 JSON 空 body（点赞/收藏等）。
- [ ] 429 要区分 `rate_limited` 与 `guest_replies_paused`；403 要区分 `missing_capability` 与 `invalid_origin`。
- [ ] 论坛写操作按 `changes` 增量合并，不要整份重拉；帖子正文单独用 `topics/:id/posts` 取。
