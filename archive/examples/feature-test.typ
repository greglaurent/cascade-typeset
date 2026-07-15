// Cascade — feature / override test (kitchen sink). The exhaustive exercise of
// per-component overrides, per-call args, the auto filter, justify, and the rhythm
// helpers. The clean shared specimen is sample.typ.
//   typst compile --root . examples/feature-test.typ

#import "@local/cascade:0.1.0": layout, rhythm

// Options come from `--input` (the viewer's Compile button passes the selected
// scale / body / heading / theme). Defaults mirror the CSS sample, so `just pdf`
// with no inputs produces the viewer's baseline.
#let _in = sys.inputs
#let _scale   = _in.at("scale",   default: "golden-ditonic")
#let _body    = _in.at("body",    default: "serif")
#let _heading = _in.at("heading", default: "")           // "" = match body
#let _theme   = _in.at("theme",   default: "light")

#let _fonts = if _heading == "" {
  (body: layout.font.bundles.at(_body))
} else {
  (body: layout.font.bundles.at(_body), heading: layout.font.bundles.at(_heading))
}

#let l = layout.make(
  theme: layout.theme.presets.at(_theme),
  measure: 65,
  scale: _scale,
  page: (paper: "us-letter", margin: 1in, numbering: "1"),
  fonts: _fonts,
)

#show: l.page
#show: l.markup

#let (
  heading-1,
  heading-2,
  heading-3,
  heading-4,
  text-1,
  text-2,
  text-3,
  link,
  emphasis,
  strong,
  code,
  code-block,
  quote,
  figure-caption,
  list,
  enum,
  figure,
  divider,
  footnote-entry,
) = l

// ─── Headings ──────────────────────────────────────────────────────────────────
#heading-1[Heading 1 — display]
#divider()
#heading-2[Heading 2]
#heading-3[Heading 3]
#heading-4[Heading 4]

// Divider customization
#divider(stroke: 1pt + rgb("#0066cc"), length: 50%)

// ─── Body text ─────────────────────────────────────────────────────────────────
#text-1(lorem(35))
#text-2(lorem(35))
#text-3(lorem(35))

// ─── Inline decorations inside body ────────────────────────────────────────────
#text-3[
  This paragraph contains #emphasis[italic emphasis], #strong[bold strong],
  and #code[inline code], plus a #link(dest: "https://typst.app")[link to Typst].
  The decorations inherit the surrounding text size.
]

// ─── Block decorations ─────────────────────────────────────────────────────────
#quote[
  The truth is rarely pure and never simple. — Oscar Wilde
]

#code-block[
  fn fibonacci(n: u32) -> u32 {
  if n < 2 { n } else { fibonacci(n - 1) + fibonacci(n - 2) }
  }
]

#figure-caption[Figure 1. A standalone caption used outside of a figure block.]

// ─── Lists ─────────────────────────────────────────────────────────────────────
#list[
  - First unordered item
  - Second item with #emphasis[emphasis] inside
  - Third item
]

#enum[
  + First ordered item
  + Second
  + Third
]

#enum(numbering: "a)", start: 3)[
  + Override numbering scheme and starting value
  + Letters in parentheses, starting from "c)"
]

// ─── Figure ────────────────────────────────────────────────────────────────────
#figure(
  caption: [A figure with a caption rendered via the layout component.],
  numbering: "1",
)[
  #rect(width: 4cm, height: 2cm, fill: rgb("#ddd"))
]

// ─── Per-call overrides ────────────────────────────────────────────────────────
#text-3(weight: 700, fill: rgb("#222"))[Per-call: bold + dark gray.]
#link(dest: "https://example.com", underline-stroke: 1pt, fill: red)[
  Red link with custom underline stroke.
]
#code-block(block-fill: rgb("#222"), fill: rgb("#eee"))[
  fn dark-theme() {}
]
#code-block(block-stroke: 0.5pt + rgb("#888"))[
  fn outlined-code() {}  // code-block with a thin gray border
]
#quote(block-inset: (left: 4em), style: "normal")[
  Quote with deeper indent and no italic override.
]
#quote(
  block-fill: rgb("#fff7d0"),
  block-inset: 1em,
  block-radius: 4pt,
)[
  Quote with yellow background, uniform padding, rounded corners.
]
#quote(
  block-stroke: (left: 3pt + rgb("#0066cc")),
  block-inset: (left: 1em),
)[
  Quote with a blue left rule \u{2014} the modern callout style.
]

// ─── auto filter ───────────────────────────────────────────────────────────────
#let partial = (weight: 700, fill: auto, tracking: auto)
#text-3(..partial)[Bold; fill and tracking stay at the layout defaults.]

// ─── Justify (per-component override) ──────────────────────────────────────────
#text-3(justify: true)[
  This paragraph has `justify: true` applied per-call so the text fills the full
  measure with stretched inter-word spacing, except the last line. The same
  parameter can be set globally via `layout.make(justify: true)` or per-component
  via `overrides: (text-3: (justify: true))`. Useful for body text in book-like
  documents where ragged-right would feel out of place.
]

// ─── Vertical rhythm ───────────────────────────────────────────────────────────
#let r = rhythm.make(
  scale: layout.font.bundles.serif.scale,
  font: layout.font.bundles.serif.profile,
  measure: 65,
  grid-unit: 4pt,
)

#text-3[
  Rhythm params: unit = #r.unit, baseline = #r.baseline,
  atomic-divisor = #r.params.atomic-divisor.
]

#text-3[
  Spacing scale: n1 = #r.spacing.n1, base = #r.spacing.base, p1 = #r.spacing.p1,
  p2 = #r.spacing.p2, p3 = #r.spacing.p3, p4 = #r.spacing.p4, p5 = #r.spacing.p5, p6 = #r.spacing.p6.
]

// Snap functions
#let (unit, baseline, spacing, snap, params) = r
#text-3[
  Snap 17.4pt to unit: #snap(17.4pt). Snap 17.4pt to baseline: #snap(17.4pt, multiple: baseline).
]

// Apply rhythm spacing to layout components via overrides
#let l-rhythmed = layout.make(
  measure: 65,
  overrides: (
    code-block: (block-inset: spacing.p2),
    quote: (block-inset: (left: spacing.p3)),
  ),
)
#(l-rhythmed.quote)[
  Quote with rhythm-derived left inset (#spacing.p3).
]

// ─── Native markup mode ────────────────────────────────────────────────────────
// With `#show: l.markup` applied at the top, Typst's native markup routes through
// our components automatically. The block below uses no explicit component calls.

= Native markup heading 1

== Native markup heading 2

This is a plain paragraph written in pure Typst markup. It picks up text-3's
defaults (size, tracking, word-space, leading, fill) from the markup show rule.
*Bold inline* and _italic inline_ work via our strong/emphasis components.
`Inline code` routes through the code component. And a markup link to
#link("https://typst.app")[Typst] gets the theme accent color plus underline.

=== Heading 3

- Native list item one
- Native list item two with _emphasis_
- Native list item three

+ Numbered item one
+ Numbered item two

```rust
fn native_markup() -> Result<(), Box<dyn Error>> {
    println!("Native code block via show rule");
    Ok(())
}
```

#quote(attribution: [Bringhurst])[
  Typography exists to honor content.
]

= Footnotes

This paragraph has a footnote#footnote[The footnote text is set in
  text-2 (one step below body) and uses the muted foreground color, with
  a 30% rule above the entries.] right here. And a second one#footnote[Footnotes
  appear at the bottom of the page they reference, with a thin rule above.]
to demonstrate gap spacing.
