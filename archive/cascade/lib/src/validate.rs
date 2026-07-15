//! Domain validation for the loaded spec + manifest.
//!
//! Types guarantee SHAPE; this guarantees TYPOGRAPHIC TRUTH — the invariants the type
//! system can't express: reference integrity, cross-palette parity, and range bounds.
//! Collects every problem (not just the first) so one run reports all of them.
//!
//! Run as a `#[test]` (see `tests/spec_valid.rs`) — and, if later moved behind a
//! `[build-dependency]`, from `build.rs` as a hard gate so an invalid spec fails the build.
use crate::manifest::Manifest;
use crate::spec::Spec;
use std::collections::HashSet;

/// Check every invariant. `Ok(())` means the spec + manifest are typographically valid.
pub fn validate(spec: &Spec, manifest: &Manifest) -> Result<(), Vec<String>> {
    let mut p = Vec::new();

    // ── uniqueness of names within each list ──
    dup(&mut p, "scale.presets", spec.scale.presets.iter().map(|x| x.name.as_str()));
    dup(&mut p, "optical.profiles", spec.optical.profiles.iter().map(|x| x.name.as_str()));
    dup(&mut p, "generics.stacks", spec.generics.stacks.iter().map(|x| x.name.as_str()));
    dup(&mut p, "generics.bundles", spec.generics.bundles.iter().map(|x| x.name.as_str()));
    dup(&mut p, "rhythm.multipliers", spec.rhythm.multipliers.iter().map(|x| x.name.as_str()));
    dup(&mut p, "theme.light", spec.theme.light.iter().map(|x| x.name.as_str()));
    dup(&mut p, "theme.dark", spec.theme.dark.iter().map(|x| x.name.as_str()));
    dup(&mut p, "manifest.fonts", manifest.fonts.iter().map(|x| x.name.as_str()));

    // ── reference integrity ──
    let presets: HashSet<&str> = spec.scale.presets.iter().map(|x| x.name.as_str()).collect();
    let profiles: HashSet<&str> = spec.optical.profiles.iter().map(|x| x.name.as_str()).collect();
    let stacks: HashSet<&str> = spec.generics.stacks.iter().map(|x| x.name.as_str()).collect();
    let bundles: HashSet<&str> = spec.generics.bundles.iter().map(|x| x.name.as_str()).collect();

    if !presets.contains(spec.scale.default.as_str()) {
        p.push(format!("scale.default '{}' is not a preset", spec.scale.default));
    }
    if !bundles.contains(spec.generics.default.as_str()) {
        p.push(format!("generics.default '{}' is not a bundle", spec.generics.default));
    }
    for b in &spec.generics.bundles {
        if !stacks.contains(b.stack.as_str()) {
            p.push(format!("bundle '{}' → stack '{}' not found", b.name, b.stack));
        }
        if !profiles.contains(b.profile.as_str()) {
            p.push(format!("bundle '{}' → profile '{}' not found", b.name, b.profile));
        }
    }
    for s in &manifest.scales {
        if !presets.contains(s.as_str()) {
            p.push(format!("manifest.scales '{s}' is not a spec preset"));
        }
    }
    for t in &manifest.themes {
        if t != "light" && t != "dark" {
            p.push(format!("manifest.themes '{t}' is not light|dark"));
        }
    }

    // ── cross-palette parity: light and dark must define the same token set ──
    let light: HashSet<&str> = spec.theme.light.iter().map(|x| x.name.as_str()).collect();
    let dark: HashSet<&str> = spec.theme.dark.iter().map(|x| x.name.as_str()).collect();
    for missing in light.difference(&dark) {
        p.push(format!("theme.dark is missing color '{missing}' (present in light)"));
    }
    for extra in dark.difference(&light) {
        p.push(format!("theme.light is missing color '{extra}' (present in dark)"));
    }

    // ── colors are #RRGGBB ──
    for c in spec.theme.light.iter().chain(&spec.theme.dark) {
        if !is_hex6(&c.hex) {
            p.push(format!("color '{}' hex '{}' is not #RRGGBB", c.name, c.hex));
        }
    }

    // ── scale domain ──
    if !(spec.scale.steps.min < 0 && spec.scale.steps.max > 0) {
        p.push("scale.steps must span below and above 0".into());
    }
    for pr in &spec.scale.presets {
        if pr.ratio <= 1.0 {
            p.push(format!("preset '{}' ratio {} must be > 1", pr.name, pr.ratio));
        }
        if pr.n < 1 {
            p.push(format!("preset '{}' n must be ≥ 1", pr.name));
        }
    }

    // ── optical domain ──
    let lc = &spec.optical.leading_clamp;
    if lc.min > lc.max {
        p.push(format!("optical.leading_clamp.min {} > max {}", lc.min, lc.max));
    }
    if !(lc.min >= 1.0 && lc.max <= 2.0) {
        p.push(format!("optical.leading_clamp [{}, {}] out of [1.0, 2.0]", lc.min, lc.max));
    }
    if spec.optical.tracking_clamp <= 0.0 {
        p.push("optical.tracking_clamp must be > 0".into());
    }
    if !(30..=120).contains(&spec.optical.measure) {
        p.push(format!("optical.measure {} out of 30..=120", spec.optical.measure));
    }
    for pf in &spec.optical.profiles {
        check_profile(&mut p, &pf.name, pf.x_height, pf.leading_base, pf.k_tracking, pf.word_space);
    }

    // ── fonts (manifest) domain ──
    for f in &manifest.fonts {
        check_profile(&mut p, &f.name, f.profile.x_height, f.profile.leading_base, f.profile.k_tracking, f.profile.word_space);
        let m = &f.measured;
        if !(0.0 < m.x_height && m.x_height < m.cap_height && m.cap_height < 1.0) {
            p.push(format!("font '{}': need 0 < x_height {} < cap_height {} < 1", f.name, m.x_height, m.cap_height));
        }
        if m.units_per_em == 0 {
            p.push(format!("font '{}': units_per_em must be > 0", f.name));
        }
        if !matches!(m.asc.parse::<f64>(), Ok(v) if v > 0.0) {
            p.push(format!("font '{}': asc '{}' must parse to > 0", f.name, m.asc));
        }
        if !matches!(m.desc.parse::<f64>(), Ok(v) if v < 0.0) {
            p.push(format!("font '{}': desc '{}' must parse to < 0", f.name, m.desc));
        }
    }

    // ── rhythm: the spacing ladder is strictly increasing ──
    for w in spec.rhythm.multipliers.windows(2) {
        if w[1].value <= w[0].value {
            p.push(format!(
                "rhythm.multipliers not strictly increasing at '{}' ({} ≤ {})",
                w[1].name, w[1].value, w[0].value
            ));
        }
    }

    if p.is_empty() { Ok(()) } else { Err(p) }
}

/// Shared profile-range checks (an optical profile and a font's inline profile).
fn check_profile(p: &mut Vec<String>, name: &str, x_height: f64, leading_base: f64, k_tracking: f64, word_space: f64) {
    if !(0.0 < x_height && x_height < 1.0) {
        p.push(format!("'{name}': x_height {x_height} must be in (0, 1)"));
    }
    if !(1.0..=2.0).contains(&leading_base) {
        p.push(format!("'{name}': leading_base {leading_base} out of [1.0, 2.0]"));
    }
    if k_tracking < 0.0 {
        p.push(format!("'{name}': k_tracking {k_tracking} must be ≥ 0"));
    }
    if word_space < 0.0 {
        p.push(format!("'{name}': word_space {word_space} must be ≥ 0"));
    }
}

fn dup<'a>(p: &mut Vec<String>, list: &str, it: impl Iterator<Item = &'a str>) {
    let mut seen = HashSet::new();
    for name in it {
        if !seen.insert(name) {
            p.push(format!("{list}: duplicate name '{name}'"));
        }
    }
}

fn is_hex6(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 7 && b[0] == b'#' && b[1..].iter().all(u8::is_ascii_hexdigit)
}
