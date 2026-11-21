//! `oxyn`: add Oxyn components to a Dioxus or GPUI project, shadcn style. The code is copied into
//! your project (`src/oxyn/`), so it is yours to change. See https://ui.oxyn.workers.dev/docs/cli/.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use oxyn_cli::install::{self, FileDiff};
use oxyn_cli::{Client, Config, Error, PRICING_URL, Source, registry_base, resolve};
use oxyn_registry_schema::{Framework, IndexEntry, Tier};

#[derive(Parser)]
#[command(name = "oxyn", version, about = "Add Oxyn UI components (Dioxus, GPUI) to your Rust project.", long_about = None)]
struct Cli {
    /// Registry base URL or directory (default https://ui.oxyn.workers.dev/r; env OXYN_REGISTRY).
    #[arg(long, global = true, value_name = "URL")]
    registry: Option<String>,
    /// Project directory (default: the nearest Cargo.toml at or above the current directory).
    #[arg(long, short = 'C', global = true, value_name = "DIR")]
    cwd: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Set up this project: oxyn.toml, src/oxyn/, `mod oxyn;` and the oxyn-core dependency.
    Init {
        /// dioxus or gpui (detected from Cargo.toml when there is exactly one).
        #[arg(long, short)]
        framework: Option<Framework>,
        /// Where components go, inside src/ (default src/oxyn).
        #[arg(long)]
        path: Option<String>,
        /// Use a local checkout of the Oxyn repository for oxyn-core instead of git.
        #[arg(long, value_name = "DIR")]
        oxyn_path: Option<String>,
        /// Write oxyn.toml again even if it exists.
        #[arg(long)]
        force: bool,
    },
    /// List the components (free and Pro).
    List {
        /// Only this category (chat, inputs, agents, ...).
        #[arg(long)]
        category: Option<String>,
        /// Only free or only Pro items.
        #[arg(long)]
        tier: Option<Tier>,
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Search names, categories and descriptions.
    Search {
        /// Words to look for.
        #[arg(required = true)]
        query: Vec<String>,
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Show one item: what it is, its files and dependencies.
    View {
        /// The item.
        slug: String,
        /// JSON output (the full registry item for free items).
        #[arg(long)]
        json: bool,
    },
    /// Add components (and the components they need) to this project.
    Add {
        /// One or more items.
        #[arg(required = true)]
        slugs: Vec<String>,
        /// Override the framework from oxyn.toml.
        #[arg(long, short)]
        framework: Option<Framework>,
        /// Override the components directory from oxyn.toml (inside src/).
        #[arg(long)]
        path: Option<String>,
        /// Replace files that were changed locally.
        #[arg(long)]
        overwrite: bool,
    },
    /// Compare an installed item with the registry.
    Diff {
        /// The item.
        slug: String,
    },
}

fn main() -> ExitCode {
    quiet_broken_pipe();
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(e.exit_code() as u8)
        }
    }
}

/// `oxyn list | head` closes stdout early; end quietly instead of panicking in `println!`.
fn quiet_broken_pipe() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info
            .payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("");
        if msg.contains("Broken pipe") {
            std::process::exit(0);
        }
        default(info);
    }));
}

fn project_dir(cwd: &Option<PathBuf>) -> Result<PathBuf, Error> {
    let start = match cwd {
        Some(d) => d.clone(),
        None => std::env::current_dir().map_err(|e| Error::Io(PathBuf::from("."), e))?,
    };
    oxyn_cli::config::find_project(&start).ok_or(Error::NoCargoProject(start))
}

fn json_out<T: serde::Serialize>(v: &T) -> Result<(), Error> {
    println!(
        "{}",
        serde_json::to_string_pretty(v).map_err(|e| Error::Invalid(e.to_string()))?
    );
    Ok(())
}

fn table(rows: &[&IndexEntry]) {
    let w = rows.iter().map(|e| e.slug.len()).max().unwrap_or(4).max(4);
    let c = rows
        .iter()
        .map(|e| e.category.len())
        .max()
        .unwrap_or(8)
        .max(8);
    println!("{:<w$}  {:<4}  {:<c$}  NAME", "SLUG", "TIER", "CATEGORY");
    for e in rows {
        let tier = if e.tier == Tier::Pro { "pro" } else { "free" };
        println!(
            "{:<w$}  {:<4}  {:<c$}  {}",
            e.slug, tier, e.category, e.name
        );
    }
}

