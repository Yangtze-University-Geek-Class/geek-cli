use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::io::{IsTerminal, Read};

use super::current_sid;
use crate::api::auth as auth_api;
use crate::api::console as console_api;
use crate::api::me as me_api;
use crate::api::public as public_api;
use crate::api::{ApiError, Client, UsageError};
use crate::output::{emit, print_message, Format};
use crate::session::{self, Session};

/// `geek login`：导入浏览器会话（sid），校验后按 base 保存。
pub async fn login(base: &str, sid: Option<String>, sid_stdin: bool, fmt: Format) -> Result<()> {
    let sid = obtain_sid(base, sid, sid_stdin)?;
    let client = Client::new(base, Some(sid.clone()))?;
    let me: Value = auth_api::me(&client).await?;
    if me["signed_in"] != json!(true) {
        let hint = if me["session_expired"] == json!(true) {
            "（服务端标记 session_expired）"
        } else {
            ""
        };
        return Err(ApiError::unauthenticated(format!(
            "会话无效或已过期{hint}：请在浏览器登录 {base}/console 后，重新复制 Cookie 里的 sid"
        ))
        .into());
    }
    let login_name = me["login"].as_str().unwrap_or_default().to_string();
    let user_id = me["user_id"].as_u64();
    let avatar_url = me["avatar_url"].as_str().map(str::to_string);
    let saved_at = chrono::Local::now().to_rfc3339();
    let session = Session {
        sid,
        login: Some(login_name.clone()),
        user_id,
        avatar_url,
        saved_at: Some(saved_at.clone()),
    };
    session::put(base, &session)?;
    if session::env_sid().is_some() {
        print_message(
            "注意：环境里已设置 GEEK_SID，其他命令将优先用它（本次仍已保存到本地会话文件）",
        );
    }
    print_message(&format!(
        "已登录 @{login_name}（{base}），会话保存到 {}",
        session::path_display()?
    ));
    emit(
        &json!({ "ok": true, "base": base, "login": login_name, "user_id": user_id, "saved_at": saved_at }),
        fmt,
    )
}

fn obtain_sid(base: &str, flag: Option<String>, sid_stdin: bool) -> Result<String> {
    if let Some(value) = flag {
        let value = value.trim().to_string();
        if value.is_empty() {
            return Err(UsageError("--sid 不能为空".into()).into());
        }
        print_message(
            "警告：命令行参数会出现在进程列表 / shell 历史里，建议改用 --sid-stdin 或 GEEK_SID",
        );
        return Ok(value);
    }
    if sid_stdin {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("read sid from stdin")?;
        let value = buf.trim().to_string();
        if value.is_empty() {
            return Err(
                UsageError("标准输入为空：请把 sid 通过管道传给 --sid-stdin".into()).into(),
            );
        }
        return Ok(value);
    }
    if !std::io::stdin().is_terminal() {
        return Err(
            UsageError("非交互环境：请用 `--sid-stdin` 或 GEEK_SID 提供会话".into()).into(),
        );
    }
    prompt_sid(base)
}

fn prompt_sid(base: &str) -> Result<String> {
    use crossterm::event::{read, Event, KeyCode, KeyEventKind, KeyModifiers};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

    eprintln!("1) 在浏览器登录 {base}/console");
    eprintln!("2) 开发者工具 → Application → Cookies → 复制 sid 的值");
    eprintln!("3) 粘贴到下面（不回显），回车确认；Esc / Ctrl-C 取消");

    enable_raw_mode().context("enable raw mode")?;
    let mut buf = String::new();
    let outcome: Result<String> = loop {
        match read() {
            Ok(Event::Key(key)) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Enter => break Ok(buf.trim().to_string()),
                    KeyCode::Esc => break Err(UsageError("已取消".into()).into()),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        break Err(UsageError("已取消".into()).into());
                    }
                    KeyCode::Backspace => {
                        buf.pop();
                    }
                    KeyCode::Char(c) => buf.push(c),
                    _ => {}
                }
            }
            Ok(_) => {}
            Err(error) => break Err(anyhow::Error::from(error)),
        }
    };
    let _ = disable_raw_mode();
    eprintln!();
    let sid = outcome?;
    if sid.is_empty() {
        return Err(UsageError("未输入内容".into()).into());
    }
    Ok(sid)
}

/// `geek logout`：尽力登出服务端会话，并清理本地会话。
pub async fn logout(base: &str, fmt: Format) -> Result<()> {
    let mut server_signout = None;
    let mut warning = None;
    if let Some(sid) = current_sid(base)? {
        let client = Client::new(base, Some(sid))?;
        match auth_api::signout(&client).await {
            Ok(_) => server_signout = Some(true),
            Err(error) => {
                server_signout = Some(false);
                warning = Some(error.to_string());
            }
        }
    }
    let removed = session::delete(base)?;
    let summary = match (server_signout, removed) {
        (Some(true), _) => "已登出服务端会话，本地会话已清理",
        (Some(false), _) => "服务端登出未确认，本地会话已清理",
        (None, true) => "本地会话已清理",
        (None, false) => "本地没有存储的会话",
    };
    print_message(&format!("{summary}（{base}）"));
    emit(
        &json!({ "ok": true, "base": base, "removed": removed, "server_signout": server_signout, "warning": warning }),
        fmt,
    )
}

/// `geek whoami`：聚合身份（auth + console? + orgs），形状见设计 02 §7。
pub async fn whoami(base: &str, fmt: Format) -> Result<()> {
    let client = Client::new(base, current_sid(base)?)?;
    let auth: Value = auth_api::me(&client).await?;
    if auth["signed_in"] != json!(true) {
        return Err(ApiError::unauthenticated(format!(
            "未登录或会话已失效：先运行 `geek login --base {base}`"
        ))
        .into());
    }
    let mut out = json!({ "auth": auth });
    match console_api::me(&client).await {
        Ok(value) => out["console"] = value,
        Err(error) => {
            if !is_forbidden(&error) {
                return Err(error);
            }
        }
    }
    out["orgs"] = me_api::orgs(&client).await?;
    emit(&out, fmt)
}

/// `geek status`：连通性 / 会话 / 能力自检（同一聚合形状，含 `health`）。
pub async fn status(base: &str, fmt: Format) -> Result<()> {
    let client = Client::new(base, current_sid(base)?)?;
    let health: Value = public_api::healthz(&client).await?;
    let auth: Value = auth_api::me(&client).await?;
    let mut out = json!({ "health": health, "auth": auth });
    if auth["signed_in"] == json!(true) {
        if let Ok(value) = console_api::me(&client).await {
            out["console"] = value;
        }
        if let Ok(value) = me_api::orgs(&client).await {
            out["orgs"] = value;
        }
    }
    emit(&out, fmt)
}

fn is_forbidden(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<ApiError>()
        .map(|api| api.status == Some(403))
        .unwrap_or(false)
}
