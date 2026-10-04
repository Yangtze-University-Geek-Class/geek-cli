//! 控制台端点（手册 §4）。

use anyhow::Result;
use serde_json::Value;

use super::Client;

/// `GET /api/console/me`：身份、称号与能力（只要求登录；403 表示能力门外的情况由调用方处理）。
pub async fn me(client: &Client) -> Result<Value> {
    client.get("/api/console/me").await
}
