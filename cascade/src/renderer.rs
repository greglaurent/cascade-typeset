//! The renderer contract — the one rendering-related thing the protocol defines.
//!
//! Stated entirely over the spec's generated types (it is handed a [`Font`], never a
//! string — so it can only be given something the spec defines). Implementations live in
//! their own crates (cascade-css, cascade-typst), depend on this one, bring their own
//! templates, and supply target BEHAVIOR — like how a font resolves to a family with
//! fallbacks. The protocol renders nothing itself.
use crate::{Font, ScalePreset, Theme, FONT_BODY, FONT_HEADING, SCALE_DEFAULT, THEME_DEFAULT};

/// One generated output file: a target-relative path and its contents.
pub struct Output {
    pub path: String,
    pub body: String,
}

/// The LIMITED surface a consumer may change per build: which modular [`ScalePreset`], the body /
/// heading typefaces, and the colour [`Theme`] (palette). Everything else is the spec (compiled,
/// not tunable). This is the single input to [`Renderer::render`], so every target projects the
/// SAME choices — the CLI resolves a `cascade.ron` (or flags) into one of these and hands it to
/// each renderer. [`Default`] is the spec's compiled defaults (`SCALE_DEFAULT`, `FONT_BODY`,
/// `FONT_HEADING`, `THEME_DEFAULT`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub scale: ScalePreset,
    pub body: Font,
    pub heading: Font,
    pub theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Self { scale: SCALE_DEFAULT, body: FONT_BODY, heading: FONT_HEADING, theme: THEME_DEFAULT }
    }
}

pub trait Renderer {
    /// Short target name, e.g. `"css"` / `"typst"`.
    fn name(&self) -> &'static str;

    /// Resolve a spec font into this target's family representation — the renderer's
    /// FALLBACK contract. It gets the abstract font from the spec (`font.family()`,
    /// `font.category()`) and produces the target form: cascade-css builds a stack,
    /// cascade-typst uses the family alone. The spec never holds this.
    fn font_family(&self, font: Font) -> String;

    /// Produce all of this target's output files for the given [`Config`].
    fn render(&self, cfg: &Config) -> Vec<Output>;

    /// Verify rendered output is correct FOR THIS MEDIUM, using that medium's OWN tooling — CSS
    /// parses it with a browser-grade parser and checks for dangling references; a print target
    /// compiles it. Returns the problems found (empty = OK).
    ///
    /// This is a REQUIRED obligation of the contract, deliberately WITHOUT a default: a renderer may
    /// not ship output it cannot stand behind, so every target must supply a real check with the
    /// right tool — a no-op won't compile as one. Drives `cascade build --verify`.
    fn verify(&self, outputs: &[Output]) -> Vec<String>;
}
