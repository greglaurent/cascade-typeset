//! cascade-css — a `Renderer` implementation for CSS.
//!
//! It CONSUMES the spec's types (`Font`, `Color`, `ScalePreset`, `Multiplier`, …) and
//! produces CSS via its own templates (`templates/*.css`). It DEFINES nothing typographic —
//! it only supplies CSS *behavior*: the fallback stacks, the `--ct-`/`--cs-`/`--cr-`/`--cf-`
//! conventions, the fluid base size, the grid-unit fraction, the `pow()`/`clamp()` formulas.
//! The spec stays renderer-agnostic; this crate is "what it looks like in CSS."
use askama::Template;
use cascade::formula::{self, Val};
use cascade::renderer::{Config, FontDelivery, Output, Renderer, ResolvedFont};
use cascade::{
    role_color, Category, Color, Font, FontRole, Multiplier, Role, ScalePreset, Semantic, BASE_PT,
    LEADING_CLAMP, MEASURE, RHYTHM_UNIT_RATIO, SIZE_MIN_PT, STEPS_MAX, STEPS_MIN, TRACKING_CLAMP,
    WORD_SPACE_K,
};

/// cascade-css's default base body size, in points, driven from the user's perspective
// The reading size (`BASE_PT`), the rhythm-unit ratio (`RHYTHM_UNIT_RATIO`), and the readability
// floor (`SIZE_MIN_PT`) are SPEC-OWNED (`cascade::*`), NOT renderer choices — the same values
// cascade-typst reads, so the two formats cannot diverge on size / rhythm / floor. cascade-css only
// adds its own CONTEXT on top: it wraps `BASE_PT` in a fluid `clamp()` for the screen.
/// cascade-css's own fluid parameters (screen-context behavior): pt→rem, the viewport window the
/// base grows across, and how much it grows. These are legitimately renderer-local (CSS-only).
const PT_PER_REM: f64 = 12.0; // 12pt = 1rem = 16px at 96dpi
const FLUID_MIN_VW: f64 = 20.0; // rem (~320px)
const FLUID_MAX_VW: f64 = 80.0; // rem (~1280px)
const FLUID_GROWTH: f64 = 1.2;

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

// ── Calc: cascade-css's projection of a spec formula into a live CSS math expression ──
//
// It implements `cascade::formula::Val`, so it receives EVERY spec formula unchanged and
// projects it here. Its operators only ever build `calc()`/`pow()`/`clamp()` strings — a
// formula can never collapse to a literal in CSS, so the browser stays the evaluator and the
// output re-resolves live on a class swap. cascade-css supplies only this projection + the
// variable names it binds in; it cannot deviate from the spec's calculation.
struct Calc {
    s: String,
    prec: u8, // 3 = atom/function (var, number, pow(), clamp(), log()); 2 = `* /`; 1 = `+ -`
}
impl Calc {
    /// Bind a typed [`Var`] key into a reactive expression — `var(--x)`, prec = atom.
    fn of(v: Var) -> Self {
        Calc { s: v.get(), prec: 3 }
    }
    /// As a standalone CSS value — wraps bare arithmetic in `calc()`, leaves a function/atom as-is.
    fn value(&self) -> String {
        if self.prec == 3 { self.s.clone() } else { format!("calc({})", self.s) }
    }
    /// `self <op> rhs` at precedence `prec`, parenthesising only where the grammar demands it:
    /// the left side when it binds looser, the right side when it binds looser-or-equal (all four
    /// operators are left-associative, so an equal-precedence right operand must be grouped).
    fn bin(self, op: &str, prec: u8, rhs: Calc) -> Self {
        let l = if self.prec < prec { format!("({})", self.s) } else { self.s };
        let r = if rhs.prec <= prec { format!("({})", rhs.s) } else { rhs.s };
        Calc { s: format!("{l} {op} {r}"), prec }
    }
}
impl Clone for Calc {
    fn clone(&self) -> Calc {
        Calc { s: self.s.clone(), prec: self.prec }
    }
}
impl std::ops::Add for Calc {
    type Output = Calc;
    fn add(self, o: Calc) -> Calc {
        self.bin("+", 1, o)
    }
}
impl std::ops::Sub for Calc {
    type Output = Calc;
    fn sub(self, o: Calc) -> Calc {
        self.bin("-", 1, o)
    }
}
impl std::ops::Mul for Calc {
    type Output = Calc;
    fn mul(self, o: Calc) -> Calc {
        self.bin("*", 2, o)
    }
}
impl std::ops::Div for Calc {
    type Output = Calc;
    fn div(self, o: Calc) -> Calc {
        self.bin("/", 2, o)
    }
}
impl std::ops::Neg for Calc {
    type Output = Calc;
    fn neg(self) -> Calc {
        // unary minus as `-1 * x` at multiply precedence; group x if it binds looser-or-equal.
        let r = if self.prec <= 2 { format!("({})", self.s) } else { self.s };
        Calc { s: format!("-1 * {r}"), prec: 2 }
    }
}
impl Val for Calc {
    fn lit(x: f64) -> Calc {
        Calc { s: format!("{x}"), prec: 3 }
    }
    fn pow(self, e: Calc) -> Calc {
        Calc { s: format!("pow({}, {})", self.s, e.s), prec: 3 }
    }
    fn ln(self) -> Calc {
        Calc { s: format!("log({})", self.s), prec: 3 }
    }
    fn clamp(self, lo: Calc, hi: Calc) -> Calc {
        Calc { s: format!("clamp({}, {}, {})", lo.s, self.s, hi.s), prec: 3 }
    }
    fn round(self, step: Calc) -> Calc {
        Calc { s: format!("round({}, {})", self.s, step.s), prec: 3 }
    }
}

/// Minimal standard base64 (padded) — for embedding font bytes in a `@font-face` `data:` URI. Avoids
/// a dependency for the one place cascade-css needs it (external-font delivery).
fn base64(data: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        out.push(A[(n >> 18 & 63) as usize] as char);
        out.push(A[(n >> 12 & 63) as usize] as char);
        out.push(if c.len() > 1 { A[(n >> 6 & 63) as usize] as char } else { '=' });
        out.push(if c.len() > 2 { A[(n & 63) as usize] as char } else { '=' });
    }
    out
}

/// A scale step's label: `0`, `n{k}` (negative), `p{k}` (positive). Shared by every var name.
fn step_label(i: i32) -> String {
    if i == 0 {
        "0".to_string()
    } else if i < 0 {
        format!("n{}", -i)
    } else {
        format!("p{i}")
    }
}

