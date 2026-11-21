//! The registry builder: reads the renderer crates and the site's catalogue, and writes the static
//! registry the `oxyn` CLI installs from (`site/public/r/`). See `oxyn-registry-schema` for the
//! shapes and `docs/cli.md` for the design.

pub mod build;
pub mod graph;
pub mod scan;

pub use build::{
    BuildConfig, MetaItem, Registry, THEME_SLUG, build, content_hash, write, write_pro,
};
