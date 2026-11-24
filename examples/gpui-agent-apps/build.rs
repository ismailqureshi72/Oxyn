//! Turns on `cfg(oxyn_pro)` when the Oxyn Pro sources are overlaid on this checkout.
//!
//! The Pro components live in the private repo `ismailqureshi72/oxyn-pro` at their original paths;
//! CI checks it out and copies it over the tree before building (see CONTRIBUTING.md). Its root
//! holds the `.oxyn-pro` marker. Without it this crate builds the free set only.

use std::path::Path;

fn main() {
    // Read at run time: the compiled build script can be reused for another checkout.
    let dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let marker = Path::new(&dir).join("../../.oxyn-pro");
    println!("cargo::rustc-check-cfg=cfg(oxyn_pro)");
    if marker.exists() {
        println!("cargo::rerun-if-changed={}", marker.display());
        println!("cargo::rustc-cfg=oxyn_pro");
    } else {
        // A missing file named in rerun-if-changed counts as changed on every build (the crate was
        // rebuilt each time in the free set). Without the marker, rerun when this script changes or
        // when the Pro overlay says it ran (OXYN_PRO_MODE, set by .github/actions/pro-overlay).
        println!("cargo::rerun-if-changed=build.rs");
        println!("cargo::rerun-if-env-changed=OXYN_PRO_MODE");
    }
}
