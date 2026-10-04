pub mod auth;
pub mod console;
pub mod org;
pub mod repo;

use anyhow::Result;

use crate::api::{ApiError, Client};
use crate::session;

/// 当前 base 的会话来源：`GEEK_SID`（不落盘）优先，其次本地会话文件。
pub fn current_sid(base: &str) -> Result<Option<String>> {
    if let Some(sid) = session::env_sid() {
        return Ok(Some(sid));
    }
    Ok(session::get(base)?.map(|stored| stored.sid))
}

/// 需要会话的命令：无会话直接按退出码契约（3）报错，不发无谓请求。
pub fn session_client(base: &str) -> Result<Client> {
    match current_sid(base)? {
        Some(sid) => Client::new(base, Some(sid)),
        None => Err(ApiError::unauthenticated(format!(
            "未登录：先运行 `geek login --base {base}`（或在环境里设置 GEEK_SID）"
        ))
        .into()),
    }
}
