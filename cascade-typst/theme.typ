// cascade-typst — theme.typ  (GENERATED from tokens.mjs by `just gen` — do not edit by hand)
// Semantic color tokens. presets.light is curated; presets.dark flips HSL lightness
// (derive-dark). Derived tokens (link, code-bg, code-fg, quote-rule, quote-bg) fall
// back to accent / bg-subtle / fg / fg-muted / none in make().

#let _flip-lightness(c) = {
  let parts = color.hsl(c).components()
  color.hsl(parts.at(0), parts.at(1), 100% - parts.at(2))
}

#let derive-dark(palette) = {
  let r = (:)
  for (k, v) in palette {
    r.insert(k, if type(v) == color { _flip-lightness(v) } else { v })
  }
  r
}

#let _or-fallback(v, fb) = if v == auto { fb } else { v }

#let make(
  fg:         none,
  fg-muted:   none,
  fg-subtle:  none,
  bg:         none,
  bg-subtle:  none,
  accent:     none,
  rule:       none,
  link:       auto,
  code-bg:    auto,
  code-fg:    auto,
  quote-rule: auto,
  quote-bg:   auto,
) = (
  fg: fg,
  fg-muted: fg-muted,
  fg-subtle: fg-subtle,
  bg: bg,
  bg-subtle: bg-subtle,
  accent: accent,
  rule: rule,
  link:       _or-fallback(link, accent),
  code-bg:    _or-fallback(code-bg, bg-subtle),
  code-fg:    _or-fallback(code-fg, fg),
  quote-rule: _or-fallback(quote-rule, fg-muted),
  quote-bg:   _or-fallback(quote-bg, none),
)

#let _light-palette = (
  fg:        rgb("#1A1A1A"),
  fg-muted:  rgb("#525250"),
  fg-subtle: rgb("#6C6C69"),
  bg:        rgb("#F8F7F2"),
  bg-subtle: rgb("#F0EEE7"),
  accent:    rgb("#1F3A5F"),
  rule:      rgb("#8E8E8A"),
)

#let presets = (
  light: make(.._light-palette),
  dark:  make(..derive-dark(_light-palette)),
)
