//! The cascade build MANIFEST — decoded from `cascade.ron` (RON + serde). The simple,
//! consumer-editable surface: which options a build exposes (scales, themes, sidenote
//! modes) and the specific typefaces it packages. To support another font, add an entry
//! to `fonts`. The deep engine lives in [`crate::spec`] (tokens.ron).
//!
//! `deny_unknown_fields` makes a mistyped key a decode error (not a silent drop);
//! `Category` makes a bad category value a decode error (not a runtime surprise).
use serde::Deserialize;
use std::path::Path;

/// Load and decode `cascade.ron` into the typed manifest.
pub fn load(path: &Path) -> Result<Manifest, Box<dyn std::error::Error>> {
    Ok(ron::from_str(&std::fs::read_to_string(path)?)?)
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Exposed scale-preset names (defined in the spec).
    pub scales: Vec<String>,
    /// Exposed theme names.
    pub themes: Vec<String>,
    /// Expose the margin-notes vs footnotes toggle.
    pub sidenotes: bool,
    /// Packaged / supported typefaces — the axis a consumer grows.
    pub fonts: Vec<Font>,
}

/// A font's generic category — closed set, so a typo fails at decode.
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Serif,
    Sans,
    Mono,
}

impl Category {
    /// Lowercase name, as used in the generated `<category>-text` profile reference.
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Serif => "serif",
            Category::Sans => "sans",
            Category::Mono => "mono",
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Font {
    pub name: String,
    pub category: Category,
    pub family: Family,
    pub profile: FontProfile,
    pub measured: Measured,
    /// Optional cross-font size normalization.
    #[serde(default)]
    pub normalize: Option<Normalize>,
    /// Path to a bundled font asset, if this build ships the file itself.
    #[serde(default)]
    pub asset: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Family {
    pub typst: String,
    pub css: String,
}

/// Same knobs as [`crate::spec::Profile`] but anonymous — inline in a font entry.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FontProfile {
    pub optical_size: String,
    pub x_height: f64,
    pub k_tracking: f64,
    pub leading_base: f64,
    pub word_space: f64,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Measured {
    pub x_height: f64,
    pub cap_height: f64,
    pub units_per_em: u32,
    pub sx: String,
    pub asc: String,
    pub desc: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Normalize {
    pub alias: String,
    pub to_x_height: f64,
    pub size_adjust: String,
    pub ascent: String,
    pub descent: String,
}
