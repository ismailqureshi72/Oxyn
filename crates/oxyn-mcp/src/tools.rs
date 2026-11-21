//! The tools: what an agent can ask the Oxyn registry, and how it installs components.
//!
//! Results are compact JSON in one text block (`isError` set when the tool could not do it), so a
//! model reads them cheaply and a client shows them as they are.

use std::path::{Path, PathBuf};

use oxyn_cli::config::{self, Config, Source};
use oxyn_cli::install::{self, Planned};
use oxyn_cli::{Error, PRICING_URL, resolve};
use oxyn_registry_schema::{FileKind, Framework, IndexEntry, RegistryIndex, RegistryItem, Tier};
use serde_json::{Map, Value, json};

use crate::Server;
use crate::props;

/// Where the MCP docs live.
pub const DOCS_URL: &str = "https://ui.oxyn.workers.dev/docs/mcp/";
/// How to install the CLI (the same repository as the server).
pub const CLI_INSTALL: &str =
    "cargo install --git https://github.com/ismailqureshi72/Oxyn oxyn-cli";

const LIST_LIMIT: u64 = 100;
const SEARCH_LIMIT: u64 = 15;
/// Bytes of file contents `get_component` returns before it lists the rest by path.
const CODE_BUDGET: usize = 40_000;

fn schema(props: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": props, "required": required, "additionalProperties": false })
}

fn annotations(title: &str, read_only: bool) -> Value {
    json!({
        "title": title,
        "readOnlyHint": read_only,
        "destructiveHint": false,
        "idempotentHint": true,
        "openWorldHint": true
    })
}

/// The tool definitions for `tools/list`.
pub fn list() -> Vec<Value> {
    let fw = json!({ "type": "string", "enum": ["dioxus", "gpui"] });
    vec![
        json!({
            "name": "list_components",
            "title": "List Oxyn components",
            "description": "List Oxyn UI components (free and Pro) with optional filters. Returns slug, name, tier, category, kind and frameworks; use get_component for details.",
            "inputSchema": schema(json!({
                "category": { "type": "string", "description": "Only this category, e.g. chat, inputs, agents, display (the result lists all categories)." },
                "tier": { "type": "string", "enum": ["free", "pro"] },
                "framework": fw.clone(),
                "limit": { "type": "integer", "minimum": 1, "maximum": 500, "description": "Default 100." },
                "offset": { "type": "integer", "minimum": 0 }
            }), &[]),
            "annotations": annotations("List Oxyn components", true)
        }),
        json!({
            "name": "search_components",
            "title": "Search Oxyn components",
            "description": "Search Oxyn components by words in their name, slug, category and description, best matches first.",
            "inputSchema": schema(json!({
                "query": { "type": "string", "description": "Words to look for, e.g. \"approval\" or \"file upload\"." },
                "limit": { "type": "integer", "minimum": 1, "maximum": 100, "description": "Default 15." }
            }), &["query"]),
            "annotations": annotations("Search Oxyn components", true)
        }),
        json!({
            "name": "get_component",
            "title": "Get an Oxyn component",
            "description": "One component: metadata, props (Dioxus) or builder API (GPUI), the components and crates it needs, the install command, its docs URL, and for free items the source files exactly as add_component writes them.",
            "inputSchema": schema(json!({
                "slug": { "type": "string", "description": "The component, e.g. composer." },
                "framework": { "type": "string", "enum": ["dioxus", "gpui"], "description": "Which build's files to include (default dioxus when it has one)." },
                "include_code": { "type": "boolean", "description": "Include file contents (default true; about 40 KB at most, the rest listed by path). Pro items: only when true is passed, since fetching a Pro item counts toward the month's Pro items (re-downloads are free)." },
                "file": { "type": "string", "description": "Return only this file's contents (a path from an earlier result, e.g. src/oxyn/composer/mod.rs)." }
            }), &["slug"]),
            "annotations": annotations("Get an Oxyn component", true)
        }),
        json!({
            "name": "add_component",
            "title": "Add Oxyn components to a project",
            "description": "Copy components (and the components they need) into a Rust project's src/oxyn/, add the crates they need to Cargo.toml and declare `mod oxyn;`. Sets the project up (oxyn.toml) on first use. Files changed locally are kept unless overwrite is true. Pro items return a Pro message instead.",
            "inputSchema": schema(json!({
                "slugs": { "type": "array", "items": { "type": "string" }, "minItems": 1, "description": "Components to add, e.g. [\"composer\", \"button\"]." },
                "project_path": { "type": "string", "description": "Directory of the project (the one with Cargo.toml, or inside it). Absolute is best; default: the server's working directory." },
                "framework": { "type": "string", "enum": ["dioxus", "gpui"], "description": "Detected from Cargo.toml when the project depends on exactly one." },
                "path": { "type": "string", "description": "Components folder inside src/ (default src/oxyn)." },
                "overwrite": { "type": "boolean", "description": "Replace files that were changed locally (default false)." }
            }), &["slugs"]),
            "annotations": annotations("Add Oxyn components", false)
        }),
        json!({
            "name": "get_install_instructions",
            "title": "Oxyn install instructions",
            "description": "How to set up Oxyn in a Dioxus or GPUI project: dependencies, the oxyn CLI, styles and theme.",
            "inputSchema": schema(json!({ "framework": fw }), &[]),
            "annotations": annotations("Oxyn install instructions", true)
        }),
    ]
}

