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
    role_color, Color, FontRole, Multiplier, Role, RoleKind, Semantic, Theme, BASE_PT, CODE_SCALE,
    LEADING_CLAMP, MEASURE, RHYTHM_UNIT_RATIO, SIZE_MIN_PT, TRACKING_CLAMP, VERSION, WORD_SPACE_K,
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
    /// The role's baked Typst dict body (the text inside `(…)`). A SIZED role carries an absolute
    /// `size`/`tracking`/line-box `te`/`be`/`spacing`; an inline DECORATION carries only its decoration
    /// (weight / style / colour), inheriting size & leading from context — see [`type_row`].
    body: String,
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

/// True for an inline DECORATION — a role that carries no scale step (`strong`, `emphasis`, `link`,
/// inline `code`). CSS gives these NO `font-size`: they inherit the surrounding size (and leading &
/// tracking), applying only their one decoration. `text-1…5` / `small` / `sidenote` are inline too
/// but carry a step, so they ARE sized spans, not decorations. `code-block`/`quote` are blocks.
fn is_decoration(role: Role) -> bool {
    role.kind() == RoleKind::Inline && role.step().is_none()
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

    let fill = resolve_color(cfg.theme, role_color(role).fg.unwrap_or("fg"))
        .unwrap_or_else(|| cfg.theme.light(Color::Fg).to_string());

    // Inline decorations INHERIT size/leading/tracking (exactly like CSS, where `strong`/`em`/`a`/
    // inline-`code` set no `font-size`). Baking an absolute size would make e.g. italic inside a 9pt
    // footnote snap back to the 11pt body size. So emit ONLY the decoration.
    if is_decoration(role) {
        let body = if fr == FontRole::Code {
            // Code still matches the body's x-height — but as a RELATIVE em factor (the print analogue
            // of CSS `font-size-adjust`), so it scales with whatever context it sits in, times the
            // spec's `CODE_SCALE` (mono reads heavier than proportional type at equal apparent size).
            // Tracking and word-space are reset (CSS `letter-spacing: 0; word-spacing: normal`).
            let xh = cfg.body.x_height / cfg.code.x_height * CODE_SCALE;
            format!("size: {}em, tracking: 0em, spacing: 0em", f(xh))
        } else {
            let mut d = Vec::new();
            if let Some(w) = role.weight() {
                d.push(format!("weight: {w}"));
            }
            if role.italic() {
                d.push("style: \"italic\"".to_string());
            }
            if role.underline() {
                d.push(format!("fill: rgb(\"{fill}\")"));
            }
            d.join(", ")
        };
        return Some(TypeRow { name: role.id(), body });
    }

    // Scale size, floored at the readability minimum (exactly as the CSS output clamps it).
    let mut size = (BASE_PT * formula::size_factor::<f64>(step, n, ratio)).max(SIZE_MIN_PT);
    // Code (block: `pre`) optically matched to the body's x-height (print analogue of CSS
    // `font-size-adjust`), then scaled by the spec's `CODE_SCALE` so a code block reads as clearly
    // secondary and ordinary command lines stay inside the reading measure.
    if fr == FontRole::Code {
        size *= cfg.body.x_height / cfg.code.x_height * CODE_SCALE;
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

    let body = format!(
        "size: {}pt, tracking: {}em, te: {}em, be: {}em, spacing: {}em, fill: rgb(\"{}\"){}",
        f(size),
        f(tracking),
        f(top_edge),
        f(bottom_edge),
        f(word),
        fill,
        extras,
    );
    Some(TypeRow { name: role.id(), body })
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

    /// The line of `s` whose first non-blank token is `prefix` (e.g. `"body: ("`, `"unit: "`).
    fn line_starting<'a>(s: &'a str, prefix: &str) -> &'a str {
        s.lines()
            .find(|l| l.trim_start().starts_with(prefix))
            .unwrap_or_else(|| panic!("no line starting `{prefix}`"))
    }

    /// The numeric literal following `"{key}: "` on `line` — unit suffix (`pt`/`em`) ignored. The
    /// key must sit at a field boundary (preceded by `(` or space) so `te` doesn't match `quote`.
    fn num(line: &str, key: &str) -> f64 {
        let needle = format!("{key}: ");
        let at = line
            .match_indices(&needle)
            .find(|&(i, _)| i > 0 && matches!(line.as_bytes()[i - 1], b'(' | b' '))
            .map(|(i, _)| i)
            .unwrap_or_else(|| panic!("no `{key}` in: {line}"));
        let rest = &line[at + needle.len()..];
        let lit: String =
            rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect();
        lit.parse().unwrap_or_else(|_| panic!("bad `{key}` in: {line}"))
    }

    /// THE PARITY ANCHOR. cascade-css proves its `calc()` output folds to the shared `formula::f64`
    /// (`css_projection_folds_to_the_f64_projection`). This proves the Typst BAKED output equals the
    /// SAME `formula::f64` — for every SIZED role's size / tracking / word-space / line-height, the
    /// rhythm unit / baseline / spacing tokens, and the reading measure. Both renderers pinned to one
    /// f64 ground truth ⇒ CSS and Typst cannot diverge on typography; only on render-specific mechanism
    /// (CSS's fluid clamp & `font-size-adjust` vs Typst's fixed pt & baked size). Reads the rendered
    /// artifact (not the private helpers), so a wrong constant / dropped floor / bad edge model here
    /// diverges from the spec-derived truth and fails.
    ///
    /// Inline DECORATIONS (`strong`/`emphasis`/`link`/inline-`code`) are checked the OTHER way: CSS
    /// sets them no `font-size`, so they must carry NO absolute pt dimension and inherit — otherwise
    /// e.g. italic inside a 9pt footnote snaps back to the 11pt body size.
    #[test]
    fn baked_output_equals_the_f64_projection_so_css_and_typst_agree() {
        let cfg = Config::default();
        let s = lib();
        let n = cfg.scale.n() as f64;
        let ratio = cfg.scale.ratio();
        let ln = ratio.ln();
        // Baked literals are rounded to 3 decimals by `f()`; allow that plus fp slack.
        let close = |got: f64, want: f64, what: &str| {
            assert!(
                (got - want).abs() <= 3e-3 * (1.0 + want.abs()),
                "{what}: baked {got}, f64 projection {want}"
            );
        };

        // ── per-role type table ──────────────────────────────────────────────
        for &role in Role::ALL.iter().filter(|&&r| r != Role::Root) {
            let line = line_starting(&s, &format!("{}: (", role.id()));
            let fr = role.font().unwrap_or(FontRole::Body);
            let id = role.id();

            // Inline decorations INHERIT size (CSS sets them no font-size): they must carry NO absolute
            // pt dimension. `code` keeps an x-height match, but RELATIVE (em) so it scales with context.
            // The decoration set is named EXPLICITLY here (not via the renderer's own `is_decoration`)
            // so this test independently pins the right answer — if the renderer regresses to baking an
            // absolute size on any of them, the `pt` assertion fires.
            if matches!(role, Role::Strong | Role::Emphasis | Role::Link | Role::Code) {
                assert!(
                    !line.contains("pt"),
                    "{id} is an inline decoration and must inherit size (no absolute pt): {line}"
                );
                if fr == FontRole::Code {
                    close(
                        num(line, "size"),
                        cfg.body.x_height / cfg.code.x_height * CODE_SCALE,
                        &format!("{id} x-height factor (relative em)"),
                    );
                }
                continue;
            }

            let step = role.step().unwrap_or(0) as f64;
            let font = font_for(fr, &cfg);

            // size: base × scale, floored at the minimum; code (block) matched to body x-height × CODE_SCALE.
            let mut size = (BASE_PT * formula::size_factor::<f64>(step, n, ratio)).max(SIZE_MIN_PT);
            if fr == FontRole::Code {
                size *= cfg.body.x_height / cfg.code.x_height * CODE_SCALE;
            }
            close(num(line, "size"), size, &format!("{id} size"));
            close(
                num(line, "tracking"),
                formula::tracking::<f64>(step, n, ln, font.k_tracking, TRACKING_CLAMP),
                &format!("{id} tracking"),
            );
            close(
                num(line, "spacing"),
                formula::word_space::<f64>(step, n, ln, font.word_space, WORD_SPACE_K),
                &format!("{id} word-space"),
            );
            // Line-box height (te − be) must equal the role's line-height — i.e. CSS `line-height`.
            let lead0 = formula::lead0::<f64>(font.leading_base, MEASURE as f64, font.x_height);
            let lead = formula::leading::<f64>(step, n, ln, lead0, LEADING_CLAMP.0, LEADING_CLAMP.1);
            close(num(line, "te") - num(line, "be"), lead, &format!("{id} line-height (te−be)"));
        }

        // ── rhythm ───────────────────────────────────────────────────────────
        let unit = BASE_PT * RHYTHM_UNIT_RATIO;
        close(num(line_starting(&s, "unit: "), "unit"), unit, "rhythm unit");
        let bl0 = formula::lead0::<f64>(cfg.body.leading_base, MEASURE as f64, cfg.body.x_height);
        let blr = formula::leading::<f64>(0.0, n, ln, bl0, LEADING_CLAMP.0, LEADING_CLAMP.1);
        close(
            num(line_starting(&s, "baseline: "), "baseline"),
            formula::baseline::<f64>(BASE_PT, blr),
            "rhythm baseline",
        );
        let sp = line_starting(&s, "spacing: (");
        for m in Multiplier::ALL {
            close(num(sp, m.id()), formula::spacing::<f64>(unit, m.factor()), &format!("spacing {}", m.id()));
        }

        // ── reading measure (page geometry) ──────────────────────────────────
        // Same copyfit width cascade-css bakes into `--cf-measure-inline`, centred on us-letter.
        let measure = MEASURE as f64 * cfg.body.avg_advance * BASE_PT;
        let margin_x = ((US_LETTER_PT - measure) / 2.0).max(72.0);
        let mline = s.lines().find(|l| l.contains("margin: (x:")).expect("margin line");
        close(num(mline, "x"), margin_x, "measure margin_x");
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
