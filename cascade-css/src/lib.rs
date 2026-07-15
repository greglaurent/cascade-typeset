//! cascade-css — a `Renderer` implementation for CSS.
//!
//! It CONSUMES the spec's types (`Font`, `Color`, `ScalePreset`, `Multiplier`, …) and
//! produces CSS via its own templates (`templates/*.css`). It DEFINES nothing typographic —
//! it only supplies CSS *behavior*: the fallback stacks, the `--ct-`/`--cs-`/`--cr-`/`--cf-`
//! conventions, the fluid base size, the grid-unit fraction, the `pow()`/`clamp()` formulas.
//! The spec stays renderer-agnostic; this crate is "what it looks like in CSS."
use askama::Template;
use cascade::renderer::{Output, Renderer};
use cascade::{
    optical, Category, Color, Font, Multiplier, ScalePreset, LEADING_CLAMP, MEASURE, SCALE_DEFAULT,
    STEPS_MAX, STEPS_MIN, TRACKING_CLAMP, WORD_SPACE_K,
};

/// cascade-css's default base body size, in points, driven from the user's perspective
/// (e.g. 11pt) — a renderer default, overridable via the user-input boundary. cascade-css
/// CALCULATES its fluid clamp from this pt; the spec never holds a base.
const DEFAULT_BASE_PT: f64 = 11.0;
/// cascade-css's own fluid parameters (its behavior, like the fallbacks): pt→rem, the
/// viewport window the base grows across, and how much it grows.
const PT_PER_REM: f64 = 12.0; // 16px/rem at 96dpi
const FLUID_MIN_VW: f64 = 20.0; // rem (~320px)
const FLUID_MAX_VW: f64 = 80.0; // rem (~1280px)
const FLUID_GROWTH: f64 = 1.2;
/// The grid unit as a fraction of the base — also CSS behavior.
const CSS_UNIT: f64 = 0.375;
/// cascade-css's default mono/code family and readability floor — its defaults, overridable.
const DEFAULT_MONO_FAMILY: &str = "DejaVu Sans Mono";
const DEFAULT_SIZE_MIN_PT: f64 = 9.0; // → 0.75rem

pub struct Css;

impl Css {
    /// cascade-css's own fallback chains, per spec category — renderer behavior. The spec
    /// gives only the abstract `Category`; the concrete CSS stack lives here.
    fn fallbacks(category: Category) -> &'static str {
        match category {
            Category::Serif => "Georgia, \"Times New Roman\", serif",
            Category::Sans => "system-ui, -apple-system, sans-serif",
            Category::Mono => "ui-monospace, SFMono-Regular, monospace",
        }
    }

    /// Calculate cascade-css's fluid base `clamp()` from a point size: pt in, CSS's own
    /// fluid range out. The renderer derives its context-specific value; the spec supplies
    /// no clamp, no range — only the renderer knows how CSS is fluid.
    fn base_clamp(pt: f64) -> String {
        let min = pt / PT_PER_REM;
        let max = min * FLUID_GROWTH;
        let slope = (max - min) / (FLUID_MAX_VW - FLUID_MIN_VW);
        let intercept = min - slope * FLUID_MIN_VW;
        let vw = slope * 100.0;
        format!("clamp({min:.4}rem, {intercept:.4}rem + {vw:.4}vw, {max:.4}rem)")
    }
}

// ── templates (cascade-css owns these; they consume the spec) ────────────────────
#[derive(Template)]
#[template(path = "theme.css", escape = "none")]
struct ThemeCss {
    colors: Vec<ColorRow>,
}
struct ColorRow {
    id: &'static str,
    light: &'static str,
    dark: &'static str,
}

#[derive(Template)]
#[template(path = "scale.css", escape = "none")]
struct ScaleCss {
    base: String,
    default_id: &'static str,
    default_ratio: f64,
    default_n: u32,
    sizes: Vec<SizeRow>,
    presets: Vec<PresetRow>,
}
struct SizeRow {
    label: String,
    expr: String,
}
struct PresetRow {
    id: &'static str,
    ratio: f64,
    n: u32,
}

