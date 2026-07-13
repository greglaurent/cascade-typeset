// cascade-typst — scale.typ  (GENERATED from tokens.mjs by `just gen` — do not edit by hand)
// Typographic scale (Spencer Mortensen):  f_i = f_0 · r^(i / n)
//   f_0 = base size, r = ratio, n = notes per interval.  Pure math, font-agnostic.

#let make(base: 11pt, ratio: 2, n: 5) = {
  let size = i => base * calc.pow(ratio, i / n)
  (
    n5: size(-5),
    n4: size(-4),
    n3: size(-3),
    n2: size(-2),
    n1: size(-1),
    base: size(0),
    p1: size(1),
    p2: size(2),
    p3: size(3),
    p4: size(4),
    p5: size(5),
    size: size,
    params: (base: base, ratio: ratio, n: n),
  )
}

#let presets = (
  classical:      make(ratio: 2, n: 5),
  golden-ratio:   make(ratio: 1.618033988749895, n: 1),
  golden-ditonic: make(ratio: 1.618033988749895, n: 2),
  tritonic:       make(ratio: 2, n: 3),
  tetratonic:     make(ratio: 2, n: 4),
  major-third:    make(ratio: 1.25, n: 1),
  minor-third:    make(ratio: 1.2, n: 1),
)
