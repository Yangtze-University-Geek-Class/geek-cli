# 05 · 链路对接协议与安全治理

> 状态：`accepted` · 更新：2026-10-02 · 适用范围：把 geek-cli 接入 CI、脚本、agent 编排，以及凭据与操作安全。
> 事实来源：API 手册 §1.4/§1.5（错误与来源校验）、§2（会话）、§4–§6（能力/角色/限流）、§6.2（`changes`）；本设计 02–04。
> 约束与验证：脱敏与请求头有测试断言；链路示例必须可复制运行（见 §A.8）；评审清单见 §B.8。

## A. 链路对接协议

### A.1 进程契约（所有链路的地基）

| 通道 | 契约 |
|---|---|
| stdout | **只有数据**：默认单行 JSON，与 API 手册同形状（可 jq / serde 直读） |
| stderr | 提示、进度、更新通知、错误（结构化错误在 json 模式也是单行 JSON） |
| 退出码 | 稳定契约 0/1/2/3/4/5/6/7/8/9（[02](02-session-and-transport.md) §6）——链路用退出码分支，不解析文本 |
| 交互 | 非 TTY 一律不弹确认，直接以退出码 2 失败；破坏性操作显式 `--yes` |
| 幂等 | 写操作默认不重试（真 toggle 永不重试；`pin`/`close`/`view` 幂等）；链路重试责任见 §A.4 |

### A.2 环境变量契约

| 变量 | 用途 | 说明 |
|---|---|---|
| `GEEK_BASE` | 目标 base | 与 `--env/--base` 同义；CI 指向预发布做演练 |
| `GEEK_SID` | 会话注入 | **不落盘**；CI secret 注入；优先级最高 |
| `GEEK_FORMAT` | 默认输出格式 | 等价 `--format` |
| `GEEK_TIMEOUT` | HTTP 超时（秒） | |
| `GEEK_NO_UPDATE_CHECK` | 关版本检查 | CI 建议置 1（TTY 检测之外的双保险） |
| `GEEK_DEBUG` | HTTP 调试日志到 stderr | 已脱敏，不含 `sid` |
| `NO_COLOR` / `CI` | 颜色 / CI 检测 | 标准语义 |

### A.3 数据通道（JSON 透传）

- 字段名、嵌套、编号（`t73`/`p10001`/`n9`）与手册一致；CLI 不重命名、不重排、不拆包 → **链路文档可以直接引用 API 手册的 jq 路径**。
- 例外：聚合命令（`whoami`/`status`）用 CLI 定义的对象，附 `sources` 说明来源；命令帮助里列明。
- 时间戳一律毫秒整数（手册 §8.8）；需要人类格式用 `--format table`，链路不应依赖 table。

### A.4 重试与「结果未知」协议

| 场景 | 链路允许的动作 |
|---|---|
| 读请求失败（网络/5xx） | 可重试；CLI 自身已重试 2 次 |
| 提交类写请求失败 | 不自动重试；链路应报告人工/上游处理，或先用读接口核对状态再决定 |
| 真 toggle（`like` / `bookmark` / `follow`） | **禁止重试**（重试=翻转两次） |
| `pin` / `close` / `view`（幂等） | 可安全重试（`pin`/`close` 为显式目标状态；`view` 服务端去重），默认仍不自动重试 |
| 503「邀请结果待核对」等结果未知 | 停手，人工核对（手册 §3.3 明确要求） |
| `pow_invalid` | 可安全重算重发一次（本地计算无副作用） |

### A.5 限流与批量

- 各接口限流以手册为准（§4.2 控制台、§6.3 论坛、§8.7 常量）；CLI **默认串行、不并发扇出**。
- 常用额度提醒：`application export` 5 次/分钟；成员发帖 10/分钟、回复 30/分钟；游客回复 5/分钟+30/天/IP；`state` 120/分钟/IP。
- 429 分类处理：`rate_limited`（#191/#197 起统一形状）、`guest_replies_paused`（全站上限，改用登录或稍后）、`apply_limited`（官网投递，未收录）——都按限流处理；旧 `request_error` 形态已随修复移除，遗留部署若仍返回按限流兜底。
- 批量场景建议：链路按 `--limit`/分页串行拉取，两次请求间隔 ≥ 手册额度倒数；不要循环 `repo show`（服务端有 60s 缓存，重复读没有收益）。
- 错误体 `request_id` 可能缺失（仅部分路径带，见 02 §6）——链路不得假设其存在。

### A.6 匿名写与 PoW

- `join redeem` 与游客回复由 CLI **本地计算 PoW**（`sha256(timestamp:body:nonce)`，难度取 `/api/public/config` 或 `guestPolicy.powDifficulty`）。
- Turnstile 启用（`turnstile_site_key` 非空）时 CLI 无法求解：明确报错并提示改走浏览器，不静默失败。
- 蜜罐字段（`website` 等）不由 CLI 暴露，恒空。

### A.7 服务端推送与轮询

