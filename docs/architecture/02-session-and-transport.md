# 02 · 会话与传输契约

> 状态：`accepted` · 更新：2026-10-02 · 适用范围：CLI 底座（会话、HTTP、错误、输出）。
> 事实来源：API 手册 §1（通用约定）、§2（登录与会话）、§2.3（CLI 拿 `sid` 的可行路径）。
> 约束与验证：传输层的每条行为（头、重试、退出码）都有 mock server 测试；脱敏有专门用例（06 §3）。

## 1. 会话模型

- 唯一凭据：`sid`（浏览器 OAuth 登录 `<base>` 后由服务端签发的 HttpOnly cookie；手册 §1.2）。CLI 侧按**凭据**保存，等同密码。
- 有效期 7 天（服务端为准）；host-only、按 base 隔离；正式 / 预发布 / 本机三套互不相通。
- **不存在** PAT、Device flow、论坛独立登录、刷新令牌；不做「自动续期」，过期即重新登录。
- 纪律：不写入日志 / 报错 / `--dry-run` 输出 / shell history；文件权限 0600；CI 用环境变量注入。

## 2. 获取：`geek login`

### 2.1 今日唯一可行路径（手册 §2.3 路线 1）

交互模式（TTY）：

```text
$ geek login
1) 已在浏览器打开 https://yangtzeu.work/console （未打开可自行访问）
2) 登录后：开发者工具 → Application → Cookies → 复制 sid 的值
3) 粘贴到下面（输入不回显）：
sid: ********
✓ 已登录 @crosery（提督，3 项能力）· 会话预计 7 天
```

- `--sid-stdin`：从标准输入读（脚本推荐）；`GEEK_SID` 环境变量优先级最高且**不落盘**（CI 推荐）。
- `--sid <值>` 允许但发出警告（进程列表/历史可见），文档不推荐。
- 校验：先 `GET /auth/me` 确认 `signed_in:true`；随后 `GET /api/me/orgs` 记录组织快照。失败按 `signin` 原因给出对应提示（见 §4）。
- `--env prev|local` / `--base <url>`：对对应环境登录（每个环境单独执行一次）。

### 2.2 非交互（CI / agent / 脚本）

- 优先级：`GEEK_SID`（不落盘）> `--sid-stdin` > `--sid` > 本地会话文件。
- `--base`/`GEEK_BASE` 决定目标环境；不同环境互不共享会话。
- 任何需要交互确认的操作在非 TTY 下**直接失败退出**（§6 退出码 2），不挂起等待。

### 2.3 未来路径（`proposed`，需 geek_main 后端支持）

| 方案 | 需要的后端改动 | CLI 预留 |
|---|---|---|
| A. 本地回环回调 | `return_to` 白名单允许 `http://127.0.0.1:<port>`（手册 §2.1 同源白名单当前会拒绝） | `AuthMethod::Loopback`：起本地监听 → 用户浏览器完成登录 → 捕获 Set-Cookie |
| B. CLI 一次性授权 | 新增 device flow / PKCE 交换端点，签发 `sid` 或 CLI 专用 token | `AuthMethod::CliToken` |

命令面不变，只替换 `login` 的获取策略；后端就绪前不得在文档/提示中宣称可用。 **评审定夺（issue #2）：本期不做**；后续若仍有需求，走 geek_main 的 issue 重新评估。

## 3. 存储：`session.json`

路径：`~/.config/geek/session.json`（Unix 0600；Windows 使用用户 ACL；写入用「临时文件 + 原子 rename」）。

```json
{
  "version": 1,
  "sessions": {
    "https://yangtzeu.work": {
      "sid": "…",
      "login": "crosery",
      "user_id": 123456,
      "avatar_url": "https://avatars.githubusercontent.com/u/…",
      "saved_at": "2026-10-02T18:00:00+08:00"
    }
  }
}
```

- key = 规范化 base（小写 scheme/host、去末尾 `/`）；`--env prod|prev|local` 映射到三个固定 base。
- `GEEK_SID` 命中时不读也不写文件。
- 另有 `config.json`（偏好：`update.mode`、默认 format、超时等，见 04 §5）与 `state/`（更新检查缓存），互不混存。

