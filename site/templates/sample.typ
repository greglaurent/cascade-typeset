// cascade — site specimen (print). Mirrors `templates/sample.html` so the CSS and Typst (PDF) tabs
// show the SAME document. The config (scale, fonts, theme) is BAKED into `cascade.typ` by the site's
// renderer before this compiles; the only runtime knob is the notes edition (margin notes vs. plain
// footnotes), passed as `--input sidenotes=<bool>` — everything else is fixed at bake time.

#import "cascade.typ": make

#let _sidenotes = sys.inputs.at("sidenotes", default: "false") == "true"
#let l = make(sidenotes: _sidenotes, page: (numbering: "1"))

#show: l.page
#show: l.markup

#let (text-1, text-2, text-3, text-4, text-5, figure, divider, sidenote, marginnote) = l

= The Well-Set Page

#text-1[A specimen exercising every cascade component — the modular scale, optical profile,
vertical rhythm, and theme — written about the craft it demonstrates.]

Typography begins with one decision: the size of the body text. Everything else — the headings
above it, the captions beneath, the space between the lines — follows from that size by
_proportion_ rather than by whim. A *modular scale* fixes those ratios, so a page holds together
the way intervals hold a chord.

Body text asks to be read, not admired. It wants a comfortable measure — around sixty-six
characters to the line — and leading loose enough to carry the eye back to the left margin without
losing the row. The relevant knob is `line-height`; set it too tight and the lines congeal, too
loose and they drift. The reasoning behind these defaults lives in
#link("https://example.com")[the notes].

The finer adjustments compound quietly.#sidenote[Bringhurst calls these the micro-typographic
controls — the ones a reader feels but never consciously notices.] Tracking opens at display sizes
and closes at reading sizes; word spacing does the opposite. None of it should announce itself —
the reader simply finds the page easy to move through.#sidenote[On the sixty-six-character line and
much else, see Robert Bringhurst, _The Elements of Typographic Style_.]#sidenote[And, on vertical
rhythm specifically, the same volume's chapter on the baseline.]

== On measure and rhythm

A column of text is a grid in disguise. Every line rests on an invisible baseline, and the space
between paragraphs is a whole multiple of it, so the vertical rhythm never
stumbles.#marginnote[Bringhurst's comfortable measure runs 45–75 characters; cascade defaults to
sixty-six.] Even inline code such as `font-size` is pulled onto the same measure.#footnote[A
traditional footnote, placed by hand at the foot of the document — numbered automatically, linked
back to its marker, and coexisting with the margin notes above.]

#quote(attribution: [Robert Bringhurst])[Typography exists to honour content.]

=== A scale, from small to large

#text-1[text-1 · caption and footnote size]
#text-2[text-2 · small print]
#text-3[text-3 · the reading size]
#text-4[text-4 · a subhead or lead]
#text-5[text-5 · a small display line]

// A display line and a subhead wouldn't sit adjacent; add a baseline of air before the next section.
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

```
cascade build --out dist --verify
```

#figure(caption: [Figure 1. A figure with its caption set in the caption style.])[
  #rect(width: 100%, height: 3.5cm, fill: l.theme.bg-subtle, stroke: none)
]

#divider()
