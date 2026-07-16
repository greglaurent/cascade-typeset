//! The UNIVERSAL typographic formulas — written ONCE as generic Rust over [`Val`].
//!
//! A formula is an ordinary function using ordinary arithmetic. The type parameter chooses
//! the projection: `f64` evaluates it to a number (a fixed medium, e.g. Typst); a renderer's
//! symbolic type (e.g. cascade-css's `Css`) builds a live `calc()`/`pow()`/`clamp()`
//! expression, so nothing is ever collapsed to a literal and the browser stays the evaluator.
//! The formula is never restated per medium — the renderer supplies only the [`Val`] impl (the
//! operator vocabulary) and the variables it passes in, so it CANNOT deviate from the spec's
//! calculation. Variables are NOT part of `Val`; they enter as arguments, which is what lets a
//! symbolic renderer pass tokens while a numeric one passes values.

use std::ops::{Add, Div, Mul, Neg, Sub};

/// The operator vocabulary a spec formula may use. A renderer implements this once to receive
/// EVERY formula, projected into its medium. `lit` is the only literal a formula body may
/// introduce (spec constants); everything else arrives as a bound argument.
pub trait Val:
    Sized
    + Clone
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    fn lit(x: f64) -> Self;
    fn pow(self, exp: Self) -> Self;
    fn ln(self) -> Self;
    fn clamp(self, lo: Self, hi: Self) -> Self;
    /// Round `self` to the nearest multiple of `step` (CSS `round()`, Typst `calc.round`).
    fn round(self, step: Self) -> Self;
}

/// Numeric projection — the fixed-medium (Typst) evaluation. A number in, a number out.
impl Val for f64 {
    fn lit(x: f64) -> Self {
        x
    }
    fn pow(self, exp: Self) -> Self {
        self.powf(exp)
    }
    fn ln(self) -> Self {
        f64::ln(self)
    }
    fn clamp(self, lo: Self, hi: Self) -> Self {
        f64::clamp(self, lo, hi)
    }
    fn round(self, step: Self) -> Self {
        (self / step).round() * step
    }
}

// Shape constants of the optical model — spec-owned, projected verbatim into every medium.
pub const MEASURE_REF: f64 = 65.0; // reference line length (characters)
pub const MEASURE_GAIN: f64 = 0.006; // leading added per character over the reference
pub const XH_REF: f64 = 0.5; // reference relative x-height
pub const XH_GAIN: f64 = 0.8; // leading removed per unit x-height over the reference
pub const STEP_GAIN: f64 = 0.10; // leading removed per scale-step of size

/// Modular-scale size multiplier at `step`: `ratio^(step/n)`.
pub fn size_factor<T: Val>(step: T, n: T, ratio: T) -> T {
    ratio.pow(step / n)
}

/// Baseline (step-0) leading: `lb + (measure − REF)·GAIN − (xh − REF)·GAIN`.
pub fn lead0<T: Val>(lb: T, measure: T, xh: T) -> T {
    lb + (measure - T::lit(MEASURE_REF)) * T::lit(MEASURE_GAIN)
        - (xh - T::lit(XH_REF)) * T::lit(XH_GAIN)
}

/// Optical leading at `step`, clamped: `clamp(lmin, lead0 − STEP_GAIN·(step/n)·ln_ratio, lmax)`.
/// `lead0` enters as a variable so a renderer can share one resolved baseline.
pub fn leading<T: Val>(step: T, n: T, ln_ratio: T, lead0: T, lmin: T, lmax: T) -> T {
    (lead0 - T::lit(STEP_GAIN) * (step / n) * ln_ratio).clamp(lmin, lmax)
}

/// Optical tracking at `step` — a unitless em fraction — clamped to ±`tc`:
/// `clamp(−tc, −kt·(step/n)·ln_ratio, tc)`.
pub fn tracking<T: Val>(step: T, n: T, ln_ratio: T, kt: T, tc: T) -> T {
    (-kt * (step / n) * ln_ratio).clamp(-tc.clone(), tc)
}

/// Optical word-space at `step` — a unitless em fraction: `base_ws − k_ws·(step/n)·ln_ratio`.
/// Inverse with size (Bringhurst §2.1.4–5, Tracy): smaller sizes want relatively MORE word
/// space, display sizes less. At step 0 it is exactly `base_ws`. Unclamped, per the original.
pub fn word_space<T: Val>(step: T, n: T, ln_ratio: T, base_ws: T, k_ws: T) -> T {
    base_ws - k_ws * (step / n) * ln_ratio
}

// ── vertical rhythm (rhythm.typ) — derived from the scale + optical model, not independent ──

/// The body baseline — one line's height: `body_size × leading_ratio`. Paragraph spacing and the
/// rhythm unit derive from it. A print renderer may additionally snap it to a grid (see `snap`).
pub fn baseline<T: Val>(body_size: T, leading_ratio: T) -> T {
    body_size * leading_ratio
}

/// A vertical-rhythm spacing token: the grid `unit` times a `multiplier`.
pub fn spacing<T: Val>(unit: T, multiplier: T) -> T {
    unit * multiplier
}

