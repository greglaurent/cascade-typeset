//! cascade-typst — a [`Renderer`] implementation for Typst (print / PDF).
//!
//! Typst is the NUMERIC side of the spec's dual projection: every size / tracking / leading /
//! word-space / rhythm / colour is computed here in Rust via the shared `cascade::formula::*`
//! (over `f64`, per `formula.rs`) and BAKED as a concrete value into Typst source. Print has no
//! runtime, so the config (scale, body/heading/code faces, theme) is resolved by the CLI and fixed
//! at build time — the CSS renderer's reactive class switches have no analogue. The generated
//! `cascade.typ` reproduces the styling decisions of the archive Typst library (`archive/
//! cascade-typst/*.typ`) with baked numbers instead of its formulas-in-Typst machinery.
//!
//! `verify` shells out to the `typst` CLI to compile the output — the medium's own check.

use std::process::Command;

use askama::Template;
use cascade::formula;
use cascade::renderer::{Config, Output, Renderer, ResolvedFont};
use cascade::{
    role_color, Color, FontRole, Multiplier, Role, Semantic, Theme, BASE_PT, LEADING_CLAMP, MEASURE,
    RHYTHM_UNIT_RATIO, SIZE_MIN_PT, TRACKING_CLAMP, VERSION, WORD_SPACE_K,
};

// The typographic constants — base reading size, rhythm-unit ratio, readability floor — are
// SPEC-OWNED (`cascade::{BASE_PT, RHYTHM_UNIT_RATIO, SIZE_MIN_PT}`), the SAME values cascade-css
// reads. A renderer never chooses them, so the two formats cannot diverge on size/rhythm/floor.

/// us-letter width in pt (8.5in × 72) — a print-context default (which paper), the renderer's own
/// concern; the measure is centred within it.
const US_LETTER_PT: f64 = 612.0;

/// The Typst renderer.
pub struct Typst;

// ── template binding ─────────────────────────────────────────────────────────
#[derive(Template)]
#[template(path = "cascade.typ", escape = "none")]
struct CascadeTyp {
    body_family: String,
    heading_family: String,
    code_family: String,
    // palette (baked light variant)
    fg: String,
    fg_muted: String,
    fg_subtle: String,
    bg: String,
    bg_subtle: String,
    rule: String,
    accent: String,
    accent_rule: String,
    link: String,
    code_fg: String,
    code_bg: String,
    quote_rule: String,
    quote_bg_expr: String,
    // per-role type/optical table
    rows: Vec<TypeRow>,
    // page geometry
    margin_x: String,
    // rhythm
    unit: String,
    baseline: String,
    sp_n1: String,
    sp_base: String,
    sp_p1: String,
    sp_p2: String,
    sp_p3: String,
    sp_p4: String,
    sp_p5: String,
    sp_p6: String,
}

struct TypeRow {
    name: &'static str,
    size: String,
    tracking: String,
    // Line box edges (em) that make Typst's box the full line-height with `leading: 0` — so a block's
    // height is `N × line-height` (half-leading above the first line + below the last), exactly like
    // CSS. Replaces the earlier 1em-box + inter-line leading, which left blocks a half-leading short.
    top_edge: String,
    bottom_edge: String,
    word: String,
    fill: String,
    extras: String,
}

/// Trim a float to 3 decimals and drop trailing zeros — clean Typst literals.
fn f(x: f64) -> String {
    let r = (x * 1000.0).round() / 1000.0;
    let r = if r == 0.0 { 0.0 } else { r }; // normalise -0.0 → 0
    format!("{r}")
}

/// The resolved font for a role's [`FontRole`].
fn font_for(fr: FontRole, cfg: &Config) -> &ResolvedFont {
    match fr {
        FontRole::Body => &cfg.body,
        FontRole::Heading => &cfg.heading,
        FontRole::Code => &cfg.code,
    }
}

