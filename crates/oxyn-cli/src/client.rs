//! Reads the registry: over HTTPS (the default, `https://oxynui.com/r`), plain HTTP, or a
//! local directory (a registry built with `oxyn-registry`, handy offline and in CI).

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use oxyn_registry_schema::{RegistryIndex, RegistryItem, SCHEMA_VERSION, valid_slug};

use crate::Error;

/// The public registry.
pub const DEFAULT_REGISTRY: &str = "https://oxynui.com/r";
/// Where Oxyn Pro is sold.
pub const PRICING_URL: &str = "https://oxynui.com/pricing";
/// Environment variable that overrides the registry base.
pub const REGISTRY_ENV: &str = "OXYN_REGISTRY";
/// Environment variable holding an Oxyn CLI token; wins over the one `oxyn login` saved.
pub const TOKEN_ENV: &str = "OXYN_TOKEN";
/// Environment variable that overrides the API base (default: the registry base without `/r`, plus
/// `/api`, for an HTTP registry; none for a directory registry).
pub const API_ENV: &str = "OXYN_API";

/// The `Authorization` header for the Pro registry: `OXYN_TOKEN`, else the token `oxyn login`
/// saved (see [`crate::auth`]).
pub fn auth_header() -> Option<String> {
    crate::auth::token().map(|t| format!("Bearer {t}"))
}

/// The API base for a registry base: `https://host/r` → `https://host/api`; `OXYN_API` wins.
pub fn api_base(registry: &str) -> Option<String> {
    if let Ok(a) = std::env::var(API_ENV)
        && !a.trim().is_empty()
    {
        return Some(a.trim().trim_end_matches('/').to_string());
    }
    let r = registry.trim_end_matches('/');
    if !(r.starts_with("http://") || r.starts_with("https://")) {
        return None;
    }
    Some(format!("{}/api", r.strip_suffix("/r").unwrap_or(r)))
}

/// An API answer: status, JSON body (or `null`), and the usage line the Pro registry sends.
pub struct ApiReply {
    /// HTTP status.
    pub status: u16,
    /// The JSON body, `Value::Null` when it was not JSON.
    pub body: serde_json::Value,
    /// `x-oxyn-usage`: "3 of 20 components, 1 of 8 screens, resets 2026-12-05".
    pub usage: Option<String>,
    /// `x-oxyn-counted`: this download took one of the month's items.
    pub counted: bool,
}

impl ApiReply {
    /// The server's error message, if any.
    pub fn message(&self) -> Option<String> {
        self.body["error"]["message"].as_str().map(String::from)
    }
    /// The server's error code, if any.
    pub fn code(&self) -> &str {
        self.body["error"]["code"].as_str().unwrap_or("")
    }
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
    api: Option<String>,
    token: Option<String>,
    via: &'static str,
    usage: RefCell<Option<String>>,
    unlocked: RefCell<Vec<String>>,
}

impl Client {
    /// A client for a base URL (`https://…/r`, `http://…`, `file:///…` or a directory path).
    pub fn new(base: &str) -> Client {
        let base_str = base;
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
        let api = api_base(base_str);
        Client {
            base,
            agent,
            cache: RefCell::new(HashMap::new()),
            api,
            token: crate::auth::token(),
            via: "cli",
            usage: RefCell::new(None),
            unlocked: RefCell::new(Vec::new()),
        }
    }

    /// Puts a registry item's JSON in the cache (installed Pro items, kept in the project, so a
    /// later `add` or `diff` does not download them again).
    pub fn prime(&self, slug: &str, json: String) {
        self.cache
            .borrow_mut()
            .entry(format!("{slug}.json"))
            .or_insert(json);
    }

    /// Who is asking, for the server's records: `cli` (default) or `mcp`.
    pub fn with_via(mut self, via: &'static str) -> Client {
        self.via = via;
        self
    }

    /// Uses this token instead of `OXYN_TOKEN` / the saved one (`None`: no token).
    pub fn with_token(mut self, token: Option<String>) -> Client {
        self.token = token.filter(|t| !t.trim().is_empty());
        self
    }

    /// Uses this API base (`https://host/api`) instead of the one derived from the registry.
    pub fn with_api(mut self, api: Option<String>) -> Client {
        self.api = api.map(|a| a.trim_end_matches('/').to_string());
        self
    }

    /// The API base, if this registry has one.
    pub fn api(&self) -> Option<&str> {
        self.api.as_deref()
    }

    /// The token this client sends, if any.
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// The month's usage the Pro registry last reported ("3 of 20 components, …").
    pub fn last_usage(&self) -> Option<String> {
        self.usage.borrow().clone()
    }

    /// Pro items this client downloaded that took one of the month's items.
    pub fn counted(&self) -> Vec<String> {
        self.unlocked.borrow().clone()
    }

