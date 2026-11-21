//! Signing the CLI in to Oxyn: the token `oxyn login` saves in the user config directory, the
//! `OXYN_TOKEN` override, the browser sign-in (a device flow) and `whoami`.
//!
//! The token file is `<config dir>/oxyn/credentials.toml` (Linux `$XDG_CONFIG_HOME` or
//! `~/.config`, macOS `~/Library/Application Support`, Windows `%APPDATA%`; `OXYN_CONFIG_DIR`
//! overrides the whole path's directory). On Unix it is written readable by the owner only.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::Error;
use crate::client::{Client, TOKEN_ENV};

/// Overrides the directory holding `credentials.toml`.
pub const CONFIG_DIR_ENV: &str = "OXYN_CONFIG_DIR";

/// What `oxyn login` saves.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Credentials {
    /// The CLI token (`oxyn_…`).
    pub token: String,
    /// The account it belongs to, for messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The API it was made for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
}

/// The directory holding `credentials.toml`, if one can be found.
pub fn config_dir() -> Option<PathBuf> {
    let env = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if let Some(d) = env(CONFIG_DIR_ENV) {
        return Some(d);
    }
    let base = if cfg!(windows) {
        env("APPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support"))
    } else {
        env("XDG_CONFIG_HOME").or_else(|| env("HOME").map(|h| h.join(".config")))
    }?;
    Some(base.join("oxyn"))
}

/// `<config dir>/credentials.toml`.
pub fn credentials_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("credentials.toml"))
}

/// The saved credentials, if any.
pub fn load() -> Option<Credentials> {
    let p = credentials_path()?;
    let s = fs::read_to_string(p).ok()?;
    toml::from_str::<Credentials>(&s)
        .ok()
        .filter(|c| !c.token.trim().is_empty())
}

/// Saves the credentials (owner-only on Unix). Returns the file written.
pub fn save(c: &Credentials) -> Result<PathBuf, Error> {
    let p = credentials_path().ok_or_else(|| {
        Error::Invalid(format!(
            "no config directory (set HOME or {CONFIG_DIR_ENV}); use {TOKEN_ENV} instead"
        ))
    })?;
    let dir = p.parent().expect("has a parent");
    fs::create_dir_all(dir).map_err(|e| Error::Io(dir.to_path_buf(), e))?;
    let body = format!(
        "# Written by `oxyn login`. Remove with `oxyn logout`; revoke on your account page.\n{}",
        toml::to_string(c).map_err(|e| Error::Invalid(e.to_string()))?
    );
    write_private(&p, &body)?;
    Ok(p)
}

#[cfg(unix)]
fn write_private(p: &std::path::Path, body: &str) -> Result<(), Error> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::fs::PermissionsExt;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(p)
        .map_err(|e| Error::Io(p.to_path_buf(), e))?;
    // An older file keeps its mode on open: make sure.
    fs::set_permissions(p, fs::Permissions::from_mode(0o600))
        .map_err(|e| Error::Io(p.to_path_buf(), e))?;
    f.write_all(body.as_bytes())
        .map_err(|e| Error::Io(p.to_path_buf(), e))
}

#[cfg(not(unix))]
fn write_private(p: &std::path::Path, body: &str) -> Result<(), Error> {
    fs::write(p, body).map_err(|e| Error::Io(p.to_path_buf(), e))
}

/// Removes the saved credentials. `Ok(false)` when there were none.
pub fn remove() -> Result<bool, Error> {
    let Some(p) = credentials_path() else {
        return Ok(false);
    };
    match fs::remove_file(&p) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(Error::Io(p, e)),
    }
}

/// The token to send: `OXYN_TOKEN`, else the saved one.
pub fn token() -> Option<String> {
    std::env::var(TOKEN_ENV)
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .or_else(|| load().map(|c| c.token.trim().to_string()))
}

/// Where the token in use comes from, for `whoami`.
pub fn token_source() -> &'static str {
    if std::env::var(TOKEN_ENV).is_ok_and(|t| !t.trim().is_empty()) {
        "OXYN_TOKEN"
    } else if load().is_some() {
        "oxyn login"
    } else {
        "none"
    }
}

