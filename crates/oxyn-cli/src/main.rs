//! `oxyn`: add Oxyn components to a Dioxus or GPUI project, shadcn style. The code is copied into
//! your project (`src/oxyn/`), so it is yours to change. See https://oxynui.com/docs/cli/.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use oxyn_cli::install::{self, FileDiff};
use oxyn_cli::{Client, Config, Error, PRICING_URL, Source, auth, registry_base, resolve};
use oxyn_registry_schema::{Framework, IndexEntry, Tier};

#[derive(Parser)]
#[command(name = "oxyn", version, about = "Add Oxyn UI components (Dioxus, GPUI) to your Rust project.", long_about = None)]
struct Cli {
    /// Registry base URL or directory (default https://oxynui.com/r; env OXYN_REGISTRY).
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
    /// Set up this project: oxyn.toml, src/oxyn/, `mod oxyn;` and the oxyn-core dependency
    /// (from crates.io unless --oxyn-path or --oxyn-git).
    Init {
        /// dioxus or gpui (detected from Cargo.toml when there is exactly one).
        #[arg(long, short)]
        framework: Option<Framework>,
        /// Where components go, inside src/ (default src/oxyn).
        #[arg(long)]
        path: Option<String>,
        /// Use a local checkout of the Oxyn repository for oxyn-core instead of crates.io.
        #[arg(long, value_name = "DIR", conflicts_with = "oxyn_git")]
        oxyn_path: Option<String>,
        /// Use the Oxyn git repository, pinned to the commit the registry was built from, instead of
        /// crates.io (oxyn-core = "0.1" by default).
        #[arg(long)]
        oxyn_git: bool,
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
    /// Sign in for Pro items: approve this computer in the browser, or save a token.
    Login {
        /// A token from your account page (https://oxynui.com/account/#cli) instead of
        /// the browser.
        #[arg(long, value_name = "TOKEN")]
        token: Option<String>,
        /// Print the sign-in link instead of opening the browser.
        #[arg(long)]
        no_browser: bool,
    },
    /// Forget the saved token on this computer (revoke it on your account page).
    Logout,
    /// Show the signed-in account, its plan and this month's Pro items.
    Whoami {
        /// JSON output.
        #[arg(long)]
        json: bool,
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
            oxyn_git,
            force,
        } => {
            let p = project_dir(&cli.cwd)?;
            let source = match oxyn_path {
                Some(dir) => {
                    let abs = std::fs::canonicalize(&dir)
                        .map_err(|e| Error::Io(PathBuf::from(&dir), e))?;
                    Source::local(abs.to_string_lossy().into_owned())
                }
                None if oxyn_git => {
                    let idx = client.index()?;
                    Source::git(idx.source.git, Some(idx.source.rev))
                }
                None => Source::crates_io(),
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
                client.item(&e.slug)?
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
                None if e.tier == Tier::Pro && e.installable => println!(
                    "\n  Part of Oxyn Pro ({PRICING_URL}/). With a plan: oxyn login, then oxyn add {slug}\n  (each new Pro item counts toward this month's; `oxyn whoami` shows the count)."
                ),
                None if e.tier == Tier::Pro => println!(
                    "\n  Part of Oxyn Pro, with no CLI build yet: the code is on its page ({}).",
                    e.url
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
            if let Some(u) = client.last_usage() {
                let n = client.counted().len();
                println!(
                    "  Pro this month: {u}{}",
                    if n > 0 {
                        format!(" ({n} new)")
                    } else {
                        String::new()
                    }
                );
            }
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
        Cmd::Login { token, no_browser } => login(&client, token, no_browser)?,
        Cmd::Logout => {
            if auth::remove()? {
                println!(
                    "Signed out on this computer. The token still works until you revoke it on your account page (https://oxynui.com/account/#cli)."
                );
            } else {
                println!("No saved token on this computer.");
            }
            if auth::token_source() == "OXYN_TOKEN" {
                println!("OXYN_TOKEN is still set in this shell.");
            }
        }
        Cmd::Whoami { json } => {
            let v = auth::whoami(&client)?;
            if json {
                return json_out(&v);
            }
            let plan = if v["admin"].as_bool() == Some(true) {
                "Admin (everything, no limits)".to_string()
            } else {
                match v["tier"].as_str().unwrap_or("free") {
                    "team" => "Team".to_string(),
                    "pro" => "Pro".to_string(),
                    _ => "Free".to_string(),
                }
            };
            println!("{}", v["email"].as_str().unwrap_or("?"));
            println!("  plan   {plan}");
            if let Some(u) = auth::usage_line(&v) {
                println!("  usage  {u}");
            }
            println!("  token  from {}", auth::token_source());
            if v["pro"].as_bool() != Some(true) {
                println!("\nPro items need a plan: {PRICING_URL}/");
            }
        }
    }
    Ok(())
}

fn login(client: &Client, token: Option<String>, no_browser: bool) -> Result<(), Error> {
    let (token, email) = match token {
        Some(t) => {
            let t = t.trim().to_string();
            if !t.starts_with("oxyn_") {
                return Err(Error::Invalid(
                    "that is not an Oxyn token (they start with oxyn_): make one on your account page".into(),
                ));
            }
            let check = Client::new(&client.describe())
                .with_api(client.api().map(String::from))
                .with_token(Some(t.clone()));
            let v = auth::whoami(&check)?;
            (t, v["email"].as_str().map(String::from))
        }
        None => {
            let d = auth::start_login(client, &auth::host_name())?;
            println!(
                "Sign in to Oxyn in your browser and approve this code: {}",
                d.user_code
            );
            if no_browser || !auth::open_browser(&d.url) {
                println!("Open: {}", d.url);
            } else {
                println!("Opened {} (if nothing opened, visit it yourself)", d.url);
            }
            println!("Waiting for approval…");
            auth::wait_login(client, &d)?
        }
    };
    let path = auth::save(&auth::Credentials {
        token: token.clone(),
        email: email.clone(),
        api: client.api().map(String::from),
    })?;
    println!(
        "Signed in{}. Token saved to {}",
        email.map(|e| format!(" as {e}")).unwrap_or_default(),
        path.display()
    );
    let after = Client::new(&client.describe())
        .with_api(client.api().map(String::from))
        .with_token(Some(token));
    if let Ok(v) = auth::whoami(&after)
        && let Some(u) = auth::usage_line(&v)
    {
        println!("  {u}");
    }
    Ok(())
}
