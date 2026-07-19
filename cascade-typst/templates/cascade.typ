// cascade — GENERATED Typst library. Baked from the spec by cascade-typst; do not edit by hand.
//
// Print is the NUMERIC projection: every size / tracking / leading / word-space / rhythm / colour
// below is a concrete value computed in Rust via the shared `formula::*` (f64) for the resolved
// config, then baked here. There is no runtime, so the CSS renderer's reactive class switches are
// gone — the config (scale, body/heading/code faces, theme) is fixed at build time.
//
// Usage:
//   #import "cascade.typ": make
//   #let l = make()                 // or make(sidenotes: true) for the margin-note edition
//   #show: l.page
//   #show: l.markup
//   = Heading
//   Body with *bold*, _italic_, `code`, and a footnote.#footnote[...]

// ── baked palette (light) ─────────────────────────────────────────────────────
#let theme = (
  fg:          rgb("{{ fg }}"),
  fg-muted:    rgb("{{ fg_muted }}"),
  fg-subtle:   rgb("{{ fg_subtle }}"),
  bg:          rgb("{{ bg }}"),
  bg-subtle:   rgb("{{ bg_subtle }}"),
  rule:        rgb("{{ rule }}"),
  accent:      rgb("{{ accent }}"),
  accent-rule: rgb("{{ accent_rule }}"),
  link:        rgb("{{ link }}"),
  code-fg:     rgb("{{ code_fg }}"),
  code-bg:     rgb("{{ code_bg }}"),
  quote-rule:  rgb("{{ quote_rule }}"),
  quote-bg:    {{ quote_bg_expr }},
)

// ── baked families ────────────────────────────────────────────────────────────
#let fonts = (body: "{{ body_family }}", heading: "{{ heading_family }}", code: "{{ code_family }}")

// ── baked type scale + optical (per document role) ────────────────────────────
// SIZED roles carry size (pt), tracking / spacing (word-space) in em, te/be = the line-box top/bottom
// edges (em) that make the box the full line-height with leading:0, above/below = block margins (pt),
// and fill = the role's colour. INLINE DECORATIONS (strong/emphasis/link/inline-code) carry ONLY their
// decoration and inherit size/leading/tracking from context — like CSS, which sets them no font-size
// (code keeps an x-height match, but as a relative em factor so it scales with the surrounding size).
#let typ = (
{% for r in rows %}  {{ r.name }}: ({{ r.body }}),
{% endfor %})

// ── baked vertical rhythm ─────────────────────────────────────────────────────
#let rhythm = (
  unit: {{ unit }}pt,
  baseline: {{ baseline }}pt,
  spacing: (n1: {{ sp_n1 }}pt, base: {{ sp_base }}pt, p1: {{ sp_p1 }}pt, p2: {{ sp_p2 }}pt, p3: {{ sp_p3 }}pt, p4: {{ sp_p4 }}pt, p5: {{ sp_p5 }}pt, p6: {{ sp_p6 }}pt),
)

// ── render helpers ────────────────────────────────────────────────────────────
// Apply a baked role to a body: text() with the role's size/tracking/word-space/fill (+ weight/
// style when the role carries them). `font` and `fill` may be overridden per call.
// word-space is cascade's CSS `word-spacing` ADDITION (an em fraction added to the normal space),
// so in Typst it is `100% + <ws>` (relative to the font's own space), NOT an absolute width — a bare
// `0em` would collapse spaces (e.g. in mono code, whose word-space is 0).
// te/be set the line box to the role's full line-height (with `set par(leading: 0pt)`), so line
// spacing is METRIC-INDEPENDENT like CSS `line-height` AND a block is `N × line-height` tall — the
// half-leading appears above the first line and below the last, matching CSS's box model exactly.
#let _apply(t, body, font: fonts.body, fill: auto) = {
  let args = (
    font: font,
    size: t.size,
    tracking: t.tracking,
    spacing: 100% + t.spacing,
    top-edge: t.te,
    bottom-edge: t.be,
    fill: if fill == auto { t.fill } else { fill },
  )
  if "weight" in t { args.insert("weight", t.weight) }
  if "style" in t { args.insert("style", t.style) }
  text(..args, body)
}

// A document role. Roles that carry block margins (headings, code-block, quote) become a `block`;
// the rest flow as ordinary paragraphs (so `par.spacing` separates them) — matching CSS block flow.
#let _role(t, body, font: fonts.body) = {
  let inner = { set par(leading: 0pt); _apply(t, body, font: font) }
  if "above" in t or "below" in t {
    block(above: t.at("above", default: auto), below: t.at("below", default: auto), inner)
  } else {
    inner
  }
}

