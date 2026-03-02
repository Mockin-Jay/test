/// SVG Loader — Reads SVGs, replaces sentinel colors, rasterizes via resvg.
///
/// Sentinel color mapping (same as SVGRecolorer.ts):
///   red   → base color
///   lime  → shadow  (L × 0.82)
///   blue  → shadow2 (L × 0.67)
///   #000 / black → outline (kept as-is)

use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

use crate::color::compute_variants;

/// Pre-compiled sentinel color regexes (compiled once, reused forever).
struct SentinelPatterns {
    fill_red_css: Regex,
    stroke_red_css: Regex,
    fill_lime_css: Regex,
    stroke_lime_css: Regex,
    fill_blue_css: Regex,
    stroke_blue_css: Regex,
    fill_red_xml: Regex,
    stroke_red_xml: Regex,
    fill_lime_xml: Regex,
    stroke_lime_xml: Regex,
    fill_blue_xml: Regex,
    stroke_blue_xml: Regex,
}

static PATTERNS: LazyLock<SentinelPatterns> = LazyLock::new(|| SentinelPatterns {
    fill_red_css: Regex::new(r#"(?i)(\bfill\s*:\s*)red(\s*[;"])"#).unwrap(),
    stroke_red_css: Regex::new(r#"(?i)(\bstroke\s*:\s*)red(\s*[;"])"#).unwrap(),
    fill_lime_css: Regex::new(r#"(?i)(\bfill\s*:\s*)lime(\s*[;"])"#).unwrap(),
    stroke_lime_css: Regex::new(r#"(?i)(\bstroke\s*:\s*)lime(\s*[;"])"#).unwrap(),
    fill_blue_css: Regex::new(r#"(?i)(\bfill\s*:\s*)blue(\s*[;"])"#).unwrap(),
    stroke_blue_css: Regex::new(r#"(?i)(\bstroke\s*:\s*)blue(\s*[;"])"#).unwrap(),
    fill_red_xml: Regex::new(r#"(?i)(\bfill=")red(")"#).unwrap(),
    stroke_red_xml: Regex::new(r#"(?i)(\bstroke=")red(")"#).unwrap(),
    fill_lime_xml: Regex::new(r#"(?i)(\bfill=")lime(")"#).unwrap(),
    stroke_lime_xml: Regex::new(r#"(?i)(\bstroke=")lime(")"#).unwrap(),
    fill_blue_xml: Regex::new(r#"(?i)(\bfill=")blue(")"#).unwrap(),
    stroke_blue_xml: Regex::new(r#"(?i)(\bstroke=")blue(")"#).unwrap(),
});

/// Replace sentinel colors in SVG text with user-chosen colors.
pub fn recolor_svg(svg_text: &str, base_hex: &str) -> String {
    let v = compute_variants(base_hex);
    let p = &*PATTERNS;

    let mut result = svg_text.to_string();

    let replacements: &[(&Regex, &str)] = &[
        (&p.fill_red_css, &v.base),
        (&p.stroke_red_css, &v.base),
        (&p.fill_lime_css, &v.shadow),
        (&p.stroke_lime_css, &v.shadow),
        (&p.fill_blue_css, &v.shadow2),
        (&p.stroke_blue_css, &v.shadow2),
        (&p.fill_red_xml, &v.base),
        (&p.stroke_red_xml, &v.base),
        (&p.fill_lime_xml, &v.shadow),
        (&p.stroke_lime_xml, &v.shadow),
        (&p.fill_blue_xml, &v.shadow2),
        (&p.stroke_blue_xml, &v.shadow2),
    ];

    for (re, color) in replacements {
        result = re
            .replace_all(&result, format!("${{1}}{}${{2}}", color))
            .into_owned();
    }

    result
}

/// Load an SVG from disk, optionally recolor, and rasterize to RGBA pixels.
/// Returns (width, height, rgba_bytes).
pub fn load_and_rasterize_svg(
    svg_path: &Path,
    base_hex: Option<&str>,
    target_width: u32,
    target_height: u32,
) -> Result<(u32, u32, Vec<u8>), String> {
    let svg_text =
        std::fs::read_to_string(svg_path).map_err(|e| format!("Failed to read SVG: {e}"))?;

    let svg_text = match base_hex {
        Some(hex) => recolor_svg(&svg_text, hex),
        None => svg_text,
    };

    let opts = usvg::Options::default();
    let tree =
        usvg::Tree::from_str(&svg_text, &opts).map_err(|e| format!("Failed to parse SVG: {e}"))?;

    let mut pixmap = tiny_skia::Pixmap::new(target_width, target_height)
        .ok_or("Failed to create pixmap")?;

    let tree_size = tree.size();
    let sx = target_width as f32 / tree_size.width();
    let sy = target_height as f32 / tree_size.height();

    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(sx, sy),
        &mut pixmap.as_mut(),
    );

    // resvg/tiny_skia output premultiplied alpha — keep as-is.
    // Our pipeline uses PREMULTIPLIED_ALPHA_BLENDING, so no conversion needed.
    // Converting to straight alpha and back is lossy (edge artifacts at low alpha).
    let pixels = pixmap.data().to_vec();

    Ok((target_width, target_height, pixels))
}
