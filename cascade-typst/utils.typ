// Layout mechanics — shared helpers, render functions, and the component factory.
//
// Imported by layout.typ (and any future style modules that need the same machinery).
// All names here are intended to be called from other style files; they have no
// leading underscore.

// Meta-keys trigger recomputation of size + tracking + word-space + leading
// from scale/font/measure. Other args are passed through as plain style tweaks.
// `step` lets a component sit at a different scale step coherently.
// `size` overrides the size directly; tracking/word-space/leading are recomputed
// from the given size against the active font profile.
// Pass either per-call (`text-3(size: 14pt)[...]`), via `overrides:` at make
// time, or at the `fonts.<category>.` layer (applies to every component in
// that category). `scale`, `font-profile`, and `measure` remain configured
// once at `layout.make()`.
#let _meta-keys = ("step", "size")

#let filter-auto(d) = {
  let r = (:)
  for (k, v) in d {
    if v != auto { r.insert(k, v) }
  }
  r
}

#let split-meta(d) = {
  let meta = (:)
  let real = (:)
  for (k, v) in d {
    if _meta-keys.contains(k) { meta.insert(k, v) }
    else { real.insert(k, v) }
  }
  (meta: meta, real: real)
}

#let compute(step, scale, font, measure, size-min: 0pt) = {
  // Readability floor: never smaller than size-min (a rendering concern, not the
  // scale — scale.size stays pure). Optical is recomputed from the floored size.
  let size = calc.max((scale.size)(step), size-min)
  (
    size: size,
    tracking: (font.tracking)(size),
    spacing: (font.word-space)(size),
    leading: (font.leading)(size, measure: measure),
  )
}

// ─── Render functions ──────────────────────────────────────────────────────────
// Each render takes the fully-resolved param dict and the body, extracts its own
// decoration-specific keys, and delegates remaining text/par keys to render-text.

// Par-level keys we recognize in the final dict. Extracted before passing to text().
#let _par-keys = ("leading", "justify", "first-line-indent", "hanging-indent", "linebreaks")
// Block-spacing keys: extracted and applied via a block() wrapper.
#let _block-spacing-keys = ("above", "below")

#let render-text(final, body) = {
  let f = final
  let par-args = (:)
  for k in _par-keys {
    if k in f { par-args.insert(k, f.remove(k)) }
  }
  let spacing-args = (:)
  for k in _block-spacing-keys {
    if k in f { spacing-args.insert(k, f.remove(k)) }
  }
  let use-smallcaps = if "smallcaps" in f { f.remove("smallcaps") } else { false }
  let b = if use-smallcaps { smallcaps(body) } else { body }
  let inner = if par-args.len() > 0 {
    [
      #set par(..par-args)
      #text(..f, b)
    ]
  } else {
    text(..f, b)
  }
  if spacing-args.len() > 0 { block(..spacing-args, inner) } else { inner }
}

#let render-link(final, body) = {
  let f = final
  let dest = if "dest" in f { f.remove("dest") } else { none }
  let stroke = if "underline-stroke" in f { f.remove("underline-stroke") } else { auto }
  let offset = if "underline-offset" in f { f.remove("underline-offset") } else { auto }
  let inner = render-text(f, body)
  let decorated = underline(stroke: stroke, offset: offset, inner)
  if dest != none { link(dest, decorated) } else { decorated }
}

// Extract block-* prefixed keys and return (extracted-args, remaining-dict).
// Recognized: block-fill, block-stroke, block-inset, block-radius, block-width,
// block-height, block-breakable, block-outset, block-clip, block-above, block-below.
// Mutates a local copy and returns it — caller receives the modified dict.
#let extract-block-args(f) = {
  let args = (:)
  let keys = (
    "fill", "stroke", "inset", "radius", "width", "height",
    "breakable", "outset", "clip", "above", "below",
  )
  for k in keys {
    let our-key = "block-" + k
    if our-key in f { args.insert(k, f.remove(our-key)) }
  }
  (args, f)
}

#let render-code-block(final, body) = {
  let (block-args, f) = extract-block-args(final)
  let inner = render-text(f, body)
  block(breakable: true, width: 100%, ..block-args, inner)
}