#[derive(Template)]
#[template(path = "rhythm.css", escape = "none")]
struct RhythmCss {
    unit_fraction: f64,
    spaces: Vec<SpaceRow>,
}
struct SpaceRow {
    label: String,
    factor: f64,
}

#[derive(Template)]
#[template(path = "fonts.css", escape = "none")]
struct FontsCss {
    fonts: Vec<FontRow>,
}
struct FontRow {
    slug: String,
    family: String,
    x_height: f64,
    k_tracking: f64,
    leading_base: f64,
    word_space: f64,
}

#[derive(Template)]
#[template(path = "font.css", escape = "none")]
struct FontCss {
    serif_family: String,
    sans_family: String,
    mono_family: String,
    xh: f64,
    kt: f64,
    lb: f64,
    bws: f64,
    kws: f64,
    tc: f64,
    lmin: f64,
    lmax: f64,
    measure: u32,
    size_min: String,
    // Optical model, PROJECTED from the spec (cascade::optical) — not re-encoded here. Each
    // string is `Expr::to_css_value` with CSS bindings, so a class swap re-resolves it live.
    lead0: String,
    h_lead0: String,
    tracks: Vec<Row>,
    leads: Vec<Row>,
    h_tracks: Vec<Row>,
    h_leads: Vec<Row>,
    profiles: Vec<ProfileClassRow>,
}
struct Row {
    label: String,
    value: String,
}
struct ProfileClassRow {
    id: &'static str,
    xh: f64,
    kt: f64,
    lb: f64,
    bws: f64,
}