/// Runs `tools/call`. `Err` only for a malformed call (unknown tool, missing name): a tool that
/// fails returns `Ok` with `isError: true`.
pub fn call(server: &Server, params: &Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or("tools/call needs a tool name")?;
    let empty = Value::Object(Map::new());
    let args = params.get("arguments").unwrap_or(&empty);
    let out = match name {
        "list_components" => list_components(server, args),
        "search_components" => search_components(server, args),
        "get_component" => get_component(server, args),
        "add_component" => add_component(server, args),
        "get_install_instructions" => install_instructions(args).map(Value::String),
        other => return Err(format!("unknown tool: {other}")),
    };
    Ok(match out {
        Ok(Value::String(text)) => text_result(text, false),
        Ok(v) => text_result(v.to_string(), false),
        Err(e) => text_result(e.to_string(), true),
    })
}

fn text_result(text: String, is_error: bool) -> Value {
    json!({ "content": [{ "type": "text", "text": text }], "isError": is_error })
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn arg_u64(args: &Value, key: &str, default: u64, max: u64) -> u64 {
    args.get(key)
        .and_then(Value::as_u64)
        .unwrap_or(default)
        .min(max)
}

fn arg_bool(args: &Value, key: &str, default: bool) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn arg_framework(args: &Value) -> Result<Option<Framework>, Error> {
    arg_str(args, "framework")
        .map(|s| s.parse::<Framework>().map_err(Error::Invalid))
        .transpose()
}

fn short(e: &IndexEntry) -> Value {
    let mut v = json!({
        "slug": e.slug,
        "name": e.name,
        "tier": e.tier,
        "category": e.category,
        "kind": e.kind,
        "frameworks": e.frameworks,
    });
    if e.tier == Tier::Free && !e.installable {
        v["installable"] = json!(false);
    }
    v
}

fn visible(idx: &RegistryIndex) -> impl Iterator<Item = &IndexEntry> {
    // `theme` is the tokens stylesheet the CLI installs on its own, not a component.
    idx.items.iter().filter(|e| e.slug != "theme")
}

fn list_components(server: &Server, args: &Value) -> Result<Value, Error> {
    let idx = server.client.index()?;
    let tier = arg_str(args, "tier")
        .map(|s| s.parse::<Tier>().map_err(Error::Invalid))
        .transpose()?;
    let fw = arg_framework(args)?;
    let category = arg_str(args, "category");
    let rows: Vec<&IndexEntry> = visible(&idx)
        .filter(|e| category.is_none_or(|c| e.category.eq_ignore_ascii_case(c)))
        .filter(|e| tier.is_none_or(|t| e.tier == t))
        .filter(|e| fw.is_none_or(|f| e.frameworks.contains(&f)))
        .collect();
    let limit = arg_u64(args, "limit", LIST_LIMIT, 500) as usize;
    let offset = arg_u64(args, "offset", 0, u64::MAX) as usize;
    let page: Vec<Value> = rows
        .iter()
        .skip(offset)
        .take(limit)
        .map(|e| short(e))
        .collect();
    let mut cats: Vec<&str> = visible(&idx).map(|e| e.category.as_str()).collect();
    cats.sort_unstable();
    cats.dedup();
    let mut out = json!({
        "total": rows.len(),
        "free": rows.iter().filter(|e| e.tier == Tier::Free).count(),
        "items": page,
        "categories": cats,
        "pro": format!("Pro items need Oxyn Pro: {PRICING_URL}"),
    });
    if offset + limit < rows.len() {
        out["next_offset"] = json!(offset + limit);
    }
    Ok(out)
}

fn search_components(server: &Server, args: &Value) -> Result<Value, Error> {
    let q = arg_str(args, "query").ok_or_else(|| Error::Invalid("query is required".into()))?;
    let idx = server.client.index()?;
    let limit = arg_u64(args, "limit", SEARCH_LIMIT, 100) as usize;
    let hits: Vec<&IndexEntry> = resolve::search(&idx, q)
        .into_iter()
        .filter(|e| e.slug != "theme")
        .collect();
    let items: Vec<Value> = hits
        .iter()
        .take(limit)
        .map(|e| {
            let mut v = short(e);
            v["description"] = json!(clip(&e.description, 160));
            v
        })
        .collect();
    Ok(json!({ "query": q, "total": hits.len(), "items": items }))
}

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let mut t: String = s.chars().take(n - 1).collect();
    t.push('…');
    t
}

