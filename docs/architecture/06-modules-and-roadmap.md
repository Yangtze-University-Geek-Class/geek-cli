# 06 · 模块拆分、技术选型、测试策略与分期路线

> 状态：`accepted` · 更新：2026-10-02 · 适用范围：实现者（geek-cli v2 编码工作）。
> 事实来源：现行 `src/` 结构、[01](01-facts-and-migration.md)–[05](05-integration-and-security.md)、`docs/10` 工程规则。
> 约束与验证：模块依赖方向不允许反向；每期验收标准（§4）逐条可查；测试矩阵（§3）进 CI 后为 required check（`docs/10` §7-2）。

## 1. 目标模块结构

```text
src/
├── main.rs            # 装配：全局参数 → session/config → 分发（不含业务）
├── cli.rs             # clap 命令树声明（只声明，不写业务）
├── commands/
│   ├── auth.rs        # login / logout / whoami / status
│   ├── admin.rs       # org / member / team / invite / link / repo（大文件按子域再拆）
│   ├── console.rs     # console *
│   ├── forum.rs       # forum *
│   ├── public.rs      # join * / health
│   └── self_.rs       # self update / rollback / info
├── api/
│   ├── mod.rs         # Client：base、请求头策略、错误解析、重试矩阵、分页辅助
│   ├── error.rs       # ApiError ↔ 退出码（02 §6）
│   ├── auth.rs / admin.rs / console.rs / forum.rs / public.rs
│   └── models.rs      # 写请求的强类型模型（响应默认 serde_json::Value 透传）
├── session.rs         # session.json、GEEK_SID、多 base
├── config.rs          # config.json、env 与优先级
├── output.rs          # json/pretty/table、北京时间、颜色、脱敏输出
├── pow.rs             # PoW 计算（join / 游客回复共用）
└── update.rs          # 版本检查、自更新、回滚、渠道识别
```

- 依赖方向：`commands → api → (session | config | pow)`；`output` 被 commands 使用；`api` 不反向依赖 commands；**禁止跨域命令互调**（forum 命令不得直调 admin 内部函数；跨域装配放 `commands/mod.rs`）。
- 单文件建议 ≤400 行；超限按子域拆文件。
- 读响应透传用 `serde_json::Value`；**写请求体一律强类型**（`additionalProperties:false` 的 400 是编译期能挡的错误）。

## 2. 技术选型

| 项 | 选择 | 理由 |
|---|---|---|
| clap 4（derive） | 保留 | 全局参数 + 嵌套子命令成熟；help 即文档 |
| reqwest + rustls | 保留 | 免系统库、musl 静态分发（`docs/10` §2） |
| serde / serde_json | 保留 | 透传 + 强类型写体 |
| thiserror（新增） | `ApiError` 枚举 | 状态码 / 机器码 / 网络错误分类，退出码映射单点 |
| sha2（新增） | PoW + 自更新校验 | 手册 §3.2；04 §3 |
| self-replace 或等价（新增） | Windows 运行中 EXE 替换 | 04 §3-6 |
| chrono + chrono-tz | 保留（补 tz） | 毫秒时间戳 → 北京时间展示（手册 §8.8） |
| comfy-table | 保留 | table 模式 |
| edition / MSRV | edition 2024，`rust-version = "1.85"` | 本机 rustc 1.88 可用；MSRV 明示避免误用 |

## 3. 测试策略（无真实网络、无真实凭据）

| 层 | 方式 / 断言 |
|---|---|
| `api::Client` | mock server：路径、请求头（无 `Origin`、无空 `Content-Type`）、错误 → 退出码、重试次数、3xx 不跟随 |
| session/config | 临时 HOME：0600、原子写、多 base 隔离、`GEEK_SID` 优先级与不落盘 |
| PoW | 已知向量（难度 1–4）、时间偏差边界 |
| output | 快照：JSON 透传字段与手册/01 §8 实测一致；table 投影稳定；脱敏（`sid`、token 不出现） |
| commands | `assert_cmd` 跑二进制：退出码、`--dry-run`、非 TTY 下确认失败（码 2） |
| update | 本地假 Release 服务（04 §8）：成功/校验失败/中断/回滚/渠道识别 |
| 真机 | 预发布环境只读冒烟（`status`、`org list`、`repo list`）；写命令只在预发布演练 |

