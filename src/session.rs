use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 一个 base 上的会话（`sid` 等同密码级凭据；见 docs/architecture/02 §1）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub sid: String,
    #[serde(default)]
    pub login: Option<String>,
    #[serde(default)]
    pub user_id: Option<u64>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub saved_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Store {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default)]
    sessions: BTreeMap<String, Session>,
}

fn default_version() -> u32 {
    1
}

impl Default for Store {
    fn default() -> Self {
        Self {
            version: 1,
            sessions: BTreeMap::new(),
        }
    }
}

/// 归一化 base：去首尾空白、去末尾 `/`、统一小写（三环境各自独立会话）。
pub fn normalize_base(base: &str) -> String {
    base.trim().trim_end_matches('/').to_lowercase()
}

fn store_path() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("could not resolve user config directory")?
        .join("geek");
    Ok(dir.join("session.json"))
}

pub fn path_display() -> Result<String> {
    Ok(store_path()?.display().to_string())
}

fn read_store(path: &Path) -> Result<Store> {
    if !path.exists() {
        return Ok(Store::default());
    }
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

fn write_store(path: &Path, store: &Store) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    }
    let tmp = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(store)?;
    std::fs::write(&tmp, &bytes).with_context(|| format!("write {}", tmp.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&tmp)?.permissions();
        perm.set_mode(0o600);
        std::fs::set_permissions(&tmp, perm)?;
    }
    std::fs::rename(&tmp, path).with_context(|| format!("replace {}", path.display()))?;
    Ok(())
}

pub fn get_at(path: &Path, base: &str) -> Result<Option<Session>> {
    Ok(read_store(path)?
        .sessions
        .get(&normalize_base(base))
        .cloned())
}

pub fn put_at(path: &Path, base: &str, session: &Session) -> Result<()> {
    let mut store = read_store(path)?;
    store.version = 1;
    store.sessions.insert(normalize_base(base), session.clone());
    write_store(path, &store)
}

pub fn delete_at(path: &Path, base: &str) -> Result<bool> {
    let mut store = read_store(path)?;
    if store.sessions.remove(&normalize_base(base)).is_none() {
        return Ok(false);
    }
    write_store(path, &store)?;
    Ok(true)
}

pub fn get(base: &str) -> Result<Option<Session>> {
    get_at(&store_path()?, base)
}

pub fn put(base: &str, session: &Session) -> Result<()> {
    put_at(&store_path()?, base, session)
}

pub fn delete(base: &str) -> Result<bool> {
    delete_at(&store_path()?, base)
}

/// `GEEK_SID`（不落盘）优先于本地会话文件。
pub fn env_sid() -> Option<String> {
    std::env::var("GEEK_SID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(tag: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("geek-session-{tag}-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        path
    }

    fn sample(sid: &str) -> Session {
        Session {
            sid: sid.to_string(),
            login: None,
            user_id: None,
            avatar_url: None,
            saved_at: None,
        }
    }

    #[test]
    fn roundtrip_with_base_normalization() {
        let path = tmp_path("roundtrip");
        put_at(&path, "https://YangtzeU.Work/", &sample("abc")).unwrap();
        let got = get_at(&path, "https://yangtzeu.work")
            .unwrap()
            .expect("stored");
        assert_eq!(got.sid, "abc");
        assert!(delete_at(&path, "https://yangtzeu.work").unwrap());
        assert!(get_at(&path, "https://yangtzeu.work").unwrap().is_none());
        assert!(!delete_at(&path, "https://yangtzeu.work").unwrap());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn multi_base_isolation() {
        let path = tmp_path("multi");
        put_at(&path, "https://a.example", &sample("a")).unwrap();
        put_at(&path, "https://b.example", &sample("b")).unwrap();
        assert_eq!(
            get_at(&path, "https://a.example").unwrap().unwrap().sid,
            "a"
        );
        assert_eq!(
            get_at(&path, "https://b.example").unwrap().unwrap().sid,
            "b"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[cfg(unix)]
    #[test]
    fn store_file_is_0600() {
        use std::os::unix::fs::PermissionsExt;
        let path = tmp_path("mode");
        put_at(&path, "https://x.example", &sample("s")).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let _ = std::fs::remove_file(&path);
    }
}