fn entry<'a>(idx: &'a RegistryIndex, slug: &str) -> Result<&'a IndexEntry, Error> {
    idx.get(slug)
        .ok_or_else(|| Error::UnknownItem(slug.to_string(), resolve::suggest(idx, slug)))
}

fn build_json(b: &oxyn_registry_schema::FrameworkBuild) -> Value {
    let mut v = json!({
        "framework": b.framework,
        "entry": b.entry,
        "module": b.module,
    });
    if !b.oxyn_deps.is_empty() {
        v["needs"] = json!(b.oxyn_deps);
    }
    let crates: Vec<&str> = b.cargo_deps.iter().map(|d| d.name.as_str()).collect();
    if !crates.is_empty() {
        v["crates"] = json!(crates);
    }
    if !b.css.is_empty() {
        v["css"] = json!(b.css);
    }
    let chunks = || {
        b.files
            .iter()
            .filter(|f| f.kind == FileKind::Rust)
            .flat_map(|f| f.chunks.iter().map(|c| c.code.as_str()))
    };
    if b.entry.is_empty() {
        return v;
    }
    match b.framework {
        Framework::Dioxus => {
            if let Some(p) = props::dioxus_props(chunks(), &b.entry) {
                v["props"] = Value::Array(p.iter().map(props::Prop::to_json).collect());
            }
        }
        Framework::Gpui => {
            if let Some((ctors, methods)) = props::gpui_builder(chunks(), &b.entry) {
                if !ctors.is_empty() {
                    v["constructors"] = json!(ctors);
                }
                if !methods.is_empty() {
                    v["builder"] = Value::Array(methods.iter().map(props::Prop::to_json).collect());
                }
            }
        }
    }
    v
}

/// The item's own files for one framework, as `add_component` would write them with nothing else
/// installed. Stylesheets shared by many items are counted, not inlined.
fn own_files(
    server: &Server,
    item: &RegistryItem,
    fw: Framework,
    only: Option<&str>,
) -> Result<Value, Error> {
    let mut solo = item.clone();
    let mut shared = 0;
    for b in &mut solo.frameworks {
        b.files.retain(|f| {
            let keep = f.kind == FileKind::Rust || f.url.is_none();
            if !keep && b.framework == fw {
                shared += 1;
            }
            keep
        });
    }
    let cfg = Config::new(
        fw,
        Source {
            git: None,
            rev: None,
            path: None,
        },
    );
    let planned: Vec<Planned> = install::compose(&[solo], fw, &cfg, &server.client)?;
    // The entry's own file first, then the rest; past CODE_BUDGET only paths and sizes.
    let module = item
        .build(fw)
        .map(|b| b.module.replace("::", "/"))
        .unwrap_or_default();
    let is_entry = |p: &str| {
        !module.is_empty()
            && (p.ends_with(&format!("/{module}.rs")) || p.ends_with(&format!("/{module}/mod.rs")))
    };
    let mut planned = planned;
    planned.sort_by_key(|p| !is_entry(&p.path));
    if let Some(want) = only {
        let want = want.trim_start_matches("./");
        let hit = planned
            .into_iter()
            .find(|p| p.path == want || p.path.ends_with(&format!("/{want}")))
            .ok_or_else(|| Error::Invalid(format!("`{}` has no file `{want}`", item.slug)))?;
        return Ok(
            json!({ "framework": fw, "files": [{ "path": hit.path, "content": hit.content }] }),
        );
    }
    let mut used = 0;
    let mut omitted = 0;
    let files: Vec<Value> = planned
        .into_iter()
        .map(|p| {
            if used + p.content.len() <= CODE_BUDGET {
                used += p.content.len();
                json!({ "path": p.path, "content": p.content })
            } else {
                omitted += 1;
                json!({ "path": p.path, "bytes": p.content.len() })
            }
        })
        .collect();
    let mut out = json!({ "framework": fw, "files": files, "shared_stylesheets": shared });
    if omitted > 0 {
        out["omitted"] = json!(format!(
            "{omitted} files left out to keep this short: call get_component again with file set to a path to read one"
        ));
    }
    Ok(out)
}

