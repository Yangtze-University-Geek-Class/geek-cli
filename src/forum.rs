use anyhow::{Context, Result, bail};
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_BASE: &str = "https://yangtzeu.work";

pub struct ForumClient {
    http: Client,
    token: String,
    base: String,
}

impl ForumClient {
    pub fn new(token: String) -> Result<Self> {
        let http = Client::builder()
            .user_agent("geek-cli/forum")
            .build()
            .context("build http client")?;
        let base = std::env::var("GEEK_FORUM_BASE").unwrap_or_else(|_| DEFAULT_BASE.to_string());
        Ok(Self { http, token, base })
    }

    fn req(&self, method: Method, path: &str) -> RequestBuilder {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", self.base, path)
        };
        self.http
            .request(method, url)
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let res = self.req(Method::GET, path).send().await.with_context(|| format!("GET {path}"))?;
        check(&res).await?;
        Ok(res.json().await?)
    }

    pub async fn post<B: Serialize, T: for<'de> Deserialize<'de>>(&self, path: &str, body: &B) -> Result<T> {
        let res = self.req(Method::POST, path).json(body).send().await.with_context(|| format!("POST {path}"))?;
        check(&res).await?;
        Ok(res.json().await?)
    }

    pub async fn patch<B: Serialize>(&self, path: &str, body: &B) -> Result<Value> {
        let res = self.req(Method::PATCH, path).json(body).send().await.with_context(|| format!("PATCH {path}"))?;
        check(&res).await?;
        Ok(res.json().await.unwrap_or(Value::Null))
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let res = self.req(Method::DELETE, path).send().await.with_context(|| format!("DELETE {path}"))?;
        check(&res).await?;
        Ok(())
    }
}

async fn check(res: &reqwest::Response) -> Result<()> {
    let status = res.status();
    if status.is_success() {
        return Ok(());
    }
    if status == StatusCode::UNAUTHORIZED {
        bail!("论坛拒绝 token (401)。CLI 用 GitHub PAT，需要先 `geek login`，并在论坛首次登录过一次确认 github_id 绑定");
    }
    if status == StatusCode::FORBIDDEN {
        bail!("权限不足 (403)。该操作仅论坛负责人 / 协管可用，或目标分类是 legacy 只读区");
    }
    if status == StatusCode::NOT_FOUND {
        bail!("找不到 (404)");
    }
    bail!("论坛 API 返回 {status}");
}