impl Renderer for Css {
    fn name(&self) -> &'static str {
        "css"
    }

    /// The fallback contract: the abstract font (from the spec) + this renderer's stack.
    fn font_family(&self, font: Font) -> String {
        format!("\"{}\", {}", font.family(), Self::fallbacks(font.category()))
    }

    fn render(&self) -> Vec<Output> {
        // theme.css — the spec's Color type.
        let colors = Color::ALL
            .into_iter()
            .map(|c| ColorRow { id: c.id(), light: c.light(), dark: c.dark() })
            .collect();
        let theme = ThemeCss { colors }.render().expect("render theme.css");

        // scale.css — the spec's ScalePreset + steps; the base + pow() formula are ours.
        let label = |i: i32| {
            if i == 0 {
                "0".to_string()
            } else if i < 0 {
                format!("n{}", -i)
            } else {
                format!("p{i}")
            }
        };
        // The modular-scale factor is a spec formula (optical::size_factor), projected to a
        // live CSS calc(); cascade-css only multiplies it by its own fluid base.
        let scale_bind = |k: &str| match k {
            "ratio" => "var(--cs-ratio)".to_string(),
            "n" => "var(--cs-n)".to_string(),
            other => panic!("scale: unbound {other}"),
        };
        let sizes = (STEPS_MIN..=STEPS_MAX)
            .map(|i| SizeRow {
                label: label(i),
                expr: if i == 0 {
                    "var(--cs-base)".to_string()
                } else {
                    format!("calc(var(--cs-base) * {})", optical::size_factor(i).to_css_value(&scale_bind))
                },
            })
            .collect();
        let presets = ScalePreset::ALL
            .into_iter()
            .map(|p| PresetRow { id: p.id(), ratio: p.ratio(), n: p.n() })
            .collect();
        let scale = ScaleCss {
            base: Self::base_clamp(DEFAULT_BASE_PT),
            default_id: SCALE_DEFAULT.id(),
            default_ratio: SCALE_DEFAULT.ratio(),
            default_n: SCALE_DEFAULT.n(),
            sizes,
            presets,
        }
        .render()
        .expect("render scale.css");

        // rhythm.css — the spec's Multiplier; the unit fraction is ours. `base` → `0`.
        let spaces = Multiplier::ALL
            .into_iter()
            .map(|m| SpaceRow {
                label: if m.id() == "base" { "0".to_string() } else { m.id().to_string() },
                factor: m.factor(),
            })
            .collect();
        let rhythm = RhythmCss { unit_fraction: CSS_UNIT, spaces }.render().expect("render rhythm.css");

        // fonts.css — each spec Font as a bundle; `font_family` (the fallback contract)
        // builds the stack, the optical vars come straight from the spec.
        let fonts = Font::ALL
            .into_iter()
            .map(|f| FontRow {
                slug: f.family().to_lowercase(),
                family: self.font_family(f),
                x_height: f.x_height(),
                k_tracking: f.k_tracking(),
                leading_base: f.leading_base(),
                word_space: f.word_space(),
            })
            .collect();
        let fonts_css = FontsCss { fonts }.render().expect("render fonts.css");

        // font.css — the optical model, PROJECTED from the spec. `body`/`head` bind each
        // formula variable to a CSS custom property; the SAME optical::* definitions drive
        // both (and Typst, via eval). A class swap re-resolves the emitted calc() live.
        let body = |k: &str| -> String {
            match k {
                "ratio" => "var(--cs-ratio)",
                "n" => "var(--cs-n)",
                "ln_ratio" => "var(--cs-ln-ratio)",
                "xh" => "var(--cf-xh)",
                "lb" => "var(--cf-lb)",
                "measure" => "var(--cf-measure)",
                "lead0" => "var(--cf-lead0)",
                "lmin" => "var(--cf-lmin)",
                "lmax" => "var(--cf-lmax)",
                "tc" => "var(--cf-tc)",
                "kt" => "var(--cf-kt)",
                other => panic!("body: unbound {other}"),
            }
            .to_string()
        };
        let head = |k: &str| -> String {
            match k {
                "xh" => "var(--cf-h-xh)".to_string(),
                "lb" => "var(--cf-h-lb)".to_string(),
                "lead0" => "var(--cf-h-lead0)".to_string(),
                "kt" => "var(--cf-h-kt)".to_string(),
                other => body(other),
            }
        };
        // tracking is an em fraction — apply the unit here (CSS representation); leading is unitless.
        let track = |i: i32, bind: &dyn Fn(&str) -> String| -> String {
            if i == 0 {
                "0em".to_string()
            } else {
                format!("calc({} * 1em)", optical::tracking(i).to_css_value(bind))
            }
        };
        let lead0 = optical::lead0().to_css_value(&body);
        let h_lead0 = optical::lead0().to_css_value(&head);
        let tracks: Vec<Row> =
            (STEPS_MIN..=STEPS_MAX).map(|i| Row { label: label(i), value: track(i, &body) }).collect();
        let leads: Vec<Row> = (STEPS_MIN..=STEPS_MAX)
            .map(|i| Row { label: label(i), value: optical::leading(i).to_css_value(&body) })
            .collect();
        let h_tracks: Vec<Row> =
            (1..=4).map(|k| Row { label: format!("p{k}"), value: track(k, &head) }).collect();
        let h_leads: Vec<Row> = (1..=4)
            .map(|k| Row { label: format!("p{k}"), value: optical::leading(k).to_css_value(&head) })
            .collect();
        // Per-category default optical baselines — `.cascade.profile-serif|sans|mono`, the
        // generic tuning when no specific font is applied. Straight from the spec's Category.
        let profiles = Category::ALL
            .into_iter()
            .map(|c| ProfileClassRow {
                id: c.as_str(),
                xh: c.default_x_height(),
                kt: c.default_k_tracking(),
                lb: c.default_leading_base(),
                bws: c.default_word_space(),
            })
            .collect();
        let font = FontCss {
            serif_family: self.font_family(Font::Lora),
            sans_family: self.font_family(Font::Inter),
            mono_family: format!("\"{}\", {}", DEFAULT_MONO_FAMILY, Self::fallbacks(Category::Mono)),
            xh: Font::Lora.x_height(),
            kt: Font::Lora.k_tracking(),
            lb: Font::Lora.leading_base(),
            bws: Font::Lora.word_space(),
            kws: WORD_SPACE_K,
            tc: TRACKING_CLAMP,
            lmin: LEADING_CLAMP.0,
            lmax: LEADING_CLAMP.1,
            measure: MEASURE,
            size_min: format!("{:.4}rem", DEFAULT_SIZE_MIN_PT / PT_PER_REM),
            lead0,
            h_lead0,
            tracks,
            leads,
            h_tracks,
            h_leads,
            profiles,
        }
        .render()
        .expect("render font.css");

        vec![
            Output { path: "theme.css".into(), body: theme },
            Output { path: "scale.css".into(), body: scale },
            Output { path: "font.css".into(), body: font },
            Output { path: "rhythm.css".into(), body: rhythm },
            Output { path: "fonts.css".into(), body: fonts_css },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn out(path: &str) -> String {
        Css.render().into_iter().find(|o| o.path == path).unwrap().body
    }

    #[test]
    fn font_family_is_spec_identity_plus_css_fallbacks() {
        assert_eq!(Css.font_family(Font::Lora), "\"Lora\", Georgia, \"Times New Roman\", serif");
        assert_eq!(Css.font_family(Font::Inter), "\"Inter\", system-ui, -apple-system, sans-serif");
    }

    #[test]
    fn family_is_selected_by_category_type_not_by_font() {
        let fallbacks = |f: Font| Css.font_family(f).split_once(", ").unwrap().1.to_string();
        assert_eq!(fallbacks(Font::Inter), fallbacks(Font::Jost)); // both sans → same
        assert_ne!(fallbacks(Font::Lora), fallbacks(Font::Inter)); // serif → different
    }

    #[test]
    fn renders_theme_and_scale() {
        let theme = out("theme.css");
        assert!(theme.contains("--ct-accent: #7A2E28;") && theme.contains("--ct-accent: #E09A93;"));
        let scale = out("scale.css");
        assert!(scale.contains("--cs-base: clamp("));
        assert!(scale.contains(".cascade.scale-classical { --cs-ratio: 2; --cs-n: 5; }"));
    }

    #[test]
    fn renders_rhythm_and_fonts() {
        let rhythm = out("rhythm.css");
        assert!(rhythm.contains("--cr-space-0: calc(var(--cr-unit) * 1);"));
        assert!(rhythm.contains("--cr-space-p1: calc(var(--cr-unit) * 2);"));

        let fonts = out("fonts.css");
        assert!(fonts.contains(".cascade.bundle-lora {"));
        assert!(fonts.contains("--cf-family-body: \"Lora\", Georgia, \"Times New Roman\", serif;"));
        assert!(fonts.contains("--cf-xh:  0.5;"));
    }

    #[test]
    fn renders_font_optical_model() {
        let font = out("font.css");
        // families: the confirmed defaults
        assert!(font.contains("--cf-font-serif: \"Lora\", Georgia, \"Times New Roman\", serif;"));
        assert!(font.contains("--cf-font-mono: \"DejaVu Sans Mono\", ui-monospace, SFMono-Regular, monospace;"));
        // body optical = Lora (default serif); knobs from the spec
        assert!(font.contains("--cf-xh: 0.5;"));
        assert!(font.contains("--cf-tc: 0.04;"));
        assert!(font.contains("--cf-lmin: 1.2;") && font.contains("--cf-lmax: 1.5;"));
        // clamp chains cascade-css derives, + profile classes from the spec
        assert!(font.contains("--cf-track-p1: calc(clamp("));
        assert!(font.contains("--cf-lead-n5: clamp(var(--cf-lmin),"));
        assert!(font.contains(".cascade.profile-serif { --cf-xh: 0.49;"));
    }
}