fn get_component(server: &Server, args: &Value) -> Result<Value, Error> {
    let slug = arg_str(args, "slug").ok_or_else(|| Error::Invalid("slug is required".into()))?;
    let idx = server.client.index()?;
    let e = entry(&idx, slug)?;
    let mut out = json!({
        "slug": e.slug,
        "name": e.name,
        "kind": e.kind,
        "category": e.category,
        "tier": e.tier,
        "description": e.description,
        "docs": e.url,
        "frameworks": e.frameworks,
    });
    let item = if e.tier == Tier::Pro {
        // The authenticated Pro registry: each new Pro item counts toward the month's, so the
        // code is fetched only when it is asked for (include_code: true).
        let needs = || -> Option<Value> {
            let deps: Map<String, Value> = e
                .oxyn_deps
                .iter()
                .filter(|(_, d)| !d.is_empty())
                .map(|(f, d)| (f.to_string(), json!(d)))
                .collect();
            (!deps.is_empty()).then_some(Value::Object(deps))
        };
        let wants_code = args.get("include_code").and_then(Value::as_bool) == Some(true);
        if !e.installable || !wants_code {
            out["pro"] = json!(if e.installable {
                format!(
                    "Part of Oxyn Pro ({PRICING_URL}). Call again with include_code: true (or add_component) to get the code: with a signed-in plan (`oxyn login` or OXYN_TOKEN) each new Pro item counts toward this month's, re-downloads are free."
                )
            } else {
                format!(
                    "Part of Oxyn Pro, with no CLI build yet: the code is on its page ({}).",
                    e.url
                )
            });
            if let Some(n) = needs() {
                out["needs"] = n;
            }
            return Ok(out);
        }
        match server.client.fetch_pro(slug, None) {
            Ok(it) => {
                if let Some(u) = server.client.last_usage() {
                    out["pro_usage"] = json!(u);
                }
                Some(it)
            }
            Err(err) => {
                out["pro"] = json!(err.to_string());
                if let Some(n) = needs() {
                    out["needs"] = n;
                }
                return Ok(out);
            }
        }
    } else if e.installable {
        server.client.item(slug)?
    } else {
        None
    };
    let Some(item) = item else {
        out["installable"] = json!(false);
        out["note"] = json!(
            e.note
                .clone()
                .unwrap_or_else(|| "no installable build".into())
        );
        return Ok(out);
    };
    out["builds"] = Value::Array(item.frameworks.iter().map(build_json).collect());
    out["install"] = json!({
        "cli": format!("oxyn add {slug}"),
        "mcp": format!("add_component {{\"slugs\":[\"{slug}\"]}}"),
    });
    if arg_bool(args, "include_code", true) {
        let fw = match arg_framework(args)? {
            Some(f) => f,
            None => item
                .build(Framework::Dioxus)
                .map(|b| b.framework)
                .or_else(|| item.frameworks.first().map(|b| b.framework))
                .unwrap_or(Framework::Dioxus),
        };
        if item.build(fw).is_none() {
            return Err(Error::NoFramework(
                slug.to_string(),
                fw,
                item.framework_list(),
            ));
        }
        out["code"] = own_files(server, &item, fw, arg_str(args, "file"))?;
    }
    Ok(out)
}

fn project_dir(server: &Server, args: &Value) -> Result<PathBuf, Error> {
    let start = match arg_str(args, "project_path") {
        Some(p) => {
            let p = Path::new(p);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                server.cwd.join(p)
            }
        }
        None => server.cwd.clone(),
    };
    config::find_project(&start).ok_or(Error::NoCargoProject(start))
}

