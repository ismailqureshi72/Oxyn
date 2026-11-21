//! Renders Oxyn components to Rust source; shared by the CLI and Studio.
//!
//! [`registry`] builds the static registry the `oxyn` CLI installs from: it walks the renderer
//! crates, works out each component's files and dependencies, and writes `index.json` plus one
//! JSON per free component.

pub mod registry;

/// Crate version, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_set() {
        assert!(!super::VERSION.is_empty());
    }
}