/// Grid-snap: round `value` to the nearest multiple of `unit` — Tim Brown's opt-in baseline
/// alignment, not an enforced lattice. `snap(value, unit)`.
pub fn snap<T: Val>(value: T, unit: T) -> T {
    value.round(unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway SYMBOLIC projection, defined here only to prove the trait — the real one
    /// (cascade-css's `Css`) lives in the renderer. Aggressive parens keep it valid CSS `calc()`.
    #[derive(Clone)]
    struct Sym(String);
    impl Sym {
        fn var(name: &str) -> Self {
            Sym(format!("var(--{name})"))
        }
    }
    fn fmt(x: f64) -> String {
        format!("{x}")
    }
    impl Add for Sym {
        type Output = Sym;
        fn add(self, o: Sym) -> Sym {
            Sym(format!("({} + {})", self.0, o.0))
        }
    }
    impl Sub for Sym {
        type Output = Sym;
        fn sub(self, o: Sym) -> Sym {
            Sym(format!("({} - {})", self.0, o.0))
        }
    }
    impl Mul for Sym {
        type Output = Sym;
        fn mul(self, o: Sym) -> Sym {
            Sym(format!("({} * {})", self.0, o.0))
        }
    }
    impl Div for Sym {
        type Output = Sym;
        fn div(self, o: Sym) -> Sym {
            Sym(format!("({} / {})", self.0, o.0))
        }
    }
    impl Neg for Sym {
        type Output = Sym;
        fn neg(self) -> Sym {
            Sym(format!("(-1 * {})", self.0))
        }
    }
    impl Val for Sym {
        fn lit(x: f64) -> Sym {
            Sym(fmt(x))
        }
        fn pow(self, e: Sym) -> Sym {
            Sym(format!("pow({}, {})", self.0, e.0))
        }
        fn ln(self) -> Sym {
            Sym(format!("log({})", self.0))
        }
        fn clamp(self, lo: Sym, hi: Sym) -> Sym {
            Sym(format!("clamp({}, {}, {})", lo.0, self.0, hi.0))
        }
        fn round(self, step: Sym) -> Sym {
            Sym(format!("round({}, {})", self.0, step.0))
        }
    }

    #[test]
    fn one_formula_two_projections() {
        // numeric projection (Typst-style): scale doubles at +5; lead0 collapses at the refs.
        assert!((size_factor::<f64>(5.0, 5.0, 2.0) - 2.0).abs() < 1e-9);
        let l0 = lead0::<f64>(1.4, 65.0, 0.5);
        assert!((l0 - 1.4).abs() < 1e-9);
        // leading/tracking always land inside their clamps.
        for step in -5..=5 {
            let ld = leading::<f64>(step as f64, 5.0, 2f64.ln(), l0, 1.2, 1.5);
            assert!(ld >= 1.2 && ld <= 1.5);
            let tr = tracking::<f64>(step as f64, 5.0, 2f64.ln(), 0.03, 0.04);
            assert!(tr.abs() <= 0.04 + 1e-12);
        }
        // word-space: exactly base_ws at step 0, inverse with size (more at small, less at large).
        assert!((word_space::<f64>(0.0, 5.0, 2f64.ln(), 0.28, 0.04) - 0.28).abs() < 1e-9);
        assert!(word_space::<f64>(5.0, 5.0, 2f64.ln(), 0.28, 0.04) < 0.28);
        assert!(word_space::<f64>(-5.0, 5.0, 2f64.ln(), 0.28, 0.04) > 0.28);
        // rhythm: baseline = size × leading; spacing = unit × multiplier; snap → nearest unit.
        assert!((baseline::<f64>(11.0, 1.4) - 15.4).abs() < 1e-9);
        assert!((spacing::<f64>(4.0, 3.0) - 12.0).abs() < 1e-9);
        assert!((snap::<f64>(15.4, 4.0) - 16.0).abs() < 1e-9); // 15.4 → nearest multiple of 4
        assert!((snap::<f64>(13.9, 4.0) - 12.0).abs() < 1e-9);
        let sn = snap::<Sym>(Sym::var("v"), Sym::var("u")).0;
        assert_eq!(sn, "round(var(--v), var(--u))");

        // symbolic projection (CSS-style): the SAME formula, a live expression, never a literal.
        let s = size_factor::<Sym>(Sym::var("cs-step"), Sym::var("cs-n"), Sym::var("cs-ratio")).0;
        assert_eq!(s, "pow(var(--cs-ratio), (var(--cs-step) / var(--cs-n)))");
        let ld = leading::<Sym>(
            Sym::var("s"),
            Sym::var("n"),
            Sym::var("lnr"),
            Sym::var("lead0"),
            Sym::var("lmin"),
            Sym::var("lmax"),
        )
        .0;
        assert!(ld.starts_with("clamp(var(--lmin),") && ld.contains("var(--lead0)"));
        let ws = word_space::<Sym>(
            Sym::var("s"),
            Sym::var("n"),
            Sym::var("lnr"),
            Sym::var("bws"),
            Sym::var("kws"),
        )
        .0;
        assert!(ws.contains("var(--bws) -") && ws.contains("var(--kws)"));
    }
}