fn run(cli: Cli) -> Result<(), Error> {
    // The config (when there is one) can name a registry; the flag and env win.
    let proj = project_dir(&cli.cwd).ok();
    let cfg = proj.as_ref().and_then(|p| Config::load(p).ok());
    let base = registry_base(
        cli.registry.as_deref(),
        cfg.as_ref().and_then(|c| c.registry.as_deref()),
    );
    let client = Client::new(&base);
    match cli.cmd {
        Cmd::Init {
            framework,
            path,
            oxyn_path,
            force,
        } => {
            let p = project_dir(&cli.cwd)?;
            let source = match oxyn_path {
                Some(dir) => {
                    let abs = std::fs::canonicalize(&dir)
                        .map_err(|e| Error::Io(PathBuf::from(&dir), e))?;
                    Source {
                        git: None,
                        rev: None,
                        path: Some(abs.to_string_lossy().into_owned()),
                    }
                }
                None => {
                    let idx = client.index()?;
                    Source {
                        git: Some(idx.source.git),
                        rev: Some(idx.source.rev),
                        path: None,
                    }
                }
            };
            let rep = install::init(&p, framework, path.as_deref(), source, force)?;
            for c in rep.changes {
                println!("  {c}");
            }
            println!("Ready. Add a component: oxyn add button");
        }
        Cmd::List {
            category,
            tier,
            json,
        } => {
            let idx = client.index()?;
            let rows: Vec<&IndexEntry> = idx
                .items
                .iter()
                .filter(|e| e.slug != "theme")
                .filter(|e| {
                    category
                        .as_ref()
                        .is_none_or(|c| e.category.eq_ignore_ascii_case(c))
                })
                .filter(|e| tier.is_none_or(|t| e.tier == t))
                .collect();
            if json {
                return json_out(&rows);
            }
            if rows.is_empty() {
                println!("nothing matches");
                return Ok(());
            }
            table(&rows);
            let free = rows.iter().filter(|e| e.tier == Tier::Free).count();
            println!(
                "\n{} items: {free} free, {} Pro ({PRICING_URL}/). Add one: oxyn add <slug>",
                rows.len(),
                rows.len() - free
            );
        }
        Cmd::Search { query, json } => {
            let idx = client.index()?;
            let q = query.join(" ");
            let hits: Vec<&IndexEntry> = resolve::search(&idx, &q)
                .into_iter()
                .filter(|e| e.slug != "theme")
                .collect();
            if json {
                return json_out(&hits);
            }
            if hits.is_empty() {
                println!("no items match `{q}`");
                return Ok(());
            }
            table(&hits);
        }
        Cmd::View { slug, json } => {
            let idx = client.index()?;
            let e = idx
                .get(&slug)
                .ok_or_else(|| Error::UnknownItem(slug.clone(), resolve::suggest(&idx, &slug)))?;
            let item = if e.installable {
                client.item(&slug)?
            } else {
                None
            };
            if json {
                return match item {
                    Some(it) => json_out(&it),
                    None => json_out(e),
                };
            }
            println!("{} ({})", e.name, e.slug);
            println!(
                "  {} · {} · {}",
                e.kind,
                e.category,
                if e.tier == Tier::Pro { "Pro" } else { "Free" }
            );
            if !e.description.is_empty() {
                println!("  {}", e.description);
            }
            if !e.url.is_empty() {
                println!("  {}", e.url);
            }
            match item {
                Some(it) => {
                    for b in &it.frameworks {
                        println!(
                            "\n  {}: {} (module {})",
                            b.framework,
                            b.entry,
                            if b.module.is_empty() { "-" } else { &b.module }
                        );
                        let deps: Vec<&String> = b.oxyn_deps.iter().collect();
                        if !deps.is_empty() {
                            println!(
                                "    needs: {}",
                                deps.iter()
                                    .map(|s| s.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                        }
                        let crates: Vec<&str> =
                            b.cargo_deps.iter().map(|d| d.name.as_str()).collect();
                        if !crates.is_empty() {
                            println!("    crates: {}", crates.join(", "));
                        }
                        for f in &b.files {
                            let size = if f.chunks.is_empty() {
                                f.content.len()
                            } else {
                                f.chunks
                                    .iter()
                                    .filter(|c| c.requires.is_none())
                                    .map(|c| c.code.len())
                                    .sum()
                            };
                            println!(
                                "    {:<48} {}",
                                f.target,
                                if f.url.is_some() {
                                    "shared".to_string()
                                } else {
                                    format!("{size} B")
                                }
                            );
                        }
                    }
                    println!("\n  oxyn add {slug}");
                }
                None if e.tier == Tier::Pro => println!(
                    "\n  Part of Oxyn Pro: the preview is free on the site, the code needs Pro ({PRICING_URL}/)."
                ),
                None => println!(
                    "\n  Not installable: {}",
                    e.note.clone().unwrap_or_default()
                ),
            }
        }
        Cmd::Add {
            slugs,
            framework,
            path,
            overwrite,
        } => {
            let p = project_dir(&cli.cwd)?;
            let mut cfg = Config::load(&p)?;
            if let Some(f) = framework {
                cfg.framework = f;
            }
            if let Some(path) = path {
                cfg.path = path.trim_end_matches('/').to_string();
            }
            let idx = client.index()?;
            let rep = install::add(&p, &mut cfg, &client, &idx, &slugs, overwrite)?;
            println!("added {}", rep.items.join(", "));
            for f in &rep.files.written {
                println!("  wrote {f}");
            }
            if !rep.files.unchanged.is_empty() {
                println!("  {} files already up to date", rep.files.unchanged.len());
            }
            for f in &rep.files.skipped {
                println!(
                    "  kept   {f} (changed locally; `oxyn diff` shows the difference, --overwrite replaces it)"
                );
            }
            for c in &rep.cargo {
                println!("  added {c} to Cargo.toml");
            }
            if !rep.next.is_empty() {
                println!("\nNext:");
                for n in &rep.next {
                    println!("  {n}");
                }
            }
        }
        Cmd::Diff { slug } => {
            let p = project_dir(&cli.cwd)?;
            let cfg = Config::load(&p)?;
            let idx = client.index()?;
            let diffs = install::diff(&p, &cfg, &client, &idx, &slug)?;
            let mut changed = 0;
            for d in diffs {
                match d {
                    FileDiff::Same(f) => println!("  same     {f}"),
                    FileDiff::Missing(f) => {
                        changed += 1;
                        println!("  missing  {f}");
                    }
                    FileDiff::Changed(_, u) => {
                        changed += 1;
                        print!("{u}");
                    }
                }
            }
            if changed == 0 {
                println!("{slug} matches the registry");
            }
        }
    }
    Ok(())
}
