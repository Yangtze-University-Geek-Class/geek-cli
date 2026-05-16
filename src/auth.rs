use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::time::Duration;

// GitHub's "official" client ID for the gh CLI. Reusing it gives us OAuth Device
// flow without registering our own app — the trade-off is the consent screen
// shows "GitHub CLI" instead of "Geek CLI". For a polished product we'd
// register our own GitHub App; for an internal tool this is the pragmatic move.
const CLIENT_ID: &str = "178c6fc778ccc68e1d6a";
const SCOPES: &str = "read:user,user:email,admin:org,read:org,repo";
const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_URL: &str = "https://api.github.com/user";

#[derive(Debug, Deserialize)]
struct DeviceCode {
    device_code: String,
    user_code: String,
    verification_uri: String,
    interval: u64,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TokenResp {
    Ok { access_token: String },
    Pending { error: String, #[allow(dead_code)] error_description: Option<String> },
}

#[derive(Debug, Deserialize)]
struct User {
    login: String,
}

pub async fn login() -> Result<(String, String)> {
    // GitHub's OAuth endpoint frequently closes keep-alive connections between
    // polls. Reusing a pooled connection after the server has half-closed it
    // surfaces as "connection closed before message completed". Disable the
    // pool entirely + retry once on transient errors.
    let client = reqwest::Client::builder()
        .user_agent("geek-cli")
        .pool_max_idle_per_host(0)
        .timeout(Duration::from_secs(15))
        .build()?;

    let dc: DeviceCode = client
        .post(DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .form(&[("client_id", CLIENT_ID), ("scope", SCOPES)])
        .send()
        .await
        .context("request device code")?
        .error_for_status()
        .context("device code endpoint refused")?
        .json()
        .await
        .context("parse device code response")?;

    eprintln!();
    eprintln!("  打开浏览器访问: {}", dc.verification_uri);
    eprintln!("  输入这个一次性 code: \x1b[1m\x1b[32m{}\x1b[0m", dc.user_code);
    eprintln!("  此 code 有效期 {} 秒", dc.expires_in);
    eprintln!();

    let _ = open_browser(&dc.verification_uri);

    let mut interval = dc.interval.max(5);
    let deadline = std::time::Instant::now() + Duration::from_secs(dc.expires_in);
    loop {
        if std::time::Instant::now() > deadline {
            bail!("授权超时，请重新运行 `geek login`");
        }
        tokio::time::sleep(Duration::from_secs(interval)).await;

        let mut attempt = 0u8;
        let resp: TokenResp = loop {
            attempt += 1;
            let r = client
                .post(TOKEN_URL)
                .header("Accept", "application/json")
                .form(&[
                    ("client_id", CLIENT_ID),
                    ("device_code", dc.device_code.as_str()),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()
                .await;
            match r {
                Ok(res) => match res.json::<TokenResp>().await {
                    Ok(parsed) => break parsed,
                    Err(e) if attempt < 3 => {
                        eprintln!("[geek] token parse retry {attempt}: {e}");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                    Err(e) => return Err(e).context("parse token response"),
                },
                Err(e) if attempt < 3 => {
                    eprintln!("[geek] token poll retry {attempt}: {e}");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
                Err(e) => return Err(e).context("token poll"),
            }
        };

        match resp {
            TokenResp::Ok { access_token } => {
                let user: User = client
                    .get(USER_URL)
                    .bearer_auth(&access_token)
                    .header("User-Agent", "geek-cli")
                    .header("Accept", "application/vnd.github+json")
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;
                return Ok((access_token, user.login));
            }
            TokenResp::Pending { error, .. } => match error.as_str() {
                "authorization_pending" => continue,
                "slow_down" => {
                    interval += 5;
                    continue;
                }
                "expired_token" => bail!("device code 已过期，请重新运行 `geek login`"),
                "access_denied" => bail!("授权被拒绝"),
                other => bail!("GitHub 返回错误: {other}"),
            },
        }
    }
}

fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "linux")]
    let cmd = "xdg-open";
    #[cfg(target_os = "windows")]
    let cmd = "explorer";
    std::process::Command::new(cmd).arg(url).spawn()?;
    Ok(())
}
