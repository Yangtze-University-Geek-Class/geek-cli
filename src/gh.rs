use anyhow::{Context, Result, bail};
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct GhClient {
    http: Client,
    token: String,
}

impl GhClient {
    pub fn new(token: String) -> Result<Self> {
        let http = Client::builder()
            .user_agent("geek-cli")
            .build()
            .context("build http client")?;
        Ok(Self { http, token })
    }

    fn req(&self, method: Method, path: &str) -> RequestBuilder {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("https://api.github.com{path}")
        };
        self.http
            .request(method, url)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let res = self
            .req(Method::GET, path)
            .send()
            .await
            .with_context(|| format!("GET {path}"))?;
        check(&res)?;
        let status = res.status();
        let text = res.text().await?;
        serde_json::from_str(&text)
            .with_context(|| format!("parse GET {path} (status {status}): {}", snip(&text)))
    }

    pub async fn paginate<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<Vec<T>> {
        let mut out = Vec::new();
        let join = if path.contains('?') { '&' } else { '?' };
        let mut next = Some(format!("{path}{join}per_page=100"));
        while let Some(url) = next.take() {
            let res = self
                .req(Method::GET, &url)
                .send()
                .await
                .with_context(|| format!("GET {url}"))?;
            check(&res)?;
            next = parse_next_link(res.headers().get("link"));
            let mut batch: Vec<T> = res.json().await?;
            out.append(&mut batch);
        }
        Ok(out)
    }

    pub async fn post<B: Serialize, T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let res = self
            .req(Method::POST, path)
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {path}"))?;
        check(&res)?;
        Ok(res.json().await?)
    }

    pub async fn patch<B: Serialize, T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let res = self
            .req(Method::PATCH, path)
            .json(body)
            .send()
            .await
            .with_context(|| format!("PATCH {path}"))?;
        check(&res)?;
        Ok(res.json().await?)
    }

    pub async fn put<B: Serialize>(&self, path: &str, body: &B) -> Result<Value> {
        let res = self
            .req(Method::PUT, path)
            .json(body)
            .send()
            .await
            .with_context(|| format!("PUT {path}"))?;
        check(&res)?;
        Ok(res.json().await.unwrap_or(Value::Null))
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let res = self
            .req(Method::DELETE, path)
            .send()
            .await
            .with_context(|| format!("DELETE {path}"))?;
        check(&res)?;
        Ok(())
    }
}

fn check(res: &reqwest::Response) -> Result<()> {
    let status = res.status();
    if status.is_success() {
        return Ok(());
    }
    if status == StatusCode::UNAUTHORIZED {
        bail!("GitHub 拒绝 token (401)。重新运行 `geek login` 拿新 token");
    }
    if status == StatusCode::FORBIDDEN {
        bail!("权限不足 (403)。你不是该资源的 admin / 没相关 scope");
    }
    if status == StatusCode::NOT_FOUND {
        bail!("找不到 (404)。资源不存在或你没访问权");
    }
    bail!("GitHub API 返回 {status}");
}

fn parse_next_link(h: Option<&reqwest::header::HeaderValue>) -> Option<String> {
    let raw = h?.to_str().ok()?;
    for part in raw.split(',') {
        let part = part.trim();
        if part.ends_with("rel=\"next\"") {
            if let Some(start) = part.find('<') {
                if let Some(end) = part.find('>') {
                    return Some(part[start + 1..end].to_string());
                }
            }
        }
    }
    None
}

fn snip(s: &str) -> String {
    if s.len() > 200 {
        format!("{}…", &s[..200])
    } else {
        s.to_string()
    }
}
