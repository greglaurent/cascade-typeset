// cascade-typst — font.typ  ({{ banner }})
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
  leading-clamp: ({{ leading_clamp_min }}, {{ leading_clamp_max }}),   // Butterick: hold line-spacing 120–145%; `none` disables
  tracking-clamp: {{ tracking_clamp }},
  base-word-space: 0.28,
  k-word-space: {{ word_space_k }},
) = {
  let tracking = s => {
    let raw = k-tracking * calc.ln(optical-size / s)
    let val = if tracking-clamp == none { raw } else {
      calc.max(-tracking-clamp, calc.min(tracking-clamp, raw))
    }
    val * 1em
  }
  let leading-ratio = (s, measure: {{ measure }}) => {
    let m = if measure == none { 0 } else { (measure - 65) * 0.006 }
    let x = -(x-height - 0.50) * 0.8
    let z = -0.10 * calc.ln(s / optical-size)
    let raw = leading-base + m + x + z
    if leading-clamp == none { raw } else {
      calc.max(leading-clamp.at(0), calc.min(leading-clamp.at(1), raw))
    }
  }
  let leading = (s, measure: {{ measure }}) => {
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
{% for p in profiles %}  {{ p.padded_name }}{{ p.make }},
{% endfor %}
  // Font-specific profiles — x-height MEASURED from the font's OS/2 table.
{% for p in fonts %}  {{ p.padded_name }}{{ p.make }},
{% endfor %})

#let bundles = (
{% for b in generic_bundles %}  {{ b.name }}: (
    family:  "{{ b.family }}",
    scale:   scale.presets.{{ scale_default }},
    profile: presets.{{ b.profile_key }},
  ),
{% endfor %}
  // Font-specific bundles — real typeface + its measured optical profile.
{% for b in font_bundles %}  {{ b.name }}: (
    family:  "{{ b.family }}",
    scale:   scale.presets.{{ scale_default }},
    profile: presets.{{ b.profile_key }},
  ),
{% endfor %})

