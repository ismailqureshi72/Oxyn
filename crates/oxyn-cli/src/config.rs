//! `oxyn.toml` (the project's Oxyn settings) and `oxyn.lock` (what the CLI wrote, so local edits are
//! never overwritten by accident).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use oxyn_registry_schema::Framework;
use serde::{Deserialize, Serialize};

use crate::Error;

/// File name of the settings file, next to `Cargo.toml`.
pub const CONFIG_FILE: &str = "oxyn.toml";
/// File name of the lock file, next to `Cargo.toml`.
pub const LOCK_FILE: &str = "oxyn.lock";

/// The crates.io version requirement `oxyn init` writes for the Oxyn crates by default.
pub const CRATES_IO_VERSION: &str = "0.1";

/// Where the Oxyn crates (`oxyn-core`, `oxyn-tokens`) come from: crates.io (the default), a local
/// checkout (`path`) or the git repository (`git`, usually pinned to `rev`). When several are set,
/// `path` wins, then `git`, then `version`; with none set, crates.io at [`CRATES_IO_VERSION`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// crates.io version requirement (`"0.1"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Git repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    /// Pinned commit (the one the registry was built from at `oxyn init`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    /// A local checkout of the Oxyn repository instead of git (for contributors and CI).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl Source {
    /// The Oxyn crates from crates.io at [`CRATES_IO_VERSION`] (what `oxyn init` uses by default).
    pub fn crates_io() -> Source {
        Source {
            version: Some(CRATES_IO_VERSION.into()),
            ..Source::default()
        }
    }

    /// The Oxyn git repository, pinned to a commit when `rev` is given.
    pub fn git(repo: impl Into<String>, rev: Option<String>) -> Source {
        Source {
            git: Some(repo.into()),
            rev,
            ..Source::default()
        }
    }

    /// A local checkout of the Oxyn repository (its root directory).
    pub fn local(dir: impl Into<String>) -> Source {
        Source {
            path: Some(dir.into()),
            ..Source::default()
        }
    }
}

/// `oxyn.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// The renderer this project uses.
    pub framework: Framework,
    /// Directory the components go into, relative to the project (`src/oxyn`).
    pub path: String,
    /// Directory for stylesheets the app loads itself (`assets/oxyn`).
    pub css: String,
    /// Registry base URL; `OXYN_REGISTRY` and `--registry` override it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    /// Where the Oxyn crates come from.
    pub oxyn: Source,
    /// Items added so far.
    #[serde(default)]
    pub installed: Vec<String>,
}

impl Config {
    /// Default paths for a framework.
    pub fn new(framework: Framework, oxyn: Source) -> Config {
        Config {
            framework,
            path: "src/oxyn".into(),
            css: "assets/oxyn".into(),
            registry: None,
            oxyn,
            installed: Vec::new(),
        }
    }

    /// Reads `oxyn.toml` from a project directory.
    pub fn load(dir: &Path) -> Result<Config, Error> {
        let p = dir.join(CONFIG_FILE);
        let s = fs::read_to_string(&p).map_err(|_| Error::NotInitialised(dir.to_path_buf()))?;
        toml::from_str(&s).map_err(|e| Error::Invalid(format!("{}: {e}", p.display())))
    }

    /// Writes `oxyn.toml`.
    pub fn save(&self, dir: &Path) -> Result<(), Error> {
        let mut s =
            String::from("# Oxyn settings for this project (https://oxynui.com/docs/cli/).\n");
        s.push_str(&toml::to_string_pretty(self).map_err(|e| Error::Invalid(e.to_string()))?);
        write(&dir.join(CONFIG_FILE), &s)
    }

    /// The Rust module path of the components directory: `src/oxyn` → `crate::oxyn`.
    pub fn module_path(&self) -> Result<String, Error> {
        module_path_of(&self.path)
    }

    /// Records an installed item (once).
    pub fn mark_installed(&mut self, slug: &str) {
        if !self.installed.iter().any(|s| s == slug) {
            self.installed.push(slug.to_string());
            self.installed.sort();
        }
    }
}

/// `src/oxyn` → `crate::oxyn`; `src/ui/kit` → `crate::ui::kit`. The directory must be under `src/`.
pub fn module_path_of(path: &str) -> Result<String, Error> {
    let p = path.trim_matches('/').replace('\\', "/");
    let rest = p.strip_prefix("src/").ok_or_else(|| {
        Error::Invalid(format!(
            "the components path `{path}` must be inside src/ (for example src/oxyn)"
        ))
    })?;
    let parts: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() || !parts.iter().all(|s| valid_ident(s)) {
        return Err(Error::Invalid(format!(
            "`{path}` does not name a Rust module path (use lowercase names, like src/oxyn)"
        )));
    }
    Ok(format!("crate::{}", parts.join("::")))
}