- 目前**没有**服务端→客户端的推送通道（无 SSE/Webhook/长轮询接口）；通知/状态变化靠主动轮询（如 `forum notifications`）。
- CLI 不做常驻进程、不注册定时任务（除更新检查的轻量缓存）；若未来后端提供推送协议，另立 `proposed` 设计。

### A.8 典型链路示例

```yaml
# GitHub Actions：只读巡检（预发布演练）
- run: curl -fsSL https://github.com/Yangtze-University-Geek-Class/geek-cli/releases/latest/download/install.sh | sh
- run: geek status --format json | jq -e '.health.ok and .auth.signed_in'
  env: { GEEK_SID: "${{ secrets.GEEK_SID }}", GEEK_BASE: "https://prev.yangtzeu.work" }
```

```bash
# agent：先自检能力，再决定调用哪些命令；写操作先 --dry-run 给人确认
geek console me | jq -r '.capabilities[]' | grep -qx applications.review || exit 4
geek console application review "$ID" --status interview --time … --place … --dry-run
```

## B. 安全治理

### B.1 凭据（`sid` = 密码级）

- 存储：`session.json` 0600、原子写；不进日志/报错/`--dry-run`/`--verbose` 输出。
- 传递：优先 `GEEK_SID`（CI secret）或 stdin；`--sid` 仅容错（进程列表可见，发出警告）；**禁止**写进脚本、README、issue、PR、执行记录（`docs/04/05` 已禁止）。
- 撤销：`geek logout` 或浏览器 `POST /auth/signout`；怀疑泄露 → 立即 signout 并重新登录（会话 7 天，无 refresh）。
- 共享机器：不保存会话（用 `GEEK_SID` 一次性注入）并避免 `--sid`。

### B.2 最小权限

- 使用登录者本人的权限与能力；CLI 不做本地放行判断，服务端的 `missing_capability`/`requires_org_admin`/`org_not_whitelisted` 原样上报（[02](02-session-and-transport.md) §6）。
- CI/agent 场景建议使用**专门的低权限账号**（普通成员）而不是管理员 `sid`；管理类命令的自动化要有明确授权边界。

### B.3 危险操作

- 清单与 `--yes` 要求见 [03](03-command-surface.md) §9；所有写支持 `--dry-run`；非 TTY 不弹交互。
- 「先读后写」防护：`application review` 默认自动填 `expected_status`/`expected_review_id`（防陈旧覆盖），显式传参时以用户为准。

### B.4 敏感数据展示

- 邀请链接 token（持有即可发出邀请，手册 §5.3）：**JSON 输出按透传契约原样包含**（与手册同形，便于 `jq` 取用；该输出即凭据，勿转发、勿截图）；`--format table` 只显示前缀（如 `ab12cd…`）；任何日志 / `notes/` / `--dry-run` 输出不得含完整 token。
- 关注关系（`state.follows`）为**全量公开**数据（评审确认，手册待修）：CLI 仅透传，不额外聚合或用于非公开场景。
- 投递/意见数据：JSON 全量（操作需要）；`table` 只投影操作必需列；截图与分享时按 `docs/04` 的「不贴真实数据」纪律执行。

### B.5 网络与传输

- TLS（rustls），不提供关闭证书校验的开关；不跟随重定向（防 cookie 跨域外泄，[02](02-session-and-transport.md) §5）；代理尊重系统设置。
- 不发 `Origin`（来源校验通过），但也不伪造 `Sec-Fetch-Site`；CLI 的合法身份就是「无来源头的 bearer/cookie 客户端」（手册 §1.5）。

### B.6 供应链

- 自更新只认本仓库 Release + 强制 sha256（[04](04-update-and-distribution.md) §3）；release 资产由 CI 从 tag 构建。
- 新增依赖走 `docs/10` §2 的说明（目的/许可/体积）；发布产物 `strip + lto` 保持现状。

### B.7 威胁模型（简表）

| 威胁 | 对策 |
|---|---|
| `sid` 经 argv/shell history 泄露 | 首选 stdin/env；`--sid` 警告；文档不示范 |
| 日志/报错泄露凭据 | 统一脱敏层 + 测试断言（02 §8） |
| 恶意更新源/中间人 | 强制 sha256、渠道识别、拒绝包管理器目录内替换 |
| 误操作正式环境 | 每次命令显式解析 base；`status` 展示环境；预发布演练 |
| 误删/误改 | `--yes` + 服务端审计（`/audit`、`/logs`）+ `--dry-run` |
| 用错身份（个人 vs 机器人） | 会话与身份在 `whoami`/`status` 明示；CI 用专用低权限账号 |

### B.8 验证与评审

- 测试：脱敏断言、请求头断言（无 `Origin`/空 `Content-Type`）、退出码映射、确认门（非 TTY + 缺 `--yes` → 2）。
- 评审清单（并入 `docs/06`）：凭据处理、越权可能、危险操作确认、限流与重试语义、输出是否透传一致。