fn add_component(server: &Server, args: &Value) -> Result<Value, Error> {
    let slugs: Vec<String> = match args.get("slugs") {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(Value::as_str)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Some(Value::String(s)) => s
            .split([',', ' '])
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect(),
        _ => Vec::new(),
    };
    if slugs.is_empty() {
        return Err(Error::Invalid(
            "slugs is required: e.g. [\"composer\"]".into(),
        ));
    }
    let fw = arg_framework(args)?;
    let path = arg_str(args, "path");
    let idx = server.client.index()?;
    // Say "needs Pro" / "sign in" / "monthly limit" before touching the project.
    for s in &slugs {
        if let Some(e) = idx.get(s)
            && e.tier == Tier::Pro
            && e.installable
        {
            server.client.fetch_pro(s, None)?;
        }
    }
    let project = project_dir(server, args)?;
    let mut setup: Vec<String> = Vec::new();
    let mut cfg = match Config::load(&project) {
        Ok(c) => c,
        Err(Error::NotInitialised(_)) => {
            let source = Source {
                git: Some(idx.source.git.clone()),
                rev: Some(idx.source.rev.clone()),
                path: None,
            };
            let rep = install::init(&project, fw, path, source, false)?;
            setup = rep.changes;
            Config::load(&project)?
        }
        Err(e) => return Err(e),
    };
    if let Some(f) = fw {
        cfg.framework = f;
    }
    if let Some(p) = path {
        cfg.path = p.trim_end_matches('/').to_string();
    }
    let overwrite = arg_bool(args, "overwrite", false);
    let rep = install::add(&project, &mut cfg, &server.client, &idx, &slugs, overwrite)?;
    let mut out = json!({
        "project": project.display().to_string(),
        "framework": cfg.framework,
        "added": rep.items,
        "written": rep.files.written,
    });
    if !setup.is_empty() {
        out["setup"] = json!(setup);
    }
    if !rep.files.unchanged.is_empty() {
        out["unchanged"] = json!(rep.files.unchanged.len());
    }
    if !rep.files.skipped.is_empty() {
        out["kept_local_changes"] = json!(rep.files.skipped);
        out["hint"] = json!(
            "these files were edited locally and were not replaced; call again with overwrite: true to replace them"
        );
    }
    if !rep.cargo.is_empty() {
        out["cargo"] = json!(rep.cargo);
    }
    if !rep.next.is_empty() {
        out["next"] = json!(rep.next);
    }
    if let Some(u) = server.client.last_usage() {
        out["pro_usage"] = json!(u);
    }
    Ok(out)
}

fn install_instructions(args: &Value) -> Result<String, Error> {
    let fw = arg_framework(args)?;
    let dx = "## Dioxus 0.7\n\n1. `cargo add dioxus@0.7 --features web` (or desktop / mobile).\n2. Add components with the add_component tool (it writes oxyn.toml, src/oxyn/, `mod oxyn;` and the oxyn-core dependency on first use), or from a terminal: `oxyn init` then `oxyn add composer`.\n3. Load the design tokens once: `document::Stylesheet { href: asset!(\"/assets/oxyn/oxyn-tokens.css\") }`.\n4. Put each component's styles in the page once: `style { {crate::oxyn::COMPOSER_CSS} }` (add_component lists the constants under `next`).\n5. Dark theme: `data-oxyn-theme=\"dark\"` on an ancestor element (`light`, `hc-dark` too).";
    let gp = "## GPUI\n\n1. Depend on gpui at the revision the Oxyn workspace pins (see the repository's Cargo.toml).\n2. Add components with add_component (framework: gpui), or `oxyn init --framework gpui` then `oxyn add button`.\n3. Use the element builders from `crate::oxyn` (e.g. `ButtonEl::new(\"save\", \"Save\")`); get_component lists each constructor and builder method. Themes come from `Theme::DARK` / `Theme::LIGHT` in the components.";
    let common = format!(
        "\n\n## The CLI (optional)\n\n`{CLI_INSTALL}`\n\nDocs: {DOCS_URL} · Pro: {PRICING_URL}"
    );
    Ok(match fw {
        Some(Framework::Dioxus) => format!("{dx}{common}"),
        Some(Framework::Gpui) => format!("{gp}{common}"),
        None => format!("{dx}\n\n{gp}{common}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schemas_name_their_required_fields() {
        let tools = list();
        let get = tools.iter().find(|t| t["name"] == "get_component").unwrap();
        assert_eq!(get["inputSchema"]["required"], json!(["slug"]));
        let add = tools.iter().find(|t| t["name"] == "add_component").unwrap();
        assert_eq!(add["annotations"]["readOnlyHint"], false);
    }

    #[test]
    fn install_instructions_per_framework() {
        let dx = install_instructions(&json!({"framework":"dioxus"})).unwrap();
        assert!(dx.contains("Dioxus") && !dx.contains("## GPUI"));
        let both = install_instructions(&json!({})).unwrap();
        assert!(both.contains("## GPUI") && both.contains(CLI_INSTALL));
        assert!(install_instructions(&json!({"framework":"qt"})).is_err());
    }

    #[test]
    fn clip_keeps_short_text() {
        assert_eq!(clip("abc", 5), "abc");
        assert_eq!(clip("abcdef", 4), "abc…");
    }
}