// ── Var: a typed CSS custom-property KEY — the single way to name (`def`) or reference (`get`) a
// cascade custom property. Built from spec types (a scale step, a FontRole, a colour/space token),
// so a reference to a property the renderer doesn't define can't be constructed, and the naming
// convention lives here once. Output is REACTIVE: `get()` is `var(--x)`, a live custom-property
// reference the browser resolves on every class swap — the type only constrains WHICH property. ──
struct Var(String);
impl Var {
    /// Definition side: `--x` (used as `--x: <value>`).
    fn def(&self) -> String {
        format!("--{}", self.0)
    }
    /// Reference side: `var(--x)` — a live, reactive reference.
    fn get(&self) -> String {
        format!("var(--{})", self.0)
    }
    fn size(step: i32) -> Var {
        Var(format!("cs-size-{}", step_label(step)))
    }
    fn lead(fr: FontRole, step: i32) -> Var {
        Var(format!("cf-{}lead-{}", Self::opt(fr), step_label(step)))
    }
    fn track(fr: FontRole, step: i32) -> Var {
        Var(format!("cf-{}track-{}", Self::opt(fr), step_label(step)))
    }
    fn ws(fr: FontRole, step: i32) -> Var {
        Var(format!("cf-{}ws-{}", Self::opt(fr), step_label(step)))
    }
    fn family(fr: FontRole) -> Var {
        Var(format!("cf-family-{}", Self::slug(fr)))
    }
    fn size_min() -> Var {
        Var("cf-size-min".to_string())
    }
    fn measure() -> Var {
        Var("cf-measure".to_string())
    }
    /// The body font's frequency-weighted average character advance (em) — the copyfitting factor
    /// that turns the character `measure` into a real width. Reactive: `bundle-*` redefines it, so
    /// the measure re-resolves live when the body typeface changes.
    fn avg_advance() -> Var {
        Var("cf-aw".to_string())
    }
    /// The measure projected to a reading-column WIDTH (`measure × avg_advance em`, overflow-guarded),
    /// single-sourced so every layout mode (centered, banded) binds the same value.
    fn measure_inline() -> Var {
        Var("cf-measure-inline".to_string())
    }
    // scale scalars
    fn base() -> Var {
        Var("cs-base".to_string())
    }
    fn ratio() -> Var {
        Var("cs-ratio".to_string())
    }
    fn n() -> Var {
        Var("cs-n".to_string())
    }
    fn ln_ratio() -> Var {
        Var("cs-ln-ratio".to_string())
    }
    // shared optical scalars
    fn lmin() -> Var {
        Var("cf-lmin".to_string())
    }
    fn lmax() -> Var {
        Var("cf-lmax".to_string())
    }
    fn tc() -> Var {
        Var("cf-tc".to_string())
    }
    fn kws() -> Var {
        Var("cf-kws".to_string())
    }
    // rhythm unit
    fn unit() -> Var {
        Var("cr-unit".to_string())
    }
    // per-role optical scalars (body/code share the body set; heading uses `h-`)
    fn xh(fr: FontRole) -> Var {
        Var(format!("cf-{}xh", Self::opt(fr)))
    }
    // the DOCUMENT body x-height ratio, captured at the root as a concrete value so it survives
    // the code subtree's `--cf-xh` override (which swaps in the mono metrics for leading). Drives
    // `font-size-adjust` on code, optically matching the mono's x-height to the body's. Because it
    // is `var(--cf-xh)` resolved on `.cascade`, a `bundle-*` switch re-derives it reactively.
    fn xh_doc() -> Var {
        Var("cf-xh-doc".to_string())
    }
    // Code-role optical, held in a dedicated var set (not the shared body `--cf-xh` etc.) so a
    // `.code-<font>` switch can override it independently of the body. The code subtree binds
    // `--cf-xh: var(--cf-code-xh)` (and kt/lb), so the leading/tracking formulas re-resolve from
    // whichever code face is selected — the exact mirror of `.bundle-*` (body) and `.heading-*`.
    fn code_xh() -> Var {
        Var("cf-code-xh".to_string())
    }
    fn code_kt() -> Var {
        Var("cf-code-kt".to_string())
    }
    fn code_lb() -> Var {
        Var("cf-code-lb".to_string())
    }
    fn kt(fr: FontRole) -> Var {
        Var(format!("cf-{}kt", Self::opt(fr)))
    }
    fn lb(fr: FontRole) -> Var {
        Var(format!("cf-{}lb", Self::opt(fr)))
    }
    fn bws(fr: FontRole) -> Var {
        Var(format!("cf-{}bws", Self::opt(fr)))
    }
    fn lead0(fr: FontRole) -> Var {
        Var(format!("cf-{}lead0", Self::opt(fr)))
    }
    /// A colour reference by token name (a `Color` or `Semantic` id — spec-validated to exist).
    fn color(token: &str) -> Var {
        Var(format!("ct-{token}"))
    }
    /// A rhythm spacing reference: `"baseline"` → the baseline, else a `Multiplier` id (`p4`, `0`).
    fn space(token: &str) -> Var {
        if token == "baseline" {
            Var("cr-baseline".to_string())
        } else {
            Var(format!("cr-space-{token}"))
        }
    }
    /// Body and code share the body optical vars; headings use the `h-` set.
    fn opt(fr: FontRole) -> &'static str {
        match fr {
            FontRole::Heading => "h-",
            _ => "",
        }
    }
    fn slug(fr: FontRole) -> &'static str {
        match fr {
            FontRole::Body => "body",
            FontRole::Heading => "heading",
            FontRole::Code => "code",
        }
    }
}

// ── templates (cascade-css owns these; they consume the spec) ────────────────────
#[derive(Template)]
#[template(path = "theme.css", escape = "none")]
struct ThemeCss {
    default_id: &'static str,
    colors: Vec<ColorRow>, // the default palette (cfg.theme) — emitted at :root
    palettes: Vec<PaletteBlock>, // EVERY palette, scoped by [data-palette] for runtime switching
    semantics: Vec<SemanticRow>,
}
struct PaletteBlock {
    id: &'static str,
    colors: Vec<ColorRow>,
}
struct ColorRow {
    id: &'static str,
    light: &'static str,
    dark: &'static str,
}
struct SemanticRow {
    id: &'static str,
    value: String, // `var(--ct-<base>)`, or a literal like `transparent`
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
    name: String, // Var::size(step).def()
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
    unit: String,
    spaces: Vec<SpaceRow>,
    baseline: String,
}
struct SpaceRow {
    name: String, // Var::space(token).def()
    expr: String, // formula::spacing(unit, multiplier), projected
}

#[derive(Template)]
#[template(path = "typefaces.css", escape = "none")]
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
    avg_advance: f64,
}

#[derive(Template)]
#[template(path = "optical.css", escape = "none")]
struct FontCss {
    body_family: String,
    heading_family: String,
    code_family: String,
    font_serif: String,
    font_sans: String,
    font_mono: String,
    xh: f64,
    kt: f64,
    lb: f64,
    bws: f64,
    h_xh: f64,
    h_kt: f64,
    h_lb: f64,
    h_bws: f64,
    // code optical (from the code font) — the root default the code subtree binds and a `.code-*`
    // switch overrides. word-space/advance omitted: code sets word-spacing:normal and has no measure.
    code_xh: f64,
    code_kt: f64,
    code_lb: f64,
    kws: f64,
    tc: f64,
    lmin: f64,
    lmax: f64,
    measure: u32,
    aw: f64, // body font's mean character advance (em) — the copyfitting factor for the measure
    size_min: String,
    // Optical model, PROJECTED from the spec (`cascade::formula::*` over `Calc`) — not re-encoded
    // here. Each string is a `formula::*` result with CSS-var bindings, so a class swap re-resolves
    // it live in the browser.
    lead0: String,
    h_lead0: String,
    tracks: Vec<Row>,
    leads: Vec<Row>,
    word_spaces: Vec<Row>,
    h_tracks: Vec<Row>,
    h_leads: Vec<Row>,
    h_word_spaces: Vec<Row>,
    profiles: Vec<ProfileClassRow>,
}
struct Row {
    name: String, // the full custom-property definition name, from Var::…def()
    value: String,
}
struct ProfileClassRow {
    id: &'static str,
    xh: f64,
    kt: f64,
    lb: f64,
    bws: f64,
    aw: f64,
}

