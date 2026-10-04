//! 组织读命令（P0）。

use anyhow::Result;

use crate::api::{admin, me as me_api};
use crate::commands::session_client;
use crate::output::{emit, Format};

/// `geek org list`：当前会话可用的组织与本人角色（GET /api/me/orgs）。
pub async fn list(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&me_api::orgs(&client).await?, fmt)
}

/// `geek org show <org>`：组织概况与统计（GET /api/admin/:org/overview）。
pub async fn show(base: &str, org: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&admin::overview(&client, org).await?, fmt)
}
