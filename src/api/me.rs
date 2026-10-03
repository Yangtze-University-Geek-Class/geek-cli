//! 当前会话可用的组织（手册 §2.4）。

use anyhow::Result;
use serde_json::Value;

use super::Client;

/// `GET /api/me/orgs`：可用组织与本人角色（服务端缓存 120 秒）。
pub async fn orgs(client: &Client) -> Result<Value> {
    client.get("/api/me/orgs").await
}
