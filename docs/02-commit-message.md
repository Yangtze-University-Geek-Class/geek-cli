# 提交信息规范

> Conventional Commits 结构 + 中文简述 + 一次提交一个可独立回滚的目的。
> （依据 Conventional Commits 1.0.0 的消息结构；中文简述和 scope 词表是项目约定，不是该标准的语言要求。）

## 格式

```text
<type>(<scope>): <中文简述>

为什么改；改变了什么；用什么命令验证。
```

- 首行 ≤ 72 字符，半角冒号，不加句末句号或 emoji。
- 代码标识符、路径保持英文。
- 正文默认写三段信息：**为什么改** / **改变了什么** / **用什么命令验证**（可合并成两三行，但不能省验证方式）。

**type 词表**：`feat`、`fix`、`refactor`、`perf`、`docs`、`test`、`build`、`ci`、`chore`、`style`。
`style` 只表示格式，不表示界面功能改动。

**scope 词表**（按职责中心选择，不罗列全部文件）：

- 服务/端：`portal`、`forum`、`admin`、`console`、`shared`、`server`、`auth`、`db`
- 工程：`docs`、`notes`、`deploy`、`tooling`、`deps`、`release`

只补执行记录的提交用 `docs(notes): <一句话>`。

示例：

```text
fix(forum): 防止重复删除回帖破坏统计计数
test(auth): 验证密码变更撤销旧会话
docs(tooling): 统一根目录验收入口
```

## 原子性与兼容性

- 一个提交 = 一个可独立理解/回滚的逻辑目的。
- 代码与其契约、测试、文档属于同一改动（不拆成「先代码后补文档」）。
- 机械搬迁与行为改变尽量分开，但**不能故意制造不可构建的中间状态**。
- 兼容性变更用 `!` 或 `BREAKING CHANGE:` 正文说明影响和迁移路径。
- 结尾可写 `Refs #<issue>` 建立 提交 → issue 的双向链接。
- 禁止：`update`、`fix bug`、`WIP` 等无信息量消息；批量混入无关改动；回写/改写他人历史；无明确授权时创建或推送提交（Agent 尤其注意）。

## 提交 ≠ 发版

- Conventional Commits 只规定消息结构，**不自动计算或推进发布版本**。
- 禁止 semantic-release、版本机器人或「按 commit type 推算版本」的脚本；`feat`/`fix` 不是自动升号或自动部署的依据。
- 版本号何时改、改成多少由所有者决定，走普通 PR；发版与人工验收见 07。
