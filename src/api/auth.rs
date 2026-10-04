//! 会话与登录端点（手册 §2）。

use anyhow::Result;
use serde_json::Value;

use super::Client;

/// `GET /auth/me`：会话状态（无会话也返回 200，见手册 §2.2）。
pub async fn me(client: &Client) -> Result<Value> {
    client.get("/auth/me").await
}

/// `POST /auth/signout`：清服务端会话（未登录调用也成功；不发 body / Content-Type）。
pub async fn signout(client: &Client) -> Result<Value> {
    client.post_empty("/auth/signout").await
}
