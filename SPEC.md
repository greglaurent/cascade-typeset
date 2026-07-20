# cascade — Typographic Spec (canonical)

The canonical, human-readable statement of the typography. Companion to
[ARCHITECTURE.md](ARCHITECTURE.md): that file says *how* the spec is projected (renderers re-state
it, never author it); this file says *what* the spec is.

**Source of truth.** The machine-canonical source is the RON + `formula.rs`:
`cascade/tokens.ron`, `cascade/theme.ron`, `cascade/themes/*.ron`, `cascade/fonts/*.ron`,
`cascade/src/formula.rs`. They compile into types at build time. **This document mirrors them; edit
the RON/formula → update this file in the same change.** Every value below is copied from that
source, not from memory.

**References.** Where a value or formula is backed by a real source it is cited (see
[§9](#9-references)). Where a value is a **spec-chosen tuning constant** with no external source, it
says so plainly — no citation is invented. If you see a claim here you did not make, it is a bug in
this document; fix it here first.

---

## ⚠ Value provenance — READ THIS FIRST

Most numbers in this document are **not confirmed spec.** They are the *current code values*, and
several were chosen by agents (including in this session), not by the author — they are **UNVERIFIED
and may be wrong.** Do not treat them as canonical.

- **Confirmed input (author-given):** `base_pt = 11`.
- **Derived, not guessed:** the font metrics (§5) are **measured** from the font files by `measure.rs`;
  the scale ratios (§4) are mathematical (golden ratio, octave, thirds).
- **UNVERIFIED — agent-introduced, treat as suspect:** `size_min_pt`, `measure`, `word_space_k`,
  `tracking_clamp`, `leading_clamp`, `rhythm_unit_ratio`, `code_scale`, the gain coefficients inside
  the leading/tracking/word-space formulas (`MEASURE_GAIN`, `XH_GAIN`, `STEP_GAIN`), and the
  `WEIGHT_USAGE` averaging prior. Documented below as *what the code currently holds*, not as truth.
- **Open / unresolved:** many of the "constants" above are **not set values at all — they are
  calculations that were frozen into magic numbers.** Which are genuinely set vs. which must be
  *calculated* is the author's design and is **not recoverable from the code.** It is left blank here
  rather than guessed.

---

## 1. Inputs — the overridable knobs

Everything else derives from these. A renderer exposes exactly these (in the forms that make sense
for its medium) and nothing else, so no invalid state is reachable:

| Input | What it selects | Default |
|---|---|---|
| base size | reading size at scale step 0 (`base_pt`) | 11pt (web: fluid around it) |
| body / heading / code typeface | a bundled font (or category fallback) | Inter / Lora / IBM Plex Mono |
| scale preset | the modular-scale ratio + steps-per-ratio | golden-ditonic |
| theme | palette + light/dark mode | paper, light |
| measure | reading line length in characters | 65 |

---

## 2. Optical constants  (`tokens.ron` → `optical`)

Spec-owned; every renderer uses these identically.

| Constant | Value | Meaning | Basis |
|---|---|---|---|
| `base_pt` | 11.0 | reading size at scale step 0 | spec-chosen reading size |
| `size_min_pt` | 9.0 | readability floor small roles clamp up to | spec-chosen legibility floor |
| `measure` | 65 | reading line length, in characters | Bringhurst 45–75 (66 ideal) [B1]; Butterick 45–90 [Bu1]; under WCAG 1.4.8 ~80 ceiling [W2] |
| `rhythm_unit_ratio` | 0.375 | vertical-rhythm unit = `base × ratio` | spec-chosen |
| `code_scale` | 0.9 | code sized to this fraction of its reference **before** the x-height match | spec-chosen; monospace reads heavier at equal apparent size, and code stays inside the measure [Bu1] |
| `word_space_k` | 0.04 | word-space sensitivity to size (`k_ws`) | spec-chosen calibration |
| `tracking_clamp` | 0.04 | max \|tracking\| (em) | spec-chosen calibration |
| `leading_clamp` | (min 1.2, max 1.5) | leading floor/ceiling | max 1.5 = WCAG 1.4.12 body line-height [W1] |

---

## 3. Canonical calculations

The formulas ARE the spec. Below, `step` = scale step, `n` = steps per ratio doubling,
`ratio` = scale ratio, `ln_ratio = ln(ratio)`. Font-supplied inputs (`lb`, `xh`, `kt`, `base_ws`,
`avg_advance`) come from the resolved font (§5).

### 3a. In `formula.rs` today (shared, projected by both renderers)

Shape constants: `MEASURE_REF = 65`, `MEASURE_GAIN = 0.006`, `XH_REF = 0.5`, `XH_GAIN = 0.8`,
`STEP_GAIN = 0.10` (the gain coefficients are spec-chosen calibration).

| Name | Formula | Notes / basis |
|---|---|---|
| `size_factor` | `ratio^(step / n)` | modular scale |
| `lead0` | `lb + (measure − 65)·0.006 − (xh − 0.5)·0.8` | baseline (step-0) leading: looser for a wider measure, tighter for a larger x-height (direction is standard practice; coefficients are calibration) |
| `leading` | `clamp(lmin, lead0 − 0.10·(step/n)·ln_ratio, lmax)` | line-height ratio; larger sizes lead tighter |
| `tracking` | `clamp(−tc, −kt·(step/n)·ln_ratio, tc)` | em; opens at small sizes, closes at display |
| `word_space` | `base_ws − k_ws·(step/n)·ln_ratio` | em; **inverse with size** — Bringhurst §2.1.4–5 [B1], Tracy [T1]. Exactly `base_ws` at step 0 |
| `baseline` | `body_size · leading_ratio` | one line's height |
| `spacing` | `unit · multiplier` | a vertical-rhythm spacing token |
| `snap` | `round(value / unit) · unit` | opt-in baseline grid snap |

### 3b. Canonical, but currently hand-rolled in the renderers → MUST move into `formula.rs`

These are spec derivations (they must match across media) that today are computed independently in
each renderer — the divergence source ARCHITECTURE.md exists to kill. Documented here as canonical:

| Name | Formula | Note |
|---|---|---|
| `rhythm_unit` | `base · rhythm_unit_ratio` (= `base · 0.375`) | rhythm unit; spacing tokens are `unit · multiplier` |
| `measure_width` | `measure · avg_advance · base` | copyfit column width — the body font's real mean advance, **not** `ch` [W2]. Anchored to the **base** size, never element-`em` (the bug this replaced) |
| `role_size` | `max(base · size_factor(step), size_min)` | per-role size, floored at the readability minimum |
| `code_size` | `role_size · (body.x_height / code.x_height) · code_scale` | code matched to the body's x-height, then scaled by `code_scale`. Inline code uses the same factor relative to its context |

Medium-specific application (a shared value, no cross-renderer formula — see ARCHITECTURE.md §"What
is legitimately the renderer's"): CSS applies `leading` as `line-height` and the x-height match as
`font-size-adjust`; Typst applies `leading` as a `top/bottom-edge` box and the x-height match as a
relative-`em` size. Same value in; the box/adjust math is medium mechanism, verified visually.

---

## 4. Scale presets  (`tokens.ron` → `scale_presets`; steps −5…5)

`n` = number of steps that span one multiplication by `ratio`.

| Preset | ratio | n | interval |
|---|---|---|---|
| classical | 2.0 | 5 | octave over 5 steps |
| golden-ratio | 1.6180339887498949 | 1 | golden ratio per step |
| **golden-ditonic** (default) | 1.6180339887498949 | 2 | golden ratio over 2 steps |
| tritonic | 2.0 | 3 | octave over 3 steps |
| tetratonic | 2.0 | 4 | octave over 4 steps |
| major-third | 1.25 | 1 | 5:4 per step |
| minor-third | 1.2 | 1 | 6:5 per step |

---

## 5. Fonts

A **selected** font's measured metrics + profile knobs OVERRIDE the category defaults. `x_height`,
`cap_height`, `avg_advance` are **measured from the font file** (OS/2 + glyph advances); the profile
knobs (`optical_size`, `k_tracking`, `leading_base`, `word_space`) are the font's tuning.

### Category defaults (generic fallback, when no specific font is selected)

| category | optical_size | x_height | k_tracking | leading_base | word_space | avg_advance |
|---|---|---|---|---|---|---|
| serif | 11pt | 0.49 | 0.022 | 1.35 | 0.28 | 0.46 |
| sans | 12pt | 0.53 | 0.03 | 1.45 | 0.28 | 0.47 |
| mono | 12pt | 0.535 | 0.0 | 1.5 | 0.0 | 0.6 |

### Bundled fonts (measured)   — defaults: body Inter, heading Lora, code IBM Plex Mono

Profile knobs (`optical_size`, `k_tracking`, `leading_base`, `word_space`) are author tuning;
`x_height` / `cap_height` / `avg_advance` / `asc` / `desc` are measured (§5, *How a font is measured*).

| font | cat | opsz | x_height | cap_height | avg_advance | asc | desc | upm | k_tracking | leading_base | word_space |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Inter | sans | 12pt | 0.546 | 0.728 | 0.4735 | 0.969 | −0.241 | 2048 | 0.03 | 1.45 | 0.26 |
| Lora | serif | 11pt | 0.500 | 0.700 | 0.4669 | 1.006 | −0.274 | 1000 | 0.022 | 1.38 | 0.28 |
| IBM Plex Mono | mono | 12pt | 0.516 | 0.698 | 0.6000 | 1.025 | −0.275 | 1000 | 0.0 | 1.5 | 0.0 |

`avg_advance` (mono) = the single fixed advance of a monospace. `asc`/`desc` (typographic
ascender/descender, em) are measured and carried in the RON but **not consumed by the current
formulas** — kept for the human-readable note (per `measure.rs`).

### How a font is measured  (`cascade/src/measure.rs`, `measure` feature)

Font metrics are **derived by a standardized measurement, not entered by hand** — so every bundled
*or* client-added font (`cascade measure`, `cascade add`, on-the-fly `--font-path`) derives them the
same way. All ratios are em fractions (÷ `units_per_em`). "How a font is measured" is spec, not tool.

Per metric:

- **x_height** — OS/2 `sxHeight` ÷ upem. **Required**: a font without it is refused, not invented.
- **cap_height** — OS/2 `sCapHeight` ÷ upem, else the `H` glyph's top.
- **ascender / descender** — typographic (OS/2) values, else `hhea`.
- **avg_advance** — the copyfitting factor: the **character-frequency-weighted mean glyph advance**
  (÷ upem). Each glyph's advance is weighted by how often that character occurs in English running
  text (`CHAR_FREQ`: space 0.1828, e 0.1041, t 0.0729, … z 0.0005; Σ ≈ 1, **including the word space
  at ~18%**), renormalized over the glyphs the font actually provides. So it is the mean width of a
  *typical* character of prose — not a naïve 0.5em. (cf. OS/2 `xAvgCharWidth`, Capsize `xWidthAvg`
  [C1].)

Variable-font axes are handled deliberately differently — **this is the "font averaging" piece**:

- **opsz (optical size) — PINNED, not averaged.** Set to the axis text end (`min`), the reading cut.
  opsz moves x-height and advance materially (Inter x-height 0.546@14pt → 0.516@32pt); averaging
  across it would blend text with display, so we measure the reading face.
- **wght (weight) — measured at the DEFAULT INSTANCE, not averaged.** The advance thickens with weight
  **non-linearly**, but body text is set at ONE weight — the font's default instance (≈ Regular, the
  designer's representative weight). `avg_advance` is measured there, like every other metric; it is
  **not** averaged over an assumed weight distribution. Only `avg_advance` varies with weight (x-height /
  cap / asc / desc are weight-invariant). A **static** font has one instance. If a consumer overrides the
  body weight, the advance is measured at THAT weight and a renderer selects it per weight the same way
  `bundle-*` selects a font — a live pick over pre-measured data, never a baked scalar.

`avg_advance` is a **measured metric** (like `x_height`), an input to the `measure_width` formula — not
its own formula. The earlier `WEIGHT_USAGE` body-weight *usage* prior (`300:0.10 … 700:0.08`, `m* = pᵀM`)
was an agent-added hardcoded knob — it collapsed a weight that isn't actually unknown — and is
**eliminated**: measure at the body's weight instead.

The `profile` block (`optical_size`, `k_tracking`, `leading_base`, `word_space`) is **author tuning,
not measured** — seeded from the category defaults on first measure, preserved verbatim on re-measure.

The opsz-text-pin is **spec-chosen method**; the frequency-weighted advance follows OS/2
`xAvgCharWidth` / Capsize [C1]; `CHAR_FREQ` is standard English letter frequency [F1]. (The
`WEIGHT_USAGE` weight-usage prior is eliminated — see above.)

---

## 6. Document roles  (`tokens.ron` → `roles`)

The universal document model (the "h1 = p4" bindings). `kind` block/inline; `step` = scale step
(absent → inherits); `font` = body/heading/code; `space_*` = rhythm token before/after. Colour is
**not** here (§8). Inline roles with **no step** (strong/emphasis/link/code) inherit size — they
carry decoration only.

| role | elements | kind | step | font | weight | space before | space after | flags |
|---|---|---|---|---|---|---|---|---|
| root | — | block | 0 | body | | | | |
| body | p | block | 0 | body | | | baseline | |
| heading-1 | h1 | block | 4 | heading | 700 | p4 | p2 | |
| heading-2 | h2 | block | 3 | heading | 600 | p3 | p1 | |
| heading-3 | h3 | block | 2 | heading | 600 | p2 | 0 | |
| heading-4 | h4 | block | 1 | heading | 600 | p1 | 0 | |
| text-1 | .text-1 | inline | −2 | body | | | | |
| text-2 | .text-2 | inline | −1 | body | | | | |
| text-3 | .text-3 | inline | 0 | body | | | | |
| text-4 | .text-4 | inline | 1 | body | | | | |
| text-5 | .text-5 | inline | 2 | body | | | | |
| strong | strong, b | inline | — | body | 700 | | | |
| emphasis | em, i, cite | inline | — | body | | | | italic |
| small | small | inline | −1 | body | | | | |
| link | a | inline | — | body | | | | underline |
| quote | blockquote | block | — | body | | baseline | baseline | italic |
| code | code, kbd, samp | inline | — | code | | | | |
| code-block | pre | block | — | code | | | baseline | |
| list | ul, ol | block | — | body | | | baseline | |
| figure | figure | block | — | body | | p4 | p4 | |
| caption | figcaption | block | −2 | body | | | | italic |
| footnotes | .footnotes | block | −2 | body | | | | |
| sidenote | .sidenote | inline | −2 | body | | | | |
| marginnote | .marginnote | inline | −2 | body | | | | |
| divider | hr | block | — | | | p4 | p4 | |

`elements` are semantic identities; a renderer translates them (CSS selectors, Typst show rules).
Tufte note *placement* (margin float / disclosure) is renderer presentation, not spec.

---

## 7. Rhythm multipliers  (`tokens.ron` → `multipliers`)

A spacing token = `rhythm_unit × multiplier`; `"baseline"` = one body line (§3, `baseline`).

| token | n1 | base | p1 | p2 | p3 | p4 | p5 | p6 |
|---|---|---|---|---|---|---|---|---|
| ×unit | 0.5 | 1.0 | 2.0 | 3.0 | 4.0 | 6.0 | 8.0 | 12.0 |

---

## 8. Colour / theme

The colour layer is separate and swappable. Structure lives in `theme.ron`; swatch **values** live
in `themes/<name>.ron`. Both light and dark are hand-curated and ship together — light/dark is a
runtime toggle within a palette.

### Swatch vocabulary (every palette provides these `Color` names)
`fg`, `fg-muted`, `fg-subtle`, `bg`, `bg-subtle`, `rule`, `accent`, `accent-hover`, `accent-subtle`,
`accent-rule`, `accent-visited`.

### Semantic aliases (`theme.ron` → `semantic`)
`link → accent` · `link-hover → accent-hover` · `link-visited → accent-visited` · `code-fg → fg` ·
`code-bg → bg-subtle` · `quote-rule → accent-rule` · `quote-bg → transparent` ·
`selection-bg → accent-subtle`.

### Role → colour bindings (`theme.ron` → `roles`; unlisted roles inherit `fg`)
`root` fg=fg bg=bg · `text-1` fg=fg-muted · `caption` fg=fg-muted · `footnotes` fg=fg-subtle
border=rule · `sidenote`/`marginnote` fg=fg-subtle · `link` fg=link · `code`/`code-block` fg=code-fg
bg=code-bg · `quote` bg=quote-bg border=quote-rule · `divider` border=rule.

### Palettes  (default: **paper**)

**paper** — warm paper + ink, brick-red accent:

| swatch | light | dark |
|---|---|---|
| fg | #171717 | #E8E4DC |
| fg-muted | #59544C | #A8A196 |
| fg-subtle | #7A746A | #8A8378 |
| bg | #F6F2E9 | #14120E |
| bg-subtle | #EFE9DC | #1E1B16 |
| rule | #C4BDB0 | #3A362F |
| accent | #7A2E28 | #E09A93 |
| accent-hover | #5E211C | #EFB8B1 |
| accent-subtle | #F0E2DE | #2E1614 |
| accent-rule | #C9A5A0 | #4A2A26 |
| accent-visited | #5A3A52 | #C9A5C4 |

**slate** — cool neutral + slate-blue accent:

| swatch | light | dark |
|---|---|---|
| fg | #1A1D21 | #E4E7EB |
| fg-muted | #4B525B | #9AA4B0 |
| fg-subtle | #6B7480 | #7C8593 |
| bg | #F5F7FA | #0F1216 |
| bg-subtle | #E9EDF2 | #191D23 |
| rule | #C2C9D2 | #333A43 |
| accent | #2F5E8A | #8FB8E0 |
| accent-hover | #234A6E | #B0D0F0 |
| accent-subtle | #E1EAF3 | #14212E |
| accent-rule | #A6BFD6 | #294056 |
| accent-visited | #4A4370 | #B7ABE0 |

---

## 9. References

- **[B1] Robert Bringhurst, *The Elements of Typographic Style*.** Measure 45–75 characters, 66
  ideal (`measure = 65`). Word-space decreasing with size — §2.1.4–5 (`word_space`). Measure/leading
  relationship (direction of `lead0`).
- **[T1] Walter Tracy, *Letters of Credit*.** Word-space set relative to size (`word_space`).
- **[Bu1] Matthew Butterick, *Practical Typography*** — [line length](https://practicaltypography.com/line-length.html)
  45–90 characters, content-agnostic; [monospaced fonts](https://practicaltypography.com/monospaced-fonts.html):
  code set in a monospace **within** the text column, not a wider measure (`code_scale`, code stays
  in the measure).
- **[W1] WCAG 2.x, SC 1.4.12 Text Spacing.** Body line-height ≥ 1.5 (`leading_clamp` max 1.5).
- **[W2] WCAG 2.x, SC 1.4.8 Visual Presentation.** Width ceiling ~80 characters; motivates a real
  copyfit measure over `ch` (`measure`, `measure_width`).
- **[C1] OS/2 `xAvgCharWidth` / Capsize (`xWidthAvg`).** Frequency-weighted mean advance as the
  copyfitting factor (`avg_advance`).
- **[F1] Standard English letter-frequency corpus** (letters + word space ~18%). The `CHAR_FREQ`
  distribution used to weight the mean advance.

Values marked "spec-chosen" above (base size, floors, the gain/`k`/clamp calibration constants,
`rhythm_unit_ratio`, `code_scale`, the scale-preset selection) are tuning decisions of this project,
not claims from an external source. If any should be pinned to a reference, add it here first.
