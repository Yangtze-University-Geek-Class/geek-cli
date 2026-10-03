//! geek-cli v2 框架（预览分支；功能按切片逐步加入）。
//!
//! 对应设计：`docs/architecture/06-modules-and-roadmap.md`。分层与扩展点：
//!
//! - [`api`]：端点层——传输、错误模型与退出码（0–9）、按域客户端（`auth` / `me` / `console` / `public`）。
//! - [`commands`]：命令层——按域组织（`auth` 已实现；`admin` / `console` / `forum` / `public` / `self` 随功能切片加入）。
//! - [`session`]：多环境会话存储；[`output`]：输出与错误呈现；[`pow`]：工作量证明；
//!   [`update`]：版本比较与检查缓存。
//!
//! 约定：
//! - 命令层不写死端点路径——路径集中在 `api/<域>.rs`；
//! - 读响应透传（`serde_json::Value`），写请求用强类型模型（随首个写命令切片加入）；
//! - 偏好配置 `config`（`config.json`）在旧 token 存储随 P0 重写删除后落位（当前被旧命令占名）。
//!
//! 运行入口在二进制 crate（`src/main.rs` + `src/cli.rs`）：`cli.rs` 负责 clap 命令树与接线。

pub mod api;
pub mod commands;
pub mod output;
pub mod pow;
pub mod session;
pub mod update;
