// Layout — named callable components composed from per-category font bundles + theme.
//
// Each component belongs to a category (`body`, `heading`, or `code`). Each category gets a
// font bundle — `{ family, scale, profile }` — that determines:
//   - which typeface family the component uses
//   - which scale produces its sizes
//   - which optical profile produces its tracking / leading / word-space
//
// Override at three layers, low → high precedence:
//   1. Built-in component defaults (the `_build-specs` function below)
//   2. Per-component overrides at make time:  overrides: (heading-1: (...))
//   3. Per-call named arguments:              heading-1(weight: 800)[...]
//
// Meta-override keys — `scale`, `font-profile`, `measure`, `step` — trigger recomputation
// of size / tracking / spacing / leading from the new inputs.
//
// Any override value of `auto` is filtered out before merging — treated as
// "use the default that would otherwise apply." Partial override dicts compose cleanly.
//
// Usage:
//   #let l = layout.make()                                  // serif body + heading, mono code
//   #let (heading-1, text-3, link, emphasis, strong, code, code-block, quote, figure-caption,
//         list, enum, figure) = l
//   #show: l.page
//   #show: l.markup
//   = Heading
//   Body with *bold* and _italic_ and `code`.
//
// Switch to a different bundle per category:
//   #let l = layout.make(
//     fonts: (
//       body:    font.bundles.serif,
//       heading: font.bundles.sans,
//     ),
//   )
//
// Override family only (keep bundle's scale + profile):
//   #let l = layout.make(
//     fonts: (
//       body: (..font.bundles.serif, family: "Source Serif Pro"),
//     ),
//   )

#import "scale.typ"
#import "font.typ"
#import "rhythm.typ"
#import "theme.typ"
#import "utils.typ"
#import "@preview/marge:0.1.0": sidenote as _marge-note   // margin-note placement (l.sidenote / l.marginnote)

// Capture scale.make + scale.presets at module load so they survive `scale` shadowing inside make().
#let _scale-make = scale.make
#let _scale-presets = scale.presets

// ─── Component registry ────────────────────────────────────────────────────────
// Each entry: step | none, category (body/heading/code), default param dict, render fn.

#let _build-specs(t, r) = (
  text-1:         (step: -2,   category: "body",    defaults: (fill: t.fg-muted),                        render: utils.render-text),
  text-2:         (step: -1,   category: "body",    defaults: (fill: t.fg),                              render: utils.render-text),
  text-3:         (step: 0,    category: "body",    defaults: (fill: t.fg),                              render: utils.render-text),
  text-4:         (step: 1,    category: "body",    defaults: (fill: t.fg),                              render: utils.render-text),
  text-5:         (step: 2,    category: "body",    defaults: (fill: t.fg),                              render: utils.render-text),
  heading-1:      (step: 4,    category: "heading", defaults: (weight: 700, fill: t.fg, above: r.p4, below: r.p2), render: utils.render-text),
  heading-2:      (step: 3,    category: "heading", defaults: (weight: 600, fill: t.fg, above: r.p3, below: r.p1), render: utils.render-text),
  heading-3:      (step: 2,    category: "heading", defaults: (weight: 600, fill: t.fg, above: r.p2, below: r.base), render: utils.render-text),
  heading-4:      (step: 1,    category: "heading", defaults: (weight: 600, fill: t.fg, above: r.p1, below: r.base), render: utils.render-text),
  link:           (step: none, category: "body",    defaults: (fill: t.link),                            render: utils.render-link),
  emphasis:       (step: none, category: "body",    defaults: (style: "italic", fill: t.fg),             render: utils.render-text),
  strong:         (step: none, category: "body",    defaults: (weight: 700, fill: t.fg),                 render: utils.render-text),
  code:           (step: none, category: "code",    defaults: (fill: t.code-fg),                         render: utils.render-text),
  code-block:     (step: 0,    category: "code",    defaults: (
                                  fill: t.code-fg,
                                  block-fill: t.code-bg,
                                  block-inset: 1em,
                                  block-radius: 4pt,
                                ),                                                                       render: utils.render-code-block),
  quote:          (step: 0,    category: "body",    defaults: (style: "italic", fill: t.fg, block-inset: (left: 2em), block-fill: t.quote-bg), render: utils.render-quote),
  figure-caption: (step: -2,   category: "body",    defaults: (style: "italic", fill: t.fg-muted),       render: utils.render-figure-caption),
  list:           (step: 0,    category: "body",    defaults: (fill: t.fg),                              render: utils.render-list),
  enum:           (step: 0,    category: "body",    defaults: (fill: t.fg),                              render: utils.render-enum),
  figure:         (step: none, category: "body",    defaults: (:),                                       render: utils.render-figure),
  divider:        (step: none, category: "body",    defaults: (stroke: 0.5pt + t.rule, length: 100%),    render: utils.render-divider),
  footnote-entry: (step: -2,   category: "body",    defaults: (fill: t.fg-muted),                        render: utils.render-text),
)

