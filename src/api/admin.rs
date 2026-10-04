//! GitHub 组织网关端点（手册 §5）：全部经 `/api/admin/:org/*`，会话由 `sid` 提供。

use anyhow::Result;
use serde_json::Value;

use super::Client;

/// 生成查询串（键值都编码；空列表返回空串）。
fn query(pairs: &[(&str, String)]) -> String {
    if pairs.is_empty() {
        return String::new();
    }
    let encoded: Vec<String> = pairs
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                urlencoding::encode(key),
                urlencoding::encode(value)
            )
        })
        .collect();
    format!("?{}", encoded.join("&"))
}

fn org_path(org: &str, tail: &str) -> String {
    format!("/api/admin/{}/{tail}", urlencoding::encode(org))
}

fn repo_path(org: &str, repo: &str, tail: &str) -> String {
    let base = format!(
        "/api/admin/{}/repos/{}",
        urlencoding::encode(org),
        urlencoding::encode(repo)
    );
    if tail.is_empty() {
        base
    } else {
        format!("{base}/{tail}")
    }
}

/// `GET /overview`：组织概况 + 统计。
pub async fn overview(client: &Client, org: &str) -> Result<Value> {
    client.get(&org_path(org, "overview")).await
}

/// `GET /repos`：仓库列表。
pub async fn repos(client: &Client, org: &str) -> Result<Value> {
    client.get(&org_path(org, "repos")).await
}

/// `GET /repos/:repo`：仓库详情（分支 / 协作者 / 钩子）。
pub async fn repo(client: &Client, org: &str, repo: &str) -> Result<Value> {
    client.get(&repo_path(org, repo, "")).await
}

/// `GET /repos/:repo/tree?ref=&path=`：目录树。
pub async fn tree(
    client: &Client,
    org: &str,
    repo: &str,
    dir: &str,
    git_ref: Option<&str>,
) -> Result<Value> {
    let mut pairs: Vec<(&str, String)> = Vec::new();
    if let Some(r) = git_ref {
        pairs.push(("ref", r.to_string()));
    }
    if !dir.is_empty() {
        pairs.push(("path", dir.to_string()));
    }
    client
        .get(&repo_path(org, repo, &format!("tree{}", query(&pairs))))
        .await
}

/// `GET /repos/:repo/file?ref=&path=`：文件内容。
pub async fn file(
    client: &Client,
    org: &str,
    repo: &str,
    file_path: &str,
    git_ref: Option<&str>,
) -> Result<Value> {
    let mut pairs: Vec<(&str, String)> = vec![("path", file_path.to_string())];
    if let Some(r) = git_ref {
        pairs.push(("ref", r.to_string()));
    }
    client
        .get(&repo_path(org, repo, &format!("file{}", query(&pairs))))
        .await
}

/// `GET /repos/:repo/commits?sha=&path=&per_page=&page=`：提交列表（唯一吃分页参数的端点）。
#[allow(clippy::too_many_arguments)]
pub async fn commits(
    client: &Client,
    org: &str,
    repo: &str,
    sha: Option<&str>,
    path: Option<&str>,
    per_page: Option<u32>,
    page: Option<u32>,
) -> Result<Value> {
    let mut pairs: Vec<(&str, String)> = Vec::new();
    if let Some(v) = sha {
        pairs.push(("sha", v.to_string()));
    }
    if let Some(v) = path {
        pairs.push(("path", v.to_string()));
    }
    if let Some(v) = per_page {
        pairs.push(("per_page", v.to_string()));
    }
    if let Some(v) = page {
        pairs.push(("page", v.to_string()));
    }
    client
        .get(&repo_path(org, repo, &format!("commits{}", query(&pairs))))
        .await
}

/// `GET /repos/:repo/issues?state=`：Issue 列表（不含 PR；服务端每页 50 取完、不分页）。
pub async fn issues(client: &Client, org: &str, repo: &str, state: &str) -> Result<Value> {
    client
        .get(&repo_path(
            org,
            repo,
            &format!("issues{}", query(&[("state", state.to_string())])),
        ))
        .await
}

/// `GET /repos/:repo/pulls?state=`：PR 列表（服务端每页 50 取完、不分页）。
pub async fn pulls(client: &Client, org: &str, repo: &str, state: &str) -> Result<Value> {
    client
        .get(&repo_path(
            org,
            repo,
            &format!("pulls{}", query(&[("state", state.to_string())])),
        ))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_paths_and_encoded_queries() {
        assert_eq!(org_path("org", "overview"), "/api/admin/org/overview");
        assert_eq!(repo_path("org", "repo", ""), "/api/admin/org/repos/repo");
        assert_eq!(
            repo_path("a/b", "r r", "tree"),
            "/api/admin/a%2Fb/repos/r%20r/tree"
        );
        assert_eq!(query(&[]), "");
        assert_eq!(query(&[("state", "open".into())]), "?state=open");
        assert_eq!(query(&[("ref", "feature/x".into())]), "?ref=feature%2Fx");
    }
}
