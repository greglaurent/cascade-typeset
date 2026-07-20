# cascade — Equation Verification (literature cross-reference)

Every foundational equation, cross-referenced against the major typographic resources for its **form**
and its **constants**, and assigned a path — **Digital / Print / Identical**. Companion to
[SPEC.md](SPEC.md) (what the values are) and [ARCHITECTURE.md](ARCHITECTURE.md) (the boundary + why
the diff-guarantee applies to the Identical path only).

**Method.** Seven parallel research passes, each grounding one equation cluster against the sources,
under a no-fabrication rule (quote + URL, or mark ungroundable). The **five spec-changing citations
were then re-fetched and verified verbatim by hand** (marked ✔verified). Honesty caveats: Bringhurst's
*Elements*, Tracy's *Letters of Credit*, Hochuli's *Detail in Typography*, and Müller-Brockmann's
*Grid Systems* were **not** fetched as primary text — quotes attributed to them come from pages that
reproduce/paraphrase them (webtypography.net, reviews) and are labelled where load-bearing.

**Verdict key:** GROUNDED (form + value cited) · DIRECTION-ONLY (shape cited, constant is calibration)
· CONVENTION (a real practitioner range, exact value not derived) · UNSUPPORTED (no basis found /
contradicted).

---

## Summary

| # | Equation | Form | Constants | Path | Verdict |
|---|---|---|---|---|---|
| 1 | `size_factor = ratio^(step/n)` | grounded | ratios grounded; `step/n` = equal-tempered ext. | **Identical** | GROUNDED |
| 2 | `lead0` / `leading` | grounded (directions) | `0.006 / 0.8 / 0.10` calibration; **`1.5` ceiling wrong** | **Digital ≠ Print** | DIRECTION-ONLY + fix |
| 3 | `tracking` | grounded (direction) | `0.04` clamp calibration | Identical | DIRECTION-ONLY |
| 4 | `word_space` | grounded (direction) | base `M/4≈0.25` grounded; `k_ws` calibration | Identical | DIRECTION-ONLY |
| 5 | `measure = 65` | grounded | grounded (45–75/66; ≤80) | Identical | GROUNDED |
| 6 | `measure_width = measure·avg_advance·base` | grounded | grounded (xAvgCharWidth; not `ch`) | Identical | GROUNDED |
| 7 | `role_size` + `base`/`size_min` | grounded | `base 11pt` grounded; `size_min 9pt` convention | **Identical base + derived digital suggestion** | GROUNDED |
| 8 | `code_size` (x-height × `0.9`) | x-height match grounded | `0.9` convention | Identical value, medium mechanism | CONVENTION |
| 9 | `rhythm_unit = base·0.375` | **UNSUPPORTED** | `0.375` has no basis | Identical | **UNSUPPORTED — replace** |
| 10 | `spacing = unit·multiplier` | grounded | integer multiples; `0.5` half-step mildly off | Identical | GROUNDED |
| 11 | font measurement | (1) grounded, (2) grounded, (3) `avg_advance` = measured metric; `WEIGHT_USAGE` eliminable | `18%` calibration | Identical | GROUNDED + drop `WEIGHT_USAGE` |

**Actions this produces:** (a) `rhythm_unit` → the line-height/baseline, not `0.375×base`; (b) the
leading `1.5` **ceiling** contradicts WCAG (1.5 is a *floor*) and web practice — reconsider;
(c) **base size** is **Identical** — one canonical base (the cross-renderer comparison anchor); the digital size is a *derived, overridable suggestion* from base + screen size, **not** a second base. **leading**'s print-vs-screen difference is still open (a real split, or the same derived-suggestion pattern — see question below); (d) the fabricated
constants (`code_scale 0.9`, the leading `0.006/0.8/0.10`, the tracking/word-space slopes, the `18%`
space, `size_min`) were **agent-invented — they never existed in the PoC**, so there is nothing to
restore. **Ground them from the research (the literature):** take the cited value/range directly where
it exists (`size_min ≈ 9pt`, code shrink ~0.85–0.9×, space ~18%), and **derive** the coefficient to land
in the cited output ranges where the literature grounds only the form (the leading slopes, the
tracking/word-space slopes) — never keep the made-up number, never restore from a PoC that never had it;
(e) `avg_advance` is a **measured metric** (like
`x_height`) evaluated at the body weight (§11.3) — the `WEIGHT_USAGE` weight-usage prior/averaging is
agent-added and **eliminable**; weight-dependence, if body weight is overridable, is handled by
per-weight emission + live selection like `bundle-*`, never a baked scalar.

