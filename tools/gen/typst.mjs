// cascade-typeset — Typst renderer backend. Emits the token-driven .typ modules
// (scale, font, theme, rhythm) from tokens.mjs. Formulas live in the make() templates
// below; only the numbers come from tokens. `files()` is collected by ../gen.mjs.
import { scale, optical, fonts, theme, rhythm } from '../tokens.mjs';
import { GEN, steps, cssLabel } from './helpers.mjs';

// ── cascade-typst/scale.typ ───────────────────────────────────────────────────
function scaleTyp() {
  const def = scale.presets[scale.default];
  const dictLines = steps.map(i => `    ${i === 0 ? 'base' : cssLabel(i)}: size(${i}),`).join('\n');
  const names = Object.keys(scale.presets);
  const w = Math.max(...names.map(n => n.length)) + 1;
  const presetLines = names.map(k => {
    const v = scale.presets[k];
    return `  ${(k + ':').padEnd(w)} make(ratio: ${v.ratio}, n: ${v.n}),`;
  }).join('\n');

  return `// cascade-typst — scale.typ  (${GEN})
// Typographic scale (Spencer Mortensen):  f_i = f_0 · r^(i / n)
//   f_0 = base size, r = ratio, n = notes per interval.  Pure math, font-agnostic.

#let make(base: ${scale.base.print}, ratio: ${def.ratio}, n: ${def.n}) = {
  let size = i => base * calc.pow(ratio, i / n)
  (
${dictLines}
    size: size,
    params: (base: base, ratio: ratio, n: n),
  )
}

#let presets = (
${presetLines}
)
`;
}

// ── cascade-typst/font.typ ────────────────────────────────────────────────────
function fontTyp() {
  const lc = optical.leadingClamp, tc = optical.trackingClamp, kws = optical.wordSpaceK;
  const mk = p => `make(optical-size: ${p.opticalSize}, x-height: ${p.xHeight}, k-tracking: ${p.kTracking}, leading-base: ${p.leadingBase}, base-word-space: ${p.wordSpace})`;
  const W = 12;
  const profileLines = Object.entries(optical.profiles).map(([k, p]) => `  ${(k + ':').padEnd(W)}${mk(p)},`).join('\n');
  const presetLines = Object.entries(fonts.presets).map(([k, f]) => `  ${(k + ':').padEnd(W)}${mk(f.profile)},`).join('\n');

  const scaleRef = `scale.presets.${scale.default}`;
  const bundle = (name, familyTypst, profileKey) =>
`  ${name}: (
    family:  "${familyTypst}",
    scale:   ${scaleRef},
    profile: presets.${profileKey},
  ),`;
  const genericBundles = Object.entries(fonts.bundles).map(([k, b]) => bundle(k, fonts.stacks[b.stack].typst, b.profile)).join('\n');
  const fontBundles = Object.entries(fonts.presets).map(([k, f]) => bundle(k, f.family.typst, k)).join('\n');

  return `// cascade-typst — font.typ  (${GEN})
// Per-typeface optical profile: tracking, leading, word-space as size-dependent
// functions. Compose with scale.typ (scale gives sizes, font gives spacing).
//   tracking_em    = k-tracking · ln(optical-size / size), clamped to ±tracking-clamp   (Ahrens)
//   leading_ratio  = leading-base + (measure-65)·0.006 − (x-height-0.50)·0.8
//                                 − 0.10·ln(size/optical-size), clamped to leading-clamp  (Bringhurst/Dyson)
//   word_space_em  = base-word-space + k-word-space · ln(optical-size / size)             (Bringhurst/Tracy)

#import "scale.typ"

#let make(
  optical-size: 12pt,
  x-height: 0.50,
  k-tracking: 0.030,
  leading-base: 1.45,
  leading-clamp: (${lc.min}, ${lc.max}),   // Butterick: hold line-spacing 120–145%; \`none\` disables
  tracking-clamp: ${tc},
  base-word-space: 0.28,
  k-word-space: ${kws},
) = {
  let tracking = s => {
    let raw = k-tracking * calc.ln(optical-size / s)
    let val = if tracking-clamp == none { raw } else {
      calc.max(-tracking-clamp, calc.min(tracking-clamp, raw))
    }
    val * 1em
  }
  let leading-ratio = (s, measure: ${optical.measure}) => {
    let m = if measure == none { 0 } else { (measure - 65) * 0.006 }
    let x = -(x-height - 0.50) * 0.8
    let z = -0.10 * calc.ln(s / optical-size)
    let raw = leading-base + m + x + z
    if leading-clamp == none { raw } else {
      calc.max(leading-clamp.at(0), calc.min(leading-clamp.at(1), raw))
    }
  }
  let leading = (s, measure: ${optical.measure}) => {
    (leading-ratio(s, measure: measure) - 1.0) * s
  }
  let word-space = s => {
    (base-word-space + k-word-space * calc.ln(optical-size / s)) * 1em
  }
  (
    tracking: tracking,
    leading: leading,
    leading-ratio: leading-ratio,
    word-space: word-space,
    params: (
      optical-size: optical-size,
      x-height: x-height,
      k-tracking: k-tracking,
      leading-base: leading-base,
      leading-clamp: leading-clamp,
      tracking-clamp: tracking-clamp,
      base-word-space: base-word-space,
      k-word-space: k-word-space,
    ),
  )
}

#let presets = (
${profileLines}

  // Font-specific profiles — x-height MEASURED from the font's OS/2 table.
${presetLines}
)

#let bundles = (
${genericBundles}

  // Font-specific bundles — real typeface + its measured optical profile.
${fontBundles}
)
`;
}

