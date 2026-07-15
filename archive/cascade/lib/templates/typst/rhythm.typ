// cascade-typst — rhythm.typ  ({{ banner }})
// Vertical rhythm from scale + font + measure. baseline = ceil(body-size ·
// leading-ratio / grid-unit) · grid-unit; spacing tokens are multiples of the unit.
// snap() is opt-in (Tim Brown), not an enforced lattice.

#import "scale.typ"
#import "font.typ"

#let make(
  scale: scale.presets.{{ scale_default }},
  font: font.presets.sans-text,
  measure: {{ measure }},
  body-step: 0,
  grid-unit: {{ grid_unit_print }},
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
{% for m in multipliers %}    {{ m.padded_name }}grid-unit * {{ m.value }},
{% endfor %}  )

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

