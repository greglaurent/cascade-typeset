// cascade-typst — font.typ  (GENERATED from tokens.mjs by `just gen` — do not edit by hand)
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
  leading-clamp: (1.2, 1.45),   // Butterick: hold line-spacing 120–145%; `none` disables
  tracking-clamp: 0.04,
  base-word-space: 0.28,
  k-word-space: 0.04,
) = {
  let tracking = s => {
    let raw = k-tracking * calc.ln(optical-size / s)
    let val = if tracking-clamp == none { raw } else {
      calc.max(-tracking-clamp, calc.min(tracking-clamp, raw))
    }
    val * 1em
  }
  let leading-ratio = (s, measure: 65) => {
    let m = if measure == none { 0 } else { (measure - 65) * 0.006 }
    let x = -(x-height - 0.50) * 0.8
    let z = -0.10 * calc.ln(s / optical-size)
    let raw = leading-base + m + x + z
    if leading-clamp == none { raw } else {
      calc.max(leading-clamp.at(0), calc.min(leading-clamp.at(1), raw))
    }
  }
  let leading = (s, measure: 65) => {
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
  sans-ui:    make(optical-size: 14pt, x-height: 0.53, k-tracking: 0.035, leading-base: 1.45, base-word-space: 0.25),
  sans-text:  make(optical-size: 12pt, x-height: 0.53, k-tracking: 0.03, leading-base: 1.45, base-word-space: 0.28),
  serif-text: make(optical-size: 11pt, x-height: 0.49, k-tracking: 0.022, leading-base: 1.35, base-word-space: 0.28),
  sf-pro:     make(optical-size: 12pt, x-height: 0.53, k-tracking: 0.045, leading-base: 1.45, base-word-space: 0.25),

  // Font-specific profiles — x-height MEASURED from the font's OS/2 table.
  lora:       make(optical-size: 11pt, x-height: 0.5, k-tracking: 0.022, leading-base: 1.38, base-word-space: 0.28),
  inter:      make(optical-size: 12pt, x-height: 0.546, k-tracking: 0.03, leading-base: 1.45, base-word-space: 0.26),
)

#let bundles = (
  serif: (
    family:  "Libertinus Serif",
    scale:   scale.presets.classical,
    profile: presets.serif-text,
  ),
  sans: (
    family:  "Inter",
    scale:   scale.presets.classical,
    profile: presets.sans-text,
  ),
  mono: (
    family:  "DejaVu Sans Mono",
    scale:   scale.presets.classical,
    profile: presets.sans-text,
  ),

  // Font-specific bundles — real typeface + its measured optical profile.
  lora: (
    family:  "Lora",
    scale:   scale.presets.classical,
    profile: presets.lora,
  ),
  inter: (
    family:  "Inter",
    scale:   scale.presets.classical,
    profile: presets.inter,
  ),
)
