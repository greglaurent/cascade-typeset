//! The renderer contract — the one rendering-related thing the protocol defines.
//!
//! Stated entirely over the spec's generated types (it is handed a [`Font`], never a
//! string — so it can only be given something the spec defines). Implementations live in
//! their own crates (cascade-css, cascade-typst), depend on this one, bring their own
//! templates, and supply target BEHAVIOR — like how a font resolves to a family with
//! fallbacks. The protocol renders nothing itself.
use crate::Font;

/// One generated output file: a target-relative path and its contents.
pub struct Output {
    pub path: String,
    pub body: String,
}

pub trait Renderer {
    /// Short target name, e.g. `"css"` / `"typst"`.
    fn name(&self) -> &'static str;

    /// Resolve a spec font into this target's family representation — the renderer's
    /// FALLBACK contract. It gets the abstract font from the spec (`font.family()`,
    /// `font.category()`) and produces the target form: cascade-css builds a stack,
    /// cascade-typst uses the family alone. The spec never holds this.
    fn font_family(&self, font: Font) -> String;

    /// Produce all of this target's output files from the spec.
    fn render(&self) -> Vec<Output>;
}
