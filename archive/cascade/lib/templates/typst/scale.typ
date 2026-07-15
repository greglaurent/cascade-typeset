// cascade-typst — scale.typ  ({{ banner }})
// Typographic scale (Spencer Mortensen):  f_i = f_0 · r^(i / n)
//   f_0 = base size, r = ratio, n = notes per interval.  Pure math, font-agnostic.

#let make(base: {{ base_print }}, ratio: {{ default_ratio }}, n: {{ default_n }}) = {
  let size = i => base * calc.pow(ratio, i / n)
  (
{% for e in steps %}    {{ e.k }}: size({{ e.i }}),
{% endfor %}    size: size,
    params: (base: base, ratio: ratio, n: n),
  )
}

#let presets = (
{% for p in presets %}  {{ p.padded_name }} make(ratio: {{ p.ratio }}, n: {{ p.n }}),
{% endfor %})

