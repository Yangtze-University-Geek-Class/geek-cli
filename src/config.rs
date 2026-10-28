use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Stored {
    /// GitHub OAuth 令牌。只用于 api.github.com；**绝不能发给站内 API**。
    pub token: Option<String>,
    pub login: Option<String>,
    /// 站内（yangtzeu.work）的设备令牌，和 GitHub 令牌分开存。
    /// 老版本的 token.json 没有这个字段，`default` 让它们仍能读出来。
    #[serde(default)]
    pub site_token: Option<String>,
}

fn config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("could not resolve user config directory")?
        .join("geek");
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    Ok(dir)
}

pub fn token_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("token.json"))
}

pub fn load() -> Result<Stored> {
    let path = token_path()?;
    if !path.exists() {
        return Ok(Stored::default());
    }
    let bytes =
        std::fs::read(&path).with_context(|| format!("read token from {}", path.display()))?;
    let parsed: Stored = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(parsed)
}

pub fn save(stored: &Stored) -> Result<()> {
    let path = token_path()?;
    let bytes = serde_json::to_vec_pretty(stored)?;
    std::fs::write(&path, bytes).with_context(|| format!("write {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&path)?.permissions();
        perm.set_mode(0o600);
        std::fs::set_permissions(&path, perm)?;
    }
    Ok(())
}

pub fn clear() -> Result<()> {
    let path = token_path()?;
    if path.exists() {
        std::fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
    }
    Ok(())
}

pub fn require_token() -> Result<String> {
    if let Ok(t) = std::env::var("GEEK_TOKEN") {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    let stored = load()?;
    stored
        .token
        .filter(|t| !t.is_empty())
        .context("not signed in — run `geek login` first")
}

/// 站内 API 用的令牌：环境变量 `GEEK_SITE_TOKEN`，或登录时存下的设备令牌。
///
/// 这里**故意不回退到 GitHub 令牌**。那枚令牌带 `admin:org` + `repo`，
/// 发给站内 API 等于把整个组织的管理权交出去，而站内 API 也根本不认它
/// （app/server/src/middleware/bearer-auth.ts 只查 device_grants）。
pub fn require_site_token() -> Result<String> {
    if let Ok(t) = std::env::var("GEEK_SITE_TOKEN") {
        let t = t.trim().to_string();
        if !t.is_empty() {
            return Ok(t);
        }
    }
    load()?
        .site_token
        .filter(|t| !t.trim().is_empty())
        .context(concat!(
            "站内 API 需要设备令牌（不是 GitHub 令牌）：",
            "到 https://yangtzeu.work/device 授权后设置 GEEK_SITE_TOKEN",
        ))
}
