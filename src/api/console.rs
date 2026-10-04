//! 控制台端点（手册 §4）：全部需要会话；能力门由服务端判定（403 `missing_capability`）。

use anyhow::Result;
use serde_json::Value;

use super::{query, Client};

/// `GET /api/console/me`：身份、称号与能力（只要求登录）。
pub async fn me(client: &Client) -> Result<Value> {
    client.get("/api/console/me").await
}

/// `GET /api/console/summary`：概览统计（按本人能力返回子集）。
pub async fn summary(client: &Client) -> Result<Value> {
    client.get("/api/console/summary").await
}

/// `GET /api/console/catalogue`：配置类只读数据（称号 / 色标 / 能力 / 领域 / 权限包 / 图标 / 投递状态）。
pub async fn catalogue(client: &Client) -> Result<Value> {
    client.get("/api/console/catalogue").await
}

/// `GET /api/console/people`：成员全名单（含称号，按层级排序）。
pub async fn people(client: &Client) -> Result<Value> {
    client.get("/api/console/people").await
}

/// `GET /api/console/departments`：部门列表（含负责人、人数、权限包）。
pub async fn departments(client: &Client) -> Result<Value> {
    client.get("/api/console/departments").await
}

/// `GET /api/console/assignments?department_id=&role=`：指派列表。
pub async fn assignments(
    client: &Client,
    department_id: Option<&str>,
    role: Option<&str>,
) -> Result<Value> {
    let mut pairs: Vec<(&str, String)> = Vec::new();
    if let Some(v) = department_id {
        pairs.push(("department_id", v.to_string()));
    }
    if let Some(v) = role {
        pairs.push(("role", v.to_string()));
    }
    client
        .get(&format!("/api/console/assignments{}", query(&pairs)))
        .await
}

/// `GET /api/console/applications?status=&q=&limit=&offset=`：投递列表。
pub async fn applications(
    client: &Client,
    status: Option<&str>,
    q: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value> {
    let mut pairs: Vec<(&str, String)> = Vec::new();
    if let Some(v) = status {
        pairs.push(("status", v.to_string()));
    }
    if let Some(v) = q {
        pairs.push(("q", v.to_string()));
    }
    if let Some(v) = limit {
        pairs.push(("limit", v.to_string()));
    }
    if let Some(v) = offset {
        pairs.push(("offset", v.to_string()));
    }
    client
        .get(&format!("/api/console/applications{}", query(&pairs)))
        .await
}

/// `GET /api/console/applications/:application_id`：投递详情（含审核历史）。
pub async fn application(client: &Client, application_id: &str) -> Result<Value> {
    client
        .get(&format!(
            "/api/console/applications/{}",
            urlencoding::encode(application_id)
        ))
        .await
}

/// `GET /api/console/feedback?status=&limit=`：意见箱列表。
pub async fn feedback(client: &Client, status: Option<&str>, limit: Option<u32>) -> Result<Value> {
    let mut pairs: Vec<(&str, String)> = Vec::new();
    if let Some(v) = status {
        pairs.push(("status", v.to_string()));
    }
    if let Some(v) = limit {
        pairs.push(("limit", v.to_string()));
    }
    client
        .get(&format!("/api/console/feedback{}", query(&pairs)))
        .await
}

/// `GET /api/console/audit?limit=&offset=&action=`：审计日志（脱敏）。
pub async fn audit(
    client: &Client,
    limit: Option<u32>,
    offset: Option<u32>,
    action: Option<&str>,
) -> Result<Value> {
    let mut pairs: Vec<(&str, String)> = Vec::new();
    if let Some(v) = limit {
        pairs.push(("limit", v.to_string()));
    }
    if let Some(v) = offset {
        pairs.push(("offset", v.to_string()));
    }
    if let Some(v) = action {
        pairs.push(("action", v.to_string()));
    }
    client
        .get(&format!("/api/console/audit{}", query(&pairs)))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_application_id_segment() {
        // 仅验证路径拼装不 panic 且对非法字符做段编码（网络行为由 tests/api_client.rs 覆盖）。
        let path = format!("/api/console/applications/{}", urlencoding::encode("a/b"));
        assert_eq!(path, "/api/console/applications/a%2Fb");
    }
}