// sidenotes.css — the opt-in Tufte notes FEATURE. Presentation only: note typography comes from
// the sidenote/marginnote roles (layout.css). The cascade tokens it uses arrive via `Var`.
#[derive(Template)]
#[template(path = "sidenotes.css", escape = "none")]
struct SidenotesCss {
    gap: String,      // space above a note (rhythm)
    indent: String,   // aside indent (rhythm)
    baseline: String, // one body line (rhythm)
    marker: String,   // marker colour (theme link)
    rule: String,     // disclosure border colour (theme rule)
    measure: String,  // the single-sourced reading measure (banded caps its text column at it)
}

impl Renderer for Css {
    fn name(&self) -> &'static str {
        "css"
    }

    /// The fallback contract: the plain font name (what a user expects and can override with
    /// whatever they have installed) + this renderer's generic category stack. The sane-default
    /// x-height normalization is applied to that same name by faces.css's `@font-face`.
    fn font_family(&self, font: &ResolvedFont) -> String {
        format!("\"{}\", {}", font.family, Self::fallbacks(font.category))
    }

    fn render(&self, cfg: &Config) -> Vec<Output> {
        // theme.css — the spec's Color palette + the Semantic uses (each alias → a CSS var or a
        // literal). The role→colour binding is combined into the layout, not here.
        // Every palette is projected reactively: cfg.theme is the :root default, and ALL palettes
        // ship scoped by [data-palette] so markup switches them live — orthogonal to the
        // [data-theme] light/dark axis. (A print renderer, having no runtime, bakes cfg.theme
        // alone; this is the CSS projection of the same Config field.)
        let palette_rows = |t: cascade::Theme| -> Vec<ColorRow> {
            Color::ALL
                .into_iter()
                .map(|c| ColorRow { id: c.id(), light: t.light(c), dark: t.dark(c) })
                .collect()
        };
        let colors = palette_rows(cfg.theme);
        let palettes = cascade::Theme::ALL
            .into_iter()
            .map(|t| PaletteBlock { id: t.id(), colors: palette_rows(t) })
            .collect();
        let semantics = Semantic::ALL
            .into_iter()
            .map(|s| SemanticRow {
                id: s.id(),
                value: match s.base() {
                    "transparent" => "transparent".to_string(),
                    base => Var::color(base).get(),
                },
            })
            .collect();
        let theme = ThemeCss { default_id: cfg.theme.id(), colors, palettes, semantics }
            .render()
            .expect("render theme.css");

        // scale.css — the spec's ScalePreset + steps; the base + pow() formula are ours.
        // The modular-scale factor is a spec formula (formula::size_factor), projected to a
        // live CSS calc(); cascade-css only multiplies it by its own fluid base.
        let sizes = (STEPS_MIN..=STEPS_MAX)
            .map(|i| SizeRow {
                name: Var::size(i).def(),
                expr: if i == 0 {
                    Var::base().get()
                } else {
                    (Calc::of(Var::base())
                        * formula::size_factor::<Calc>(
                            Calc::lit(i as f64),
                            Calc::of(Var::n()),
                            Calc::of(Var::ratio()),
                        ))
                    .value()
                },
            })
            .collect();
        let presets = ScalePreset::ALL
            .into_iter()
            .map(|p| PresetRow { id: p.id(), ratio: p.ratio(), n: p.n() })
            .collect();
        let scale = ScaleCss {
            base: Self::base_clamp(BASE_PT),
            default_id: cfg.scale.id(),
            default_ratio: cfg.scale.ratio(),
            default_n: cfg.scale.n(),
            sizes,
            presets,
        }
        .render()
        .expect("render scale.css");

        // rhythm.css — the spec's Multiplier; the grid unit's fraction of the base is ours (CSS
        // behaviour). Each spacing token and the baseline route through the spec's rhythm formulas
        // (formula::spacing / formula::baseline) rather than inline arithmetic. `base` → `0`.
        let unit = format!("calc({} * {RHYTHM_UNIT_RATIO})", Var::base().get());
        let spaces = Multiplier::ALL
            .into_iter()
            .map(|m| {
                let token = if m.id() == "base" { "0" } else { m.id() };
                SpaceRow {
                    name: Var::space(token).def(),
                    expr: formula::spacing::<Calc>(Calc::of(Var::unit()), Calc::lit(m.factor())).value(),
                }
            })
            .collect();
        let baseline =
            formula::baseline::<Calc>(Calc::of(Var::base()), Calc::of(Var::lead(FontRole::Body, 0))).value();
        let rhythm = RhythmCss { unit, spaces, baseline }.render().expect("render rhythm.css");

        // typefaces.css — each spec Font as a swappable bundle; `font_family` (the fallback contract)
        // builds the stack, the optical vars come straight from the spec.
        let fonts = Font::ALL
            .into_iter()
            .map(|f| FontRow {
                // slug is a single-token CSS class (.bundle-<slug>); strip spaces from multi-word
                // families ("IBM Plex Mono" -> "ibmplexmono") so the selector doesn't split.
                slug: f.family().to_lowercase().replace(' ', ""),
                family: self.font_family(&f.into()),
                x_height: f.x_height(),
                k_tracking: f.k_tracking(),
                leading_base: f.leading_base(),
                word_space: f.word_space(),
                avg_advance: f.avg_advance(),
            })
            .collect();
        let fonts_css = FontsCss { fonts }.render().expect("render typefaces.css");

        // optical.css — the optical model, PROJECTED from the spec via `formula::*` over `Calc`.
        // Each call binds the spec's variables to CSS custom properties; body and heading differ
        // only in which face-vars they pass (lead0, lb, xh, kt). The scale vars (n, ln_ratio) and
        // clamp bounds (lmin/lmax/tc) are shared. The SAME formulas drive Typst, via `f64`.
        let b = FontRole::Body;
        let h = FontRole::Heading;
        let leading = |step: i32, fr: FontRole| -> String {
            formula::leading::<Calc>(
                Calc::lit(step as f64),
                Calc::of(Var::n()),
                Calc::of(Var::ln_ratio()),
                Calc::of(Var::lead0(fr)),
                Calc::of(Var::lmin()),
                Calc::of(Var::lmax()),
            )
            .value()
        };
        // tracking is an em fraction — apply the unit here (CSS representation); leading is unitless.
        let tracking = |step: i32, fr: FontRole| -> String {
            if step == 0 {
                return "0em".to_string();
            }
            let t = formula::tracking::<Calc>(
                Calc::lit(step as f64),
                Calc::of(Var::n()),
                Calc::of(Var::ln_ratio()),
                Calc::of(Var::kt(fr)),
                Calc::of(Var::tc()),
            );
            format!("calc({} * 1em)", t.value())
        };
        // word-space is an em fraction (like tracking) — apply the unit here. At step 0 it is
        // exactly the base word-space; other steps project formula::word_space.
        let word_space = |step: i32, fr: FontRole| -> String {
            if step == 0 {
                return format!("calc({} * 1em)", Var::bws(fr).get());
            }
            let w = formula::word_space::<Calc>(
                Calc::lit(step as f64),
                Calc::of(Var::n()),
                Calc::of(Var::ln_ratio()),
                Calc::of(Var::bws(fr)),
                Calc::of(Var::kws()),
            );
            // word_space is bare arithmetic (a subtraction); group it and apply the unit in one calc.
            format!("calc(({}) * 1em)", w.s)
        };
        let lead0_of = |fr: FontRole| -> String {
            formula::lead0::<Calc>(Calc::of(Var::lb(fr)), Calc::of(Var::measure()), Calc::of(Var::xh(fr)))
                .value()
        };
        let lead0 = lead0_of(b);
        let h_lead0 = lead0_of(h);
        let tracks: Vec<Row> = (STEPS_MIN..=STEPS_MAX)
            .map(|i| Row { name: Var::track(b, i).def(), value: tracking(i, b) })
            .collect();
        let leads: Vec<Row> = (STEPS_MIN..=STEPS_MAX)
            .map(|i| Row { name: Var::lead(b, i).def(), value: leading(i, b) })
            .collect();
        let word_spaces: Vec<Row> = (STEPS_MIN..=STEPS_MAX)
            .map(|i| Row { name: Var::ws(b, i).def(), value: word_space(i, b) })
            .collect();
        let h_tracks: Vec<Row> =
            (1..=4).map(|k| Row { name: Var::track(h, k).def(), value: tracking(k, h) }).collect();
        let h_leads: Vec<Row> =
            (1..=4).map(|k| Row { name: Var::lead(h, k).def(), value: leading(k, h) }).collect();
        let h_word_spaces: Vec<Row> =
            (1..=4).map(|k| Row { name: Var::ws(h, k).def(), value: word_space(k, h) }).collect();
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
                aw: c.default_avg_advance(),
            })
            .collect();
        let font = FontCss {
            body_family: self.font_family(&cfg.body),
            heading_family: self.font_family(&cfg.heading),
            code_family: self.font_family(&cfg.code),
            font_serif: Self::fallbacks(Category::Serif).to_string(),
            font_sans: Self::fallbacks(Category::Sans).to_string(),
            font_mono: Self::fallbacks(Category::Mono).to_string(),
            xh: cfg.body.x_height,
            kt: cfg.body.k_tracking,
            lb: cfg.body.leading_base,
            bws: cfg.body.word_space,
            h_xh: cfg.heading.x_height,
            h_kt: cfg.heading.k_tracking,
            h_lb: cfg.heading.leading_base,
            h_bws: cfg.heading.word_space,
            code_xh: cfg.code.x_height,
            code_kt: cfg.code.k_tracking,
            code_lb: cfg.code.leading_base,
            kws: WORD_SPACE_K,
            tc: TRACKING_CLAMP,
            lmin: LEADING_CLAMP.0,
            lmax: LEADING_CLAMP.1,
            measure: MEASURE,
            aw: cfg.body.avg_advance,
            size_min: format!("{:.4}rem", SIZE_MIN_PT / PT_PER_REM),
            lead0,
            h_lead0,
            tracks,
            leads,
            word_spaces,
            h_tracks,
            h_leads,
            h_word_spaces,
            profiles,
        }
        .render()
        .expect("render optical.css");

        // ── delivery: @font-face for any SELECTED font shipped WITH bytes (an external `Embed`).
        // Bundled fonts are `System` → no @font-face, so the default output is untouched. Embedded as
        // a self-contained data: URI so the font renders with no extra files. Deduped by family
        // (body == heading emits once); prepended to optical.css, where families live. ──
        let mut seen = std::collections::HashSet::new();
        let mut faces = String::new();
        for f in [&cfg.body, &cfg.heading, &cfg.code] {
            let FontDelivery::Faces(set) = &f.delivery else { continue };
            if !seen.insert(f.family.as_str()) {
                continue; // body == heading: emit the family's face set once
            }
            // One @font-face per face — each declares the weight RANGE (variable) or value (static) +
            // slant it covers, so the browser engages the font's axis / picks the right static face
            // instead of faux-bolding cascade's headings. Embed → data: URI; linked → url(href).
            for face in set {
                let src = match &face.href {
                    Some(href) => href.clone(),
                    None => format!("data:{};base64,{}", face.format.mime(), base64(&face.bytes)),
                };
                let (lo, hi) = face.style.weight;
                let weight = if lo == hi { lo.to_string() } else { format!("{lo} {hi}") };
                let slant = if face.style.italic { "italic" } else { "normal" };
                faces.push_str(&format!(
                    "@font-face {{ font-family: \"{}\"; src: url(\"{}\") format(\"{}\"); font-weight: {}; font-style: {}; font-display: swap; }}\n",
                    f.family,
                    src,
                    face.format.css_format(),
                    weight,
                    slant,
                ));
            }
        }
        let font = if faces.is_empty() { font } else { format!("{faces}{font}") };

        // layout.css — the DOCUMENT MODEL, projected. Each Role's structure (spec) × its
        // role_color (theme) → an element rule. Every role→step/weight/spacing binding comes from
        // the spec; cascade-css supplies only selector syntax, unit application, and (next) the
        // CSS-specific representation. No role→step is authored here.
        let sel = |r: Role| -> String {
            if r.elements().is_empty() {
                ".cascade".to_string()
            } else {
                r.elements().iter().map(|e| format!(".cascade {e}")).collect::<Vec<_>>().join(", ")
            }
        };
        let mut type_rules =
            String::from("/* cascade-css — layout.css: the document model, projected from the spec */\n");
        for r in Role::ALL {
            let mut d: Vec<String> = Vec::new();
            if let Some(fr) = r.font() {
                d.push(format!("font-family: {}", Var::family(fr).get()));
            }
            if let Some(step) = r.step() {
                // sub-base sizes hold a readability floor (cascade-css representation).
                if step < 0 {
                    d.push(format!("font-size: max({}, {})", Var::size(step).get(), Var::size_min().get()));
                } else {
                    d.push(format!("font-size: {}", Var::size(step).get()));
                }
                // heading roles read the `h-` optical set; body (and code) read the body set.
                let ofr = if r.font() == Some(FontRole::Heading) { FontRole::Heading } else { FontRole::Body };
                d.push(format!("line-height: {}", Var::lead(ofr, step).get()));
                d.push(format!("letter-spacing: {}", Var::track(ofr, step).get()));
                d.push(format!("word-spacing: {}", Var::ws(ofr, step).get()));
            }
            if let Some(w) = r.weight() {
                d.push(format!("font-weight: {w}"));
            }
            if r.italic() {
                d.push("font-style: italic".to_string());
            }
            if r.underline() {
                d.push("text-decoration: underline".to_string());
            }
            if r.space_before().is_some() || r.space_after().is_some() {
                let m = |t: Option<&str>| t.map(|t| Var::space(t).get()).unwrap_or_else(|| "0".to_string());
                d.push(format!("margin: {} 0 {}", m(r.space_before()), m(r.space_after())));
            }
            let rc = role_color(r);
            if let Some(fg) = rc.fg {
                d.push(format!("color: {}", Var::color(fg).get()));
            }
            if let Some(bg) = rc.bg {
                d.push(format!("background: {}", Var::color(bg).get()));
            }
            if d.is_empty() {
                continue;
            }
            type_rules.push_str(&format!("{} {{ {}; }}\n", sel(r), d.join("; ")));
        }

        // ── representation: cascade-css BEHAVIOUR — box model, states, chrome. References the
        // spec's tokens/optical + the theme colours, but authors no role→step binding. Border
        // colours are pulled from role_color (single source); placement/width are CSS reps. ──
        let border = |r: Role| role_color(r).border.unwrap_or("rule");
        // Every var REFERENCE below goes through `Var` too — the representation is authored CSS,
        // but its cascade custom-property references are still typed and single-sourced.
        let mut layout = type_rules;
        layout.push_str("\n/* ── representation: box model, states, chrome (cascade-css behaviour) ── */\n");
        // The reading measure, single-sourced. `--cf-measure-inline` is the TEXT line length: the
        // spec's character `measure` × the body font's real mean advance (`--cf-aw`, em) via
        // copyfitting — a TRUE character count per typeface, not a `1ch` (the "0" advance, which
        // overshoots and breaches the WCAG 1.4.8 ~80ch ceiling) nor a textbook 0.5em. `--cf-aw` is
        // reactive, so switching the body face (`bundle-*`) re-resolves it. Every layout mode binds
        // this SAME text width: banded caps its offset text column at it (sidenotes.css), and the
        // centered container below sizes to it PLUS the page padding — with border-box, padding is
        // inside max-inline-size, so the container must be `measure + 2×pad` for the TEXT (not the
        // text-minus-padding) to equal the measure. min(100%, …) keeps it inside a narrow viewport.
        let pad = Var::space("p5").get();
        layout.push_str(&format!(".cascade {{ box-sizing: border-box; {mi}: calc({m} * {aw} * 1em); max-inline-size: min(100%, calc({mir} + 2 * {pad})); margin-inline: auto; padding: {pad}; text-rendering: optimizeLegibility; font-kerning: normal; font-feature-settings: \"kern\", \"liga\", \"clig\"; -webkit-font-smoothing: antialiased; hyphens: none; {xhd}: {xhb}; }}\n", mi = Var::measure_inline().def(), m = Var::measure().get(), aw = Var::avg_advance().get(), mir = Var::measure_inline().get(), pad = pad, xhd = Var::xh_doc().def(), xhb = Var::xh(b).get()));
        layout.push_str(".cascade *, .cascade *::before, .cascade *::after { box-sizing: border-box; }\n");
        layout.push_str(".cascade > *:last-child { margin-bottom: 0; }\n");
        layout.push_str(".cascade :is(h1, h2, h3, h4):first-child { margin-top: 0; }\n");
        layout.push_str(".cascade h1, .cascade h2, .cascade h3, .cascade h4 { text-wrap: balance; }\n");
        // link states + focus (semantic colours from the theme)
        layout.push_str(".cascade a { text-decoration-thickness: 1px; text-underline-offset: 0.15em; }\n");
        layout.push_str(&format!(".cascade a:visited {{ color: {}; }}\n", Var::color("link-visited").get()));
        layout.push_str(&format!(".cascade a:hover {{ color: {}; text-decoration-thickness: 2px; }}\n", Var::color("link-hover").get()));
        layout.push_str(&format!(".cascade a:focus-visible {{ outline: 2px solid {}; outline-offset: 2px; border-radius: 1px; }}\n", Var::color("accent").get()));
        // code: the mono category optical, applied as overrides (definitions) so the leading/tracking
        // formulas re-resolve for the code subtree; + inline/block chrome. `font-size-adjust` pins the
        // mono's rendered x-height to the DOCUMENT body's (`--cf-xh-doc`): a monospace face carries a
        // larger x-height than a text face, so at an equal em it reads oversized in running prose. This
        // normalises apparent size across families (and across whatever the mono fallback resolves to),
        // which is why inline code needs no hand-tuned em fudge -- the x-height match supplies it.
        layout.push_str(&format!(".cascade code, .cascade kbd, .cascade samp, .cascade pre {{ letter-spacing: 0; word-spacing: normal; font-size-adjust: {}; {}: {}; {}: {}; {}: {}; }}\n", Var::xh_doc().get(), Var::xh(b).def(), Var::code_xh().get(), Var::kt(b).def(), Var::code_kt().get(), Var::lb(b).def(), Var::code_lb().get()));
        layout.push_str(".cascade :not(pre) > code { padding: 0.1em 0.34em; border-radius: 3px; }\n");
        layout.push_str(&format!(".cascade pre {{ font-size: {}; line-height: {}; padding: 1em; border-radius: 4px; overflow-x: auto; }}\n", Var::size(0).get(), Var::lead(b, 0).get()));
        layout.push_str(".cascade pre code { background: none; padding: 0; font-size: inherit; }\n");
        // quote
        layout.push_str(&format!(".cascade blockquote {{ padding: {} 2em; border-inline-start: 3px solid {}; }}\n", Var::space("p1").get(), Var::color(border(Role::Quote)).get()));
        layout.push_str(".cascade blockquote > *:last-child { margin-bottom: 0; }\n");
        // lists
        layout.push_str(".cascade ul, .cascade ol { padding-inline-start: 1.6em; }\n");
        layout.push_str(".cascade li { margin: 0; }\n");
        layout.push_str(".cascade li > ul, .cascade li > ol { margin-bottom: 0; }\n");
        // figures
        layout.push_str(".cascade figure img, .cascade img { max-inline-size: 100%; height: auto; }\n");
        layout.push_str(&format!(".cascade figcaption {{ text-align: center; margin-top: {}; }}\n", Var::space("0").get()));
        // divider
        layout.push_str(&format!(".cascade hr {{ border: 0; block-size: 0; border-top: 0.5px solid {}; }}\n", Var::color(border(Role::Divider)).get()));
        // footnotes: the styled endnotes REGION (above) + the standard author-placed reference
        // markers and back-links. This is NOT a switchable mode like the side/margin notes -- it is
        // plain styling for the conventional markup (a `.footnote-ref`/[role="doc-noteref"] anchor,
        // usually wrapping a <sup> number, in the text; an <ol> of notes in the region, each ending
        // in a `.footnote-back`/[role="doc-backlink"] anchor). Numbering comes from the markup / the
        // <ol>, not CSS counters, so it composes with the auto-numbered side/margin notes rather
        // than competing with them. (Author-placed because pure CSS can't collect inline notes to
        // the page foot on screen -- that's Paged-Media `float: footnote`, i.e. the Typst renderer.)
        layout.push_str(&format!(".cascade .footnotes, .cascade [role=\"doc-endnotes\"] {{ border-top: 0.5px solid {}; padding-top: {}; margin-top: {}; }}\n", Var::color(border(Role::Footnotes)).get(), Var::space("p1").get(), Var::space("p4").get()));
        layout.push_str(&format!(".cascade .footnote-ref, .cascade [role=\"doc-noteref\"] {{ color: {}; text-decoration: none; }}\n", Var::color("link").get()));
        layout.push_str(&format!(".cascade .footnote-back, .cascade [role=\"doc-backlink\"] {{ color: {}; text-decoration: none; margin-inline-start: 0.35em; }}\n", Var::color("link").get()));

        // sidenotes.css — the opt-in notes feature; presentation only, cascade tokens via Var.
        let sidenotes = SidenotesCss {
            gap: Var::space("n1").get(),
            indent: Var::space("p1").get(),
            baseline: Var::space("baseline").get(),
            marker: Var::color("link").get(),
            rule: Var::color("rule").get(),
            measure: Var::measure_inline().get(),
        }
        .render()
        .expect("render sidenotes.css");

        // The modules, in logical load order (tokens → typefaces → binding). Custom-property
        // resolution is order-independent, so this order is for readability, not correctness.
        let modules = vec![
            Output { path: "theme.css".into(), body: theme },
            Output { path: "scale.css".into(), body: scale },
            Output { path: "rhythm.css".into(), body: rhythm },
            Output { path: "typefaces.css".into(), body: fonts_css },
            Output { path: "optical.css".into(), body: font },
            Output { path: "layout.css".into(), body: layout },
            Output { path: "sidenotes.css".into(), body: sidenotes },
        ];

        // cascade.css — the ENTRYPOINT. `@import` this one file to load the whole system. The
        // import list is derived from the modules above, so it can never drift from what's emitted.
        // (@import rules must lead the file, which they do — the rest is comments.)
        let imports =
            modules.iter().map(|o| format!("@import \"{}\";", o.path)).collect::<Vec<_>>().join("\n");
        let entrypoint = format!(
            "/* cascade-css — cascade.css: the entrypoint. `@import \"cascade.css\"` (or bundle this\n \
             * file) to load the whole system. Opt into features with the documented hooks:\n \
             *   palettes  → [data-palette=\"<id>\"]      (theme.css)\n \
             *   light/dark → [data-theme=\"light|dark\"]  (theme.css)\n \
             *   typeface   → class .bundle-<name>        (typefaces.css)\n \
             *   notes      → class .sidenotes[.banded]   (sidenotes.css) */\n{imports}\n"
        );

        let mut outputs = vec![Output { path: "cascade.css".into(), body: entrypoint }];
        outputs.extend(modules);
        outputs
    }

    /// CSS's medium check — the same guards the test-suite enforces on the defaults, here run on
    /// whatever [`Config`] was rendered. Empty = correct.
    fn verify(&self, outputs: &[Output]) -> Vec<String> {
        verify_css(outputs)
    }
}