/// Resolve a colour token (a base swatch or a semantic alias) to a hex string for `theme` (light).
/// `transparent` (and unknown) resolve to `None`.
fn resolve_color(theme: Theme, name: &str) -> Option<String> {
    if name == "transparent" {
        return None;
    }
    if let Some(c) = Color::ALL.iter().find(|c| c.id() == name) {
        return Some(theme.light(*c).to_string());
    }
    if let Some(s) = Semantic::ALL.iter().find(|s| s.id() == name) {
        return resolve_color(theme, s.base());
    }
    None
}

/// A spacing token id (`"baseline"` or a [`Multiplier`] id like `"p4"`) → absolute pt, from the
/// rhythm `unit` — the same `formula::spacing(unit, multiplier)` cascade-css projects.
fn space_pt(tok: Option<&str>, baseline: f64, unit: f64) -> Option<f64> {
    tok.map(|t| match t {
        "baseline" => baseline,
        id => Multiplier::ALL
            .iter()
            .find(|m| m.id() == id)
            .map(|m| formula::spacing::<f64>(unit, m.factor()))
            .unwrap_or(0.0),
    })
}

/// Project one role into a baked `typ` row. `Root` is the page/base, not a component.
fn type_row(role: Role, cfg: &Config, baseline: f64, unit: f64) -> Option<TypeRow> {
    if role == Role::Root {
        return None;
    }
    let n = cfg.scale.n() as f64;
    let ratio = cfg.scale.ratio();
    let ln_ratio = ratio.ln();
    let step = role.step().unwrap_or(0) as f64;
    let fr = role.font().unwrap_or(FontRole::Body);
    let font = font_for(fr, cfg);

    // Scale size, floored at the readability minimum (exactly as the CSS output clamps it).
    let mut size = (BASE_PT * formula::size_factor::<f64>(step, n, ratio)).max(SIZE_MIN_PT);
    // Code optically matched to the body's x-height (print analogue of CSS `font-size-adjust`):
    // a mono face's larger x-height would otherwise read oversized beside the text face.
    if fr == FontRole::Code {
        size *= cfg.body.x_height / cfg.code.x_height;
    }

    let lead0 = formula::lead0::<f64>(font.leading_base, MEASURE as f64, font.x_height);
    let lead_ratio =
        formula::leading::<f64>(step, n, ln_ratio, lead0, LEADING_CLAMP.0, LEADING_CLAMP.1);
    // Line box = the full line-height (`lead_ratio` em), with `leading: 0`. The em-box (baseline
    // ~0.25em above the box bottom, per typical descent) is centred in it, so the leading appears as
    // half above the first line and half below the last — CSS's box model. Box height = top - bottom
    // = lead_ratio, so a block is `N × line-height` tall, matching CSS exactly.
    let top_edge = (lead_ratio + 1.0) / 2.0 - 0.25;
    let bottom_edge = -((lead_ratio - 1.0) / 2.0 + 0.25);
    let tracking = formula::tracking::<f64>(step, n, ln_ratio, font.k_tracking, TRACKING_CLAMP);
    let word = formula::word_space::<f64>(step, n, ln_ratio, font.word_space, WORD_SPACE_K);

    let mut extras = String::new();
    if let Some(w) = role.weight() {
        extras += &format!(", weight: {w}");
    }
    if role.italic() {
        extras += ", style: \"italic\"";
    }
    // Only emit a margin when it's positive: a `0pt` margin would KILL Typst's natural block flow
    // (CSS gets away with margin:0 via line-height; Typst does not). Omitted → default paragraph flow.
    if let Some(a) = space_pt(role.space_before(), baseline, unit).filter(|&x| x > 0.001) {
        extras += &format!(", above: {}pt", f(a));
    }
    if let Some(b) = space_pt(role.space_after(), baseline, unit).filter(|&x| x > 0.001) {
        extras += &format!(", below: {}pt", f(b));
    }

    let fill = resolve_color(cfg.theme, role_color(role).fg.unwrap_or("fg"))
        .unwrap_or_else(|| cfg.theme.light(Color::Fg).to_string());

    Some(TypeRow {
        name: role.id(),
        size: f(size),
        tracking: f(tracking),
        top_edge: f(top_edge),
        bottom_edge: f(bottom_edge),
        word: f(word),
        fill,
        extras,
    })
}

