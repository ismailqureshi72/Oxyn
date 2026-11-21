//! Builds the static registry.
//!
//! ```text
//! oxyn-registry --input site/src/generated/registry-input.json --out site/public/r \
//!     [--pro-out site/public/_pro/r] [--root .] [--rev <commit>] [--homepage https://oxynui.com]
//! ```
//!
//! Writes `index.json` (every item, free and Pro, without file contents) and `<slug>.json` for
//! each installable free item. With `--pro-out`, also `<slug>.json` for each installable Pro item
//! there: the Worker-only Pro registry (served by `GET /api/r/<slug>.json` to a signed-in plan, see
//! site/worker/pro.ts). Run by `site/scripts/gen.mjs` before the site build.

use std::path::PathBuf;
use std::process::{Command, ExitCode};

use oxyn_codegen::registry::{self, BuildConfig, MetaItem};

const USAGE: &str = "usage: oxyn-registry --input <registry-input.json> --out <dir> [--pro-out <dir>] [--root <repo>] [--rev <commit>] [--homepage <url>] [--git <url>] [--quiet]";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("oxyn-registry: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut input = None;
    let mut out = None;
    let mut pro_out: Option<PathBuf> = None;
    let mut root = PathBuf::from(".");
    let mut rev = std::env::var("GITHUB_SHA").ok();
    let mut homepage = "https://oxynui.com".to_string();
    let mut git = "https://github.com/ismailqureshi72/Oxyn".to_string();
    let mut quiet = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut val = || {
            args.next()
                .ok_or_else(|| format!("{a} needs a value\n{USAGE}"))
        };
        match a.as_str() {
            "--input" => input = Some(PathBuf::from(val()?)),
            "--out" => out = Some(PathBuf::from(val()?)),
            "--pro-out" => pro_out = Some(PathBuf::from(val()?)),
            "--root" => root = PathBuf::from(val()?),
            "--rev" => rev = Some(val()?),
            "--homepage" => homepage = val()?.trim_end_matches('/').to_string(),
            "--git" => git = val()?,
            "--quiet" => quiet = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            _ => return Err(format!("unknown argument {a}\n{USAGE}")),
        }
    }
    let input = input.ok_or(USAGE)?;
    let out = out.ok_or(USAGE)?;
    let rev = match rev {
        Some(r) => r,
        None => {
            let o = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&root)
                .output()
                .map_err(|e| format!("git rev-parse: {e}"))?;
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        }
    };
    if rev.len() != 40 || !rev.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("--rev must be a full commit hash, got `{rev}`"));
    }
    let raw = std::fs::read_to_string(&input).map_err(|e| format!("{}: {e}", input.display()))?;
    let meta: Vec<MetaItem> =
        serde_json::from_str(&raw).map_err(|e| format!("{}: {e}", input.display()))?;
    let cfg = BuildConfig {
        repo_root: root,
        homepage,
        git,
        rev,
    };
    let reg = registry::build(&meta, &cfg)?;
    let (n, bytes) = registry::write(&reg, &out)?;
    let pro_written = match &pro_out {
        Some(p) => Some(registry::write_pro(&reg, p, &out)?),
        None => None,
    };
    let total = reg.index.items.len();
    let pro = reg
        .index
        .items
        .iter()
        .filter(|e| e.tier == oxyn_registry_schema::Tier::Pro)
        .count();
    let blocked = reg
        .index
        .items
        .iter()
        .filter(|e| e.tier == oxyn_registry_schema::Tier::Free && !e.installable)
        .count();
    println!(
        "registry: {total} items ({pro} pro), {n} free item JSONs ({:.1} MiB), {blocked} free items not installable -> {}",
        bytes as f64 / 1048576.0,
        out.display()
    );
    if let (Some((np, pb)), Some(p)) = (pro_written, &pro_out) {
        println!(
            "registry: {np} Pro item JSONs ({:.1} MiB, Worker-only) -> {}",
            pb as f64 / 1048576.0,
            p.display()
        );
    }
    if !quiet {
        for line in &reg.report {
            println!("  note: {line}");
        }
    }
    Ok(())
}