// ─── Defaults ─────────────────────────────────────────────────────────────────

#let _default-fonts = (
  body:    font.bundles.serif,
  heading: font.bundles.serif,
  code:    font.bundles.mono,
)

#let _default-page = (
  paper: "us-letter",
  margin: 1in,
  numbering: none,
)

// ─── Public API ────────────────────────────────────────────────────────────────

#let make(
  theme: theme.presets.light,
  theme-overrides: (:),
  scale: auto,
  base: 11pt,
  measure: 65,
  size-min: 8pt,   // readability floor for small roles; set 0pt to disable
  justify: false,
  sidenotes: false,   // Tufte page: widen the outer margin for l.sidenote / l.marginnote
  font: none,
  fonts: (:),
  page: (:),
  overrides: (:),
) = {
  let theme = theme + theme-overrides

  // Resolve top-level scale: string → preset lookup; dict → use directly; auto → use bundle defaults.
  let global-scale = if scale == auto {
    auto
  } else if type(scale) == str {
    _scale-presets.at(scale)
  } else {
    scale
  }

  // Rebuild a scale dict with the user's base, keeping its ratio + n.
  let with-base = s => _scale-make(base: base, ratio: s.params.ratio, n: s.params.n)

  // Merge user fonts with defaults, per category. Apply global scale (if set), then global base.
  // Top-level `font:` sets the family across every category as a shortcut; per-category
  // `fonts.<cat>.family` still wins for explicit overrides.
  //
  // Within `fonts.<cat>`:
  //   - `family` / `scale` / `profile` configure the bundle itself.
  //   - `size` rebuilds the bundle's scale with that size as the new base, so every
  //     component in the category recomputes coherently from its step.
  //   - any other keys (`weight`, `fill`, etc.) are collected as per-category overrides
  //     applied to every component. Per-component `overrides:` beat these; per-call args
  //     beat both.
  let _bundle-keys = ("family", "scale", "profile")
  let resolved-fonts = (:)
  let cat-overrides = (:)
  for (cat, default-bundle) in _default-fonts {
    let user-bundle = fonts.at(cat, default: (:))
    let bundle-cfg = (:)
    let cat-ov = (:)
    let cat-base = none
    for (k, v) in user-bundle {
      if _bundle-keys.contains(k) { bundle-cfg.insert(k, v) }
      else if k == "size" { cat-base = v }
      else { cat-ov.insert(k, v) }
    }
    let base-bundle = if font != none { default-bundle + (family: font) } else { default-bundle }
    let merged = base-bundle + bundle-cfg
    if global-scale != auto {
      merged.insert("scale", global-scale)
    }
    let scale-base = if cat-base != none { cat-base } else { base }
    merged.insert("scale", _scale-make(base: scale-base, ratio: merged.scale.params.ratio, n: merged.scale.params.n))
    resolved-fonts.insert(cat, merged)
    cat-overrides.insert(cat, cat-ov)
  }

  let body-bundle = resolved-fonts.body
  let r = rhythm.make(scale: body-bundle.scale, font: body-bundle.profile, measure: measure)
  let specs = _build-specs(theme, r.spacing)
  // Sidenote page: the outer (right) margin is the note band. Derived from the reading
  // measure (`measure` chars at the body size, ~0.5em each) rather than a frozen 2.5in,
  // so it scales with `measure` and `base`. The band is half the measure — mirroring the
  // CSS note = 50% of the column. Left/top/bottom keep the base page margin.
  let _measure-width = measure * 0.5 * base                     // the measure, as a length (≈ 4.97in default)
  let sidenote-page = if sidenotes {
    (margin: (left: _default-page.margin, right: _measure-width * 0.5, y: _default-page.margin))
  } else { (:) }
  let merged-page = _default-page + (fill: theme.bg) + sidenote-page + page

  let result = (:)
  for (name, spec) in specs {
    let cat-bundle = resolved-fonts.at(spec.category)
    let cat-state = (
      scale: cat-bundle.scale,
      font: cat-bundle.profile,
      measure: measure,
      size-min: size-min,
    )
    // Family becomes a text-param default; component overrides win over it.
    let cat-defaults = (font: cat-bundle.family) + spec.defaults
    let merged-spec = (
      step: spec.step,
      defaults: cat-defaults,
      render: spec.render,
    )
    let cat-ov = cat-overrides.at(spec.category, default: (:))
    let comp-overrides = cat-ov + overrides.at(name, default: (:))
    result.insert(name, utils.make-component(cat-state, merged-spec, comp-overrides))
  }

  // Notes. `sidenotes: true` places them in the wide outer margin via `marge`
  // (`sidenote` numbered, `marginnote` unnumbered, styled as the caption role) — the
  // Tufte edition. `sidenotes: false` (the default, "standards" edition) drops both to
  // ordinary numbered footnotes at the foot of the page, so the same document compiles
  // to a submission-safe form with no source changes.
  let _note-body = it => text(size: (body-bundle.scale.size)(-2), fill: theme.fg-muted, it)
  if sidenotes {
    result.insert("sidenote", body => _marge-note(numbering: "1", _note-body(body)))
    result.insert("marginnote", body => _marge-note(numbering: none, _note-body(body)))
  } else {
    result.insert("sidenote", body => std.footnote(body))
    result.insert("marginnote", body => std.footnote(body))
  }

  // Page rule — applies page settings and sets default text font/fill from body bundle.
  let page-rule = body => {
    set std.page(..merged-page)
    set text(font: body-bundle.family, fill: theme.fg)
    body
  }
  result.insert("page", page-rule)

  // Markup rule — binds native Typst elements to our components. Apply via
  // `#show: l.markup`. Body defaults match text-3 (body-category scale step 0).
  let body-size = (body-bundle.scale.size)(0)
  let body-tracking = (body-bundle.profile.tracking)(body-size)
  let body-spacing = (body-bundle.profile.word-space)(body-size)
  let body-leading = (body-bundle.profile.leading)(body-size, measure: measure)

  let markup-rule = body => {
    // ── set rules ──
    set text(
      font: body-bundle.family,
      size: body-size,
      tracking: body-tracking,
      spacing: body-spacing,
      fill: theme.fg,
    )
    // Paragraph spacing derives from the rhythm baseline (one line), so inter-
    // paragraph gaps stay on the vertical grid instead of Typst's default.
    set par(leading: body-leading, justify: justify, spacing: r.baseline)
    set std.footnote.entry(
      separator: line(length: 30%, stroke: 0.5pt + theme.rule),
      clearance: 1em,
      gap: 0.5em,
      indent: 1em,
    )

    // ── show rules: native Typst elements → our components ──
    show std.heading.where(level: 1): it => (result.heading-1)(it.body)
    show std.heading.where(level: 2): it => (result.heading-2)(it.body)
    show std.heading.where(level: 3): it => (result.heading-3)(it.body)
    show std.heading.where(level: 4): it => (result.heading-4)(it.body)
    show std.strong: it => (result.strong)(it.body)
    show std.emph: it => (result.emphasis)(it.body)
    // `link` styling — break recursion by detecting an already-underlined body.
    // First pass wraps in underline + colored text + re-emits link to preserve
    // click semantics. Second pass sees body is already an underline element
    // and passes through.
    show std.link: it => if it.body.func() == underline {
      it
    } else {
      link(it.dest, underline(
        stroke: 0.5pt + theme.link,
        text(fill: theme.link, it.body),
      ))
    }
    show std.raw.where(block: false): it => (result.code)(it)
    show std.raw.where(block: true): it => (result.code-block)(it)
    show std.list: it => (result.list)(it)
    show std.enum: it => (result.enum)(it)
    show std.quote: it => (result.quote)(attribution: it.attribution, it.body)
    show std.figure.caption: it => (result.figure-caption)(it.body)
    show std.footnote.entry: it => (result.footnote-entry)(it)
    // `figure` itself isn't bound — our figure component re-emits a figure
    // element, which would recurse. Use `#figure-comp(...)` explicitly.

    body
  }
  result.insert("markup", markup-rule)

  result.insert("theme", theme)
  result.insert("scale", resolved-fonts.body.scale)
  result.insert("rhythm", r)

  result
}
