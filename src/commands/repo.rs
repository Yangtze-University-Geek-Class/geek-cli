//! 仓库读命令（P0）。

use anyhow::Result;

use crate::api::admin;
use crate::commands::session_client;
use crate::output::{emit, Format};

/// `geek repo list <org>`：组织仓库列表（GET /api/admin/:org/repos）。
pub async fn list(base: &str, org: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&admin::repos(&client, org).await?, fmt)
}

/// `geek repo show <org>/<repo>`：仓库详情（GET /api/admin/:org/repos/:repo）。
pub async fn show(base: &str, full: &str, fmt: Format) -> Result<()> {
    let (org, repo) = split_full(full)?;
    let client = session_client(base)?;
    emit(&admin::repo(&client, org, repo).await?, fmt)
}

/// `geek repo tree <org>/<repo> [--path] [--ref]`：目录树（GET …/tree）。
pub async fn tree(
    base: &str,
    full: &str,
    path: &str,
    git_ref: Option<&str>,
    fmt: Format,
) -> Result<()> {
    let (org, repo) = split_full(full)?;
    let client = session_client(base)?;
    emit(&admin::tree(&client, org, repo, path, git_ref).await?, fmt)
}

/// `geek repo file <org>/<repo> <path> [--ref]`：文件内容（GET …/file）。
pub async fn file(
    base: &str,
    full: &str,
    path: &str,
    git_ref: Option<&str>,
    fmt: Format,
) -> Result<()> {
    let (org, repo) = split_full(full)?;
    let client = session_client(base)?;
    emit(&admin::file(&client, org, repo, path, git_ref).await?, fmt)
}

/// `geek repo commits <org>/<repo> [--sha --path --per-page --page]`：提交列表。
#[allow(clippy::too_many_arguments)]
pub async fn commits(
    base: &str,
    full: &str,
    sha: Option<&str>,
    path: Option<&str>,
    per_page: Option<u32>,
    page: Option<u32>,
    fmt: Format,
) -> Result<()> {
    let (org, repo) = split_full(full)?;
    let client = session_client(base)?;
    emit(
        &admin::commits(&client, org, repo, sha, path, per_page, page).await?,
        fmt,
    )
}

/// `geek repo issues <org>/<repo> [--state]`：Issue 列表（不含 PR）。
pub async fn issues(base: &str, full: &str, state: &str, fmt: Format) -> Result<()> {
    let (org, repo) = split_full(full)?;
    let client = session_client(base)?;
    emit(&admin::issues(&client, org, repo, state).await?, fmt)
}

/// `geek repo prs <org>/<repo> [--state]`：PR 列表。
pub async fn pulls(base: &str, full: &str, state: &str, fmt: Format) -> Result<()> {
    let (org, repo) = split_full(full)?;
    let client = session_client(base)?;
    emit(&admin::pulls(&client, org, repo, state).await?, fmt)
}

fn split_full(full: &str) -> Result<(&str, &str)> {
    full.split_once('/')
        .ok_or_else(|| crate::api::UsageError("expected <org>/<repo> form".into()).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_full_name() {
        assert_eq!(split_full("org/repo").unwrap(), ("org", "repo"));
        assert!(split_full("nope").is_err());
    }
}
