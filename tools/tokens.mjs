// @ts-check
// cascade-typeset — design tokens: the single source of truth.
//
// These are the NUMBERS of the type system. The FORMULAS that consume them stay in
// each renderer (CSS pow()/log() + clamp chains; Typst make()). Run `just gen` to
// regenerate the token-driven parts of cascade-css/ and cascade-typst/ from this
// file, so the two renderers are identical by construction and can't drift.
//
// Two kinds of value appear here:
//   • shared        — one number both renderers use (e.g. an x-height, a ratio).
//   • { print, web }/{ typst, css } — genuinely target-specific representations
//                     (a fixed pt vs a fluid clamp; a single family vs a stack).
// Everything else — the model coefficients (Dyson 0.006, x-height 0.8, display
// 0.10, the 65ch / 0.50 baselines) — lives in the renderer formulas, not here.
//
// Status: scale + optical + fonts + theme + rhythm wired end-to-end.

/**
 * @typedef {{ ratio: number, n: number }} ScalePreset
 * @typedef {{ base: { print: string, web: string }, steps: { min: number, max: number },
 *             default: string, presets: Record<string, ScalePreset> }} Scale
 *
 * @typedef {{ opticalSize: string, xHeight: number, kTracking: number,
 *             leadingBase: number, wordSpace: number }} Profile
 * @typedef {{ wordSpaceK: number, trackingClamp: number,
 *             leadingClamp: { min: number, max: number }, measure: number,
 *             sizeMin: { print: string, web: string },
 *             profiles: Record<string, Profile> }} Optical
 *
 * @typedef {{ typst: string, css: string }} Stack
 * @typedef {{ stack: string, profile: string }} Bundle
 * @typedef {{ xHeight: number, capHeight: number, unitsPerEm: number,
 *             sx: string, asc: string, desc: string }} Measured
 * @typedef {{ alias: string, toXHeight: number, sizeAdjust: string,
 *             ascent: string, descent: string }} Normalize
 * @typedef {{ family: Stack, category: string, profile: Profile,
 *             measured: Measured, normalize?: Normalize }} FontPreset
 * @typedef {{ stacks: Record<string, Stack>, default: string,
 *             bundles: Record<string, Bundle>, presets: Record<string, FontPreset> }} Fonts
 *
 * @typedef {{ light: Record<string, string>, dark: Record<string, string> }} Theme
 * @typedef {{ unit: { print: string, web: number },
 *             multipliers: Record<string, number> }} Rhythm
 */

/** @type {Scale} */
export const scale = {
  // Fundamental body size. Print takes a fixed point size; web takes a fluid clamp
  // (Utopia-style) so the whole scale tracks the viewport.
  base: {
    print: '11pt',
    web:   'clamp(1.125rem, 1.02rem + 0.42vw, 1.375rem)',
  },
  // The scale spans i ∈ [min, max], each size = base · ratio^(i/n).
  steps: { min: -5, max: 5 },
  // Default preset — applied on the bare .cascade box and as scale.typ's make() defaults.
  default: 'classical',
  // Mortensen presets: ratio (r) and notes-per-interval (n).
  presets: {
    'classical':      { ratio: 2,                  n: 5 },
    'golden-ratio':   { ratio: 1.6180339887498949, n: 1 },
    'golden-ditonic': { ratio: 1.6180339887498949, n: 2 },
    'tritonic':       { ratio: 2,                  n: 3 },
    'tetratonic':     { ratio: 2,                  n: 4 },
    'major-third':    { ratio: 1.25,               n: 1 },
    'minor-third':    { ratio: 1.2,                n: 1 },
  },
};

/** @type {Optical} */
export const optical = {
  // Global model knobs shared by every profile.
  wordSpaceK:    0.04,                     // k-word-space
  trackingClamp: 0.04,                     // max |tracking|, em
  leadingClamp:  { min: 1.2, max: 1.45 },  // Butterick: hold line-spacing 120–145%
  measure:       65,                       // default line length, ch
  sizeMin:       { print: '8pt', web: '0.75rem' },   // readability floor for small roles

  // Optical profiles. opticalSize is Typst-only (CSS reinterprets it as the body
  // base, so ln(size/optical) collapses to (step/n)·ln(ratio)).
  profiles: {
    'sans-ui':    { opticalSize: '14pt', xHeight: 0.53, kTracking: 0.035, leadingBase: 1.45, wordSpace: 0.25 },
    'sans-text':  { opticalSize: '12pt', xHeight: 0.53, kTracking: 0.030, leadingBase: 1.45, wordSpace: 0.28 },
    'serif-text': { opticalSize: '11pt', xHeight: 0.49, kTracking: 0.022, leadingBase: 1.35, wordSpace: 0.28 },
    'sf-pro':     { opticalSize: '12pt', xHeight: 0.53, kTracking: 0.045, leadingBase: 1.45, wordSpace: 0.25 },
  },
};

