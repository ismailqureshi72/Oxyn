//! `oxyn-tokens-gen`: validates the DTCG token files and writes the generated outputs.
//!
//! ```text
//! cargo run -p oxyn-tokens-gen            # validate, then write the outputs
//! cargo run -p oxyn-tokens-gen -- --check # validate, then fail if the outputs are stale (CI)
//! ```
//!
//! Inputs: `tokens/oxyn.tokens.json`, `tokens/oxyn.motion.json`.
//! Outputs: `crates/oxyn-tokens/src/generated.rs`, `tokens/dist/oxyn.css`.
//! Validation follows `docs/tokens.md` §11 (structure, aliases, tiers, theme parity, contrast).

mod contrast;
mod css;
mod model;
mod rust;

use std::{fs, path::PathBuf, process::ExitCode};

use model::TokenSet;

const TOKENS: &str = "tokens/oxyn.tokens.json";
const MOTION: &str = "tokens/oxyn.motion.json";
const OUT_RUST: &str = "crates/oxyn-tokens/src/generated.rs";
const OUT_CSS: &str = "tokens/dist/oxyn.css";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let check = std::env::args().any(|a| a == "--check");
    let root = repo_root();
    let read = |p: &str| {
        let text = fs::read_to_string(root.join(p)).unwrap_or_else(|e| panic!("reading {p}: {e}"));
        serde_json::from_str::<serde_json::Value>(&text)
            .unwrap_or_else(|e| panic!("parsing {p}: {e}"))
    };

    let tokens = TokenSet::load(&read(TOKENS));
    let motion = TokenSet::load(&read(MOTION));

    let mut errors = tokens.validate_tokens_file();
    errors.extend(motion.validate_common());
    let mut seen = std::collections::HashSet::new();
    errors.retain(|e| seen.insert(e.clone()));
    // Contrast needs every colour to resolve, so it only runs on a structurally valid file.
    let rows = if errors.is_empty() {
        contrast::rows(&tokens)
    } else {
        Vec::new()
    };
    let fails: Vec<_> = rows.iter().filter(|r| r.fails()).collect();

    println!(
        "tokens: {} ({} aliases), motion: {} ({} aliases), contrast rows: {}, fails: {}",
        tokens.len(),
        tokens.alias_count(),
        motion.len(),
        motion.alias_count(),
        rows.len(),
        fails.len()
    );
    for e in &errors {
        eprintln!("error: {e}");
    }
    for r in &fails {
        eprintln!("contrast: {r}");
    }
    if !errors.is_empty() || !fails.is_empty() {
        return ExitCode::FAILURE;
    }

    let outputs = [
        (OUT_RUST, rust::generate(&tokens, &motion)),
        (OUT_CSS, css::generate(&tokens, &motion)),
    ];
    let mut stale = false;
    for (path, text) in outputs {
        let full = root.join(path);
        let current = fs::read_to_string(&full).unwrap_or_default();
        if current == text {
            continue;
        }
        if check {
            eprintln!("stale: {path} (run `cargo run -p oxyn-tokens-gen`)");
            stale = true;
        } else {
            if let Some(dir) = full.parent() {
                fs::create_dir_all(dir).expect("create output dir");
            }
            fs::write(&full, text).unwrap_or_else(|e| panic!("writing {path}: {e}"));
            println!("wrote {path}");
        }
    }
    if stale {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
