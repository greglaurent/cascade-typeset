//! cascade — THE SPEC, compiled into types, plus the renderer contract.
//!
//! `tokens.ron` is the single source of typographic definition; `build.rs` compiles it
//! into the types below. Nothing is loaded at runtime — the spec *is* the type system, so
//! every consumer (renderers, cli) is handed values valid by construction. Add a font /
//! preset / color in the RON → the types recompile → it's usable (or a compile error)
//! everywhere at once. [`renderer::Renderer`] is the contract those consumers implement.
include!(concat!(env!("OUT_DIR"), "/spec.rs"));

pub mod optical;
pub mod renderer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_compiled_into_types() {
        // `Font::Lora` exists because the RON defined it — validity is compile-time.
        assert_eq!(Font::Lora.family(), "Lora");
        assert_eq!(Font::Lora.category(), Category::Serif);
        assert_eq!(Font::Inter.x_height(), 0.546);
        assert_eq!(Font::ALL.len(), 3);

        assert_eq!(SCALE_DEFAULT, ScalePreset::GoldenDitonic);
        assert_eq!(ScalePreset::GoldenRatio.n(), 1);
        assert_eq!(Color::Accent.light(), "#7A2E28");
        assert_eq!(Multiplier::P1.factor(), 2.0);
        assert_eq!(MEASURE, 65);
    }
}
