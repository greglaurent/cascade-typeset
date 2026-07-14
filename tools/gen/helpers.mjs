// cascade-typeset — shared generator helpers, used by every renderer backend
// (gen/css.mjs, gen/typst.mjs, …). Dependency-free (Node ESM).
import { scale } from '../tokens.mjs';

export const GEN = 'GENERATED from tokens.mjs by `just gen` — do not edit by hand';
export const cap = s => s[0].toUpperCase() + s.slice(1);

// Scale step range (n5 … 0 … p5), shared by the scale + font backends.
export const steps = [];
for (let i = scale.steps.min; i <= scale.steps.max; i++) steps.push(i);
export const cssLabel = i => (i < 0 ? `n${-i}` : i > 0 ? `p${i}` : '0');