// ── public API ────────────────────────────────────────────────────────────────
#let make(sidenotes: false, page: (:)) = {
  let sp = rhythm.spacing
  let _note-body = it => text(size: typ.text-1.size, fill: theme.fg-subtle, it)

  let result = (:)

  // scale roles (text-1 … text-5) — block text at a fixed step.
  for key in ("text-1", "text-2", "text-3", "text-4", "text-5") {
    result.insert(key, body => _role(typ.at(key), body))
  }
  // headings — block role, heading family, own leading + margins.
  for (key, lvl) in (("heading-1", 1), ("heading-2", 2), ("heading-3", 3), ("heading-4", 4)) {
    result.insert(key, body => _role(typ.at(key), body, font: fonts.heading))
  }
  // Inline decorations — apply ONLY the decoration and INHERIT size/leading/tracking from context
  // (matching CSS `strong{font-weight}`, `em{font-style}`, `a{color;underline}`, which set no size).
  // Using an absolute `_apply` here would reset e.g. italic inside a 9pt footnote back to 11pt.
  result.insert("strong", body => text(weight: typ.strong.weight, body))
  result.insert("emphasis", body => text(style: typ.emphasis.style, body))
  result.insert("link", (dest, body) => link(dest, underline(
    stroke: 0.5pt + theme.link, offset: 0.15em, text(fill: typ.link.fill, body),
  )))
  // Inline code: a tinted pill that HUGS the glyphs, sitting on the text baseline like ordinary
  // inline text — NOT the tall line-height box (te/be), which would tower over the line. The code
  // text uses the font's own edges (default ascender/descender); `outset` extends the fill a touch
  // above/below without changing layout (so the line height is untouched); radius 2.25pt = CSS 3px.
  result.insert("code", body => box(
    fill: theme.code-bg, inset: (x: 0.34em), outset: (y: 0.12em), radius: 2.25pt,
    text(
      font: fonts.code, size: typ.code.size, tracking: typ.code.tracking,
      spacing: 100% + typ.code.spacing, fill: theme.code-fg, body,
    ),
  ))
  result.insert("code-block", body => block(
    fill: theme.code-bg, inset: 1em, radius: 4pt, width: 100%, breakable: true,
    below: typ.code-block.at("below", default: rhythm.baseline),
    {
      // Code never hyphenates or justifies: a line that exceeds the measure soft-wraps at whitespace,
      // never mid-token (so a flag like `--verify` can't split into `--` / `verify`). Print's analogue
      // of the CSS block's `overflow-x: auto` (which keeps the line intact and scrolls instead).
      set par(leading: 0pt, justify: false)
      set text(hyphenate: false)
      _apply(typ.code-block, body, font: fonts.code, fill: theme.code-fg)
    },
  ))
  // Quote: matches CSS `padding: <p1> 2em; border-inline-start: 3px`. Vertical padding (p1) top AND
  // bottom + 2em on both sides → box height + left-rule length match CSS; stroke 2.25pt (= 3px @96ppi)
  // = the CSS border. The attribution is a SEPARATE right-aligned, UPRIGHT paragraph (a full baseline
  // gap before it) — as the CSS specimen sets it — not an italic line-break tacked on.
  result.insert("quote", (body, attribution: none) => block(
    inset: (x: 2em, y: rhythm.spacing.p1), stroke: (left: 2.25pt + theme.quote-rule),
    fill: theme.quote-bg, below: rhythm.baseline,
    {
      set par(leading: 0pt, spacing: rhythm.baseline)
      _apply(typ.quote, body)
      if attribution != none {
        parbreak()
        align(right, text(size: typ.quote.size, fill: typ.quote.fill, [— #attribution]))
      }
    },
  ))
  result.insert("figure-caption", body => align(center, _apply(typ.caption, body)))
  result.insert("divider", () => block(
    above: rhythm.baseline, below: rhythm.baseline, line(length: 100%, stroke: 0.5pt + theme.rule),
  ))
  result.insert("figure", (body, caption: none) => figure(
    body, caption: if caption != none { figure.caption(caption) } else { none },
  ))

  // Notes. sidenotes:true → margin notes (best-effort marge; falls back to footnotes if the
  // package is unavailable). sidenotes:false (default) → native numbered footnotes, no deps.
  if sidenotes {
    result.insert("sidenote", body => footnote(_note-body(body)))
    result.insert("marginnote", body => footnote(_note-body(body)))
  } else {
    result.insert("sidenote", body => footnote(body))
    result.insert("marginnote", body => footnote(body))
  }

  // Page — paper, margins, background, default text.
  // Horizontal margin centres the reading measure (65 chars) on the page — matching the CSS
  // `--cf-measure-inline`; vertical margin is the usual 1in. Override via `page:` per call.
  let _default-page = (paper: "us-letter", margin: (x: {{ margin_x }}pt, y: 1in), numbering: none, fill: theme.bg)
  let merged-page = _default-page + page
  result.insert("page", body => {
    set std.page(..merged-page)
    set text(font: fonts.body, fill: theme.fg)
    body
  })

  // Markup — bind native Typst elements to the baked roles. Body defaults = the `body` role.
  let b = typ.body
  result.insert("markup", body => {
    set text(font: fonts.body, size: b.size, tracking: b.tracking, spacing: 100% + b.spacing, top-edge: b.te, bottom-edge: b.be, fill: theme.fg)
    set par(leading: 0pt, spacing: rhythm.baseline, justify: false)
    set footnote.entry(
      separator: line(length: 30%, stroke: 0.5pt + theme.rule),
      clearance: 1em, gap: 0.5em, indent: 1em,
    )
    show heading.where(level: 1): it => (result.heading-1)(it.body)
    show heading.where(level: 2): it => (result.heading-2)(it.body)
    show heading.where(level: 3): it => (result.heading-3)(it.body)
    show heading.where(level: 4): it => (result.heading-4)(it.body)
    show strong: it => (result.strong)(it.body)
    show emph: it => (result.emphasis)(it.body)
    show link: it => if it.body.func() == underline { it } else { (result.link)(it.dest, it.body) }
    show raw.where(block: false): it => (result.code)(it)
    show raw.where(block: true): it => (result.code-block)(it)
    show quote: it => (result.quote)(it.body, attribution: it.attribution)
    show std.figure.caption: it => (result.figure-caption)(it.body)
    show footnote.entry: it => text(size: typ.footnotes.size, fill: theme.fg-subtle, it)
    body
  })

  result.insert("theme", theme)
  result.insert("rhythm", rhythm)
  result.insert("typ", typ)
  result
}
