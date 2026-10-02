# geek-cli v2 架构（对接 yangtzeu.work 核心服务）

> 状态：`accepted`（**评审通过**：issue #2 · @Crosery · 2026-10-02；本目录已并入 5 条修订）· 更新：2026-10-02 · 适用范围：geek-cli 全部后续开发。
> 事实来源：[`docs/api-forum-console.md`](../api-forum-console.md)（**只读**，接口唯一真相）、geek-cli 现行源码、`docs/01–10` 协作规则。
> 约束与验证：任何实现 PR 必须能对回本目录的决策编号与命令表；验证方式=契约测试（mock server）+ 本机冒烟 + 发布前人工验收（见 06）。

## 0. 一句话

把 geek-cli 从「GitHub PAT 直连 `api.github.com` + 旧论坛接口（Bearer PAT）」重构为
「**`yangtzeu.work` 核心服务的同域会话型 CLI**」：唯一凭据是浏览器登录得到的 `sid`，命令按域映射到
`/api/admin/:org/*`（GitHub 网关）、`/api/console/*`（控制台）、`/api/forum/*`（论坛）、`/api/public/*` 与 `/api/join/*`（匿名）。

## 1. 为什么必须重构（旧实现过期判定，详见 01）

| 旧模块 | 旧做法 | 判定 |
|---|---|---|
| `src/auth.rs` | GitHub OAuth Device flow 换 PAT（借 gh CLI client id） | **过期**：新 API 没有 PAT / Device flow / 论坛独立登录，只有 `sid` cookie |
| `src/gh.rs` | 用 PAT 直连 `api.github.com` | **过期**：组织操作全部收归 `/api/admin/:org/*` 网关（白名单 + 角色 + 审计 + 缓存） |
| `src/forum.rs` | Bearer PAT + 旧 `/api/forum/{categories,threads,thread,…}` | **过期**：路径、编号（`t/p/m/g/u`）、响应形状（增量 `changes`）、写策略全变 |
| `src/config.rs` | `token.json` 单凭据 | **过期**：改多环境 `session.json` |
| `src/cli.rs` | 命令面 | **部分过期**：旧论坛管理命令（teacher/role/groups 等）在新 API 中不存在，直接删除 |

> 项目已半年未维护，凡与 API 手册不一致的旧行为一律视为**过期且不可用**，不做兼容层。

## 2. 设计目标（六条，冲突时按序）

1. **会话唯一**：所有需要身份的调用只认 `sid`；不引入第二套认证。
2. **API 即契约**：CLI 不重命名、不重组 API 数据；JSON 输出与手册形状逐字段一致（jq 路径=手册字段路径）。
3. **人机双模**：默认 JSON 给 agent，`--format table` 给人；stdout 只放数据，stderr 只放提示。
4. **安全默认**：凭据等同密码（0600 / 不回显 / 不进日志）；破坏性操作必须显式确认；自更新必须校验签名级完整性并可回滚。
5. **自动化友好**：稳定退出码、非交互不挂起、`GEEK_*` 环境契约、`--dry-run`、幂等/非幂等语义明确。
6. **可演进**：后端加接口时只加 `api/*` 客户端与命令，不改底座；登录方式可替换而不动命令面。

## 3. 关键决策清单（D1–D10）

| # | 决策 | 取舍与理由 |
|---|---|---|
| D1 | 登录 = 浏览器登录后导入 `sid`（粘贴 / stdin / `GEEK_SID`）；loopback、CLI token 列为 proposed，需后端支持 | 手册 §2.3 明确当前唯一可行路径；不伪造不存在的机制 |
| D2 | 单 `base` 同域；三环境（prod/prev/local）三套独立会话，按 base 键存储 | 手册 §1.1：三套数据独立；`return_to` 要求同源 |
| D3 | 传输层薄：读响应**透传**，写请求体**强类型**（serde，拒绝未知字段） | 读透传=文档即契约；写强类型=规避 `additionalProperties:false` 的 400 |
| D4 | 命令面按 API 域重组：保留 `org/member/invite/repo/activity/forum` 的旧肌忆，新增 `console/join/self`，删除新 API 不存在的命令 | 旧用户迁移成本低，新能力完整暴露 |
| D5 | 退出码契约 0/1/2/3–9（见 02 §6），错误体 `{error,message,request_id}` 结构化落到 stderr | agent 可判定失败类型，不再猜输出文本 |
| D6 | 自更新三级：默认仅检查提示；`geek self update` 显式升级；`update.mode=auto` 可选 | 「推送与控制的安全性」：默认不静默替换二进制 |
| D7 | 自更新 = 下载 Release 资产 + 强制 sha256 校验 + 原子替换 + 保留上一版可回滚；npm/cargo 渠道识别后拒绝自行替换 | 供应链安全；包管理器管理的文件不能被越过 |
| D8 | 破坏性/有外部副作用的写操作要求 `--yes`；所有写支持 `--dry-run`；非 TTY 不弹交互确认 | 人机复合使用：人类可读提示，agent 显式授权 |
| D9 | 论坛写操作按 `changes` 增量消费，不整份重拉；toogle 类（like/bookmark/follow/pin/close）**永不自动重试** | 手册 §6.2/#145；toggle 重试=翻转两次 |
| D10 | 代码按 `api/`（端点层）+ `commands/`（域命令）+ `session/output/pow/update/config` 拆分；edition 2024，MSRV 1.85 | 现有 `cli.rs` 822 行单文件不可持续；模块边界对应 API 域 |

