//! Dioxus components for web demos, Studio and web apps. First component lands in #15.
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