/// Runtime verification of rendered CSS. Two independent checks:
///   1. STRUCTURAL — every file parses as valid CSS via lightningcss (browser-grade, strict), so an
///      invalid selector, at-rule, or unbalanced block is caught. (Numeric parity of the `calc()`
///      math — folding to the reference number — can't run on reactive output full of `var()`s; it
///      is a renderer invariant covered by the test-suite, not re-checked per build.)
///   2. REFERENCES — across the whole output set, no `var(--x)` points at a custom property that
///      nothing defines (lightningcss can't see this — an undefined `var()` is valid CSS that
///      silently resolves empty).
/// Returns a list of problems (empty = OK). Shared by `Css::verify` and the test-suite.
fn verify_css(outputs: &[Output]) -> Vec<String> {
    let mut problems = Vec::new();
    for o in outputs {
        if let Err(e) = parse_strict(&o.body) {
            problems.push(format!("{}: invalid CSS: {e}", o.path));
        }
    }
    problems.extend(undefined_vars(outputs));
    problems
}

/// Parse strictly (no error recovery) then minify — forces the full value grammar to be
/// interpreted, so bad math surfaces. `Ok(())` if it is valid CSS.
fn parse_strict(css: &str) -> Result<(), String> {
    use lightningcss::stylesheet::{MinifyOptions, ParserOptions, StyleSheet};
    let mut ss = StyleSheet::parse(css, ParserOptions { error_recovery: false, ..Default::default() })
        .map_err(|e| e.to_string())?;
    ss.minify(MinifyOptions::default()).map_err(|e| e.to_string())
}