/** @type {Fonts} */
export const fonts = {
  // Font stacks. Typst takes a single family; CSS wants a fallback stack.
  stacks: {
    serif: { typst: 'Libertinus Serif', css: '"Libertinus Serif", Georgia, "Times New Roman", serif' },
    sans:  { typst: 'Inter',            css: '"Inter", system-ui, -apple-system, sans-serif' },
    mono:  { typst: 'DejaVu Sans Mono', css: '"DejaVu Sans Mono", ui-monospace, SFMono-Regular, monospace' },
  },
  // Default body bundle (sets the .cascade defaults + scale.typ make() fallbacks).
  default: 'serif',
  // Generic bundles: which stack + which optical profile. Scale = the default scale.
  bundles: {
    serif: { stack: 'serif', profile: 'serif-text' },
    sans:  { stack: 'sans',  profile: 'sans-text' },
    mono:  { stack: 'mono',  profile: 'sans-text' },
  },
  // Font-specific presets: a real typeface + its MEASURED optical profile. x-height
  // is measured from the font's OS/2 table; the rest starts from the matching
  // category profile with per-font judgment. Add one per typeface you use.
  presets: {
    lora: {
      family:   { typst: 'Lora', css: '"Lora", Georgia, "Times New Roman", serif' },
      category: 'serif',   // which generic profile it derives from (for the comment)
      profile:  { opticalSize: '11pt', xHeight: 0.50, kTracking: 0.022, leadingBase: 1.38, wordSpace: 0.28 },
      measured: { xHeight: 0.50, capHeight: 0.70, unitsPerEm: 1000, sx: 'sxHeight 500', asc: '1.006', desc: '-0.274' },
    },
    inter: {
      family:   { typst: 'Inter', css: '"Inter", system-ui, -apple-system, sans-serif' },
      category: 'sans',
      profile:  { opticalSize: '12pt', xHeight: 0.546, kTracking: 0.030, leadingBase: 1.45, wordSpace: 0.26 },
      measured: { xHeight: 0.546, capHeight: 0.728, unitsPerEm: 2048, sx: 'sxHeight 1118', asc: '0.969', desc: '-0.241' },
      // Optional cross-font size normalization: an @font-face alias scaled so Inter's
      // x-height matches a lower one (e.g. Lora 0.50). Omit to skip.
      normalize: { alias: 'Inter xh-normalized', toXHeight: 0.50, sizeAdjust: '91.6%', ascent: '90%', descent: '22%' },
    },
    jost: {
      family:   { typst: 'Jost', css: '"Jost", "Century Gothic", system-ui, sans-serif' },
      category: 'sans',   // geometric sans; note the low measured x-height (0.46)
      // x-height MEASURED (OS/2); rest derived from the sans-text profile — tune to taste.
      profile:  { opticalSize: '12pt', xHeight: 0.46, kTracking: 0.030, leadingBase: 1.45, wordSpace: 0.28 },
      measured: { xHeight: 0.46, capHeight: 0.70, unitsPerEm: 1000, sx: 'sxHeight 460', asc: '1.07', desc: '-0.375' },
    },
  },
};

/** @type {Theme} */
export const theme = {
  // Curated light + dark palettes (10 base tokens each). Both are explicit hex — dark
  // is hand-tuned (warm-neutral, not a mechanical HSL flip of light). The semantic
  // derived tokens (link, link-hover, code-*, quote-*) are formula, kept in the
  // generator templates. Add or edit a color here, then `just gen`.
  light: {
    'fg':            '#171717',
    'fg-muted':      '#59544C',
    'fg-subtle':     '#7A746A',
    'bg':            '#F6F2E9',
    'bg-subtle':     '#EFE9DC',
    'rule':          '#C4BDB0',
    'accent':         '#7A2E28',   // oxblood
    'accent-hover':   '#5E211C',
    'accent-subtle':  '#F0E2DE',
    'accent-rule':    '#C9A5A0',
    'accent-visited': '#5A3A52',   // muted plum — visited convention, warmed (CSS only)
  },
  dark: {
    'fg':            '#E8E4DC',
    'fg-muted':      '#A8A196',
    'fg-subtle':     '#8A8378',
    'bg':            '#14120E',
    'bg-subtle':     '#1E1B16',
    'rule':          '#3A362F',
    'accent':         '#E09A93',   // lifted for contrast on the dark ground
    'accent-hover':   '#EFB8B1',
    'accent-subtle':  '#2E1614',
    'accent-rule':    '#4A2A26',
    'accent-visited': '#C9A5C4',   // muted mauve
  },
};

/** @type {Rhythm} */
export const rhythm = {
  // Grid unit: print is a fixed grid; web is a fraction of the fluid base, so spacing
  // scales with the type (4pt at 11pt ≈ base·0.36, rounded to 0.375).
  unit: { print: '4pt', web: 0.375 },
  // Spacing multipliers (× unit). Practical steps, not geometric — Apple/Material/Carbon.
  multipliers: { n1: 0.5, base: 1, p1: 2, p2: 3, p3: 4, p4: 6, p5: 8, p6: 12 },
};
