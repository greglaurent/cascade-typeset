//! cascade — THE SPEC, compiled into types, plus the renderer contract.
//!
//! `tokens.ron` is the single source of typographic definition; [`spec`] includes the types that
//! build.rs generates from it at build time. Nothing is loaded at runtime — the spec *is* the
//! type system, so every consumer (renderers, cli) is handed values valid by construction. Add a
//! font / preset / color in the RON → the types recompile → it's usable (or a compile error)
//! everywhere at once. [`renderer::Renderer`] is the contract those consumers implement.
mod spec;
pub use spec::*;

/// The spec's version — stamped onto a shipped distribution (`cascade dist`). Travels with the crate
/// (compile-time `CARGO_PKG_VERSION`), so nothing has to parse Cargo.toml to learn it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod formula;
pub mod renderer;

/// Font measurement — the standardized derivation of a font's optical metrics + the `fonts/*.ron`
/// format. Feature-gated (`measure`) so the core spec stays dependency-free; the CLI enables it.
#[cfg(feature = "measure")]
pub mod measure;

/// Look a value up by its spec name against the compiled closed set — the inverse of `id()` /
/// `family()`. Used at the CLI boundary to turn a consumer's string (`cascade.ron`, a `--flag`)
/// into a type valid by construction, or a listable error. Membership is still the type system's:
/// these only search `ALL`, so an unknown name is unrepresentable, never a bad variant.
impl ScalePreset {
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.id() == id)
    }
}

impl Theme {
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.id() == id)
    }
}

impl Font {
    /// Case-insensitive on the family name (`Font::family`), so `inter` resolves `Font::Inter`.
    pub fn from_family(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.family().eq_ignore_ascii_case(name))
    }
}

impl Category {
    pub fn from_str(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_str().eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_compiled_into_types() {
        // `Font::Lora` exists because the RON defined it — validity is compile-time.
        assert_eq!(Font::Lora.family(), "Lora");
        assert_eq!(Font::Lora.category(), Category::Serif);
        assert_eq!(Font::Inter.x_height(), 0.546);
        // avg_advance is a measured metric (the copyfitting factor): per-font, plus a category default.
        assert_eq!(Font::Inter.avg_advance(), 0.4714);
        assert_eq!(Category::Serif.default_avg_advance(), 0.46);
        // The bundle ships Inter (body), Lora (heading), IBM Plex Mono (code); other faces are the
        // client's (external) domain. A multi-word family compiles to a PascalCase variant.
        assert_eq!(Font::ALL.len(), 3);
        assert_eq!(Font::IBMPlexMono.family(), "IBM Plex Mono");
        assert_eq!(Font::IBMPlexMono.category(), Category::Mono);
        assert_eq!(Font::IBMPlexMono.x_height(), 0.516);

        assert_eq!(SCALE_DEFAULT, ScalePreset::GoldenDitonic);
        assert_eq!(FONT_BODY, Font::Inter);
        assert_eq!(FONT_HEADING, Font::Lora);
        assert_eq!(FONT_CODE, Font::IBMPlexMono);
        assert_eq!(ScalePreset::GoldenRatio.n(), 1);
        assert_eq!(THEME_DEFAULT, Theme::Paper);
        assert_eq!(Theme::Paper.light(Color::Accent), "#7A2E28");
        assert_eq!(Multiplier::P1.factor(), 2.0);
        assert_eq!(MEASURE, 65);
    }

    #[test]
    fn document_model_is_compiled_into_roles() {
        // The role registry is now spec data, projected — not authored in a renderer.
        assert_eq!(Role::ALL.len(), 25);
        // structure only, never colour
        assert_eq!(Role::Heading1.step(), Some(4));
        assert_eq!(Role::Heading1.font(), Some(FontRole::Heading));
        assert_eq!(Role::Heading1.weight(), Some(700));
        assert_eq!(Role::Heading1.kind(), RoleKind::Block);
        assert_eq!(Role::Heading1.space_before(), Some("p4"));
        assert_eq!(Role::Body.step(), Some(0));
        assert_eq!(Role::Body.space_after(), Some("baseline"));
        assert_eq!(Role::Emphasis.italic(), true);
        assert_eq!(Role::Emphasis.step(), None);
        assert_eq!(Role::Link.underline(), true);
        assert_eq!(Role::Code.font(), Some(FontRole::Code));
        assert_eq!(Role::Code.elements(), &["code", "kbd", "samp"]);
        assert_eq!(Role::Root.elements(), &[] as &[&str]);
    }

    #[test]
    fn palettes_are_selectable_over_a_shared_swatch_vocabulary() {
        // Multiple palettes (themes/*.ron), same swatch vocabulary, values differ per Theme.
        assert_eq!(Theme::ALL.len(), 2);
        assert_eq!(Theme::Paper.id(), "paper");
        assert_eq!(Theme::Paper.light(Color::Accent), "#7A2E28");
        assert_eq!(Theme::Slate.light(Color::Accent), "#2F5E8A");
        // dark ships alongside light within each palette (runtime toggle, not a build choice).
        assert_eq!(Theme::Paper.dark(Color::Bg), "#14120E");
        assert_eq!(Theme::Slate.dark(Color::Bg), "#0F1216");
        // `Color` is now a name-only vocabulary shared by every palette.
        assert_eq!(Color::Accent.id(), "accent");
    }

    #[test]
    fn theme_is_a_separate_layer_combined_by_role() {
        // Semantic uses alias base swatches.
        assert_eq!(Semantic::Link.base(), "accent");
        assert_eq!(Semantic::QuoteBg.base(), "transparent");
        // The role → colour binding: keyed by the structural Role, but Role itself stays colour-free.
        assert_eq!(role_color(Role::Root), RoleColor { fg: Some("fg"), bg: Some("bg"), border: None });
        assert_eq!(role_color(Role::Footnotes).border, Some("rule"));
        assert_eq!(role_color(Role::Code), RoleColor { fg: Some("code-fg"), bg: Some("code-bg"), border: None });
        // Unbound roles inherit (all-None).
        assert_eq!(role_color(Role::Body), RoleColor { fg: None, bg: None, border: None });
    }
}
