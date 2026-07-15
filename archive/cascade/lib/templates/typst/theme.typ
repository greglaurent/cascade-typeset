// cascade-typst — theme.typ  ({{ banner }})
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
{% for c in light %}  {{ c.padded_name }}rgb("{{ c.hex }}"),
{% endfor %})

#let _dark-palette = (
{% for c in dark %}  {{ c.padded_name }}rgb("{{ c.hex }}"),
{% endfor %})

#let presets = (
  light: make(.._light-palette),
  dark:  make(.._dark-palette),
)

