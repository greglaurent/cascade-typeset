//! The UNIVERSAL typographic calculations — each defined ONCE, projected per medium.
//!
//! A formula is a symbolic [`Expr`]. The spec offers two projections of the SAME
//! definition: [`Expr::eval`] → a number (a renderer that fixes its values, e.g. Typst) and
//! [`Expr::to_css_value`] → a live `calc()` string (a renderer that stays fluid, e.g. CSS,
//! where a class swap re-resolves it in the browser). A renderer supplies only the variable
//! BINDINGS — concrete values for `eval`, or the target token (`n` → `var(--cs-n)`) for CSS.
//! No renderer re-derives a formula; there is exactly one definition. Pure, no runtime deps.

// Shape constants of the optical model — spec-owned, so a projected formula (CSS's `calc()`)
// carries these exact coefficients rather than restating a curve of its own.
pub const MEASURE_REF: f64 = 65.0; // reference line length (characters)
pub const MEASURE_GAIN: f64 = 0.006; // leading added per character over the reference
pub const XH_REF: f64 = 0.5; // reference relative x-height
pub const XH_GAIN: f64 = 0.8; // leading removed per unit x-height over the reference
pub const STEP_GAIN: f64 = 0.10; // leading removed per scale-step of size

/// A typographic formula as data. Variables ([`Expr::Var`]) are bound late — numerically by
/// [`eval`](Expr::eval) or to a CSS token by [`to_css`](Expr::to_css) — so one definition
/// serves every renderer. Build via [`size_factor`], [`lead0`], [`leading`], [`tracking`].
pub enum Expr {
    Num(f64),
    Var(&'static str),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Ln(Box<Expr>),
    Clamp(Box<Expr>, Box<Expr>, Box<Expr>),
}
use Expr::*;

fn lit(x: f64) -> Box<Expr> {
    Box::new(Num(x))
}
fn vr(name: &'static str) -> Box<Expr> {
    Box::new(Var(name))
}
fn bx(e: Expr) -> Box<Expr> {
    Box::new(e)
}

/// Modular-scale size multiplier at `step`: `ratio^(step/n)`. Size at a step = base × this.
pub fn size_factor(step: i32) -> Expr {
    Pow(vr("ratio"), bx(Div(lit(step as f64), vr("n"))))
}

/// Baseline (step-0) leading for a face: `lb + (measure−REF)·GAIN − (xh−REF)·GAIN`.
pub fn lead0() -> Expr {
    Sub(
        bx(Add(vr("lb"), bx(Mul(bx(Sub(vr("measure"), lit(MEASURE_REF))), lit(MEASURE_GAIN))))),
        bx(Mul(bx(Sub(vr("xh"), lit(XH_REF))), lit(XH_GAIN))),
    )
}

/// Optical leading (unitless line-height) at `step`, clamped to the spec's leading range.
/// References `lead0` as a variable so a renderer can share one resolved baseline.
pub fn leading(step: i32) -> Expr {
    Clamp(
        vr("lmin"),
        bx(Sub(
            vr("lead0"),
            bx(Mul(bx(Mul(lit(STEP_GAIN), bx(Div(lit(step as f64), vr("n"))))), vr("ln_ratio"))),
        )),
        vr("lmax"),
    )
}

/// Optical tracking at `step` — a unitless em fraction (a renderer applies the unit),
/// clamped to ±`tc`.
pub fn tracking(step: i32) -> Expr {
    Clamp(
        bx(Neg(vr("tc"))),
        bx(Mul(bx(Neg(vr("kt"))), bx(Mul(bx(Div(lit(step as f64), vr("n"))), vr("ln_ratio"))))),
        vr("tc"),
    )
}

fn fmt(x: f64) -> String {
    format!("{x}") // 65.0 -> "65", 0.006 -> "0.006"
}

impl Expr {
    /// Numeric evaluation. `env` resolves each [`Var`](Expr::Var) name to a value.
    pub fn eval(&self, env: &impl Fn(&str) -> f64) -> f64 {
        match self {
            Num(x) => *x,
            Var(k) => env(k),
            Neg(a) => -a.eval(env),
            Add(a, b) => a.eval(env) + b.eval(env),
            Sub(a, b) => a.eval(env) - b.eval(env),
            Mul(a, b) => a.eval(env) * b.eval(env),
            Div(a, b) => a.eval(env) / b.eval(env),
            Pow(a, b) => a.eval(env).powf(b.eval(env)),
            Ln(a) => a.eval(env).ln(),
            Clamp(lo, x, hi) => x.eval(env).clamp(lo.eval(env), hi.eval(env)),
        }
    }

