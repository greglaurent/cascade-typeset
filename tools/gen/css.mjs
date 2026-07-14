// cascade-typeset — CSS renderer backend. Emits the token-driven .css modules (scale,
// font, per-font presets, theme, rhythm) from tokens.mjs. Formulas live in the CSS
// calc()/pow()/log() below; only the numbers come from tokens. `files()` is collected
// by ../gen.mjs.
import { scale, optical, fonts, theme, rhythm } from '../tokens.mjs';
import { GEN, cap, steps, cssLabel } from './helpers.mjs';

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

// ── cascade-css/theme.css ─────────────────────────────────────────────────────
function themeCss() {
  const blk = (name, ind) => Object.entries(theme[name]).map(([k, hex]) =>
    `${ind}${('--ct-' + k + ':').padEnd(21)} ${hex};`).join('\n');
  return `/* cascade-css — theme.css  (${GEN})
 * Semantic color tokens. Light and dark are both curated (explicit hex). Dark applies
 * on the system pref or [data-theme="dark"]; [data-theme="light"] forces light. The
 * derived tokens live on .cascade so they recompute from whatever base is active.
 */

/* ── base tokens: light (default) ── */
:root {
${blk('light', '  ')}
}

/* ── base tokens: dark (system preference) ── */
@media (prefers-color-scheme: dark) {
  :root {
${blk('dark', '    ')}
  }
}

/* ── explicit overrides on any element (win over system pref) ── */
[data-theme="light"] {
${blk('light', '  ')}
}
[data-theme="dark"] {
${blk('dark', '  ')}
}

/* ── derived tokens: resolve from whatever base is active on this container ── */
.cascade {
  --ct-link:         var(--ct-accent);
  --ct-link-hover:   var(--ct-accent-hover);
  --ct-link-visited: var(--ct-accent-visited);
  --ct-code-fg:      var(--ct-fg);
  --ct-code-bg:      var(--ct-bg-subtle);
  --ct-quote-rule:   var(--ct-accent-rule);
  --ct-quote-bg:     transparent;
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

export function files() {
  return [
    ['cascade-css/scale.css', scaleCss()],
    ['cascade-css/font.css', fontCss()],
    ...Object.keys(fonts.presets).map(name => [`cascade-css/fonts/${name}.css`, fontPresetCss(name)]),
    ['cascade-css/theme.css', themeCss()],
    ['cascade-css/rhythm.css', rhythmCss()],
  ];
}
