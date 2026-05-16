# geek-cli

Yangtze University Geek Class 的统一 CLI，面向 AI agent 和命令行用户。

包装 GitHub 组织管理常用操作（成员 / 邀请 / 仓库 / 活动）成单二进制 `geek`，默认 JSON 输出方便 agent 解析。

线上后台：https://github.yangtzeu.work/

## 安装

### 一键脚本（推荐）

```bash
# macOS / Linux
curl -fsSL https://github.com/Yangtze-University-Geek-Class/geek-cli/releases/latest/download/install.sh | sh

# Windows PowerShell
irm https://github.com/Yangtze-University-Geek-Class/geek-cli/releases/latest/download/install.ps1 | iex
```

脚本自动检测平台（mac/linux/windows × x64/arm64，linux 自动判 glibc/musl），下载 binary 到 `~/.local/bin/geek`（Windows: `%USERPROFILE%\.local\bin\geek.exe`），校验 sha256。装完按提示把 `~/.local/bin` 加进 `$PATH`。

环境变量：`GEEK_VERSION=v0.1.2` 锁版本，`GEEK_INSTALL_DIR=/usr/local/bin` 改路径。

### npm（scoped 包，避免重名冲突）

> 注意：包名是 `@yangtzeu/geek-cli`，不是 `geek-cli`（npm 上同名是 6 年前一个 apollo 老包）。

```bash
npm i -g @yangtzeu/geek-cli
# 或：pnpm/yarn 同理
```

如果你公司 / 学校 npm 镜像走内网（如 verdaccio），请显式指定官方 registry：

```bash
npm i -g @yangtzeu/geek-cli --registry=https://registry.npmjs.org
```

### 从源码 / Release

```bash
cargo install --git https://github.com/Yangtze-University-Geek-Class/geek-cli
```

或下载对应平台 binary：https://github.com/Yangtze-University-Geek-Class/geek-cli/releases

## 上手

```bash
geek login                 # OAuth Device flow，浏览器输 code
geek whoami                # 验登录
geek org list              # 列出我的所有 org
geek repo list <org>       # 列仓库
```

第一次 `login` 后 token 存到 `~/.config/geek/token.json`（chmod 600），后续命令自动用。

## 命令一览

```
geek login                                          OAuth Device flow 登录
geek logout                                         清掉 token
geek whoami                                         看登录信息

geek org list                                       列出我的所有 org
geek org show <org>                                 org 详情
geek org members <org>                              member list 别名

geek member list <org>                              org 成员 + 角色
geek member show <org> <login>                      单个成员信息
geek member remove <org> <login>                    移除成员（admin 才行）
geek member set-role <org> <login> <admin|member>   改成员角色

geek invite list <org>                              pending 邀请
geek invite user <org> <login> [--role ...] [--team ...]    邀请 GitHub 用户
geek invite email <org> <email> [--role ...]        邀请邮箱
geek invite cancel <org> <id>                       取消 pending 邀请

geek repo list <org>                                org 全部仓库
geek repo show <org>/<repo>                         仓库详情
geek repo create <org> <name> [--visibility private|public] [--description ...]
                                                    [--no-init] [--gitignore Node] [--license mit]
geek repo delete <org>/<repo> --yes                 删仓库（不可逆，必须 --yes）
geek repo branches <org>/<repo>                     分支列表
geek repo tree <org>/<repo> [--path src] [--ref main]    目录列表
geek repo cat <org>/<repo> <path> [--ref main]      打印文件内容
geek repo commits <org>/<repo> [--ref main] [--limit 30]
geek repo issues <org>/<repo> [--state open|closed|all]
geek repo prs <org>/<repo> [--state open|closed|all]
geek repo collab <org>/<repo> <login> [--permission pull|triage|push|maintain|admin]
geek repo uncollab <org>/<repo> <login>

geek activity events <org> [--limit 30]             org 公共事件流

geek dashboard [<org>]                              浏览器打开 yzgc-admin
```

### 输出格式

所有命令支持 `--format`：

```bash
geek repo list <org>                  # default: 紧凑 JSON
geek repo list <org> --format pretty  # 缩进 JSON
geek repo list <org> --format table   # ASCII 表格（人友好）
```

agent 用 `json`（默认），人看用 `table`。

## 给 AI agent 使用

仓库自带 `SKILL.md`，符合 [runai](https://github.com/Crosery/runai) skill 规范。安装到 Claude Code / Codex / 其它支持 skill 的 CLI：

```bash
runai install Yangtze-University-Geek-Class/geek-cli
```

或者把 `SKILL.md` 拷贝到 `~/.runai/skills/geek-cli/SKILL.md`。

## 鉴权与权限

- 登录用 GitHub OAuth Device flow，scope: `read:user user:email admin:org read:org repo`
- 所有命令用你登录账号的 token 调 GitHub API，权限就是 GitHub 端你的角色
- 移除成员 / 删仓库 等只对 org admin 生效，member 调会得 403
- 紧急关闭：`geek logout` 删本机 token；同时去 `https://github.com/settings/connections/applications/178c6fc778ccc68e1d6a` revoke

## 环境变量

| 变量 | 作用 |
|---|---|
| `GEEK_TOKEN` | 跳过本地 token 文件，临时用指定 token（CI 友好） |

## License

MIT
