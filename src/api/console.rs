//! 控制台端点（手册 §4）：全部需要会话；能力门由服务端判定（403 `missing_capability`）。
//!
//! 读响应透传（`serde_json::Value`）；写请求用强类型模型（`skip_serializing_if` 保证不多发字段，
//! 规避 `additionalProperties: false` 的 400）。

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use super::{query, Client};

// ---------- 请求模型（写） ----------

/// 审核通知信内容（按状态选用；只发送实际给出的字段）。
#[derive(Debug, Serialize)]
pub struct ReviewLetter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ReviewLetter {
    pub fn is_empty(&self) -> bool {
        self.time.is_none()
            && self.place.is_none()
            && self.notes.is_none()
            && self.message.is_none()
    }
}

/// `PATCH /api/console/applications/:id` 请求体（改状态时 `expected_*` 必填，由命令层补齐）。
#[derive(Debug, Serialize)]
pub struct ApplicationReviewBody {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_review_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter: Option<ReviewLetter>,
}

/// `PATCH /api/console/feedback/:id` 请求体（至少一项，由命令层校验）。
#[derive(Debug, Serialize)]
pub struct FeedbackUpdateBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<String>,
}

// ---------- 读 ----------

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

// ---------- 写 ----------

/// `PATCH /api/console/applications/:application_id`：改状态 / 写备注 / 可选发信。
pub async fn application_review(
    client: &Client,
    application_id: &str,
    body: &ApplicationReviewBody,
) -> Result<Value> {
    client
        .patch_json(
            &format!(
                "/api/console/applications/{}",
                urlencoding::encode(application_id)
            ),
            body,
        )
        .await
}

/// `PATCH /api/console/feedback/:id`：改意见状态 / 回复。
pub async fn feedback_update(client: &Client, id: u64, body: &FeedbackUpdateBody) -> Result<Value> {
    client
        .patch_json(&format!("/api/console/feedback/{id}"), body)
        .await
}

/// `DELETE /api/console/feedback/:id`：删除意见（不可恢复）。
pub async fn feedback_delete(client: &Client, id: u64) -> Result<Value> {
    client
        .delete_json(&format!("/api/console/feedback/{id}"))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_body_skips_absent_fields() {
        let body = ApplicationReviewBody {
            status: "interview".into(),
            expected_status: Some("received".into()),
            expected_review_id: Some(0),
            note: None,
            notify: None,
            letter: None,
        };
        assert_eq!(
            serde_json::to_value(&body).unwrap(),
            serde_json::json!({"status": "interview", "expected_status": "received", "expected_review_id": 0})
        );

        let body = ApplicationReviewBody {
            status: "interview".into(),
            expected_status: None,
            expected_review_id: None,
            note: None,
            notify: Some(false),
            letter: Some(ReviewLetter {
                time: Some("周六".into()),
                place: None,
                notes: None,
                message: None,
            }),
        };
        assert_eq!(
            serde_json::to_value(&body).unwrap(),
            serde_json::json!({"status": "interview", "notify": false, "letter": {"time": "周六"}})
        );
    }

    #[test]
    fn feedback_body_skips_absent_fields() {
        let body = FeedbackUpdateBody {
            status: None,
            reply: Some("已处理".into()),
        };
        assert_eq!(
            serde_json::to_value(&body).unwrap(),
            serde_json::json!({"reply": "已处理"})
        );
    }

    #[test]
    fn letter_emptiness() {
        let empty = ReviewLetter {
            time: None,
            place: None,
            notes: None,
            message: None,
        };
        assert!(empty.is_empty());
        let full = ReviewLetter {
            time: Some("周六".into()),
            place: None,
            notes: None,
            message: None,
        };
        assert!(!full.is_empty());
    }
}
