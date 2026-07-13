// cascade-typeset — dev server (Deno). Serves both example projects and the viewer,
// and exposes POST /compile to rebuild the Typst PDF from the viewer's options.
//   deno run --allow-net --allow-read --allow-write --allow-run --allow-env serve.ts
import { join, normalize, extname } from "node:path";

const root = new URL(".", import.meta.url).pathname.replace(/\/$/, ""); // repo root
const port = 8175;

const types: Record<string, string> = {
  ".html": "text/html", ".css": "text/css", ".js": "text/javascript", ".ts": "text/javascript",
  ".json": "application/json", ".svg": "image/svg+xml", ".pdf": "application/pdf",
  ".ttf": "font/ttf", ".woff2": "font/woff2", ".woff": "font/woff",
};

// Allowlists mirror the viewer controls + Typst presets, so nothing arbitrary
// reaches the typst CLI.
const ALLOW: Record<string, string[]> = {
  scale:   ["classical", "golden-ratio", "golden-ditonic", "tritonic", "tetratonic", "major-third", "minor-third"],
  body:    ["serif", "sans", "mono", "lora", "inter", "jost"],
  heading: ["", "serif", "sans", "mono", "lora", "inter", "jost"],
  theme:   ["light", "dark"],
  sidenotes: ["false", "true"],   // margin edition vs. footnotes (viewer's notes selector)
};

async function compile(req: Request): Promise<Response> {
  let o: Record<string, string>;
  try { o = await req.json(); } catch { return new Response("bad json", { status: 400 }); }
  const pick = (k: string) => (ALLOW[k].includes(o[k]) ? o[k] : ALLOW[k][0]);
  const opt = { scale: pick("scale"), body: pick("body"), heading: pick("heading"), theme: pick("theme"), sidenotes: pick("sidenotes") };

  const args = ["compile", "--root", ".", "--input", `scale=${opt.scale}`, "--input", `body=${opt.body}`, "--input", `theme=${opt.theme}`, "--input", `sidenotes=${opt.sidenotes}`];
  if (opt.heading) args.push("--input", `heading=${opt.heading}`); // empty ⇒ omit ⇒ match body
  args.push("cascade-typst/sample.typ", "cascade-typst/sample.pdf");

  const { success, stderr } = await new Deno.Command("typst", { args, cwd: root }).output();
  if (!success) return new Response(new TextDecoder().decode(stderr), { status: 500 });
  return Response.json({ ok: true, compiled: opt });
}

Deno.serve({ port, onListen: () => console.log(`cascade-typeset → http://localhost:${port}`) }, async (req) => {
  const url = new URL(req.url);
  if (req.method === "POST" && url.pathname === "/compile") return compile(req);

  let p = decodeURIComponent(url.pathname);
  if (p === "/") p = "/index.html";
  const file = join(root, normalize("/" + p));         // normalize; block ../ escape
  if (!file.startsWith(root)) return new Response("forbidden", { status: 403 });
  try {
    const data = await Deno.readFile(file);
    return new Response(data, {
      headers: { "content-type": types[extname(file)] ?? "application/octet-stream", "cache-control": "no-store" },
    });
  } catch {
    return new Response("not found", { status: 404 });
  }
});
