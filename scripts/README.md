# scripts/

仓库工具脚本（Node ≥16，仅标准库，无需安装依赖）。

## `note.mjs` —— 执行记录工具（对应 `docs/05`、`docs/10` §7-5）

```bash
# 追加一条记录（自动定位 notes/<+08:00 日期>/<负责人>/<链路>.md）
node scripts/note.mjs add \
  --stage 提交 --issue 6 --title "P0 读命令实现" \
  --did "实现 org/repo 读命令并补测试" --result "cargo test 14/14" \
  --by "agent-omp（omp coding agent）" --user LYsnowQ

# 生成索引（生成物，勿手工维护）
node scripts/note.mjs index

# 本地门禁（docs/05 §6：目录/标题/字段/阶段/时间/首条与收尾/INDEX 最新；失败非零退出）
node scripts/note.mjs check

# 自测（临时目录；正确树通过、注入违规必须失败）
node scripts/note.mjs selftest
```

约定与行为：

- 固定 **+08:00** 时区；目录 `<日期>/<GitHub用户名>/<链路>.md`，链路名 = 分支名把 `/`、`-` 换成 `_`；
- **只追加、不改写**；跨零点自动落到新日期目录下的同名链路文件；
- 阶段必须在词表内（开工/方案/开发/提交/推送/PR/审查/返工/合并/发布/验收/阻塞/收尾）；「执行者 / 做了什么 / 结果」必填；
- 时间为保证不倒退会自动 +1 秒（会提示）；「收尾」之后拒绝再记录；
- 身份：`--user` / `GEEK_NOTES_USER`（缺省 `git config user.name`）；`--by` / `GEEK_NOTES_BY` 必填；
- 所有子命令支持 `--root <dir>`（默认 `./notes`）；
- 记录文件出现冲突标记（`<<<<<<<` 等）会被 `check` 判失败。

## 未做（见 #15）

- **CI 接线**：`ci.yml` 由 PR #9（`task/6`）首次引入；#9 合并后把 `node scripts/note.mjs check` 接进工作流（#15）；
- **全库 INDEX**：本次只含本链路的 `notes/INDEX.md`；`notes/` 全树随 #9 入库后重生成（#15）；
- task / worktree 脚本（`docs/10` §7-5 的另一半）按需引入。
