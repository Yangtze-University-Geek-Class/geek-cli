pub mod auth;
pub mod console;
pub mod error;
pub mod me;
pub mod public;

pub use error::{ApiError, UsageError};

use anyhow::{Context, Result};
use reqwest::{Client as Http, Method, RequestBuilder};
use serde::de::DeserializeOwned;
use std::time::Duration;

pub const PROD: &str = "https://yangtzeu.work";
pub const PREV: &str = "https://prev.yangtzeu.work";
pub const LOCAL: &str = "http://127.0.0.1:3000";

/// 解析顺序：`--base` > `GEEK_BASE` > `--env` 语法糖 > 默认正式（docs/architecture/02 §1/§5）。
pub fn resolve_base(base: Option<&str>, env: Option<&str>) -> String {
    resolve_base_with(base, env, std::env::var("GEEK_BASE").ok())
}

fn resolve_base_with(base: Option<&str>, env: Option<&str>, env_base: Option<String>) -> String {
    let from_flag = base
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let from_env = env_base
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    if let Some(value) = from_flag.or(from_env) {
        return value.trim_end_matches('/').to_string();
    }
    match env {
        Some("prev") => PREV.to_string(),
        Some("local") => LOCAL.to_string(),
        _ => PROD.to_string(),
    }
}

pub struct Client {
    base: String,
    sid: Option<String>,
    http: Http,
}

impl Client {
    pub fn new(base: &str, sid: Option<String>) -> Result<Self> {
        let timeout = std::env::var("GEEK_TIMEOUT")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(20);
        let http = Http::builder()
            .user_agent(concat!("geek-cli/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(timeout))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .context("build http client")?;
        Ok(Self {
            base: base.trim_end_matches('/').to_string(),
            sid,
            http,
        })
    }

    fn req(&self, method: Method, path: &str) -> RequestBuilder {
        let mut builder = self.http.request(method, format!("{}{}", self.base, path));
        if let Some(sid) = &self.sid {
            builder = builder.header("Cookie", format!("sid={sid}"));
        }
        builder.header("Accept", "application/json")
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self
            .req(Method::GET, path)
            .send()
            .await
            .map_err(ApiError::network)?;
        self.decode(response).await
    }

    /// 无请求体的 POST：不发 `Content-Type`（手册 §1.3 写策略）。
    pub async fn post_empty<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self
            .req(Method::POST, path)
            .send()
            .await
            .map_err(ApiError::network)?;
        self.decode(response).await
    }

    async fn decode<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();
        let body = response.bytes().await.map_err(ApiError::network)?;
        if !status.is_success() {
            return Err(ApiError::from_response(status, &body).into());
        }
        serde_json::from_slice(&body).with_context(|| format!("parse response from {}", self.base))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_resolution_order() {
        assert_eq!(
            resolve_base_with(Some("https://Example.COM/"), None, None),
            "https://Example.COM"
        );
        assert_eq!(
            resolve_base_with(None, None, Some(" https://env.example/ ".into())),
            "https://env.example"
        );
        assert_eq!(resolve_base_with(None, Some("prev"), None), PREV);
        assert_eq!(resolve_base_with(None, Some("local"), None), LOCAL);
        assert_eq!(resolve_base_with(None, None, None), PROD);
        assert_eq!(
            resolve_base_with(
                Some("https://flag.example"),
                None,
                Some("https://env.example".into())
            ),
            "https://flag.example"
        );
    }
}