## 4. 文档地图

| 文件 | 内容 |
|---|---|
| [01-facts-and-migration.md](01-facts-and-migration.md) | 事实基座、环境与 base、旧实现过期判定、旧→新命令映射 |
| [02-session-and-transport.md](02-session-and-transport.md) | `sid` 会话（获取/存储/失效/CI）、传输契约、错误模型与退出码、双模输出 |
| [03-command-surface.md](03-command-surface.md) | 全量命令表（auth / org+member+invite+repo+activity / console / forum / join / self） |
| [04-update-and-distribution.md](04-update-and-distribution.md) | 版本自检、`geek self update`、回滚、渠道识别、分发矩阵 |
| [05-integration-and-security.md](05-integration-and-security.md) | 链路对接协议（CI / agent / 脚本）、安全治理与威胁模型 |
| [06-modules-and-roadmap.md](06-modules-and-roadmap.md) | 代码模块图、技术选型、测试策略、P0–P3 分期与验收标准 |
| [07-open-questions.md](07-open-questions.md) | **待定 / 待确认 / 暂缓事项登记表**（含所有者待确认项；新不确定事项先记这里） |

## 5. 分期路线图（摘要，详见 06 §4）

| 阶段 | 内容 | 完成判据 |
|---|---|---|
| P0 底座 | `session/transport/output/error` + `login/logout/whoami/status` + `org list/show` + `repo list/show/tree/file/commits` + `healthz` | 会话导入可用；读命令对生产输出与手册一致；契约测试通过 |
| P1 治理 | `console *`（me/summary/catalogue/people/assignments/departments/applications/feedback/audit）+ admin 写操作（成员/团队/仓库/PR 合并/评论） | 能力门（403 `missing_capability`）映射正确；破坏性操作确认齐全 |
| P2 论坛与邀请 | `forum *` 全量 + `join info/redeem`（本地 PoW）+ 邀请链接管理 | 游客 PoW 回复可用；`changes` 增量解析；限流错误分类正确 |
| P3 自更新与打磨 | `self update/check/rollback`、渠道识别、`update.mode=auto`、SKILL.md 重写、npm/install 脚本对齐 | 从旧版升级到新版成功且可回滚；npm/cargo 渠道拒绝并提示 |

## 6. 与旧版对比（一页版）

| 维度 | 旧 | 新 |
|---|---|---|
| 认证 | GitHub Device flow → PAT | 浏览器 `sid`（7 天），多环境隔离 |
| 通路 | `api.github.com` 直连 + 旧论坛 API | 全部经 `yangtzeu.work` 网关（白名单/角色/审计/缓存） |
| 命令面 | 覆盖旧后台能力 | 覆盖新 API 全部六域，删除已不存在的概念（teacher/groups/角色旧模型） |
| 输出 | 部分重组过的数组 | API 原样透传 + `table` 人类投影 |
| 失败语义 | 退出码一律非零、文本提示 | 结构化错误 + 稳定退出码 + `request_id` |
| 更新 | 手动重装 | 版本自检 + 显式自更新（可回滚），auto 可选 |
| 安全 | token 单点 | `sid` 密码级处理、`--yes`/`--dry-run`、链路凭据规范 |

## 7. 明确不做 / 待后端支持

- **不做**：旧命令 shim、PAT/Device flow 兼容、未收录接口（官网投递、公开意见墙、dev 快照）、绕过服务端能力门。
- **待后端（proposed，评审定夺：本期不做）**：CLI 专用登录通道（loopback 白名单或 CLI token，02 §2.2；`GEEK_SID` / `--sid-stdin` 已覆盖 CI 与 agent）；发布元数据接口（不做，自更新继续走 GitHub Releases）。
- **只读**：`docs/api-forum-console.md` 不得修改；发现不符即停手报告。
- **登记表**：全部待定 / 待确认 / 暂缓事项（含上述 proposed 项的当前状态）汇总于 [07-open-questions.md](07-open-questions.md)；新增不确定事项先记那里。
