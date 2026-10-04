//! 控制台读命令（P0）。

use anyhow::Result;

use crate::api::console as console_api;
use crate::commands::session_client;
use crate::output::{emit, Format};

/// `geek console me`：控制台身份 / 称号 / 能力（GET /api/console/me）。
pub async fn me(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::me(&client).await?, fmt)
}
