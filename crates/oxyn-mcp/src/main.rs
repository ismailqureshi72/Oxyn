//! `oxyn-mcp`: the Oxyn MCP server over stdio. Point your agent at it, for example
//! `claude mcp add oxyn -- oxyn-mcp`. See https://ui.oxyn.workers.dev/docs/mcp/.

use std::process::ExitCode;

use oxyn_cli::registry_base;
use oxyn_mcp::Server;

const HELP: &str = "oxyn-mcp: the Oxyn MCP server (stdio). Lets AI coding agents list, read and add Oxyn components.

Usage: oxyn-mcp [--registry <URL|DIR>]

Your agent starts it; it reads JSON-RPC on stdin and answers on stdout.
  claude mcp add oxyn -- oxyn-mcp

Options:
  --registry <URL|DIR>  Registry base (default https://ui.oxyn.workers.dev/r; env OXYN_REGISTRY)
  -h, --help            This help
  -V, --version         Version

Docs: https://ui.oxyn.workers.dev/docs/mcp/";

fn main() -> ExitCode {
    let mut registry: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("oxyn-mcp {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--registry" => match args.next() {
                Some(v) => registry = Some(v),
                None => {
                    eprintln!("error: --registry needs a value");
                    return ExitCode::from(2);
                }
            },
            s if s.starts_with("--registry=") => {
                registry = Some(s["--registry=".len()..].to_string());
            }
            other => {
                eprintln!("error: unexpected argument `{other}` (see oxyn-mcp --help)");
                return ExitCode::from(2);
            }
        }
    }
    let base = registry_base(registry.as_deref(), None);
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
    eprintln!("oxyn-mcp {}: registry {base}", env!("CARGO_PKG_VERSION"));
    match Server::new(&base, cwd).run_stdio() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
