//! cascade-typst — a `Renderer` implementation for Typst (print / PDF).
//!
//! STATUS: STUB. It builds and satisfies the [`Renderer`] contract so the workspace compiles and
//! the target's shape is fixed, but the real work is unimplemented — see the `TODO(typst)` markers.
//!
//! When it lands it must, exactly like cascade-css, project the SAME spec formulas — but over
//! `f64` (the numeric side of the dual projection, `formula::*` with `T = f64`) into Typst source,
//! never re-deriving them. Print has no runtime, so the choices CSS keeps reactive get BAKED here:
//! the selected [`cascade::Theme`] palette, and light/dark. The notes feature becomes a Typst
//! layout capability (`layout.make(sidenotes: true)`) rather than markup classes.
use cascade::renderer::{Config, Output, Renderer};
use cascade::Font;

/// The Typst renderer. See the module docs for the projection it owes.
pub struct Typst;

impl Renderer for Typst {
    fn name(&self) -> &'static str {
        "typst"
    }

    /// Typst's fallback contract is just the family name — Typst resolves its own fallbacks, unlike
    /// CSS's explicit stack. This part is the correct behaviour, not a stub.
    fn font_family(&self, font: Font) -> String {
        font.family().to_string()
    }

    fn render(&self, _cfg: &Config) -> Vec<Output> {
        // TODO(typst): project the spec into Typst source. Reuse the shared `formula::*` over `f64`
        // for scale / optical / rhythm; emit the document model as `#show` rules; BAKE `_cfg.theme`
        // and the mode (no runtime in print); realise notes via `layout.make(sidenotes: true)`.
        todo!("cascade-typst render: project the spec into Typst source")
    }

    fn verify(&self, _outputs: &[Output]) -> Vec<String> {
        // TODO(typst): verify with Typst's OWN tooling — compile the rendered .typ (via the `typst`
        // crate or the `typst` CLI) and return any diagnostics. Required by the contract; the
        // medium's real check, never a no-op.
        todo!("cascade-typst verify: compile the output with Typst and report diagnostics")
    }
}
