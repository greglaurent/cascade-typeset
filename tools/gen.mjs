// cascade-typeset — generator. Reads tokens.mjs and writes the token-driven parts
// of both renderers, so cascade-css/ and cascade-typst/ can't drift. Run `just gen`.
// Dependency-free (Node ESM). Formulas live in the templates below; only the numbers
// come from tokens.mjs.
import { writeFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { argv } from 'node:process';
import { dirname, join } from 'node:path';
import { scale, optical, fonts, theme, rhythm } from './tokens.mjs';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');   // repo root (tools/ is one level down)
const write = (rel, body) => { writeFileSync(join(root, rel), body); console.log('wrote', rel); };
const GEN = 'GENERATED from tokens.mjs by `just gen` — do not edit by hand';
const cap = s => s[0].toUpperCase() + s.slice(1);

// HSL lightness flip — mirrors Typst's theme.derive-dark, so the CSS dark palette is
// precomputed from the same light source (light hex → flip L → dark hex).
function hexToRgb(h) { const n = parseInt(h.slice(1), 16); return [(n >> 16) & 255, (n >> 8) & 255, n & 255]; }
function rgbToHsl(r, g, b) {
  r /= 255; g /= 255; b /= 255;
  const mx = Math.max(r, g, b), mn = Math.min(r, g, b), l = (mx + mn) / 2;
  let h = 0, s = 0;
  if (mx !== mn) {
    const d = mx - mn;
    s = l > 0.5 ? d / (2 - mx - mn) : d / (mx + mn);
    h = mx === r ? (g - b) / d + (g < b ? 6 : 0) : mx === g ? (b - r) / d + 2 : (r - g) / d + 4;
    h /= 6;
  }
  return [h, s, l];
}
function hslToRgb(h, s, l) {
  if (s === 0) { const v = Math.round(l * 255); return [v, v, v]; }
  const hue = (p, q, t) => { t = (t + 1) % 1; return t < 1 / 6 ? p + (q - p) * 6 * t : t < 1 / 2 ? q : t < 2 / 3 ? p + (q - p) * (2 / 3 - t) * 6 : p; };
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s, p = 2 * l - q;
  return [hue(p, q, h + 1 / 3), hue(p, q, h), hue(p, q, h - 1 / 3)].map(x => Math.round(x * 255));
}
const flipHex = hex => {
  const [h, s, l] = rgbToHsl(...hexToRgb(hex));
  return '#' + hslToRgb(h, s, 1 - l).map(x => x.toString(16).padStart(2, '0').toUpperCase()).join('');
};

// Scale step helpers ----------------------------------------------------------
const steps = [];
for (let i = scale.steps.min; i <= scale.steps.max; i++) steps.push(i);
const cssLabel = i => (i < 0 ? `n${-i}` : i > 0 ? `p${i}` : '0');   // n5 … 0 … p5

// ── cascade-css/scale.css ─────────────────────────────────────────────────────
function scaleCss() {
  const def = scale.presets[scale.default];
  const sizeLines = steps.map(i => {
    const name = `--cs-size-${cssLabel(i)}`;
    if (i === 0) return `  ${name}:  var(--cs-base);`;
    const num = String(i).padStart(2);
    return `  ${name}: calc(var(--cs-base) * pow(var(--cs-ratio), calc(${num} / var(--cs-n))));`;
  }).join('\n');
  const presetLines = Object.entries(scale.presets).map(([k, v]) =>
    `.cascade.scale-${k} { --cs-ratio: ${v.ratio}; --cs-n: ${v.n}; }`).join('\n');

  return `/* cascade-css — scale.css  (${GEN})
 * Fluid typographic scale:  f_i = base · ratio^(i/n). Pick a preset (sets
 * --cs-ratio/--cs-n) and a fluid --cs-base; every step derives via pow(). Because
 * --cs-base is a clamp(), the whole scale tracks the viewport (Utopia-style).
 * Requires pow()/log(): Chrome/Edge 111+, Safari 15.4+, Firefox 118+.
 */

.cascade {
  --cs-base: ${scale.base.web};

  /* Default preset: ${scale.default}. Override with a .scale-* class. */
  --cs-ratio: ${def.ratio};
  --cs-n: ${def.n};
  --cs-ln-ratio: log(var(--cs-ratio));   /* natural log; recomputes with --cs-ratio */

${sizeLines}
}

/* ── Scale presets — add as a class on the .cascade box ── */
${presetLines}
`;
}

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

// ── cascade-css/font.css ──────────────────────────────────────────────────────
function fontCss() {
  const defProfile = optical.profiles[fonts.bundles[fonts.default].profile];
  const defStack = fonts.bundles[fonts.default].stack;

  const trackLine = i => {
    if (i === 0) return `  --cf-track-0:  0em;`;
    const num = String(i).padStart(2);
    return `  --cf-track-${cssLabel(i)}: clamp(calc(-1 * var(--cf-tc) * 1em), calc(-1 * var(--cf-kt) * ${num} / var(--cs-n) * var(--cs-ln-ratio) * 1em), calc(var(--cf-tc) * 1em));`;
  };
  const leadLine = i => {
    if (i === 0) return `  --cf-lead-0:  clamp(var(--cf-lmin), var(--cf-lead0), var(--cf-lmax));`;
    const num = String(i).padStart(2);
    return `  --cf-lead-${cssLabel(i)}: clamp(var(--cf-lmin), calc(var(--cf-lead0) - 0.10 * ${num} / var(--cs-n) * var(--cs-ln-ratio)), var(--cf-lmax));`;
  };
  const hTrackLine = k => `  --cf-h-track-p${k}: clamp(calc(-1 * var(--cf-tc) * 1em), calc(-1 * var(--cf-h-kt) * ${k} / var(--cs-n) * var(--cs-ln-ratio) * 1em), calc(var(--cf-tc) * 1em));`;
  const hLeadLine  = k => `  --cf-h-lead-p${k}: clamp(var(--cf-lmin), calc(var(--cf-h-lead0) - 0.10 * ${k} / var(--cs-n) * var(--cs-ln-ratio)), var(--cf-lmax));`;

  const trackLines = steps.map(trackLine).join('\n');
  const leadLines  = steps.map(leadLine).join('\n');
  const hTrackLines = [1, 2, 3, 4].map(hTrackLine).join('\n');
  const hLeadLines  = [1, 2, 3, 4].map(hLeadLine).join('\n');

  const optVars = p => `--cf-xh: ${p.xHeight}; --cf-kt: ${p.kTracking}; --cf-lb: ${p.leadingBase}; --cf-bws: ${p.wordSpace};`;
  const hOptVars = p => `--cf-h-xh: ${p.xHeight}; --cf-h-kt: ${p.kTracking}; --cf-h-lb: ${p.leadingBase};`;

  const profileClasses = Object.entries(optical.profiles).map(([k, p]) =>
    `.cascade.profile-${k} { ${optVars(p)} }`).join('\n');
  const bundleClasses = Object.entries(fonts.bundles).map(([k, b]) =>
    `.cascade.bundle-${k} { --cf-family-body: var(--cf-font-${b.stack}); ${optVars(optical.profiles[b.profile])} }`).join('\n');
  const headingClasses = Object.entries(fonts.bundles).map(([k, b]) =>
    `.cascade.heading-${k} { --cf-family-heading: var(--cf-font-${b.stack}); ${hOptVars(optical.profiles[b.profile])} }`).join('\n');

  const stackLines = Object.entries(fonts.stacks).map(([k, s]) => `  --cf-font-${k}: ${s.css};`).join('\n');

  return `/* cascade-css — font.css  (${GEN})
 * Font families (per category) + the optical profile: size-dependent tracking,
 * leading, word-space. Families are per category (body / heading / code) so a
 * serif body can pair with sans headings. Optical formulas (cascade's optical-size
 * is reinterpreted as the body base, so ln(size/optical) → (step/n)·ln(ratio)):
 *   tracking_em(i) = clamp(±tc,  -kt · (i/n) · ln(ratio))
 *   line-height(i) = clamp(lmin, lb + (measure-65)·0.006 − (xh−0.5)·0.8 − 0.10·(i/n)·ln(ratio),  lmax)
 *   word_space(i)  = (bws − kws · (i/n) · ln(ratio)) em
 */

.cascade {
  /* ── Font stacks ── */
${stackLines}

  /* ── Active family per category ── */
  --cf-family-body:    var(--cf-font-${defStack});
  --cf-family-heading: var(--cf-family-body);   /* default: same as body */
  --cf-family-code:    var(--cf-font-mono);

  /* ── Body optical params (default: ${fonts.default}) ── */
  --cf-xh: ${defProfile.xHeight};
  --cf-kt: ${defProfile.kTracking};
  --cf-lb: ${defProfile.leadingBase};
  --cf-bws: ${defProfile.wordSpace};
  --cf-kws: ${optical.wordSpaceK};
  --cf-tc: ${optical.trackingClamp};
  --cf-lmin: ${optical.leadingClamp.min};
  --cf-lmax: ${optical.leadingClamp.max};
  --cf-measure: ${optical.measure};
  --cf-size-min: ${optical.sizeMin.web};   /* readability floor for small roles */
  --cf-lead0: calc(var(--cf-lb) + (var(--cf-measure) - 65) * 0.006 - (var(--cf-xh) - 0.5) * 0.8);

  /* ── Heading optical params (default: = body) ── */
  --cf-h-xh: var(--cf-xh);
  --cf-h-kt: var(--cf-kt);
  --cf-h-lb: var(--cf-lb);
  --cf-h-lead0: calc(var(--cf-h-lb) + (var(--cf-measure) - 65) * 0.006 - (var(--cf-h-xh) - 0.5) * 0.8);

  /* ── body tracking per step (em) ── */
${trackLines}

  /* ── body line-height per step (unitless, clamped) ── */
${leadLines}

  /* ── heading tracking + line-height (p1..p4 = h4..h1) ── */
${hTrackLines}
${hLeadLines}

  /* ── word-space (body, step 0) ── */
  --cf-ws-0: calc(var(--cf-bws) * 1em);
}

/* ── Body optical profiles — class on the .cascade box ── */
${profileClasses}

/* ── Body bundles (family + optical) ── */
${bundleClasses}

/* ── Heading bundles (heading family + heading optical) ── */
${headingClasses}
`;
}

// ── cascade-css/fonts/<name>.css ──────────────────────────────────────────────
function fontPresetCss(name) {
  const f = fonts.presets[name], p = f.profile, m = f.measured;
  let out = `/* cascade-css — fonts/${name}.css  (${GEN})
 * Font-specific preset for ${cap(name)}: sets the family AND the optical variables
 * that depend on this typeface (mirror of cascade's font.bundles.${name}).
 * Measured (OS/2, unitsPerEm ${m.unitsPerEm}): x-height ${m.xHeight} (${m.sx}), cap-height ${m.capHeight},
 * typo asc/desc ${m.asc} / ${m.desc}. x-height is MEASURED; the rest starts from the
 * ${f.category}-text profile. Load AFTER cascade.css.
 */

.cascade.bundle-${name} {
  --cf-family-body: ${f.family.css};
  --cf-xh:  ${p.xHeight};
  --cf-kt:  ${p.kTracking};
  --cf-lb:  ${p.leadingBase};
  --cf-bws: ${p.wordSpace};
}

.cascade.heading-${name} {
  --cf-family-heading: ${f.family.css};
  --cf-h-xh: ${p.xHeight};
  --cf-h-kt: ${p.kTracking};
  --cf-h-lb: ${p.leadingBase};
}
`;
  if (f.normalize) {
    const nz = f.normalize;
    out += `
/* Optional cross-font size normalization: an alias scaled so ${cap(name)}'s x-height
 * (${p.xHeight}) matches ${nz.toXHeight}.  size-adjust ≈ ${nz.toXHeight} / ${p.xHeight} = ${nz.sizeAdjust}. */
@font-face {
  font-family: "${nz.alias}";
  src: local("${f.family.typst}"), local("${f.family.typst}Variable");
  size-adjust: ${nz.sizeAdjust};
  ascent-override: ${nz.ascent};
  descent-override: ${nz.descent};
}
/* Use via a preset whose --cf-xh is the normalized x-height (${nz.toXHeight}):
 *   .cascade.bundle-${name}-norm { --cf-family-body: "${nz.alias}", sans-serif;
 *     --cf-xh: ${nz.toXHeight}; --cf-kt: ${p.kTracking}; --cf-lb: ${p.leadingBase}; --cf-bws: ${p.wordSpace}; } */
`;
  }
  return out;
}

// ── cascade-typst/theme.typ ───────────────────────────────────────────────────
function themeTyp() {
  const paletteLines = Object.entries(theme.light).map(([k, hex]) =>
    `  ${(k + ':').padEnd(11)}rgb("${hex}"),`).join('\n');
  return `// cascade-typst — theme.typ  (${GEN})
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
${paletteLines}
)

#let presets = (
  light: make(.._light-palette),
  dark:  make(..derive-dark(_light-palette)),
)
`;
}

// ── cascade-css/theme.css ─────────────────────────────────────────────────────
function themeCss() {
  const dark = Object.fromEntries(Object.entries(theme.light).map(([k, hex]) => [k, flipHex(hex)]));
  const blk = (pal, ind) => Object.entries(pal).map(([k, hex]) =>
    `${ind}${('--ct-' + k + ':').padEnd(15)} ${hex};`).join('\n');
  return `/* cascade-css — theme.css  (${GEN})
 * Semantic color tokens. Light is the source; dark is cascade's derive-dark (HSL
 * lightness flipped), precomputed to hex here. Dark applies on the system pref or
 * [data-theme="dark"]; [data-theme="light"] forces light. Derived tokens live on
 * .cascade so they recompute from whatever base is active for that container.
 */

/* ── base tokens: light (default) ── */
:root {
${blk(theme.light, '  ')}
}

/* ── base tokens: dark (system preference) ── */
@media (prefers-color-scheme: dark) {
  :root {
${blk(dark, '    ')}
  }
}

/* ── explicit overrides on any element (win over system pref) ── */
[data-theme="light"] {
${blk(theme.light, '  ')}
}
[data-theme="dark"] {
${blk(dark, '  ')}
}

/* ── derived tokens: resolve from whatever base is active on this container ── */
.cascade {
  --ct-link:       var(--ct-accent);
  --ct-code-fg:    var(--ct-fg);
  --ct-code-bg:    var(--ct-bg-subtle);
  --ct-quote-rule: var(--ct-fg-muted);
  --ct-quote-bg:   transparent;
}
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

// ── cascade-css/rhythm.css ────────────────────────────────────────────────────
function rhythmCss() {
  const spaceLines = Object.entries(rhythm.multipliers).map(([k, v]) => {
    const name = `--cr-space-${k === 'base' ? '0' : k}`;
    return `  ${(name + ':').padEnd(14)} calc(var(--cr-unit) * ${v});`;
  }).join('\n');
  return `/* cascade-css — rhythm.css  (${GEN})
 * Vertical rhythm: spacing scale derived from the fluid base (unit = base · factor),
 * so all spacing scales with the type. Multipliers match cascade/rhythm.typ; baseline
 * = one body line.
 */

.cascade {
  --cr-unit: calc(var(--cs-base) * ${rhythm.unit.web});

${spaceLines}

  /* One body line — paragraph separation (matches the layout.typ rhythm fix). */
  --cr-baseline: calc(var(--cs-base) * var(--cf-lead-0));
}
`;
}

// ── file map — the single source for both the CLI and verify.mjs ──────────────
export function files() {
  return [
    ['cascade-css/scale.css', scaleCss()],
    ['cascade-typst/scale.typ', scaleTyp()],
    ['cascade-typst/font.typ', fontTyp()],
    ['cascade-css/font.css', fontCss()],
    ...Object.keys(fonts.presets).map(name => [`cascade-css/fonts/${name}.css`, fontPresetCss(name)]),
    ['cascade-typst/theme.typ', themeTyp()],
    ['cascade-css/theme.css', themeCss()],
    ['cascade-typst/rhythm.typ', rhythmTyp()],
    ['cascade-css/rhythm.css', rhythmCss()],
  ];
}

// CLI: `node gen.mjs` writes them. (Importing the module — e.g. from verify.mjs —
// does not, so the verifier can regenerate in-memory and diff against disk.)
if (import.meta.url === pathToFileURL(argv[1]).href) {
  for (const [rel, body] of files()) write(rel, body);
}
