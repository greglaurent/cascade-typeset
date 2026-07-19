// Chrome driver, ported from the PoC site. Pico proves the specimen coexists with another CSS
// library; datastar is part of the stack. The chrome manages the Options selects, the theme
// toggle, and the CSS/Typst tabs, and drives BOTH projections of the same document: the CSS tab
// via the specimen's setCascade() hook (instant, in-browser), the Typst (PDF) tab by pointing the
// iframe at /sample.pdf?<options> (the server bakes cascade.typ and compiles it). Both react to the
// same options; there is no separate "compile" step.
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

// Typst (PDF) tab: the print projection, compiled server-side from the same options. The iframe just
// points at /sample.pdf?<opts>; identical options resolve to the same URL, so the browser and the
// server both cache it. Theme is omitted deliberately — print is paper (the light palette).
const pdfUrl = () => {
  const s = state();
  const q = new URLSearchParams({ scale: s.scale, body: s.body, heading: s.heading, code: s.code, notes: s.notes });
  return `/sample.pdf?${q}`;
};
let pdfTimer = 0;
const reloadPdf = () => {
  window.clearTimeout(pdfTimer);
  pdfTimer = window.setTimeout(() => { view.src = pdfUrl(); }, 250);
};

// An option changed: refresh whichever projection is showing.
const refreshView = () => { if (isCss()) applyCss(); else reloadPdf(); };

const closeMenu = () => { opts.open = false; };

// Theme drives the chrome via [data-theme] on <html> (the --ui-* palette flips from one switch);
// color-scheme is set so native controls + scrollbars match. It also flips the CSS specimen; the
// PDF is unaffected (print = the light palette), so no recompile on toggle.
const applyChrome = () => {
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme = theme;
};

const themeBtn = document.getElementById("theme-toggle") as HTMLButtonElement;
const ICON: Record<"light" | "dark", string> = {
  light: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></svg>',
  dark: '<svg viewBox="0 0 24 24" fill="currentColor" stroke="none"><path d="M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8z"/></svg>',
};
const setTheme = (t: "light" | "dark") => {
  theme = t;
  themeBtn.innerHTML = ICON[t];
  themeBtn.setAttribute("aria-label", t === "dark" ? "Switch to light mode" : "Switch to dark mode");
  applyChrome();
  applyCss();
};
themeBtn.addEventListener("click", () => setTheme(theme === "dark" ? "light" : "dark"));

Object.values(sels).forEach((s) => s.addEventListener("change", refreshView));
view.addEventListener("load", () => {
  applyCss();
  try {
    view.contentDocument?.addEventListener("click", () => { if (opts.open) closeMenu(); });
  } catch { /* cross-origin (PDF tab) */ }
});

const isPdf = (t: HTMLButtonElement) => (t.dataset.src ?? "").endsWith(".pdf");
const show = (kind: "css" | "pdf") => {
  const css = kind === "css";
  view.dataset.kind = kind;
  view.src = css ? "/sample" : pdfUrl();
  tabs.forEach((t) => t.setAttribute("aria-selected", String(isPdf(t) === !css)));
  location.hash = css ? "css" : "typst";
};
tabs.forEach((t) => t.addEventListener("click", () => show(isPdf(t) ? "pdf" : "css")));

document.addEventListener("click", (e) => {
  if (opts.open && !opts.contains(e.target as Node)) closeMenu();
});

setTheme(matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
show(location.hash === "#typst" ? "pdf" : "css");