/// The Typst package manifest — makes `dist/typst` importable as `@local/cascade`.
fn manifest() -> String {
    format!(
        "[package]\nname = \"cascade\"\nversion = \"{VERSION}\"\nentrypoint = \"cascade.typ\"\n\
         description = \"cascade typography, baked for Typst (print).\"\n"
    )
}

impl Renderer for Typst {
    fn name(&self) -> &'static str {
        "typst"
    }

    /// Typst resolves its own fallbacks, so the family name alone is the contract (unlike CSS's
    /// explicit stack).
    fn font_family(&self, font: &ResolvedFont) -> String {
        font.family.clone()
    }

    fn render(&self, cfg: &Config) -> Vec<Output> {
        let n = cfg.scale.n() as f64;
        let ratio = cfg.scale.ratio();
        let ln_ratio = ratio.ln();

        // Rhythm: unit = base × ratio (same as cascade-css); baseline = body step-0 line height,
        // NOT grid-snapped — the CSS `--cr-baseline` is the raw `formula::baseline`, so we match it.
        let unit = BASE_PT * RHYTHM_UNIT_RATIO;
        let lead0 = formula::lead0::<f64>(cfg.body.leading_base, MEASURE as f64, cfg.body.x_height);
        let lr0 = formula::leading::<f64>(0.0, n, ln_ratio, lead0, LEADING_CLAMP.0, LEADING_CLAMP.1);
        let baseline = formula::baseline::<f64>(BASE_PT, lr0);
        let sp = |m: Multiplier| f(formula::spacing::<f64>(unit, m.factor()));

        let rows: Vec<TypeRow> =
            Role::ALL.iter().filter_map(|&role| type_row(role, cfg, baseline, unit)).collect();

        // Reading measure: MEASURE chars × the body face's mean advance (em) × base — the SAME
        // copyfitting cascade-css bakes into `--cf-measure-inline`. Centre it on us-letter by
        // widening the horizontal margin, so a Typst line is the same length as a CSS line.
        let measure_pt = MEASURE as f64 * cfg.body.avg_advance * BASE_PT;
        let margin_x = ((US_LETTER_PT - measure_pt) / 2.0).max(72.0);

        let color = |name: &str| resolve_color(cfg.theme, name).unwrap_or_default();
        let quote_bg_expr = match resolve_color(cfg.theme, "quote-bg") {
            Some(hex) => format!("rgb(\"{hex}\")"),
            None => "none".to_string(),
        };

        let lib = CascadeTyp {
            body_family: cfg.body.family.clone(),
            heading_family: cfg.heading.family.clone(),
            code_family: cfg.code.family.clone(),
            fg: color("fg"),
            fg_muted: color("fg-muted"),
            fg_subtle: color("fg-subtle"),
            bg: color("bg"),
            bg_subtle: color("bg-subtle"),
            rule: color("rule"),
            accent: color("accent"),
            accent_rule: color("accent-rule"),
            link: color("link"),
            code_fg: color("code-fg"),
            code_bg: color("code-bg"),
            quote_rule: color("quote-rule"),
            quote_bg_expr,
            rows,
            margin_x: f(margin_x),
            unit: f(unit),
            baseline: f(baseline),
            sp_n1: sp(Multiplier::N1),
            sp_base: sp(Multiplier::Base),
            sp_p1: sp(Multiplier::P1),
            sp_p2: sp(Multiplier::P2),
            sp_p3: sp(Multiplier::P3),
            sp_p4: sp(Multiplier::P4),
            sp_p5: sp(Multiplier::P5),
            sp_p6: sp(Multiplier::P6),
        };

        vec![
            Output { path: "cascade.typ".into(), body: lib.render().expect("render cascade.typ") },
            Output { path: "typst.toml".into(), body: manifest() },
        ]
    }

    fn verify(&self, outputs: &[Output]) -> Vec<String> {
        // Compile the output with Typst's own tooling in a scratch dir; report its diagnostics.
        let dir = std::env::temp_dir().join(format!("cascade-typst-verify-{}", std::process::id()));
        if let Err(e) = std::fs::create_dir_all(&dir) {
            return vec![format!("could not create scratch dir {}: {e}", dir.display())];
        }
        for o in outputs {
            if let Err(e) = std::fs::write(dir.join(&o.path), &o.body) {
                return vec![format!("could not write {}: {e}", o.path)];
            }
        }
        // A probe document exercising every bound element (headings, emphasis, link, code, lists,
        // quote, footnote). sidenotes:false → native footnotes, so verify needs no external package.
        let probe = "#import \"cascade.typ\": make\n\
             #let l = make()\n\
             #show: l.page\n\
             #show: l.markup\n\
             = Heading one\n\
             == Heading two\n\
             Body with *bold*, _italic_, a #link(\"https://example.com\")[link], `inline`, and a \
             note.#footnote[A footnote.]\n\n\
             ```\ncode block\n```\n\n\
             - item\n+ step\n\n\
             #quote(attribution: [Someone])[A quotation.]\n\
             #(l.divider)()\n";
        let probe_path = dir.join("_verify.typ");
        if let Err(e) = std::fs::write(&probe_path, probe) {
            return vec![format!("could not write probe: {e}")];
        }

        let out = Command::new("typst")
            .arg("compile")
            .arg("--root")
            .arg(&dir)
            .arg(&probe_path)
            .arg(dir.join("_verify.pdf"))
            .output();

        let problems = match out {
            Ok(o) if o.status.success() => Vec::new(),
            Ok(o) => String::from_utf8_lossy(&o.stderr)
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.to_string())
                .collect(),
            Err(e) => vec![format!("could not run `typst` (is it installed?): {e}")],
        };
        let _ = std::fs::remove_dir_all(&dir);
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cascade::renderer::Config;

    fn outputs() -> Vec<Output> {
        Typst.render(&Config::default())
    }

    fn lib() -> String {
        outputs().into_iter().find(|o| o.path == "cascade.typ").expect("cascade.typ").body
    }

    #[test]
    fn renders_library_and_manifest() {
        let o = outputs();
        let paths: Vec<&str> = o.iter().map(|x| x.path.as_str()).collect();
        assert!(paths.contains(&"cascade.typ"), "missing cascade.typ");
        assert!(paths.contains(&"typst.toml"), "missing typst.toml");
    }

    #[test]
    fn bakes_the_default_config_from_the_spec() {
        let s = lib();
        // families resolved from the spec's default_fonts
        assert!(s.contains(r#"body: "Inter""#));
        assert!(s.contains(r#"heading: "Lora""#));
        assert!(s.contains(r#"code: "IBM Plex Mono""#));
        // paper light foreground swatch, baked
        assert!(s.contains(r##"rgb("#171717")"##));
        // body reading size = the SPEC's BASE_PT (not a renderer-local constant)
        assert!(s.contains(&format!("body: (size: {}pt", BASE_PT)));
        // heading weight carried through from the role
        assert!(s.contains("weight: 700"));
        // the public entry point
        assert!(s.contains("#let make("));
    }

    #[test]
    fn applies_the_css_parity_mappings() {
        let s = lib();
        // line box = the role's full line-height via per-role te/be, with leading:0 — so block height
        // is N×line-height like CSS (not a half-leading short). Body ratio 1.413 → te 0.957/be -0.457.
        assert!(s.contains("te: 0.957em, be: -0.457em"), "body edges");
        assert!(s.contains("top-edge: t.te") && s.contains("bottom-edge: t.be"));
        assert!(s.contains("set par(leading: 0pt"));
        // word-space is an ADDITION (100% + …), matching CSS `word-spacing`, so code (ws 0) keeps spaces
        assert!(s.contains("spacing: 100% +"));
        // measure centred via a computed horizontal page margin
        assert!(s.contains("margin: (x:"));
    }

    #[test]
    fn verify_compiles_when_typst_is_present() {
        // Gated: skip where the `typst` CLI isn't installed (the check is real, not a no-op).
        if std::process::Command::new("typst").arg("--version").output().is_err() {
            return;
        }
        let problems = Typst.verify(&outputs());
        assert!(problems.is_empty(), "typst verify reported: {problems:?}");
    }
}
