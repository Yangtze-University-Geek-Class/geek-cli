use serde::Serialize;
use std::fmt;

/// 统一错误体（手册 §1.4）：`{ error, message, request_id }`。
/// `request_id` 可能缺失（直接 `send` 的错误没有）；`error` 不保证是机器码、`message` 不保证中文。
#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    #[serde(rename = "http_status")]
    pub status: Option<u16>,
    pub error: String,
    pub message: String,
    pub request_id: Option<String>,
}

impl ApiError {
    pub fn network(error: reqwest::Error) -> Self {
        Self {
            status: None,
            error: "network_error".into(),
            message: error.to_string(),
            request_id: None,
        }
    }

    pub fn from_response(status: reqwest::StatusCode, body: &[u8]) -> Self {
        let value: serde_json::Value =
            serde_json::from_slice(body).unwrap_or(serde_json::Value::Null);
        let error = value
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| status.canonical_reason().unwrap_or("http_error"))
            .to_string();
        let message = value
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let request_id = value
            .get("request_id")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        Self {
            status: Some(status.as_u16()),
            error,
            message,
            request_id,
        }
    }

    pub fn unauthenticated(message: impl Into<String>) -> Self {
        Self {
            status: Some(401),
            error: "not_signed_in".into(),
            message: message.into(),
            request_id: None,
        }
    }

    /// 退出码契约（docs/architecture/02 §6）。
    pub fn exit_code(&self) -> i32 {
        match self.status {
            Some(401) => 3,
            Some(403) => 4,
            Some(404) | Some(410) => 5,
            Some(405) | Some(409) => 6,
            Some(429) => 7,
            Some(400) | Some(413) | Some(415) | Some(422) => 8,
            Some(_) | None => 9,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            Some(status) => write!(f, "HTTP {status}（{}）：{}", self.error, self.message),
            None => write!(f, "网络错误（{}）：{}", self.error, self.message),
        }
    }
}

impl std::error::Error for ApiError {}

/// 用法错误（退出码 2）：非交互环境需要输入、参数缺失等。
#[derive(Debug)]
pub struct UsageError(pub String);

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for UsageError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(status: u16, body: &str) -> ApiError {
        ApiError::from_response(
            reqwest::StatusCode::from_u16(status).unwrap(),
            body.as_bytes(),
        )
    }

    #[test]
    fn parses_unified_body() {
        let err = response(
            403,
            r#"{"error":"missing_capability","message":"需要「查看投递」权限","request_id":"r1"}"#,
        );
        assert_eq!(err.error, "missing_capability");
        assert_eq!(err.message, "需要「查看投递」权限");
        assert_eq!(err.request_id.as_deref(), Some("r1"));
        assert_eq!(err.exit_code(), 4);
    }

    #[test]
    fn tolerates_non_json_body() {
        let err = response(401, "not json");
        assert_eq!(err.error, "Unauthorized");
        assert_eq!(err.request_id, None);
        assert_eq!(err.exit_code(), 3);
    }

    #[test]
    fn exit_code_contract() {
        assert_eq!(response(410, "{}").exit_code(), 5);
        assert_eq!(response(405, "{}").exit_code(), 6);
        assert_eq!(response(409, "{}").exit_code(), 6);
        assert_eq!(response(422, "{}").exit_code(), 8);
        assert_eq!(response(429, "{}").exit_code(), 7);
        assert_eq!(response(503, "{}").exit_code(), 9);
    }
}
