pub mod auth;
pub mod console;
pub mod org;
pub mod repo;

use anyhow::{Context, Result};
use std::io::IsTerminal;

use crate::api::{ApiError, Client, UsageError};
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

/// 写操作通用开关（全局 `--yes` / `--dry-run`）。
#[derive(Debug, Clone, Copy, Default)]
pub struct WriteOpts {
    pub yes: bool,
    pub dry_run: bool,
}

/// dry-run：打印将发送的请求（方法 / 路径 / body），返回 `true` 表示调用方应直接返回成功。
pub fn plan(
    opts: &WriteOpts,
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
) -> Result<bool> {
    if !opts.dry_run {
        return Ok(false);
    }
    eprintln!("[dry-run] {method} {path}");
    if let Some(value) = body {
        eprintln!("[dry-run] body: {value}");
    }
    Ok(true)
}

/// 交互确认：`--yes` 直通；非交互环境（非 TTY）按用法错误（退出码 2）拒绝，绝不挂起。
pub fn confirm(action: &str, opts: &WriteOpts) -> Result<()> {
    confirm_with(action, opts.yes, std::io::stdin().is_terminal())
}

fn confirm_with(action: &str, yes: bool, interactive: bool) -> Result<()> {
    if yes {
        return Ok(());
    }
    if !interactive {
        return Err(UsageError(format!("需要确认：非交互环境请加 --yes（{action}）")).into());
    }
    eprint!("{action}，确认执行？[y/N] ");
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("read confirmation")?;
    if line.trim().eq_ignore_ascii_case("y") {
        Ok(())
    } else {
        Err(UsageError("已取消".into()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_requires_yes_outside_tty() {
        assert!(confirm_with("删除意见", false, false).is_err());
        assert!(confirm_with("删除意见", true, false).is_ok());
        let error = confirm_with("删除意见", false, false).unwrap_err();
        let usage = error.downcast_ref::<UsageError>().expect("usage error");
        assert!(usage.0.contains("--yes"));
    }

    #[test]
    fn plan_prints_only_in_dry_run() {
        let body = serde_json::json!({"a": 1});
        assert!(!plan(&WriteOpts::default(), "PATCH", "/x", Some(&body)).unwrap());
        assert!(plan(
            &WriteOpts {
                yes: false,
                dry_run: true
            },
            "PATCH",
            "/x",
            Some(&body)
        )
        .unwrap());
    }
}
