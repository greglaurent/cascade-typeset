// Chrome driver, ported from the PoC site. Pico proves the specimen coexists with another CSS
// library; datastar is part of the stack. The chrome manages the Options selects, the theme
// toggle, and the CSS/Typst tabs, and drives the specimen (an iframe) via its setCascade() hook.
import "@picocss/pico/css/pico.min.css";
import "./vendor/datastar.js";

const view = document.getElementById("view") as HTMLIFrameElement;
const tabs = [...document.querySelectorAll<HTMLButtonElement>(".tab")];
const opts = document.getElementById("opts") as HTMLDetailsElement;
const sels = {
  scale: document.getElementById("sel-scale") as HTMLSelectElement,
  body: document.getElementById("sel-style") as HTMLSelectElement,
  heading: document.getElementById("sel-heading") as HTMLSelectElement,
  code: document.getElementById("sel-code") as HTMLSelectElement,
  notes: document.getElementById("sel-notes") as HTMLSelectElement,
};
let theme: "light" | "dark" = "light";

const state = () => {
  const body = sels.body.value;
  // "match body" (empty heading) mirrors the body typeface via the matching heading-* class
  // (bundle-serif -> heading-serif, bundle-lora -> heading-lora, ...).
  const heading = sels.heading.value || body.replace("bundle-", "heading-");
  // code is independent (empty = the built-in IBM Plex Mono default); no "match body" mirroring.
  return { scale: sels.scale.value, body, heading, code: sels.code.value, notes: sels.notes.value, theme };
};
const isCss = () => view.dataset.kind === "css";
// The specimen exposes setCascade() as a global on its own window (same origin).
const applyCss = () => {
  const w = view.contentWindow as (Window & { setCascade?: (s: unknown) => void }) | null;
  if (isCss() && w?.setCascade) w.setCascade(state());
};

const closeMenu = () => { opts.open = false; };

// Theme drives the chrome via [data-theme] on <html> (the --ui-* palette flips from one switch);
// color-scheme is set so native controls + scrollbars match.
const applyChrome = () => {
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme = theme;
};

const compileBtn = document.getElementById("compile") as HTMLButtonElement;
const typstOptions = () => ({
  scale: sels.scale.value.replace("scale-", ""),
  body: sels.body.value.replace("bundle-", ""),
  heading: sels.heading.value.replace("heading-", ""),
  theme,
  sidenotes: sels.notes.value === "banded" ? "true" : "false",
});
const updateCompile = () => {
  compileBtn.disabled = false;
  compileBtn.title = "Recompile the Typst PDF with the current options";
};
let compiling = false;
const doCompile = async () => {
  if (compiling || compileBtn.disabled) return;
  compiling = true;
  compileBtn.disabled = true;
  compileBtn.textContent = "Compiling...";
  try {
    const r = await fetch("/compile", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(typstOptions()),
    });
    if (!r.ok) throw new Error(await r.text());
    view.src = "/sample.pdf?t=" + Date.now();
  } catch (e) {
    console.error("compile failed:", e);
    compileBtn.textContent = "Error";
    setTimeout(() => { compileBtn.textContent = "Compile"; updateCompile(); }, 1800);
    compiling = false;
    return;
  }
  compileBtn.textContent = "Compile";
  compiling = false;
  updateCompile();
};
compileBtn.addEventListener("click", () => void doCompile());

const apply = () => { applyChrome(); applyCss(); };

const themeBtn = document.getElementById("theme-toggle") as HTMLButtonElement;
const ICON: Record<"light" | "dark", string> = {
  light: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></svg>',
  dark: '<svg viewBox="0 0 24 24" fill="currentColor" stroke="none"><path d="M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8z"/></svg>',
};
const setTheme = (t: "light" | "dark") => {
  theme = t;
  themeBtn.innerHTML = ICON[t];
  themeBtn.setAttribute("aria-label", t === "dark" ? "Switch to light mode" : "Switch to dark mode");
  apply();
};
themeBtn.addEventListener("click", () => setTheme(theme === "dark" ? "light" : "dark"));

Object.values(sels).forEach((s) => s.addEventListener("change", apply));
view.addEventListener("load", () => {
  applyCss();
  try {
    view.contentDocument?.addEventListener("click", () => { if (opts.open) closeMenu(); });
  } catch { /* cross-origin (PDF tab) */ }
});

const show = (src: string) => {
  const css = src.endsWith("/sample");
  view.dataset.kind = css ? "css" : "pdf";
  view.src = src;
  tabs.forEach((t) => t.setAttribute("aria-selected", String(t.dataset.src === src)));
  opts.hidden = !css;
  compileBtn.hidden = css;
  if (!css) { closeMenu(); updateCompile(); }
  location.hash = css ? "css" : "typst";
};
tabs.forEach((t) => t.addEventListener("click", () => show(t.dataset.src as string)));

document.addEventListener("click", (e) => {
  if (opts.open && !opts.contains(e.target as Node)) closeMenu();
});

setTheme(matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
show(location.hash === "#typst" ? "/sample.pdf" : "/sample");
