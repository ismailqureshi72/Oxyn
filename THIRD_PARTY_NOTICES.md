# Third-party notices

Code adapted from other projects, with the file that carries it. See [docs/licensing.md](docs/licensing.md) for the policy.

## GPUI `input.rs` example (zed-industries/zed)

- Used in: `crates/oxyn-gpui/src/parity/ime.rs` (the element and platform input-handler plumbing of `ImeInput`).
- Source: https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/input.rs
- License: Apache License 2.0 (the `gpui` crate). Copyright Zed Industries, Inc.
- Changes: the editing model moved to `oxyn_core::parity::edit::TextBuffer`; Oxyn theming, a bare style for composer shells, line breaks, ghost text, and Enter / accept / dismiss events were added.
