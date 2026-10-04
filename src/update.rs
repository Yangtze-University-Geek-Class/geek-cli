//! 版本比较与更新检查缓存（设计 `docs/architecture/04` §2/§5 的框架部分）。
//!
//! 本模块只含纯逻辑与本地缓存；网络检查与 `self update` 命令属后续切片。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 比较语义化版本（允许 `v` 前缀与 `-rc.N` 后缀；非数字段按 0 处理）。
/// 例：`v0.2.0 > v0.2.0-rc.2 > v0.2.0-rc.1 > v0.1.9`。
pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    fn parse(version: &str) -> (Vec<u64>, bool, u64) {
        let trimmed = version.trim().trim_start_matches('v');
        let (core, pre) = match trimmed.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (trimmed, None),
        };
        let numbers = core
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect();
        match pre {
            None => (numbers, true, 0),
            Some(pre) => {
                let n = pre
                    .rsplit('.')
                    .next()
                    .and_then(|part| part.parse::<u64>().ok())
                    .unwrap_or(0);
                (numbers, false, n)
            }
        }
    }

    let (numbers_a, stable_a, pre_a) = parse(a);
    let (numbers_b, stable_b, pre_b) = parse(b);
    numbers_a
        .cmp(&numbers_b)
        .then_with(|| stable_a.cmp(&stable_b)) // 正式版 > 预发布
        .then_with(|| pre_a.cmp(&pre_b))
}

/// 更新检查缓存（`~/.config/geek/state/update.json`）。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateCache {
    #[serde(default)]
    pub checked_at: Option<i64>,
    #[serde(default)]
    pub latest_version: Option<String>,
}

fn cache_path() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("could not resolve user config directory")?
        .join("geek")
        .join("state");
    Ok(dir.join("update.json"))
}

impl UpdateCache {
    pub fn load() -> Result<Self> {
        Self::load_at(&cache_path()?)
    }

    pub fn save(&self) -> Result<()> {
        self.save_at(&cache_path()?)
    }

    pub fn load_at(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
    }

    pub fn save_at(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        }
        let tmp = path.with_extension("tmp");
        let bytes = serde_json::to_vec_pretty(self)?;
        std::fs::write(&tmp, &bytes).with_context(|| format!("write {}", tmp.display()))?;
        std::fs::rename(&tmp, path).with_context(|| format!("replace {}", path.display()))?;
        Ok(())
    }

    /// 距上次检查是否已超过 TTL（从未检查过视为过期）。
    pub fn is_stale(&self, now_unix: i64, ttl_secs: i64) -> bool {
        self.checked_at
            .map(|checked| now_unix - checked >= ttl_secs)
            .unwrap_or(true)
    }
}

/// 更新检查闸门（设计 04 §2）：CI / 显式关闭 / 非交互时不检查。
#[derive(Debug, Clone)]
pub struct CheckGate {
    pub disabled_by_flag: bool,
    pub disabled_by_env: bool,
    pub is_ci: bool,
    pub interactive: bool,
}

impl CheckGate {
    /// `interactive` 由调用方传入（判断 stderr 是否 TTY）；`--no-update-check` 是否为真也由调用方传入。
    pub fn from_env(disabled_by_flag: bool, interactive: bool) -> Self {
        Self {
            disabled_by_flag,
            disabled_by_env: std::env::var("GEEK_NO_UPDATE_CHECK")
                .map(|v| !v.is_empty())
                .unwrap_or(false),
            is_ci: std::env::var("CI")
                .map(|v| !v.is_empty() && v != "false")
                .unwrap_or(false),
            interactive,
        }
    }

    pub fn allowed(&self) -> bool {
        !(self.disabled_by_flag || self.disabled_by_env || self.is_ci || !self.interactive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn version_ordering() {
        assert_eq!(compare_versions("v0.2.0", "v0.1.9"), Ordering::Greater);
        assert_eq!(
            compare_versions("v0.2.0-rc.2", "v0.2.0-rc.1"),
            Ordering::Greater
        );
        assert_eq!(compare_versions("v0.2.0", "v0.2.0-rc.2"), Ordering::Greater);
        assert_eq!(compare_versions("0.2.0", "v0.2.0"), Ordering::Equal);
        assert_eq!(compare_versions("v0.1.10", "v0.1.9"), Ordering::Greater);
    }

    #[test]
    fn cache_roundtrip_and_staleness() {
        let mut path = std::env::temp_dir();
        path.push(format!("geek-update-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);

        let cache = UpdateCache {
            checked_at: Some(1000),
            latest_version: Some("v0.3.0".into()),
        };
        cache.save_at(&path).unwrap();
        let loaded = UpdateCache::load_at(&path).unwrap();
        assert_eq!(loaded.latest_version.as_deref(), Some("v0.3.0"));
        assert!(!loaded.is_stale(1500, 1000));
        assert!(loaded.is_stale(2500, 1000));
        assert!(UpdateCache::default().is_stale(0, 1000));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn check_gate_combinations() {
        let gate = |flag, env, ci, tty| CheckGate {
            disabled_by_flag: flag,
            disabled_by_env: env,
            is_ci: ci,
            interactive: tty,
        };
        assert!(gate(false, false, false, true).allowed());
        assert!(!gate(true, false, false, true).allowed());
        assert!(!gate(false, true, false, true).allowed());
        assert!(!gate(false, false, true, true).allowed());
        assert!(!gate(false, false, false, false).allowed());
    }
}
