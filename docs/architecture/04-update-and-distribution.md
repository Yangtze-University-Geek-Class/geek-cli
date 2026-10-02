# 04 · 版本自检、自动更新与分发

> 状态：`accepted` · 更新：2026-10-02 · 适用范围：CLI 分发与升级全链路。
> 事实来源：`.github/workflows/release.yml`、`install.sh` / `install.ps1`、`npm/**`（现状）+ 本设计；API 手册未提供发布元数据接口。
> 约束与验证：自更新逻辑用本地假 Release 服务测试（校验成功/不匹配/中断/回滚/渠道识别）；发布检查单见 §7。

## 1. 分发矩阵与「谁可以自我替换」

| 渠道 | 二进制落点 | 自更新策略 |
|---|---|---|
| Release 二进制（`install.sh` / `install.ps1`） | `~/.local/bin/geek`（Win: `%USERPROFILE%\.local\bin\geek.exe`） | **允许** `geek self update` |
| npm `@yangtzeu/geek-cli` | `node_modules/**`（postinstall 下载） | **拒绝**替换（目录归 npm 管）→ 提示 `npm i -g @yangtzeu/geek-cli` |
| `cargo install --git` | `~/.cargo/bin` | **拒绝** → 提示 `cargo install --git … --force` |

- 识别：可执行文件路径启发式（含 `node_modules` / `.cargo/bin`）+ 编译期渠道标记；**无法确定时只提示不替换**。
- `--force` 可在人工确认后越过「拒绝」，但要在输出中说明后果（如被包管理器下次安装覆盖）。

## 2. 版本自检（默认行为）

- 数据源：`GET https://api.github.com/repos/Yangtze-University-Geek-Class/geek-cli/releases/latest`（stable 渠道；GitHub 的 `latest` 不含预发布）。
- 时机：命令启动时后台发起，主命令结束后最多等 1 秒取结果；**绝不阻塞主命令、绝不影响退出码**。
- 缓存：`~/.config/geek/state/update.json`（`checked_at`、`latest_version`）；TTL 24 小时，读缓存不发请求。
- 提示：仅当 stderr 是 TTY、且未 `--quiet`/`--no-update-check` 时，输出一行 `提示：vX.Y.Z 可用（当前 vA.B.C）→ geek self update`；JSON 数据流不受影响。
- 关闭条件：`--no-update-check`、`GEEK_NO_UPDATE_CHECK=1`、配置 `update.check=false`、`CI` 为真、非 TTY。
- 失败静默（网络/限流/无 Release）；只有 `geek self update --check` 才显式报错。

## 3. 显式升级：`geek self update`

```text
geek self update [--check] [--version vX.Y.Z] [--force]
```

1. 解析目标：默认 latest；`--version` 指定 tag；`--check` 只汇报（含当前/最新/渠道/安装方式）。
2. 渠道识别（§1）：非 self-managed 且无 `--force` → 退出码 4，打印对应包管理器命令。
3. 目标平台：编译期 `TARGET`（构建注入）；资产名 `geek-<target>[.exe]`（与 release.yml 现状一致）。
4. 下载 `geek-<target>` 与同名 `.sha256`（GET 可重试 ≤3 次，指数退避）；**sha256 必验、缺失即失败**（fail closed）。
5. 备份：当前 exe 复制为 `geek.old`（同目录）。
6. 原子替换：同目录临时文件 → Unix `rename` 覆盖；Windows 运行中的 exe 不可覆盖，采用 self-replace 类方案（重命名旧文件后就位新文件）。
7. 自检：运行 `geek --version` 确认新二进制可执行；失败 → 自动回滚到 `geek.old` 并报错退出码 1。
8. 输出 `vA → vB`；保留 `geek.old`，供 `geek self rollback`（成功回滚后删除备份）。
9. 全程**不执行任何远端脚本**；只下载本仓库 Release 的二进制资产。

## 4. 自动更新（opt-in，默认关闭）

- `update.mode = check（默认）| off | auto`。
- `auto` 执行条件（全部满足）：交互式 TTY、非 CI、非 `--quiet`、渠道为 self-managed、距上次升级 ≥24h。
- 执行位置：主命令**结束之后**、退出之前；升级结果只影响下一次启动；任何失败自动回滚并不打扰当前命令（stderr 一行说明）。
- `auto` 永不用于 rc 渠道；永不静默跳过 sha256 校验。

## 5. 配置与优先级

| 开关 | 默认 | 配置文件（`config.json`） | 环境变量 | 命令行 |
|---|---|---|---|---|
| 更新检查 | 开 | `update.check` | `GEEK_NO_UPDATE_CHECK=1` | `--no-update-check` |
| 更新模式 | `check` | `update.mode` | `GEEK_UPDATE_MODE` | — |
| 更新渠道 | `stable` | `update.channel` | `GEEK_CHANNEL` | — |

优先级：命令行 > 环境变量 > 配置文件 > 默认。配置路径 `~/.config/geek/config.json`（与 `session.json` 同目录、0600、原子写）。

## 6. 预发布渠道（`proposed`，P3+）

- `update.channel=rc`：走 `GET /repos/…/releases`（列表）过滤 `-rc.` 取最新；不参与 `auto`；`--check` 输出渠道名。
- 后端若提供发布元数据接口可替代 GitHub API（可选，不是前置）。

## 7. 与发版流程（`docs/07`、`docs/10` §5）的衔接

- **tag 即发布**：`vX.Y.Z-rc.N` → 人工验收 → 同提交 `vX.Y.Z`；自更新只认正式 tag（stable）。
- 发布检查单新增：Release 资产 = 7 平台二进制 + `.sha256` + `install.sh`/`install.ps1`；资产名与 `self update` 期望一致；`Cargo.toml` 版本 = tag。
- npm 渠道版本由 release workflow 从 tag 覆写（现状）；发布后核对 `npm view`。
- 用户侧回滚旧路径仍可用：`GEEK_VERSION=vX.Y.Z-1` 重装（install.sh）。

## 8. 验证方法

- 单元/集成：假 Release 服务（本项目资产形状）覆盖：正常升级、sha256 不匹配（必须失败且不替换）、下载中断、替换失败回滚、`--check`、缓存 TTL、非 TTY 抑制、CI 抑制、npm/cargo 渠道拒绝。
- 真机冒烟：从 `v0.2.0` 升级到测试 tag，再 `self rollback` 回退；npm 安装环境下确认提示正确。
- 发布前人工验收步骤中增加一条：`geek self update --check` 输出与最新 tag 一致。