// ── cascade-typst/theme.typ ───────────────────────────────────────────────────
function themeTyp() {
  const pal = name => Object.entries(theme[name]).map(([k, hex]) =>
    `  ${(k + ':').padEnd(16)}rgb("${hex}"),`).join('\n');
  return `// cascade-typst — theme.typ  (${GEN})
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
${pal('light')}
)

#let _dark-palette = (
${pal('dark')}
)

#let presets = (
  light: make(.._light-palette),
  dark:  make(.._dark-palette),
)
`;
}

// ── cascade-typst/rhythm.typ ──────────────────────────────────────────────────
function rhythmTyp() {
  const spacingLines = Object.entries(rhythm.multipliers).map(([k, v]) =>
    `    ${(k + ':').padEnd(6)}grid-unit * ${v},`).join('\n');
  return `// cascade-typst — rhythm.typ  (${GEN})
// Vertical rhythm from scale + font + measure. baseline = ceil(body-size ·
// leading-ratio / grid-unit) · grid-unit; spacing tokens are multiples of the unit.
// snap() is opt-in (Tim Brown), not an enforced lattice.

#import "scale.typ"
#import "font.typ"

#let make(
  scale: scale.presets.${scale.default},
  font: font.presets.sans-text,
  measure: ${optical.measure},
  body-step: 0,
  grid-unit: ${rhythm.unit.print},
) = {
  let body-size = (scale.size)(body-step)
  let body-leading-ratio = (font.leading-ratio)(body-size, measure: measure)
  let raw-baseline = body-size * body-leading-ratio
  let n = calc.ceil(raw-baseline / grid-unit)
  let baseline = n * grid-unit

  let snap = (value, multiple: grid-unit) => {
    calc.round(value / multiple) * multiple
  }

  let spacing = (
${spacingLines}
  )

  (
    unit: grid-unit,
    baseline: baseline,
    spacing: spacing,
    snap: snap,
    params: (
      grid-unit: grid-unit,
      body-size: body-size,
      body-leading-ratio: body-leading-ratio,
      body-baseline: baseline,
      atomic-divisor: n,
    ),
  )
}
`;
}

export function files() {
  return [
    ['cascade-typst/scale.typ', scaleTyp()],
    ['cascade-typst/font.typ', fontTyp()],
    ['cascade-typst/theme.typ', themeTyp()],
    ['cascade-typst/rhythm.typ', rhythmTyp()],
  ];
}