    /// Calls the API: GET when `body` is `None`, else POST JSON. Sends the token when `auth`.
    pub fn call_api(
        &self,
        path: &str,
        body: Option<&serde_json::Value>,
        auth: bool,
    ) -> Result<ApiReply, Error> {
        let Some(api) = &self.api else {
            return Err(Error::Invalid(format!(
                "the registry {} has no Oxyn API (Pro, sign-in): use the hosted registry, or set {API_ENV}",
                self.describe()
            )));
        };
        let url = format!("{api}{path}");
        let mut res = match body {
            None => {
                let mut r = self
                    .agent
                    .get(&url)
                    .header("accept", "application/json")
                    .header("x-oxyn-client", self.via);
                if auth && let Some(t) = &self.token {
                    r = r.header("authorization", &format!("Bearer {t}"));
                }
                r.call()
            }
            Some(b) => {
                let mut r = self
                    .agent
                    .post(&url)
                    .header("accept", "application/json")
                    .header("content-type", "application/json")
                    .header("x-oxyn-client", self.via);
                if auth && let Some(t) = &self.token {
                    r = r.header("authorization", &format!("Bearer {t}"));
                }
                r.send(b.to_string())
            }
        }
        .map_err(|e| Error::Network(url.clone(), e.to_string()))?;
        let status = res.status().as_u16();
        let header = |k: &str| {
            res.headers()
                .get(k)
                .and_then(|v| v.to_str().ok())
                .map(String::from)
        };
        let usage = header("x-oxyn-usage");
        let counted = header("x-oxyn-counted").as_deref() == Some("1");
        let text = res
            .body_mut()
            .with_config()
            .limit(64 * 1024 * 1024)
            .read_to_string()
            .map_err(|e| Error::Network(url.clone(), e.to_string()))?;
        let body = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
        Ok(ApiReply {
            status,
            body,
            usage,
            counted,
        })
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
                "the registry uses schema {} and this oxyn understands {SCHEMA_VERSION}: update it with `cargo install oxyn-cli --force`",
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

    /// A Pro item from the authenticated Pro registry (`GET <api>/r/<slug>.json`). Each distinct
    /// item counts once per billing month; `parent` names the item being installed when this one
    /// is its dependency, which comes with it without counting. The answer is cached, so
    /// [`Client::item`] returns it too.
    pub fn fetch_pro(&self, slug: &str, parent: Option<&str>) -> Result<RegistryItem, Error> {
        if !valid_slug(slug) {
            return Err(Error::Invalid(format!("`{slug}` is not a valid item name")));
        }
        let rel = format!("{slug}.json");
        if let Some(hit) = self.cache.borrow().get(&rel) {
            return serde_json::from_str(hit).map_err(|e| Error::Invalid(format!("{rel}: {e}")));
        }
        if self.api.is_none() {
            return Err(Error::ProRequired(slug.to_string()));
        }
        if self.token.is_none() {
            return Err(Error::SignInRequired(slug.to_string()));
        }
        let path = match parent.filter(|p| *p != slug && valid_slug(p)) {
            Some(p) => format!("/r/{slug}.json?for={p}"),
            None => format!("/r/{slug}.json"),
        };
        let r = self.call_api(&path, None, true)?;
        if let Some(u) = &r.usage {
            *self.usage.borrow_mut() = Some(u.clone());
        }
        match r.status {
            200 => {
                let it: RegistryItem = serde_json::from_value(r.body.clone())
                    .map_err(|e| Error::Invalid(format!("{rel}: {e}")))?;
                if r.counted {
                    self.unlocked.borrow_mut().push(slug.to_string());
                }
                self.cache.borrow_mut().insert(rel, r.body.to_string());
                Ok(it)
            }
            401 => Err(Error::SignInRequired(slug.to_string())),
            403 if r.code() == "quota_exceeded" => Err(Error::Quota(
                r.message()
                    .unwrap_or_else(|| "monthly Pro limit reached".into()),
            )),
            403 => Err(Error::ProRequired(slug.to_string())),
            404 => Err(Error::Invalid(r.message().unwrap_or_else(|| {
                format!("`{slug}` has no installable build in the Pro registry")
            }))),
            s => Err(Error::Network(
                format!("{}{path}", self.api.as_deref().unwrap_or("")),
                r.message().unwrap_or_else(|| format!("HTTP {s}")),
            )),
        }
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
        // A directory registry has no API: Pro items stop with "needs Pro".
        assert!(matches!(c.fetch_pro("x", None), Err(Error::ProRequired(_))));
    }

    #[test]
    fn refuses_a_newer_schema() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("index.json"), r#"{"schema":99,"name":"oxyn","homepage":"h","pricing":"p","source":{"git":"g","rev":"r"},"items":[]}"#).unwrap();
        let c = Client::new(&format!("file://{}", d.path().display()));
        assert!(matches!(c.index(), Err(Error::Invalid(m)) if m.contains("update")));
    }

    #[test]
    fn api_from_registry() {
        if std::env::var(API_ENV).is_ok() {
            return;
        }
        assert_eq!(
            api_base("https://oxynui.com/r").as_deref(),
            Some("https://oxynui.com/api")
        );
        assert_eq!(
            api_base("http://127.0.0.1:9/r/").as_deref(),
            Some("http://127.0.0.1:9/api")
        );
        assert_eq!(api_base("/tmp/r"), None);
    }

    #[test]
    fn base_precedence() {
        assert_eq!(
            registry_base(Some("http://x/r/"), Some("http://y")),
            "http://x/r"
        );
    }
}