/// `GET /api/cli/whoami`: account, plan and this month's usage.
pub fn whoami(client: &Client) -> Result<Value, Error> {
    if client.token().is_none() {
        return Err(Error::Invalid(format!(
            "not signed in: run `oxyn login` (or set {TOKEN_ENV})"
        )));
    }
    let r = client.call_api("/cli/whoami", None, true)?;
    match r.status {
        200 => Ok(r.body),
        401 => Err(Error::Invalid(format!(
            "this token is not valid any more (revoked?): run `oxyn login` again{}",
            if token_source() == "OXYN_TOKEN" {
                " or change OXYN_TOKEN"
            } else {
                ""
            }
        ))),
        s => Err(Error::Network(
            format!("{}/cli/whoami", client.api().unwrap_or("")),
            r.message().unwrap_or_else(|| format!("HTTP {s}")),
        )),
    }
}

/// One line for the month's usage, from a whoami / error body's `usage`.
pub fn usage_line(v: &Value) -> Option<String> {
    v["usage"]["summary"].as_str().map(String::from)
}

/// A started browser sign-in.
#[derive(Debug, Clone)]
pub struct DeviceLogin {
    /// Secret: polled with.
    pub device_code: String,
    /// Shown to the person and in the URL.
    pub user_code: String,
    /// The page to open.
    pub url: String,
    /// Seconds between polls.
    pub interval: u64,
    /// Seconds until it expires.
    pub expires_in: u64,
}

/// Starts `oxyn login`: the server answers with a code and the page that approves it.
pub fn start_login(client: &Client, host: &str) -> Result<DeviceLogin, Error> {
    let r = client.call_api("/cli/auth/start", Some(&json!({ "client": host })), false)?;
    if r.status != 200 {
        return Err(Error::Network(
            format!("{}/cli/auth/start", client.api().unwrap_or("")),
            r.message().unwrap_or_else(|| format!("HTTP {}", r.status)),
        ));
    }
    let s = |k: &str| r.body[k].as_str().unwrap_or("").to_string();
    Ok(DeviceLogin {
        device_code: s("device_code"),
        user_code: s("user_code"),
        url: s("verification_url"),
        interval: r.body["interval"].as_u64().unwrap_or(3).clamp(1, 30),
        expires_in: r.body["expires_in"].as_u64().unwrap_or(600),
    })
}

/// Polls until the browser approves (the token and email), declines or the request expires.
pub fn wait_login(client: &Client, d: &DeviceLogin) -> Result<(String, Option<String>), Error> {
    let until = Instant::now() + Duration::from_secs(d.expires_in);
    loop {
        let r = client.call_api(
            "/cli/auth/poll",
            Some(&json!({ "device_code": d.device_code })),
            false,
        )?;
        match (r.status, r.body["status"].as_str()) {
            (200, Some("approved")) => {
                let t = r.body["token"].as_str().unwrap_or("").to_string();
                if t.is_empty() {
                    return Err(Error::Invalid("the server sent no token".into()));
                }
                return Ok((t, r.body["email"].as_str().map(String::from)));
            }
            (200, _) => {}
            _ => {
                return Err(Error::Invalid(
                    r.message()
                        .unwrap_or_else(|| format!("sign-in failed (HTTP {})", r.status)),
                ));
            }
        }
        if Instant::now() >= until {
            return Err(Error::Invalid(
                "the sign-in request expired: run `oxyn login` again".into(),
            ));
        }
        std::thread::sleep(Duration::from_secs(d.interval));
    }
}

/// This machine's name, for the token's name on the account page.
pub fn host_name() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .ok()
        .or_else(|| fs::read_to_string("/etc/hostname").ok())
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| "this computer".into())
}

/// Opens a URL in the default browser; `false` when no opener worked.
pub fn open_browser(url: &str) -> bool {
    let tries: &[(&str, &[&str])] = if cfg!(windows) {
        &[("cmd", &["/C", "start", ""])]
    } else if cfg!(target_os = "macos") {
        &[("open", &[])]
    } else {
        &[("xdg-open", &[]), ("gio", &["open"])]
    };
    tries.iter().any(|(cmd, pre)| {
        std::process::Command::new(cmd)
            .args(*pre)
            .arg(url)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    })
}
