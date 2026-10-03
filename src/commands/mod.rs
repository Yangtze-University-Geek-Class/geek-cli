pub mod auth;

use anyhow::Result;

use crate::session;

/// 当前 base 的会话来源：`GEEK_SID`（不落盘）优先，其次本地会话文件。
pub fn current_sid(base: &str) -> Result<Option<String>> {
    if let Some(sid) = session::env_sid() {
        return Ok(Some(sid));
    }
    Ok(session::get(base)?.map(|stored| stored.sid))
}
