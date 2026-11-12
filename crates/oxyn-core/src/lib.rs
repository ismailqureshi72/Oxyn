//! UI-free agent state machine, event stream model and verb packs. Filled in by #14.
//!
//! Placeholder crate so the workspace, CI and crate names exist from M0.

/// Crate version, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_set() {
        assert!(!super::VERSION.is_empty());
    }
}
