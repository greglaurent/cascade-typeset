// cascade-typeset — generator orchestrator. Reads tokens.mjs and writes the token-driven
// parts of every renderer. Each renderer is a backend under gen/ that exports files();
// adding a renderer = one new file there + one import + spread here. Run `just gen`.
// Dependency-free (Node ESM). Formulas live in each backend's templates; only the numbers
// come from tokens.mjs.
import { writeFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { argv } from 'node:process';
import { dirname, join } from 'node:path';
import { files as cssFiles } from './gen/css.mjs';
import { files as typstFiles } from './gen/typst.mjs';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');   // repo root (tools/ is one level down)
const write = (rel, body) => { writeFileSync(join(root, rel), body); console.log('wrote', rel); };

// The single source for both the CLI and verify.mjs — every renderer backend, collected.
export function files() {
  return [...cssFiles(), ...typstFiles()];
}

// CLI: `node gen.mjs` writes them. (Importing the module — e.g. from verify.mjs —
// does not, so the verifier can regenerate in-memory and diff against disk.)
if (import.meta.url === pathToFileURL(argv[1]).href) {
  for (const [rel, body] of files()) write(rel, body);
}
