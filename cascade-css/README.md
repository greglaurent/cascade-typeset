# cascade-css

A faithful CSS port of the **cascade** Typst typography system
([`../cascade-typst`](../cascade-typst)) — the whole thing: the modular scale (all
presets), the optical profile, vertical rhythm, the light/dark theme, and every
component. Everything is driven by CSS custom properties, so you tweak it the
same way you'd pass arguments to `layout.make(...)`. The token-driven modules are
generated from the repo's `tools/tokens.mjs`; see the top-level README.

## Use

```html
<link rel="stylesheet" href="cascade.css">

<article class="cascade">
  <h1>…</h1>
  <p>…</p>
</article>
```

Everything inside `.cascade` is styled. Content is capped at the measure and
centered.

## Modules (mirror cascade's)

| file | cascade module | what it defines |
|---|---|---|
| `theme.css`  | `theme.typ`  | color tokens `--ct-*`, light + auto-derived dark |
| `scale.css`  | `scale.typ`  | fluid modular scale `--cs-*`, all 7 presets |
| `font.css`   | `font.typ`   | optical profile `--cf-*` (tracking/leading/word-space) + families |
| `rhythm.css` | `rhythm.typ` | spacing scale `--cr-*` |
| `layout.css` | `layout.typ` | elements bound to components |
| `cascade.css`| entry        | `@import`s all of the above |

## Switch scale / profile / theme

Add a class or attribute to the `.cascade` box; everything recomputes:

```html
<article class="cascade scale-golden-ditonic profile-serif-text" data-theme="dark">
```

- **scales:** `scale-classical` (default) · `scale-golden-ratio` ·
  `scale-golden-ditonic` · `scale-tritonic` · `scale-tetratonic` ·
  `scale-major-third` · `scale-minor-third`
- **bundles** (body family + optical profile, the visible "style" switch, =
  cascade's `font.bundles`): generic `bundle-serif` (default) · `bundle-sans` ·
  `bundle-mono`; and the measured font presets `bundle-lora` · `bundle-inter` ·
  `bundle-jost` — each needs its `fonts/<name>.css` loaded (see below)
- **heading bundles** (heading family + its optical profile — pair with a body
  bundle for serif/sans mixes): `heading-serif` · `heading-sans` · `heading-mono`,
  plus `heading-lora` · `heading-inter` · `heading-jost`
  (e.g. `class="cascade bundle-serif heading-sans"` = serif body, sans headings)
- **profiles** (body optical only — tracking/leading/word-space, no family change):
  `profile-serif-text` · `profile-sans-text` · `profile-sans-ui` · `profile-sf-pro`
- **theme:** `data-theme="light"` / `data-theme="dark"` (defaults to system pref)
- **font presets:** the font-specific `bundle-*` / `heading-*` classes need their preset
  stylesheet loaded — e.g. `<link rel="stylesheet" href="fonts/jost.css">` (likewise
  `lora.css`, `inter.css`) — which sets the family and that face's measured optical
  metrics. The typeface itself must be installed/available.

## Tweak anything

Override a custom property on `.cascade` (or `:root`):

```css
.cascade {
  --cs-base: clamp(1.2rem, 1rem + 0.6vw, 1.5rem);  /* fluid body size */
  --cf-measure: 85;                                 /* line length, chars */
  --ct-accent: #8B2F2A;                             /* oxblood */

  /* fonts are set PER CATEGORY (cascade's fonts.body / .heading / .code) */
  --cf-family-body:    "Lora", Georgia, serif;      /* body */
  --cf-family-heading: "Inter", sans-serif;         /* headings (serif/sans mix) */
  --cf-family-code:    "JetBrains Mono", monospace; /* code */
}
```

Set only `--cf-family-body` and headings follow it (single-font); set
`--cf-family-heading` too for a pairing. Or swap a whole stack
(`--cf-font-serif`/`--cf-font-sans`/`--cf-font-mono`) to redefine what the bundles
resolve to.

Key knobs: `--cs-base` (fluid fundamental), `--cs-ratio`/`--cs-n` (scale),
`--cf-measure`, `--cf-lb`/`--cf-lmin`/`--cf-lmax` (leading + Butterick clamp),
`--cf-kt`/`--cf-tc` (tracking), `--cf-bws`/`--cf-kws` (word-space),
`--cf-size-min` (readability floor for small roles — catches aggressive scales
like golden-ratio; set `0` to disable), the `--cf-font-*` families /
`--cf-family-*` per category, `--cr-unit` (rhythm), and the `--ct-*` colors.

## How it maps (faithful notes)

- **Scale** `f_i = base · ratio^(i/n)` is computed live with `pow()`. Because
  `--cs-base` is a fluid `clamp()`, the whole scale scales with the viewport
  (Utopia-style) while keeping cascade's single ratio.
- **Optical** formulas use `log()`, with cascade's `optical-size` reinterpreted as
  the body base (tracking 0 at body, tighter at display, looser at captions).
  Leading is clamped to Butterick's 1.2–1.45, matching the cascade fix.
- **Dark** is cascade's `derive-dark` (HSL lightness flipped), precomputed to hex.
- **Approximations** (Typst is programmatic, CSS is coarser): there's no true CSS
  baseline grid, so rhythm approximates with line-height + margins; and optical
  tracking/leading are per-scale-step rather than per-exact-size.

## Requirements

Modern CSS: `pow()`, `log()` (Chrome/Edge 111+, Safari 15.4+, Firefox 118+).
No build step, no dependencies.

See `sample.html` for a full specimen with live scale/profile/theme switching.