---

## 1. Modular scale — `size_factor = ratio^(step / n)`

**FORM: GROUNDED.** The formula is Spencer Mortensen's typographic scale verbatim ✔verified:
`f_i = f_0 · r^(i/n)`, f_0 = base, r = ratio, n = notes/steps per interval —
<https://spencermortensen.com/articles/typographic-scale/>. Modular scale as a concept: Bringhurst
("a prearranged set of harmonious proportions", via secondary), Tim Brown / modularscale.com, Rutter
§3.1.1 ("limit yourself… to a modest set of distinct and related intervals", <http://webtypography.net/3.1.1>).

**CONSTANTS (ratios): GROUNDED.** modularscale.com names each verbatim: minor third 1.2, major third
1.25, golden 1.618, octave 2.0 — <https://www.modularscale.com/>. The `step/n` subdivision (n>1) is the
**equal-tempered** generalization (Mortensen; precise-type.com) — a refinement beyond canonical
Bringhurst/Brown (who use n=1 whole-ratio steps). Your n=5-per-octave = Mortensen's "five sizes in an
interval" exactly.

**PATH: Identical.** A proportion system; Rutter ports the print scale to web unchanged. Only the
unit binding (pt vs px/rem) is medium-specific. Lupton: **silent** on modular scale (not a supporter).

---

## 2. Leading — `lead0` and `leading`

`lead0 = lb + (measure−65)·0.006 − (x_height−0.5)·0.8`;
`leading = clamp(1.2, lead0 − 0.10·(step/n)·ln(ratio), 1.5)`

**FORM / DIRECTIONS: GROUNDED.**
- base ~1.3–1.5: Rutter §2.2.1 ("figures upwards of 1.3 are common… line-height of 1.5"), Butterick
  "120% and 145%" (<https://practicaltypography.com/line-spacing.html>), MIT ("print… 120%… websites
  often use 140% to 160%").
- wider measure → more leading: Pimp My Type ("Longer lines need more line height"); Rutter §2.2.1 title.
- larger x-height → less leading: Bringhurst ("small x-heights need more line-spacing", via inkwell.ie).
- larger size → less leading: Pimp My Type ladder (body ~1.5, headings ~1.1).

**CONSTANTS: calibration.** No source gives the `0.006`, `0.8`, or `0.10` coefficients — direction only.

**CLAMP `1.5` ceiling: CONTRADICTED.** WCAG SC 1.4.12 ✔verified: *"Line height (line spacing) to at
least 1.5 times the font size"* and *"…encouraged to allow spacing to surpass the values specified, not
see them as a ceiling"* — <https://www.w3.org/WAI/WCAG22/Understanding/text-spacing.html>. So **1.5 is a
floor, not a max.** Floor `1.2` is fine (Butterick/MIT print ~120%). Web practice exceeds 1.5 (MIT 180%).

**PATH: Digital ≠ Print.** Butterick's range is print-inclusive; MIT ✔(same source as base-size)
states print ~120% vs web 140–160%; Rutter: "the web almost benefits from an increase." Screen leads
looser than print. Your ~1.35–1.5 base is a *digital-leaning* value; a print path centers nearer 1.2.

---

## 3. Tracking — `clamp(−0.04, −k·(step/n)·ln(ratio), 0.04)`

**DIRECTION: GROUNDED.** Butterick "Letterspacing": add letterspacing to lowercase < 9pt, remove it at
large sizes (<https://practicaltypography.com/letterspacing.html>); Unger via Typotheque ("the smaller
the type size, the wider… and the reverse goes for the larger sizes"); Fonts.com.
**CONSTANT `0.04`: calibration** — no per-step rate in the literature (the 5–10% figure is the *all-caps*
case, a different phenomenon, correctly excluded). **PATH: Identical** (a size-optics principle; digital
merely lost metal type's automatic per-size correction — Typotheque).

---

## 4. Word spacing — `base_ws − 0.04·(step/n)·ln(ratio)`

**DIRECTION: GROUNDED.** Bringhurst §2.1.1 verbatim (via webtypography.net): *"At larger sizes, when
letterfit is tightened, the spacing of words can be tightened as well."* — <http://webtypography.net/2.1.1>.
**BASE value GROUNDED:** Bringhurst's standard word space is **M/4 ≈ 0.25em**; your fonts' measured
0.26–0.28 sit in-band. **Slope `k_ws=0.04`: calibration.** **PATH: Identical.**

---

## 5. Measure — `65` characters

**GROUNDED.** Bringhurst/Rutter 45–75, **66 ideal** (<http://webtypography.net/2.1.2>); Butterick 45–90
(<https://practicaltypography.com/line-length.html>); WCAG SC 1.4.8 ≤80. 65 ≈ the ideal, inside all.
**PATH: Identical** — Bringhurst-for-print and Rutter-for-web cite the same numbers; it's a property of
eye movement, not medium.

---

## 6. Measure width — `measure · avg_advance · base` (not `ch`)

**GROUNDED (strongest).** OS/2 `xAvgCharWidth` ✔verified — the legacy definition is exactly a
frequency-weighted mean advance incl. the space (weight 166) — <https://learn.microsoft.com/en-us/typography/opentype/spec/os2>;
Capsize `xWidthAvg` = the same, for line-length inference. `ch` provably overshoots ~20–30% (Meyer,
<https://meyerweb.com/eric/thoughts/2018/06/28/what-is-the-css-ch-unit/>). **PATH: Identical.**

---

## 7. Role size + floor — `max(base·size_factor, size_min)`, `base=11pt`, `size_min=9pt`

**`base=11pt`: GROUNDED.** Butterick ✔verified: *"In print, the optimal point size for body text is
10–12 point"* / *"On the web, the optimal size is 15–25 pixels"* — <https://practicaltypography.com/point-size.html>.
11pt sits in the print range. **`size_min=9pt`: CONVENTION** — accessibility consensus ("<9pt is a
barrier"); WCAG mandates 200% resize, not an absolute floor.
**PATH: base size is IDENTICAL — one canonical base, the cross-renderer comparison anchor.** The digital
size (print 10–12pt vs web 15–25px, by viewing distance — ✔verified, UXmatters) is **not a second base**
but a *derived, overridable suggestion* computed from the canonical base + the screen size. Everything
else is relative to the base, so it re-derives on any change to it. A divergent base would break the
strict cross-renderer diff. (The character measure of §5 is Identical for the same reason.)

---

## 8. Code size — `role_size · (body.xh / code.xh) · 0.9`

**x-height match: GROUNDED.** MDN `font-size-adjust`: keeps "the aspect value across fonts consistent…
text appears similar regardless of the font" — <https://developer.mozilla.org/en-US/docs/Web/CSS/font-size-adjust>;
matklad extends it to intentional mono-beside-body mixing. **`×0.9`: CONVENTION** (0.8–0.9em is a real
practitioner band; Butterick confirms *why* — monospace runs large — but gives no number); arbitrary as
an exact value. "Code stays in the measure": **weak inference** from Butterick's content-agnostic line
length, not stated verbatim. **PATH: Identical value** (x-height ratio is font-intrinsic), **medium
mechanism differs** (font-size-adjust on web; baked relative size in print).

---

## 9. Rhythm unit — `base · 0.375`  ❌ UNSUPPORTED

**No source in the literature defines the vertical unit as a fraction of the raw font size, and 0.375
(3/8) appears in none.** The unit is unanimously the **line-height (leading)** — Rutter ✔verified:
*"The basic unit of vertical space is line height."* — <https://24ways.org/2006/compose-to-a-vertical-rhythm/>;
Bringhurst/Rutter §2.2.2 ("even multiple of the basic leading"). cascade already computes the correct
unit as `baseline = body_size × line_height_ratio`, then discards it — `0.375×base` works out to ~¼ of
the real line and desyncs from the grid when line-height changes. **Fix: `rhythm_unit = baseline`.**
**PATH: Identical** (a print fundamental — Bringhurst → Rutter's CSS port).

---

## 10. Spacing tokens — `rhythm_unit · multiplier`

**GROUNDED.** Spacing as integer multiples of the vertical unit (Bringhurst/Rutter §2.2.2). Caveat: the
multiplier set includes **0.5** (a half-unit); the canonical rule is *whole* multiples of the line — a
half-step is only motivated if the unit is deliberately a half-line, which cascade does not do.
**PATH: Identical.**

---

## 11. Font measurement

**(1) Frequency-weighted `avg_advance`: GROUNDED.** = legacy OS/2 `xAvgCharWidth` ✔verified (weight
table, space=166) and Capsize `xWidthAvg`. Computing it live is correct (spec says don't trust the
stored field). The **~18% space frequency** is calibration (sources: "space is the most frequent char,
~1.66–2× e" — not a pinned 18%).
**(2) opsz pinned to the text end: GROUNDED (principle).** OpenType opsz spec: optical size adapts
"overall width" and recommends "10 to 16… for typical text settings" — pinning a reading face to that
end is sound. The specific Inter x-height figures are cascade's own measurements, not cited.
**(3) Weight handling — `avg_advance` is a MEASURED METRIC, not a formula and not an average-over-usage.**
`avg_advance` is in the same category as `x_height`/`cap_height`: a *measured font metric* that feeds
the `measure_width` formula — not its own formula. It is the one metric that varies with weight, so it
is measured **at the body's actual weight** — the font's default instance (≈ Regular, the weight body
text is set at), exactly as `x_height` is measured there. The `WEIGHT_USAGE` prior (a hardcoded
body-weight *usage* distribution — agent-added, the code's own "one knob") and the averaging it drove
are **eliminable**: they only collapse a weight that isn't actually unknown. If body weight becomes an
overridable input, the advance is emitted **per weight** and a live calc selects it — the same way
`bundle-*` selects a font's pre-measured advance — never a baked guess, and no curve/interpolation
unless weight is a *continuous* live slider. (The frequency-weighted advance itself — the `CHAR_FREQ`
average of glyph widths — IS grounded: OS/2 legacy `xAvgCharWidth` / Capsize; the `18%` space weight is
minor calibration within that grounded method. That averaging stays; the weight-*usage* averaging goes.)
**PATH: Identical** (intrinsic font data, medium-agnostic).

---

## Sources (verified ✔ = re-fetched verbatim by hand)

- ✔ WCAG SC 1.4.12 Text Spacing — <https://www.w3.org/WAI/WCAG22/Understanding/text-spacing.html>
- ✔ Rutter, "Compose to a Vertical Rhythm" — <https://24ways.org/2006/compose-to-a-vertical-rhythm/>
- ✔ Butterick, "Point size" — <https://practicaltypography.com/point-size.html>
- ✔ Mortensen, "The typographic scale" — <https://spencermortensen.com/articles/typographic-scale/>
- ✔ OpenType OS/2 spec (`xAvgCharWidth`) — <https://learn.microsoft.com/en-us/typography/opentype/spec/os2>
- Bringhurst applied to the web (Rutter) — <http://webtypography.net/2.1.1>, `/2.1.2`, `/2.2.1`, `/2.2.2`, `/3.1.1`
- Butterick — line-length, line-spacing, letterspacing, monospaced fonts — practicaltypography.com
- modularscale.com; precise-type.com — modular scale ratios / equal temperament
- MDN `font-size-adjust`; matklad "font-size-adjust Is Useful"; Capsize (seek-oss)
- OpenType `opsz` axis spec — <https://learn.microsoft.com/en-us/typography/opentype/spec/dvaraxistag_opsz>
- Meyer, "What is the CSS `ch` unit" — meyerweb.com
- Pimp My Type; MIT 6.813 typography; Typotheque (Biľak/Unger); Fonts.com; letter-frequency (Wikipedia/Norvig)
- **Not fetched as primary text (flagged):** Bringhurst *Elements*, Tracy *Letters of Credit*, Hochuli
  *Detail in Typography*, Müller-Brockmann *Grid Systems*, Lupton *Thinking with Type*.