/// Every `var(--x)` reference across all files must resolve to a `--x:` definition somewhere in the
/// set (they load together via the entrypoint). Returns one message per dangling reference.
fn undefined_vars(outputs: &[Output]) -> Vec<String> {
    use std::collections::HashSet;
    let css: String = outputs.iter().map(|o| o.body.as_str()).collect::<Vec<_>>().join("\n");
    let read_name = |s: &str| -> String {
        s.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-').collect()
    };
    let referenced: HashSet<String> = css.split("var(--").skip(1).map(read_name).collect();
    let defined: HashSet<String> = css
        .split("--")
        .skip(1)
        .filter_map(|part| {
            let name = read_name(part);
            part[name.len()..].starts_with(':').then_some(name)
        })
        .collect();
    let mut missing: Vec<&String> = referenced.difference(&defined).collect();
    missing.sort();
    missing.into_iter().map(|v| format!("var(--{v}) referenced but never defined")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn out(path: &str) -> String {
        Css.render(&Config::default()).into_iter().find(|o| o.path == path).unwrap().body
    }

    #[test]
    fn font_family_is_spec_identity_plus_css_fallbacks() {
        assert_eq!(Css.font_family(&Font::Lora.into()), "\"Lora\", Georgia, \"Times New Roman\", serif");
        assert_eq!(Css.font_family(&Font::Inter.into()), "\"Inter\", system-ui, -apple-system, sans-serif");
    }

    #[test]
    fn family_is_selected_by_category_type_not_by_font() {
        // The fallback stack is chosen by CATEGORY, not the specific family — two different families
        // of the same category share it; different categories differ. Shown with synthetic resolved
        // fonts so it doesn't depend on the bundle shipping two of any one category.
        let stack = |family: &str, category: Category| {
            Css.font_family(&ResolvedFont {
                family: family.into(),
                category,
                optical_size: "12pt".into(),
                x_height: 0.5,
                cap_height: 0.7,
                avg_advance: 0.46,
                k_tracking: 0.02,
                leading_base: 1.4,
                word_space: 0.28,
                delivery: FontDelivery::System,
            })
            .split_once(", ")
            .unwrap()
            .1
            .to_string()
        };
        assert_eq!(stack("Alpha", Category::Sans), stack("Beta", Category::Sans)); // same cat → same
        assert_ne!(stack("Alpha", Category::Sans), stack("Gamma", Category::Serif)); // diff cat → diff
    }

    #[test]
    fn renders_theme_and_scale() {
        let theme = out("theme.css");
        // default palette (paper) at :root, light + dark both present
        assert!(theme.contains("--ct-accent: #7A2E28;") && theme.contains("--ct-accent: #E09A93;"));
        // explicit theme overrides win over the media query (the MODE axis)
        assert!(theme.contains("[data-theme=\"dark\"] {") && theme.contains("[data-theme=\"light\"] {"));
        // every palette ships and is switchable at runtime (the PALETTE axis), orthogonal to mode
        assert!(theme.contains("[data-palette=\"paper\"] {") && theme.contains("[data-palette=\"slate\"] {"));
        assert!(theme.contains("--ct-accent: #2F5E8A;")); // slate's accent — present though not the default
        assert!(theme.contains("[data-palette=\"slate\"][data-theme=\"dark\"] {")); // the two axes compose
        // semantic uses alias base swatches (or a literal), resolved on .cascade
        assert!(theme.contains("--ct-link: var(--ct-accent);"));
        assert!(theme.contains("--ct-code-bg: var(--ct-bg-subtle);"));
        assert!(theme.contains("--ct-quote-bg: transparent;"));
        assert!(theme.contains("::selection { background: var(--ct-selection-bg); color: var(--ct-fg); }"));
        let scale = out("scale.css");
        assert!(scale.contains("--cs-base: clamp("));
        assert!(scale.contains(".cascade.scale-classical { --cs-ratio: 2; --cs-n: 5; }"));
    }

    #[test]
    fn renders_rhythm_and_fonts() {
        let rhythm = out("rhythm.css");
        assert!(rhythm.contains("--cr-space-0: calc(var(--cr-unit) * 1);"));
        assert!(rhythm.contains("--cr-space-p1: calc(var(--cr-unit) * 2);"));

        let fonts = out("typefaces.css");
        assert!(fonts.contains(".cascade.bundle-lora {"));
        assert!(fonts.contains("--cf-family-body: \"Lora\", Georgia, \"Times New Roman\", serif;"));
        assert!(fonts.contains("--cf-xh:  0.5;"));
    }

    #[test]
    fn emits_minimally_parenthesized() {
        // Formulas project to live CSS with only the parentheses the grammar requires —
        // no literals, and no redundant grouping around already-tighter-binding sub-terms.
        let scale = out("scale.css");
        assert!(scale.contains(
            "--cs-size-p2: calc(var(--cs-base) * pow(var(--cs-ratio), 2 / var(--cs-n)));"
        ));
        let font = out("optical.css");
        assert!(font.contains(
            "--cf-lead0: calc(var(--cf-lb) + (var(--cf-measure) - 65) * 0.006 - (var(--cf-xh) - 0.5) * 0.8);"
        ));
        assert!(font.contains(
            "--cf-lead-p1: clamp(var(--cf-lmin), var(--cf-lead0) - 0.1 * (1 / var(--cs-n)) * var(--cs-ln-ratio), var(--cf-lmax));"
        ));
        assert!(font.contains(
            "--cf-track-p1: calc(clamp(-1 * var(--cf-tc), -1 * var(--cf-kt) * (1 / var(--cs-n)) * var(--cs-ln-ratio), var(--cf-tc)) * 1em);"
        ));
    }

    #[test]
    fn layout_is_projected_from_the_role_model() {
        let layout = out("layout.css");
        // h1's size/leading/weight/spacing come from the spec Role — not hardcoded here.
        assert!(layout.contains(
            ".cascade h1 { font-family: var(--cf-family-heading); font-size: var(--cs-size-p4); line-height: var(--cf-h-lead-p4); letter-spacing: var(--cf-h-track-p4); word-spacing: var(--cf-h-ws-p4); font-weight: 700; margin: var(--cr-space-p4) 0 var(--cr-space-p2); }"
        ));
        // root (the container) carries base type + the ground colour, combined from the theme.
        assert!(layout.contains(
            ".cascade { font-family: var(--cf-family-body); font-size: var(--cs-size-0); line-height: var(--cf-lead-0); letter-spacing: var(--cf-track-0); word-spacing: var(--cf-ws-0); color: var(--ct-fg); background: var(--ct-bg); }"
        ));
        // multi-element roles fan out to prefixed selectors; theme colour combined in.
        assert!(layout.contains(".cascade strong, .cascade b { font-weight: 700; }"));
        assert!(layout.contains(".cascade a { text-decoration: underline; color: var(--ct-link); }"));
        // footnotes: the styled endnotes region + the author-placed reference marker + back-link.
        assert!(layout.contains(".cascade .footnotes, .cascade [role=\"doc-endnotes\"] { border-top:"));
        assert!(layout.contains(".cascade .footnote-ref, .cascade [role=\"doc-noteref\"] { color: var(--ct-link); text-decoration: none; }"));
        assert!(layout.contains(".cascade .footnote-back, .cascade [role=\"doc-backlink\"] { color: var(--ct-link); text-decoration: none; margin-inline-start: 0.35em; }"));
    }

    #[test]
    fn measure_is_a_copyfit_width_from_the_real_advance_not_ch() {
        // The reading measure is projected ONCE, via copyfitting: characters × the body font's real
        // mean advance (--cf-aw) × 1em — never `ch` (the "0" advance, which overshoots) nor 0.5em.
        let layout = out("layout.css");
        // --cf-measure-inline is the TEXT line length (no padding); the centered container sizes to
        // it PLUS the page padding, so the text (not text-minus-padding) equals the measure.
        assert!(layout.contains("--cf-measure-inline: calc(var(--cf-measure) * var(--cf-aw) * 1em);"));
        assert!(layout.contains(
            "max-inline-size: min(100%, calc(var(--cf-measure-inline) + 2 * var(--cr-space-p5)))"
        ));
        assert!(!layout.contains("* 1ch"));
        // --cf-measure stays the raw character count (the optical lead0 formula consumes it); --cf-aw
        // carries the per-font advance, and each bundle redefines it so the measure is reactive.
        let optical = out("optical.css");
        assert!(optical.contains("--cf-measure: 65;"));
        assert!(optical.contains("--cf-aw: 0.477;")); // default body = Inter's measured advance (weight-sampled)
        assert!(out("typefaces.css").contains("--cf-aw:  0.4693;")); // Lora bundle overrides it
    }

    #[test]
    fn banded_notes_keep_the_reading_measure() {
        // Turning on margin notes (banded) must NOT discard the measure: the offset text column
        // binds the SAME single-sourced value as the centered column, not a raw percentage.
        let sn = out("sidenotes.css");
        assert!(sn.contains(
            ".cascade.sidenotes.banded > *:not(.sidenote):not(.marginnote) { width: var(--sn-text); max-inline-size: min(100%, var(--cf-measure-inline)); }"
        ));
        // and it's a defined property (layout.css defines it), so the reference resolves.
        assert!(out("layout.css").contains("--cf-measure-inline: "));
    }

    #[test]
    fn entrypoint_imports_every_module() {
        let entry = out("cascade.css");
        // one include pulls the whole system; @imports lead the file.
        for m in ["theme", "scale", "rhythm", "typefaces", "optical", "layout", "sidenotes"] {
            assert!(entry.contains(&format!("@import \"{m}.css\";")), "entrypoint missing {m}.css");
        }
        // the confusing pair is gone — no font.css / fonts.css anywhere in the output set.
        let all: Vec<String> = Css.render(&Config::default()).into_iter().map(|o| o.path).collect();
        assert!(all.contains(&"cascade.css".to_string()));
        assert!(!all.iter().any(|p| p == "font.css" || p == "fonts.css"));
        assert_eq!(all.len(), 8); // entrypoint + 7 modules
    }

    #[test]
    fn renders_sidenotes_feature() {
        let sn = out("sidenotes.css");
        // opt-in feature, scoped to .sidenotes; the counter drives numbering.
        assert!(sn.contains(".cascade.sidenotes {") && sn.contains("counter-reset: sidenote-counter;"));
        // cascade tokens arrive via Var (rhythm + theme), not hardcoded literals.
        assert!(sn.contains("--sn-gap:        var(--cr-space-n1);"));
        assert!(sn.contains("color: var(--ct-link);")); // marker
        assert!(sn.contains("border-inline-start: 2px solid var(--ct-rule);")); // disclosure border
        // per-note disclosure (a checked marker opens only its adjacent note) + the banded float layout.
        assert!(sn.contains(".margin-toggle:checked + .sidenote"));
        assert!(sn.contains(".note-group:has(.margin-toggle:checked)")); // group flexes when it has an open note
        assert!(sn.contains("@media (min-width: 62rem)") && sn.contains(".cascade.sidenotes.banded"));
        // note TYPOGRAPHY is single-sourced from the roles (layout.css), NOT re-specified here.
        assert!(!sn.contains("--cs-size-n2"));
        let layout = out("layout.css");
        assert!(layout.contains(".cascade .sidenote {") && layout.contains(".cascade .marginnote {"));
        assert!(layout.contains("var(--ct-fg-subtle)")); // note colour, from the role
    }

    #[test]
    fn renders_font_optical_model() {
        let font = out("optical.css");
        // families per font-role: body = Inter, heading = Lora, code = the bundled mono (IBM Plex Mono)
        assert!(font.contains("--cf-family-body:    \"Inter\", system-ui, -apple-system, sans-serif;"));
        assert!(font.contains("--cf-family-heading: \"Lora\", Georgia, \"Times New Roman\", serif;"));
        assert!(font.contains("--cf-family-code:    \"IBM Plex Mono\", ui-monospace, SFMono-Regular, monospace;"));
        // body optical from the body font (Inter), heading optical from the heading font (Lora)
        assert!(font.contains("--cf-xh: 0.546;")); // Inter
        assert!(font.contains("--cf-h-xh: 0.5;")); // Lora
        assert!(font.contains("--cf-h-lb: 1.38;")); // Lora leading-base
        assert!(font.contains("--cf-tc: 0.04;"));
        assert!(font.contains("--cf-lmin: 1.2;") && font.contains("--cf-lmax: 1.5;"));
        // clamp chains cascade-css derives, + profile classes from the spec
        assert!(font.contains("--cf-track-p1: calc(clamp("));
        assert!(font.contains("--cf-lead-n5: clamp(var(--cf-lmin),"));
        // generic font stacks + the body/heading bundle switches (serif/sans/mono)
        assert!(font.contains("--cf-font-serif:") && font.contains("--cf-font-sans:"));
        assert!(font.contains(".cascade.bundle-serif { --cf-family-body: var(--cf-font-serif); --cf-xh: 0.49;"));
        assert!(font.contains(".cascade.heading-serif { --cf-family-heading: var(--cf-font-serif);"));
    }

    #[test]
    fn every_font_control_option_has_a_class() {
        // Each body/heading/code dropdown option in the site must map to a real class the renderer
        // emits. All three roles are swappable, so all three switch families exist for every option.
        let optical = out("optical.css");
        let typefaces = out("typefaces.css");
        for cat in ["serif", "sans", "mono"] {
            for role in ["bundle", "heading", "code"] {
                assert!(optical.contains(&format!(".cascade.{role}-{cat} {{")), "missing {role}-{cat}");
            }
        }
        // Every bundled Font emits a bundle/heading/code class; multi-word families slugify to one token.
        for font in ["inter", "lora", "ibmplexmono"] {
            for role in ["bundle", "heading", "code"] {
                assert!(typefaces.contains(&format!(".cascade.{role}-{font} {{")), "missing {role}-{font}");
            }
        }
    }
}

/// CSS validation via lightningcss (a browser-grade parser). Two layers:
///   1. STRUCTURAL — every rendered file parses as valid CSS (balanced brackets, valid rules).
///   2. NUMERIC PARITY — each formula's `Calc` projection, emitted with LITERAL inputs (no vars),
///      is folded by lightningcss to a concrete value and must equal the `f64` projection of the
///      SAME formula. This proves the emitted CSS math is well-formed, correctly typed, and
///      browser-evaluable to exactly the reference number — the dual-projection kept honest.
#[cfg(test)]
mod validate {
    use super::*;
    use cascade::{formula, LEADING_CLAMP, SCALE_DEFAULT, TRACKING_CLAMP, WORD_SPACE_K};
    use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};

    /// Parse (strict), minify (forces the value grammar to be interpreted), and re-print.
    fn transform(css: &str) -> Result<String, String> {
        let mut ss = StyleSheet::parse(css, ParserOptions { error_recovery: false, ..Default::default() })
            .map_err(|e| e.to_string())?;
        ss.minify(MinifyOptions::default()).map_err(|e| e.to_string())?;
        ss.to_css(PrinterOptions { minify: true, ..Default::default() })
            .map(|r| r.code)
            .map_err(|e| e.to_string())
    }

    #[test]
    fn every_rendered_file_is_valid_css() {
        for o in Css.render(&Config::default()) {
            parse_strict(&o.body).unwrap_or_else(|e| panic!("{} is not valid CSS: {e}", o.path));
        }
    }

    /// The runtime verifier (what `--verify` runs) both passes on real output and HAS TEETH:
    /// a dangling `var()` and a syntax error are each reported.
    #[test]
    fn verify_passes_and_catches_faults() {
        assert!(Css.verify(&Css.render(&Config::default())).is_empty());
        let broken = vec![
            Output { path: "dangling.css".into(), body: ".a { color: var(--ct-nope); }".into() },
            Output { path: "syntax.css".into(), body: "%%% { color: red; }".into() },
        ];
        let problems = verify_css(&broken);
        assert!(problems.iter().any(|p| p.contains("--ct-nope")), "missed dangling var: {problems:?}");
        assert!(problems.iter().any(|p| p.contains("invalid CSS")), "missed syntax error: {problems:?}");
    }

    /// End-to-end backstop: no `var(--x)` reference may point at a property that isn't defined
    /// somewhere in the output. lightningcss can't catch this — an undefined `var()` is valid CSS
    /// that silently resolves empty. This does: out-of-range steps, typo'd tokens, define/reference
    /// drift all surface here.
    #[test]
    fn every_referenced_var_is_defined() {
        let problems = undefined_vars(&Css.render(&Config::default()));
        assert!(problems.is_empty(), "{problems:?}");
    }

    /// Fold a fully-concrete (var-free) CSS math expression to a number by giving it a unit,
    /// minifying, and reading the result back. Panics if lightningcss could not fold it to a
    /// single length — a residual `calc()`/`clamp()`/`pow()` means malformed or mistyped math.
    fn fold(expr: &str) -> f64 {
        let out = transform(&format!("a{{width:calc(({expr}) * 1px)}}")).expect("valid CSS");
        let inner = out
            .strip_prefix("a{width:")
            .and_then(|s| s.strip_suffix('}'))
            .unwrap_or_else(|| panic!("unexpected shape: {out}"));
        // A residual function call means lightningcss could not fold it → malformed or mistyped.
        assert!(!inner.contains('('), "did not fold to a single value (malformed/mistyped): {inner}");
        // A folded length is `<n>px`, except lightningcss drops the unit on zero (`0px` → `0`).
        let num = inner.strip_suffix("px").unwrap_or(inner);
        num.parse::<f64>().unwrap_or_else(|_| panic!("not a number: {num}"))
    }

    /// The `Calc` projection of `expr`, then folded by lightningcss, must equal `expected`.
    fn assert_parity(label: &str, expr: Calc, expected: f64) {
        let got = fold(&expr.value());
        assert!(
            (got - expected).abs() <= 1e-3 * (1.0 + expected.abs()),
            "{label}: lightningcss folded CSS to {got}, f64 projection is {expected}"
        );
    }

    #[test]
    fn css_projection_folds_to_the_f64_projection() {
        // Realistic bindings: the default scale + Lora's (default serif) optical profile.
        let ratio = SCALE_DEFAULT.ratio();
        let n = SCALE_DEFAULT.n() as f64;
        let ln_ratio = ratio.ln();
        let (lb, xh, measure) =
            (Font::Lora.leading_base(), Font::Lora.x_height(), MEASURE as f64);
        let (lmin, lmax) = LEADING_CLAMP;
        let (tc, kt) = (TRACKING_CLAMP, Font::Lora.k_tracking());
        let (bws, kws) = (Font::Lora.word_space(), WORD_SPACE_K);
        let lc = |x: f64| Calc::lit(x); // literal-input Calc → a var-free, foldable expression

        // lead0 (a single value)
        let lead0_f = formula::lead0::<f64>(lb, measure, xh);
        assert_parity("lead0", formula::lead0::<Calc>(lc(lb), lc(measure), lc(xh)), lead0_f);

        for step in STEPS_MIN..=STEPS_MAX {
            let s = step as f64;
            assert_parity(
                &format!("size_factor[{step}]"),
                formula::size_factor::<Calc>(lc(s), lc(n), lc(ratio)),
                formula::size_factor::<f64>(s, n, ratio),
            );
            assert_parity(
                &format!("leading[{step}]"),
                formula::leading::<Calc>(lc(s), lc(n), lc(ln_ratio), lc(lead0_f), lc(lmin), lc(lmax)),
                formula::leading::<f64>(s, n, ln_ratio, lead0_f, lmin, lmax),
            );
            assert_parity(
                &format!("tracking[{step}]"),
                formula::tracking::<Calc>(lc(s), lc(n), lc(ln_ratio), lc(kt), lc(tc)),
                formula::tracking::<f64>(s, n, ln_ratio, kt, tc),
            );
            assert_parity(
                &format!("word_space[{step}]"),
                formula::word_space::<Calc>(lc(s), lc(n), lc(ln_ratio), lc(bws), lc(kws)),
                formula::word_space::<f64>(s, n, ln_ratio, bws, kws),
            );
        }
    }
}
