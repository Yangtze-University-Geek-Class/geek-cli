//! 公共端点（手册 §3、§7）。

use anyhow::Result;
use serde_json::Value;

use super::Client;

/// `GET /healthz`：探活 / 连通性自检。
pub async fn healthz(client: &Client) -> Result<Value> {
    client.get("/healthz").await
}