fn valid_ident(s: &str) -> bool {
    let mut c = s.chars();
    c.next().is_some_and(|f| f.is_ascii_lowercase() || f == '_')
        && c.all(|x| x.is_ascii_lowercase() || x.is_ascii_digit() || x == '_')
}

/// `oxyn.lock`: the hash of every file the CLI wrote, by path relative to the project.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    /// Path → content hash at the time the CLI wrote it.
    pub files: BTreeMap<String, String>,
}

impl Lock {
    /// Reads the lock (empty when missing).
    pub fn load(dir: &Path) -> Lock {
        fs::read_to_string(dir.join(LOCK_FILE))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    /// Writes the lock.
    pub fn save(&self, dir: &Path) -> Result<(), Error> {
        let s = serde_json::to_string_pretty(self).map_err(|e| Error::Invalid(e.to_string()))?;
        write(&dir.join(LOCK_FILE), &(s + "\n"))
    }
}

/// FNV-1a 64-bit hex of a file's content (change detection only).
pub fn hash(s: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// Writes a file, creating its directory.
pub fn write(path: &Path, content: &str) -> Result<(), Error> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| Error::Io(dir.to_path_buf(), e))?;
    }
    fs::write(path, content).map_err(|e| Error::Io(path.to_path_buf(), e))
}

/// The nearest directory at or above `start` with a `Cargo.toml`.
pub fn find_project(start: &Path) -> Option<PathBuf> {
    let mut d = Some(start);
    while let Some(dir) = d {
        if dir.join("Cargo.toml").is_file() {
            return Some(dir.to_path_buf());
        }
        d = dir.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_paths() {
        assert_eq!(module_path_of("src/oxyn").unwrap(), "crate::oxyn");
        assert_eq!(module_path_of("src/ui/kit/").unwrap(), "crate::ui::kit");
        assert!(module_path_of("oxyn").is_err());
        assert!(module_path_of("src/").is_err());
        assert!(module_path_of("src/My-Kit").is_err());
    }

    #[test]
    fn config_round_trips() {
        let d = tempfile::tempdir().unwrap();
        let mut c = Config::new(
            Framework::Dioxus,
            Source::git("https://g", Some("abc".into())),
        );
        c.mark_installed("button");
        c.mark_installed("badge");
        c.mark_installed("button");
        c.save(d.path()).unwrap();
        let back = Config::load(d.path()).unwrap();
        assert_eq!(back, c);
        assert_eq!(
            back.installed,
            vec!["badge".to_string(), "button".to_string()]
        );
        let text = fs::read_to_string(d.path().join(CONFIG_FILE)).unwrap();
        assert!(text.contains("framework = \"dioxus\""));
        // The default source is crates.io; an oxyn.toml from before it (git + rev) still reads as git.
        let mut c = Config::new(Framework::Dioxus, Source::crates_io());
        c.save(d.path()).unwrap();
        let text = fs::read_to_string(d.path().join(CONFIG_FILE)).unwrap();
        assert!(text.contains("[oxyn]\nversion = \"0.1\""), "{text}");
        assert_eq!(Config::load(d.path()).unwrap().oxyn, Source::crates_io());
        let old: Config = toml::from_str(
            "framework = \"gpui\"\npath = \"src/oxyn\"\ncss = \"assets/oxyn\"\n[oxyn]\ngit = \"https://g\"\nrev = \"abc\"\n",
        )
        .unwrap();
        assert_eq!(old.oxyn, Source::git("https://g", Some("abc".into())));
        c = Config::new(Framework::Dioxus, Source::default());
        assert_eq!(c.oxyn.version, None);
        assert!(matches!(
            Config::load(&d.path().join("nope")),
            Err(Error::NotInitialised(_))
        ));
    }

    #[test]
    fn lock_and_project_lookup() {
        let d = tempfile::tempdir().unwrap();
        let mut l = Lock::default();
        l.files.insert("src/oxyn/mod.rs".into(), hash("x"));
        l.save(d.path()).unwrap();
        assert_eq!(Lock::load(d.path()), l);
        fs::write(d.path().join("Cargo.toml"), "[package]").unwrap();
        fs::create_dir_all(d.path().join("src/a")).unwrap();
        assert_eq!(find_project(&d.path().join("src/a")).unwrap(), d.path());
        assert_ne!(hash("a"), hash("b"));
    }
}
