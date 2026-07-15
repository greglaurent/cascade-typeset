// cascade-typst — theme.typ  (GENERATED from tokens.mjs by `just gen` — do not edit by hand)
// Semantic color tokens. presets.light and presets.dark are both curated (explicit
// hex). The derived tokens (link, code-bg, code-fg, quote-rule, quote-bg) fall back to
// accent / bg-subtle / fg / fg-muted / none in make().

#let _or-fallback(v, fb) = if v == auto { fb } else { v }

#let make(
  fg:            none,
  fg-muted:      none,
  fg-subtle:     none,
  bg:            none,
  bg-subtle:     none,
  rule:          none,
  accent:         none,
  accent-hover:   none,
  accent-subtle:  none,
  accent-rule:    none,
  accent-visited: none,
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
  rule: rule,
  accent: accent,
  accent-hover: accent-hover,
  accent-subtle: accent-subtle,
  accent-rule: accent-rule,
  accent-visited: accent-visited,
  link:       _or-fallback(link, accent),
  code-bg:    _or-fallback(code-bg, bg-subtle),
  code-fg:    _or-fallback(code-fg, fg),
  quote-rule: _or-fallback(quote-rule, accent-rule),
  quote-bg:   _or-fallback(quote-bg, none),
)

#let _light-palette = (
  fg:             rgb("#171717"),
  fg-muted:       rgb("#59544C"),
  fg-subtle:      rgb("#7A746A"),
  bg:             rgb("#F6F2E9"),
  bg-subtle:      rgb("#EFE9DC"),
  rule:           rgb("#C4BDB0"),
  accent:         rgb("#7A2E28"),
  accent-hover:   rgb("#5E211C"),
  accent-subtle:  rgb("#F0E2DE"),
  accent-rule:    rgb("#C9A5A0"),
  accent-visited: rgb("#5A3A52"),
)

#let _dark-palette = (
  fg:             rgb("#E8E4DC"),
  fg-muted:       rgb("#A8A196"),
  fg-subtle:      rgb("#8A8378"),
  bg:             rgb("#14120E"),
  bg-subtle:      rgb("#1E1B16"),
  rule:           rgb("#3A362F"),
  accent:         rgb("#E09A93"),
  accent-hover:   rgb("#EFB8B1"),
  accent-subtle:  rgb("#2E1614"),
  accent-rule:    rgb("#4A2A26"),
  accent-visited: rgb("#C9A5C4"),
)

#let presets = (
  light: make(.._light-palette),
  dark:  make(.._dark-palette),
)