## 4. 生命周期与失效

| 事件 | 行为 |
|---|---|
| 无会话 | 公开域（`/healthz`、`/api/public/*`、`/api/join/*`、论坛只读）照常；其余命令退出码 3，提示 `geek login` |
| 401 `session_expired` | 删除该 base 的本地会话；提示重新登录；不做自动重试 |
| `signin=not_member` / `invite_pending` / `cancelled` / `failed` | 登录引导页回跳原因，逐条映射为人类可读提示（`not_member`：不是组织 active 成员） |
| `geek logout` | `POST /auth/signout`（尽力而为）+ 删除本地会话；未登录调用也成功 |
| base 切换 | 每次命令独立解析（`--base` > `GEEK_BASE` > config > prod），不隐式复用其他环境的会话 |

## 5. 传输层行为（`api::Client`）

- 请求头：`User-Agent: geek-cli/<版本> (<os>/<arch>)`；有会话时 `Cookie: sid=…`；`Accept: application/json`。
- **禁止**：发 `Origin`（手册 §1.5：非 GET 带非本环境 Origin → 403 `invalid_origin`）；对无 body 的 POST/PUT/DELETE 发 `Content-Type`（Fastify 会按空 JSON 解析并 400，手册 §1.3）。
- 重定向：API 请求一律 `redirect::Policy::none()`，3xx 视为错误（`/auth/github` 是浏览器流程，CLI 不跟随）。**唯一例外**：`geek self update` 的 Release 资产下载允许跟随 302（GitHub 资产跳 `objects.githubusercontent.com`），该请求不携带任何 Cookie 与凭据（见 04 §3）。
- 超时：连接 5s、总 20s；`GEEK_TIMEOUT` 覆盖；`forum state` 允许更长（10 万级数据）。
- 缓存提示：服务端对 `repos/repo`(60s)、`commits/issues/pulls`(30s)、`me/orgs`(120s) 有缓存（手册 §5.1/§2.4）→ 写后立刻读可能拿到旧值；CLI 如实呈现，不自行缓存、不绕过。其中 `me/orgs` 的 120s 窗口**在组织成员变更时也不会失效**（评审补充）——`whoami` / `org list` 短时可能是旧数据，输出附取数时间戳提示。

### 5.1 重试矩阵（硬规则）

| 请求类 | 网络错误/超时 | 5xx | 429 |
|---|---|---|---|
| `GET`/`HEAD`（读） | 重试 2 次（200ms→800ms） | 重试 2 次 | 不自动等待，报错并提示额度 |
| `POST`/`PUT`/`PATCH`/`DELETE`（提交类） | 不重试 | 不重试 | 不重试 |
| 真 toggle（`like`/`bookmark`/`follow`） | **永不重试**（重试=翻转两次） | 永不 | 永不 |
| `pin`/`close`/`view`（幂等） | 可安全重试（`pin`/`close` 是显式目标状态、绝对赋值；`view` 按 IP+话题 1h 去重），默认仍不自动重试 | 可 | 不自动等待 |

- 上游结果未知（手册 §3.3：`503 邀请结果待核对`）→ 停止并让人复核，禁止自动重试。
- 429 区分 `rate_limited` 与 `guest_replies_paused`（手册 §6.3-5），错误消息里说明「等多久/改用登录」。
- PoW 相关失败（`pow_invalid`）可安全重算一次（本地计算，无外部副作用）。

## 6. 错误模型与退出码

- 解析统一错误体 `{ "error", "message", "request_id" }`（手册 §1.4）；例外按 HTTP 状态归类（如 `merge_failed`）。
- 退出码是**稳定契约**（agent 依赖；只增不改语义）：

