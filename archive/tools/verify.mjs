// cascade-typeset — fidelity checks (dep-free). Run `just verify`.
//
//  1. Token parity — the committed generated files equal what `just gen` produces
//     from tokens.mjs, so nothing is stale or hand-edited, and the two renderers
//     carry the same numbers by construction.
//  2. Typst model-check — the Typst FORMULAS (scale / leading / rhythm) match a
//     reference model computed from the tokens, catching formula drift in the .typ
//     templates. The reference model is the seam where a browser layer would later
//     plug in actual getComputedStyle values to cover the CSS formulas too.
import { readFileSync, writeFileSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { files } from './gen.mjs';
import { scale, optical, rhythm } from './tokens.mjs';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');   // repo root (tools/ is one level down)
let fails = 0;
const ok  = l => console.log(`  ok    ${l}`);
const bad = (l, d) => { fails++; console.log(`  FAIL  ${l}${d ? '  — ' + d : ''}`); };
const near = (a, b) => Math.abs(a - b) < 1e-6;

// ── 1. token parity ───────────────────────────────────────────────────────────
console.log('token parity — committed files == `just gen` output:');
for (const [rel, body] of files()) {
  let disk;
  try { disk = readFileSync(join(root, rel), 'utf8'); } catch { bad(rel, 'missing'); continue; }
  disk === body ? ok(rel) : bad(rel, 'drift — run `just gen`');
}

// ── 2. Typst formula model-check ──────────────────────────────────────────────
console.log('\nTypst formulas vs reference model (classical / serif-text / 11pt / measure 65):');
const probe = [
  '#import "scale.typ"', '#import "font.typ"', '#import "rhythm.typ"',
  '#let sc = scale.presets.classical',
  '#let fp = font.presets.serif-text',
  '#let rh = rhythm.make(scale: sc, font: fp, measure: 65)',
  '#let steps = range(-5, 6)',
  '#metadata((',
  '  sizes: steps.map(i => (sc.size)(i) / 1pt),',
  '  leading: steps.map(i => (fp.leading-ratio)((sc.size)(i), measure: 65)),',
  '  baseline: rh.baseline / 1pt,',
  '  spacing: (n1: rh.spacing.n1 / 1pt, base: rh.spacing.base / 1pt, p1: rh.spacing.p1 / 1pt,',
  '            p2: rh.spacing.p2 / 1pt, p3: rh.spacing.p3 / 1pt, p4: rh.spacing.p4 / 1pt,',
  '            p5: rh.spacing.p5 / 1pt, p6: rh.spacing.p6 / 1pt),',
  '))<probe>', '',
].join('\n');

const probePath = join(root, 'cascade-typst/_probe.typ');
let q;
try {
  writeFileSync(probePath, probe);
  const out = execFileSync('typst', ['query', '--root', '.', 'cascade-typst/_probe.typ', '<probe>', '--field', 'value', '--one'], { cwd: root, encoding: 'utf8' });
  q = JSON.parse(out);
} catch (e) {
  bad('typst query', String(e.stderr || e.message).split('\n')[0]);
} finally {
  rmSync(probePath, { force: true });
}

if (q) {
  // Reference model — the documented formulas, computed straight from tokens.
  const b = parseFloat(scale.base.print);                 // 11
  const { ratio, n } = scale.presets[scale.default];      // classical: 2, 5
  const size = i => b * ratio ** (i / n);
  const clamp = (lo, x, hi) => Math.max(lo, Math.min(hi, x));
  const lc = optical.leadingClamp, p = optical.profiles['serif-text'], opt = parseFloat(p.opticalSize);
  const lead = i => clamp(lc.min, p.leadingBase + (65 - 65) * 0.006 - (p.xHeight - 0.5) * 0.8 - 0.10 * Math.log(size(i) / opt), lc.max);
  const u = parseFloat(rhythm.unit.print);

  const checkArr = (name, got, ref) => {
    const i = got.findIndex((v, k) => !near(v, ref(k - 5)));
    i === -1 ? ok(name) : bad(name, `step ${i - 5}: typst ${got[i]} vs ref ${ref(i - 5)}`);
  };
  checkArr('scale sizes', q.sizes, size);
  checkArr('leading ratios', q.leading, lead);

  const refBaseline = Math.ceil(size(0) * lead(0) / u) * u;
  near(q.baseline, refBaseline) ? ok('rhythm baseline') : bad('rhythm baseline', `typst ${q.baseline} vs ref ${refBaseline}`);
  for (const [k, m] of Object.entries(rhythm.multipliers)) {
    near(q.spacing[k], u * m) ? ok(`rhythm spacing ${k}`) : bad(`rhythm spacing ${k}`, `typst ${q.spacing[k]} vs ref ${u * m}`);
  }
}

console.log(fails ? `\n${fails} check(s) FAILED` : '\nall checks passed ✓');
process.exit(fails ? 1 : 0);
