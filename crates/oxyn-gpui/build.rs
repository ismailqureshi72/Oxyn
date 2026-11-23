//! Turns on `cfg(oxyn_pro)` when the Oxyn Pro sources are overlaid on this checkout.
//!
//! The Pro components live in the private repo `ismailqureshi72/oxyn-pro` at their original paths;
//! CI checks it out and copies it over the tree before building (see CONTRIBUTING.md). Its root
//! holds the `.oxyn-pro` marker. Without it this crate builds the free set only.

use std::path::Path;

fn main() {
    let marker = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.oxyn-pro");
    println!("cargo::rustc-check-cfg=cfg(oxyn_pro)");
    println!("cargo::rerun-if-changed={}", marker.display());
    if marker.exists() {
        println!("cargo::rustc-cfg=oxyn_pro");
    }
}