    fn is_atom(&self) -> bool {
        matches!(self, Num(_) | Var(_) | Pow(..) | Ln(_) | Clamp(..))
    }

    /// An operand as it appears inside another op — parenthesised unless self-delimiting.
    fn operand(&self, bind: &impl Fn(&str) -> String) -> String {
        if self.is_atom() {
            self.to_css(bind)
        } else {
            format!("({})", self.to_css(bind))
        }
    }

    /// Math-context CSS (valid inside `calc()`/`clamp()`/…). `bind` maps a [`Var`](Expr::Var)
    /// name to its CSS token, e.g. `"n"` → `"var(--cs-n)"`.
    pub fn to_css(&self, bind: &impl Fn(&str) -> String) -> String {
        match self {
            Num(x) => fmt(*x),
            Var(k) => bind(k),
            Neg(a) => format!("-1 * {}", a.operand(bind)),
            Add(a, b) => format!("{} + {}", a.operand(bind), b.operand(bind)),
            Sub(a, b) => format!("{} - {}", a.operand(bind), b.operand(bind)),
            Mul(a, b) => format!("{} * {}", a.operand(bind), b.operand(bind)),
            Div(a, b) => format!("{} / {}", a.operand(bind), b.operand(bind)),
            Pow(a, b) => format!("pow({}, {})", a.to_css(bind), b.to_css(bind)),
            Ln(a) => format!("log({})", a.to_css(bind)),
            Clamp(lo, x, hi) => {
                format!("clamp({}, {}, {})", lo.to_css(bind), x.to_css(bind), hi.to_css(bind))
            }
        }
    }

    /// A standalone CSS property value: wraps bare arithmetic in `calc()`; leaves a
    /// self-delimiting root (`clamp()`, `pow()`, `var()`, number) as-is.
    pub fn to_css_value(&self, bind: &impl Fn(&str) -> String) -> String {
        if self.is_atom() {
            self.to_css(bind)
        } else {
            format!("calc({})", self.to_css(bind))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LEADING_CLAMP, MEASURE, TRACKING_CLAMP};

    #[test]
    fn one_definition_evaluates_and_projects() {
        let (ratio, nn) = (2.0_f64, 5.0_f64);
        let l0 = lead0().eval(&|k| match k {
            "lb" => 1.4,
            "measure" => MEASURE as f64,
            "xh" => 0.5,
            _ => panic!("unbound {k}"),
        });
        let env = |k: &str| match k {
            "ratio" => ratio,
            "n" => nn,
            "ln_ratio" => ratio.ln(),
            "lmin" => LEADING_CLAMP.0,
            "lmax" => LEADING_CLAMP.1,
            "tc" => TRACKING_CLAMP,
            "kt" => 0.03,
            "lead0" => l0,
            _ => panic!("unbound {k}"),
        };

        // numeric projection (Typst-style): scale doubles at +5; lead0 collapses at the refs.
        assert!((size_factor(5).eval(&env) - 2.0).abs() < 1e-9);
        assert!((l0 - 1.4).abs() < 1e-9);
        // leading/tracking always land inside the spec's clamps.
        for step in -5..=5 {
            let ld = leading(step).eval(&env);
            assert!(ld >= LEADING_CLAMP.0 && ld <= LEADING_CLAMP.1);
            assert!(tracking(step).eval(&env).abs() <= TRACKING_CLAMP + 1e-12);
        }

        // CSS projection (same definition): a live clamp() referencing custom properties.
        let css = |k: &str| format!("var(--{k})");
        let s = leading(-5).to_css_value(&css);
        assert!(s.starts_with("clamp("));
        assert!(s.contains("var(--lead0)") && s.contains("var(--ln_ratio)") && s.contains("var(--n)"));
    }
}
