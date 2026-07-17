//! The stylesheet, rendered IN-MEMORY via the cascade-css API -- the site is a Rust consumer of the
//! cascade library, so it renders live rather than vendoring pre-built files.

use cascade::renderer::{Config, Renderer};
use cascade_css::Css;

/// Render cascade's CSS and concatenate the modules into one served stylesheet. The specimen picks
/// its typeface with a `bundle-*` class (serif by default), so no baked body override is needed
/// here; the `cascade.css` entrypoint is only an `@import` list, so serving a single file we inline
/// the modules instead (custom-property resolution is order-independent).
pub fn stylesheet() -> String {
    Css.render(&Config::default())
        .into_iter()
        .filter(|o| o.path != "cascade.css")
        .map(|o| o.body)
        .collect::<Vec<_>>()
        .join("\n")
}
