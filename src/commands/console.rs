//! 控制台只读命令（P1）。

use anyhow::Result;

use crate::api::console as console_api;
use crate::commands::session_client;
use crate::output::{emit, Format};

/// `geek console me`：控制台身份 / 称号 / 能力（GET /api/console/me）。
pub async fn me(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::me(&client).await?, fmt)
}

/// `geek console summary`：概览统计（按本人能力返回子集）。
pub async fn summary(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::summary(&client).await?, fmt)
}

/// `geek console catalogue`：配置类只读数据。
pub async fn catalogue(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::catalogue(&client).await?, fmt)
}

/// `geek console people`：成员全名单（含称号）。
pub async fn people(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::people(&client).await?, fmt)
}

/// `geek console department list`：部门列表。
pub async fn departments(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::departments(&client).await?, fmt)
}

/// `geek console assignment list [--department --role]`：指派列表。
pub async fn assignments(
    base: &str,
    department_id: Option<&str>,
    role: Option<&str>,
    fmt: Format,
) -> Result<()> {
    let client = session_client(base)?;
    emit(
        &console_api::assignments(&client, department_id, role).await?,
        fmt,
    )
}

/// `geek console application list [--status --q --limit --offset]`：投递列表。
pub async fn applications(
    base: &str,
    status: Option<&str>,
    q: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
    fmt: Format,
) -> Result<()> {
    let client = session_client(base)?;
    emit(
        &console_api::applications(&client, status, q, limit, offset).await?,
        fmt,
    )
}

/// `geek console application show <uuid>`：投递详情（含审核历史）。
pub async fn application(base: &str, application_id: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(
        &console_api::application(&client, application_id).await?,
        fmt,
    )
}

/// `geek console feedback list [--status --limit]`：意见箱列表。
pub async fn feedback(
    base: &str,
    status: Option<&str>,
    limit: Option<u32>,
    fmt: Format,
) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::feedback(&client, status, limit).await?, fmt)
}

/// `geek console audit [--limit --offset --action]`：审计日志（脱敏）。
pub async fn audit(
    base: &str,
    limit: Option<u32>,
    offset: Option<u32>,
    action: Option<&str>,
    fmt: Format,
) -> Result<()> {
    let client = session_client(base)?;
    emit(
        &console_api::audit(&client, limit, offset, action).await?,
        fmt,
    )
}