## 4. 分期路线与验收标准

| 阶段 | 交付 | 验收（完成判据） |
|---|---|---|
| **P0 底座** | `session/transport/output/error/pow` 骨架 + `login/logout/whoami/status` + `console me` + `org list/show` + `repo list/show/tree/file/commits/issues/prs` + `health` + **`.github/workflows/ci.yml`**（fmt/clippy/test/build，汇总为一个 required check） | ①`geek login` 导入 `sid` 后 `status` 全绿；②读命令输出与手册/服务端实测（01 §8）一致（人工核对一次正式环境）；③契约测试与退出码测试通过并在 `ci.yml` 上运行 |
| **P1 治理** | `console *`（me/summary/catalogue/people/audit/assignment/application/feedback 读 + review/回复/删除写）+ admin 写（成员/团队/仓库/PR 合并/评论/组织资料） | ①`missing_capability`/`requires_org_admin` 映射到码 4；②危险操作缺 `--yes` 在非 TTY 下码 2；③`application review` 防陈旧（`expected_*`）实现且有测试 |
| **P2 论坛与邀请** | `forum *` 全量 + `join info/redeem` + `link *` | ①游客 PoW 回复成功（预发布）；②`changes` 增量解析不重拉 state；③真 toggle（like/bookmark/follow）重试次数=0、`pin`/`close`/`view` 幂等重试安全的测试通过；④Turnstile 环境报错清晰 |
| **P3 自更新与打磨** | `self update/check/rollback` + 渠道识别 + `update.mode=auto` + README/SKILL 重写 + npm/install 对齐 | ①从 P0 版本升级到 P2 版本并可回滚；②npm/cargo 渠道拒绝并提示正确；③`SKILL.md` 与 `README.md` 同 PR 更新（`docs/10` §4） |

## 5. 风险与对策

| 风险 | 影响 | 对策 |
|---|---|---|
| 后端无 CLI 登录通道 | 登录靠手工粘 cookie，自动化体验差 | 引导文档 + 推动 [02](02-session-and-transport.md) §2.3 的 proposed 方案；`GEEK_SID` 兜住 CI |
| API 手册与后端实现漂移 | CLI 行为过时 | 手册只读 + 「发现不符停手报告」；fixture 更新必须附手册 diff 说明 |
| 服务端缓存（30–120s） | 写后读旧值，链路误判 | 文档明示（02 §5）；链路写后重读需间隔或按业务容忍 |
| 限流（export 5/min 等） | 批量任务失败 | 串行 + 退避；429 分类处理（05 §A.5） |
| Windows 自替换 | 升级失败/卡死 | 备份 + self-replace + 回滚，专项测试（04 §8） |
| 命令面 ~70 条 | 文档不同步 | 域拆分；P3 评估「命令表→README 生成」（提案，不在本期） |
| `sid` 7 天过期 | CI 中断 | 退出码 3 明确；CI secret 定期更新；监控首次失败 |

## 6. 与协作规则的衔接

- 落地流程照 `docs/01–09`：先 issue → `task/<issue>/<slug>` 分支 → PR 九段 → `notes/` 执行记录；agent 入口 `AGENTS.md`。
- 命令面/输出契约变更：同步 [03](03-command-surface.md) + `README.md` + `SKILL.md`（`docs/10` §4）。
- 架构决策变更：改本目录并标注原因；`accepted` → 改行为前先改文档。
- `docs/10-geek-cli.md` 已随本次评审修订（v2 口径）；`README.md` / `SKILL.md` 的 v2 重写随实现 PR（P3）。