#let render-quote(final, body) = {
  let (block-args, f) = extract-block-args(final)
  let attribution = if "attribution" in f { f.remove("attribution") } else { none }
  // Typst's quote element passes `quotes:` through; drop it so it doesn't reach text().
  if "quotes" in f { let _ = f.remove("quotes") }
  if not "inset" in block-args { block-args.insert("inset", (left: 2em)) }
  let composed = if attribution != none {
    [#body \ #h(1fr) — #attribution]
  } else {
    body
  }
  let inner = render-text(f, composed)
  block(..block-args, inner)
}

#let render-figure-caption(final, body) = {
  let f = final
  let alignment = if "align" in f { f.remove("align") } else { center }
  let inner = render-text(f, body)
  align(alignment, inner)
}

#let render-list(final, body) = {
  let f = final
  let list-args = (:)
  for k in ("tight", "marker", "indent", "body-indent") {
    if k in f { list-args.insert(k, f.remove(k)) }
  }
  if "list-spacing" in f { list-args.insert("spacing", f.remove("list-spacing")) }
  let par-args = (:)
  if "leading" in f { par-args.insert("leading", f.remove("leading")) }
  [
    #set list(..list-args)
    #set par(..par-args)
    #text(..f, body)
  ]
}

#let render-enum(final, body) = {
  let f = final
  let enum-args = (:)
  for k in ("tight", "numbering", "start", "full", "indent", "body-indent") {
    if k in f { enum-args.insert(k, f.remove(k)) }
  }
  if "list-spacing" in f { enum-args.insert("spacing", f.remove("list-spacing")) }
  let par-args = (:)
  if "leading" in f { par-args.insert("leading", f.remove("leading")) }
  [
    #set enum(..enum-args)
    #set par(..par-args)
    #text(..f, body)
  ]
}

#let render-figure(final, body) = {
  let f = final
  let caption = if "caption" in f { f.remove("caption") } else { none }
  let fig-args = (:)
  for k in ("kind", "supplement", "numbering", "placement", "gap", "outlined") {
    if k in f { fig-args.insert(k, f.remove(k)) }
  }
  if caption != none { fig-args.insert("caption", figure.caption(caption)) }
  let styled-body = render-text(f, body)
  figure(styled-body, ..fig-args)
}

#let render-divider(final, body) = {
  let f = final
  let spacing-args = (:)
  for k in _block-spacing-keys {
    if k in f { spacing-args.insert(k, f.remove(k)) }
  }
  let inner = line(length: f.at("length", default: 100%), stroke: f.at("stroke", default: 0.5pt))
  if spacing-args.len() > 0 { block(..spacing-args, inner) } else { inner }
}

// ─── Component factory ─────────────────────────────────────────────────────────
// Builds a callable that resolves overrides at three layers (built-in defaults,
// per-component overrides, per-call args), handles meta-key recomputation, and
// dispatches to the spec's render function.

#let _from-size(s, spec-step, font, measure) = {
  let r = (
    size: s,
    tracking: (font.tracking)(s),
    spacing: (font.word-space)(s),
  )
  // Leading is paragraph-level — only emit it for block-y components (those with a step).
  // Inline components (step: none) wrapped in `set par(leading: …)` would force a line break.
  if spec-step != none {
    r.insert("leading", (font.leading)(s, measure: measure))
  }
  r
}

#let _compute-meta(meta, spec-step, state) = {
  if "size" in meta {
    _from-size(meta.size, spec-step, state.font, state.measure)
  } else {
    let step = meta.at("step", default: spec-step)
    if step == none { (:) } else {
      compute(step, state.scale, state.font, state.measure, size-min: state.at("size-min", default: 0pt))
    }
  }
}

#let make-component(state, spec, comp-overrides) = {
  let split-ov = split-meta(filter-auto(comp-overrides))
  let computed = _compute-meta(split-ov.meta, spec.step, state)
  let base-defaults = spec.defaults + split-ov.real
  let comp-base = computed + base-defaults

  (..args) => {
    let body = args.pos().at(0, default: [])
    let split-call = split-meta(filter-auto(args.named()))
    let recomputed = if split-call.meta.len() > 0 {
      _compute-meta(split-call.meta, spec.step, state)
    } else {
      (:)
    }
    let final = comp-base + recomputed + split-call.real
    (spec.render)(final, body)
  }
}
