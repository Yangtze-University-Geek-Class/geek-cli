# task/7/console_reads · LYsnowQ · 2026-10-04

负责人：LYsnowQ

## 13:47:08 +08:00 · 开工 · #7 · 建 issue #7，从 task/6 分支堆叠 task/7/console_reads

- 执行者：agent-omp
- 做了什么：建 issue #7（console 只读面）；基于 task/6/p0_read_commands 拉出 task/7（堆叠；#6 合并后 rebase 到 main）
- 结果：分支建立

## 13:47:08 +08:00 · 开发 · #7 · console 只读命令 + mock server 契约测试

- 执行者：agent-omp
- 做了什么：api/console.rs 增补 9 个只读端点（`query` 助手抽取到 `api::query`）；commands/console.rs 与 cli.rs 接线；tests/api_client.rs 进程内 mock server 三项契约测试
- 结果：cargo test --locked 16 单测 + 3 集成全过；console 冒烟：各命令未登录→exit 3、非法枚举→exit 2
