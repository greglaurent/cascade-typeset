// cascade-typst — rhythm.typ  (GENERATED from tokens.mjs by `just gen` — do not edit by hand)
// Vertical rhythm from scale + font + measure. baseline = ceil(body-size ·
// leading-ratio / grid-unit) · grid-unit; spacing tokens are multiples of the unit.
// snap() is opt-in (Tim Brown), not an enforced lattice.

#import "scale.typ"
#import "font.typ"

#let make(
  scale: scale.presets.classical,
  font: font.presets.sans-text,
  measure: 65,
  body-step: 0,
  grid-unit: 4pt,
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
    n1:   grid-unit * 0.5,
    base: grid-unit * 1,
    p1:   grid-unit * 2,
    p2:   grid-unit * 3,
    p3:   grid-unit * 4,
    p4:   grid-unit * 6,
    p5:   grid-unit * 8,
    p6:   grid-unit * 12,
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
