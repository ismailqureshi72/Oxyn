//! Reads the registry: over HTTPS (the default, `https://ui.oxyn.workers.dev/r`), plain HTTP, or a
//! local directory (a registry built with `oxyn-registry`, handy offline and in CI).

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use oxyn_registry_schema::{RegistryIndex, RegistryItem, SCHEMA_VERSION, valid_slug};

use crate::Error;

/// The public registry.
pub const DEFAULT_REGISTRY: &str = "https://ui.oxyn.workers.dev/r";
/// Where Oxyn Pro is sold.
pub const PRICING_URL: &str = "https://ui.oxyn.workers.dev/pricing";
/// Environment variable that overrides the registry base.
pub const REGISTRY_ENV: &str = "OXYN_REGISTRY";
/// Environment variable holding an Oxyn Pro token. Read by [`auth_header`]; nothing sends it yet:
/// the authenticated Pro registry arrives with P3, which wires it into [`Client::fetch_pro`].
pub const TOKEN_ENV: &str = "OXYN_TOKEN";

/// The `Authorization` header for the Pro registry, from `OXYN_TOKEN`. The seam for P3: the free
/// registry never sends it.
pub fn auth_header() -> Option<String> {
    std::env::var(TOKEN_ENV)
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .map(|t| format!("Bearer {t}"))
}

/// Picks the registry base: the flag, else `OXYN_REGISTRY`, else `oxyn.toml`, else the default.
pub fn registry_base(flag: Option<&str>, config: Option<&str>) -> String {
    let env = std::env::var(REGISTRY_ENV)
        .ok()
        .filter(|s| !s.trim().is_empty());
    flag.map(String::from)
        .or(env)
        .or(config.map(String::from))
        .unwrap_or_else(|| DEFAULT_REGISTRY.to_string())
        .trim_end_matches('/')
        .to_string()
}

enum Base {
    Http(String),
    Dir(PathBuf),
}

/// A registry client with a small in-memory cache.
pub struct Client {
    base: Base,
    agent: ureq::Agent,
    cache: RefCell<HashMap<String, String>>,
}

impl Client {
    /// A client for a base URL (`https://…/r`, `http://…`, `file:///…` or a directory path).
    pub fn new(base: &str) -> Client {
        let base = if base.starts_with("http://") || base.starts_with("https://") {
            Base::Http(base.trim_end_matches('/').to_string())
        } else {
            Base::Dir(PathBuf::from(base.strip_prefix("file://").unwrap_or(base)))
        };
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(60)))
            .user_agent(concat!("oxyn-cli/", env!("CARGO_PKG_VERSION")))
            .http_status_as_error(false)
            .build()
            .new_agent();
        Client {
            base,
            agent,
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// The base, for messages.
    pub fn describe(&self) -> String {
        match &self.base {
            Base::Http(u) => u.clone(),
            Base::Dir(p) => p.display().to_string(),
        }
    }

    /// Fetches a path relative to the base. `Ok(None)` for a 404 / missing file.
    pub fn get(&self, rel: &str) -> Result<Option<String>, Error> {
        if let Some(hit) = self.cache.borrow().get(rel) {
            return Ok(Some(hit.clone()));
        }
        let body = match &self.base {
            Base::Dir(dir) => {
                let p = dir.join(rel);
                match fs::read_to_string(&p) {
                    Ok(s) => Some(s),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                    Err(e) => return Err(Error::Io(p, e)),
                }
            }
            Base::Http(base) => {
                let url = format!("{base}/{rel}");
                let mut res = self
                    .agent
                    .get(&url)
                    .header("accept", "application/json, text/plain, */*")
                    .call()
                    .map_err(|e| Error::Network(url.clone(), e.to_string()))?;
                match res.status().as_u16() {
                    200 => Some(
                        res.body_mut()
                            .with_config()
                            .limit(64 * 1024 * 1024)
                            .read_to_string()
                            .map_err(|e| Error::Network(url.clone(), e.to_string()))?,
                    ),
                    404 => None,
                    s => return Err(Error::Network(url, format!("HTTP {s}"))),
                }
            }
        };
        if let Some(b) = &body {
            self.cache.borrow_mut().insert(rel.to_string(), b.clone());
        }
        Ok(body)
    }

    /// `index.json`.
    pub fn index(&self) -> Result<RegistryIndex, Error> {
        let s = self.get("index.json")?.ok_or_else(|| {
            Error::Network(
                format!("{}/index.json", self.describe()),
                "not found: is this an Oxyn registry?".into(),
            )
        })?;
        let idx: RegistryIndex =
            serde_json::from_str(&s).map_err(|e| Error::Invalid(format!("index.json: {e}")))?;
        if idx.schema > SCHEMA_VERSION {
            return Err(Error::Invalid(format!(
                "the registry uses schema {} and this oxyn understands {SCHEMA_VERSION}: update it with `cargo install --git https://github.com/ismailqureshi72/Oxyn oxyn-cli --force`",
                idx.schema
            )));
        }
        Ok(idx)
    }

    /// `<slug>.json`. `Ok(None)` when there is no public JSON (Pro, or unknown).
    pub fn item(&self, slug: &str) -> Result<Option<RegistryItem>, Error> {
        if !valid_slug(slug) {
            return Err(Error::Invalid(format!("`{slug}` is not a valid item name")));
        }
        let Some(s) = self.get(&format!("{slug}.json"))? else {
            return Ok(None);
        };
        let it: RegistryItem =
            serde_json::from_str(&s).map_err(|e| Error::Invalid(format!("{slug}.json: {e}")))?;
        Ok(Some(it))
    }

    /// The Pro registry (P3). Today there is none: every Pro item stops with [`Error::ProRequired`].
    /// P3 sends [`auth_header`] to the authenticated endpoint here.
    pub fn fetch_pro(&self, slug: &str) -> Result<RegistryItem, Error> {
        let _token = auth_header();
        Err(Error::ProRequired(slug.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_directory_registry_and_caches() {
        let d = tempfile::tempdir().unwrap();
        fs::write(
            d.path().join("index.json"),
            r#"{"schema":1,"name":"oxyn","homepage":"h","pricing":"p","source":{"git":"g","rev":"r"},"items":[]}"#,
        )
        .unwrap();
        let c = Client::new(d.path().to_str().unwrap());
        assert_eq!(c.index().unwrap().items.len(), 0);
        assert!(c.item("missing").unwrap().is_none());
        assert!(matches!(c.item("../x"), Err(Error::Invalid(_))));
        fs::remove_file(d.path().join("index.json")).unwrap();
        assert!(c.index().is_ok(), "served from the cache");
        assert!(matches!(c.fetch_pro("x"), Err(Error::ProRequired(_))));
    }

    #[test]
    fn refuses_a_newer_schema() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("index.json"), r#"{"schema":99,"name":"oxyn","homepage":"h","pricing":"p","source":{"git":"g","rev":"r"},"items":[]}"#).unwrap();
        let c = Client::new(&format!("file://{}", d.path().display()));
        assert!(matches!(c.index(), Err(Error::Invalid(m)) if m.contains("update")));
    }

    #[test]
    fn base_precedence() {
        assert_eq!(
            registry_base(Some("http://x/r/"), Some("http://y")),
            "http://x/r"
        );
    }
}
