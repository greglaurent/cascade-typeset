// Cascade — shared specimen. Mirrors cascade-css/sample.html word-for-word so the
// viewer's CSS and Typst tabs show the same document in both renderers. The text is
// a short, self-referential essay on setting type that exercises every component.
//
// Options come from `--input` (the viewer's Compile button); defaults mirror the
// CSS sample, so `just pdf` with no inputs is the viewer baseline. For a kitchen-sink
// test of per-call overrides and rhythm helpers, see feature-test.typ.

#import "lib.typ": layout, rhythm

#let _in = sys.inputs
#let _scale   = _in.at("scale",   default: "golden-ditonic")
#let _body    = _in.at("body",    default: "serif")
#let _heading = _in.at("heading", default: "")           // "" = match body
#let _theme   = _in.at("theme",   default: "light")
#let _sidenotes = _in.at("sidenotes", default: "false") == "true"   // margin edition vs. footnotes

#let _fonts = if _heading == "" {
  (body: layout.font.bundles.at(_body))
} else {
  (body: layout.font.bundles.at(_body), heading: layout.font.bundles.at(_heading))
}

#let l = layout.make(
  theme: layout.theme.presets.at(_theme),
  measure: 65,
  scale: _scale,
  sidenotes: _sidenotes,                          // make() widens the margin when true
  page: (paper: "us-letter", numbering: "1"),     // margin left to make() (1in, or wide for sidenotes)
  fonts: _fonts,
)

#show: l.page
#show: l.markup

#let (text-1, text-2, text-3, text-4, text-5, figure, divider, sidenote, marginnote) = l

= The Well-Set Page

#text-1[A specimen exercising every cascade component — the modular scale, optical
profile, vertical rhythm, and theme — written about the craft it demonstrates.]

Typography begins with one decision: the size of the body text. Everything else — the
headings above it, the captions beneath, the space between the lines — follows from
that size by _proportion_ rather than by whim. A *modular scale* fixes those ratios,
so a page holds together the way intervals hold a chord.

Body text asks to be read, not admired. It wants a comfortable measure — around
sixty-six characters to the line — and leading loose enough to carry the eye back to
the left margin without losing the row. The relevant knob is `line-height`; set it
too tight and the lines congeal, too loose and they drift. The reasoning behind these
defaults lives in #link("https://example.com")[the notes].

The finer adjustments compound quietly.#sidenote[Bringhurst calls these the
micro-typographic controls — the ones a reader feels but never consciously notices.]
Tracking opens at display sizes and closes at reading sizes; word spacing does the
opposite. None of it should announce itself — the reader simply finds the page easy to
move through.#sidenote[On the sixty-six-character line and much else, see Robert
Bringhurst, _The Elements of Typographic Style_.]#sidenote[And, on vertical rhythm
specifically, the same volume's chapter on the baseline.]

== On measure and rhythm

A column of text is a grid in disguise. Every line rests on an invisible baseline, and
the space between paragraphs is a whole multiple of it, so the vertical rhythm never
stumbles.#marginnote[Bringhurst's comfortable measure runs 45–75 characters; cascade
defaults to sixty-six.] Even inline code such as `font-size` is pulled onto the same measure.

#quote(attribution: [Robert Bringhurst])[Typography exists to honour content.]

=== A scale, from small to large

#text-1[text-1 · caption and footnote size]
#text-2[text-2 · small print]
#text-3[text-3 · the reading size]
#text-4[text-4 · a subhead or lead]
#text-5[text-5 · a small display line]

// Intentional break: a display line and a subhead wouldn't sit adjacent. Strong
// (additive) so it actually separates the scale demo from the next section.
#v(l.rhythm.baseline)

==== Principles

A few rules carry most of the weight:

- Contrast builds hierarchy — size, weight, and space, used sparingly.
- Restraint keeps that hierarchy legible; a page that shouts says nothing.
- Consistency is the quiet virtue: the same choice, made the same way, throughout.

Put plainly, the process is short:

+ Set the body size and the measure.
+ Derive every other size from the scale.
+ Let the rhythm follow from the leading.

```typst
#import "@local/cascade:0.1.0": layout
#let l = layout.make(scale: "golden-ditonic", measure: 66)
#show: l.page
#show: l.markup
```

#figure(
  caption: [Figure 1. A figure with its caption set in the caption style.],
)[
  #rect(width: 100%, height: 3.5cm, fill: luma(80%), stroke: none)
]

#divider()