| 码 | 含义 | 触发 |
|---|---|---|
| 0 | 成功 | — |
| 1 | 未预期错误 | bug / 无法归类 |
| 2 | 用法错误 | 参数错误；非交互下需要确认而失败 |
| 3 | 未登录 / 会话失效 | 401 `not_signed_in`、`session_expired`、`signin_required` |
| 4 | 无权限 / 来源不符 | 403 `missing_capability`、`invalid_origin`、`forbidden`、`requires_org_admin`、`org_not_whitelisted` |
| 5 | 不存在 | 404；**410 `legacy_forum_retired`（旧论坛路径已退役）** |
| 6 | 冲突 | 409 `status_changed`、`post_deleted`、`assignment_exists`、`invite_pending_review`、`invite_unavailable`；**405（上游状态冲突，如 PR 不可合并）** |
| 7 | 限流 | 429 |
| 8 | 参数校验失败 | 400 / 413 / 415（含 `letter_required`、`empty_content`）；**422（上游参数/校验拒绝）** |
| 9 | 上游 / 服务不可用 | 5xx、`internal_error`、`upstream_rejected`、网络失败、超时 |

- 错误输出一律 stderr：`--format json` 时输出单行 `{"error","message","request_id","http_status"}`；`pretty/table` 时输出中文一行 + 可操作提示。`request_id` **为可选字段**（仅抛错路径与 `session_expired` 带；缺失时不显示，不伪造）。
- 429：路由级限流统一为 `{ error: "rate_limited", message: "操作太频繁，请稍后再试", request_id }`（admin#191/#197 已并入 stage 基线）；业务上限另用专属码（`guest_replies_paused`、官网投递的 `apply_limited`）。CLI 对任何 429 一律按限流处理（兼容尚未升级的部署）。
- `merge_failed`（PR 合并失败）**不套统一错误体**，按 GitHub 实际状态码归类（405/409→6，422→8，5xx→9）。
- 评审补充映射（显式）：**410 → 5、405 → 6、422 → 8**。
- 服务端承诺（评审 2026-10-02）：已发布接口的 `error` 机器码**新增可以、改名/删除须先公告**并附版本说明；CLI 退出码映射依赖此约定。
- 映射规则：先按 HTTP 状态定退出码，再保留机器码原文（不被抹平）；`upstream_rejected` 按其实际状态码归类。

## 7. 输出契约（人机双模）

- **stdout 只放数据，stderr 只放提示/进度/更新通知**；JSON 模式不混任何人类文案。
- 默认 `--format json`：单行 JSON，**与 API 手册同形状**——不重命名、不重组、不拆包（`.members`、`.repos`、`.state` 等保持原样；jq 路径 = 手册字段路径）。
- `--format pretty`：缩进 JSON；`--format table`：人类投影（每命令定义列；不得发明字段、不得省略关键 ID；时间转北京时间）。列表类在 table 模式可省略冗余字段，JSON 模式不省。
- 聚合命令例外：`whoami`、`status` 共用**同一形状**（写入 `--help` 与 03）：`{ "health"?, "auth", "console"?, "orgs"? }`——`console` 需控制台权限、`orgs` 需登录，缺省即省略；`orgs` 附 CLI 侧取数时间戳（对应服务端 120s 缓存窗口）。
- 颜色仅 TTY；`NO_COLOR` / `--no-color` 关闭。
- `--dry-run`：打印将发送的请求（方法、路径、body 摘要；`sid`、邀请 token 等敏感值脱敏），不实际发送、不产生副作用。
- `--quiet`：仅数据，关闭提示与更新通知；`--verbose`：HTTP 层调试（含状态码与耗时，**不含**凭据）。

## 8. 验证方法（测试点）

- mock server：断言请求头（无 `Origin`、无空 `Content-Type`）、重试矩阵的请求次数、3xx 不跟随、退出码映射。
- 会话：文件权限、原子写、多 base 隔离、`GEEK_SID` 优先且不落盘、`session_expired` 清理。
- 脱敏：日志/stderr/`--dry-run` 输出中不出现 `sid` 与完整邀请 token（扫描断言）。
- 真机冒烟：`geek status`（`/healthz` + 会话）在正式与预发布各跑一次。
