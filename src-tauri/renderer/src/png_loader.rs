/// PNG Loader — Decode PNG files and apply optional tint color.

use std::path::Path;

/// Load a PNG file and return premultiplied-alpha RGBA pixel data.
/// Optionally applies a tint color (multiplies RGB channels).
/// Output is premultiplied to match our PREMULTIPLIED_ALPHA_BLENDING pipeline.
pub fn load_png(
    png_path: &Path,
    tint_hex: Option<&str>,
) -> Result<(u32, u32, Vec<u8>), String> {
    let img = image::open(png_path)
        .map_err(|e| format!("Failed to open PNG {}: {e}", png_path.display()))?
        .into_rgba8();

    let width = img.width();
    let height = img.height();
    let mut pixels = img.into_raw();

    // Apply tint (if any) and premultiply alpha in one pass using integer math.
    // Integer ops auto-vectorize much better than f32 (LLVM emits SIMD).
    // Formula: out = (val * alpha + 127) / 255  (rounds to nearest)
    if let Some(hex) = tint_hex {
        let hex = hex.trim_start_matches('#');
        let tr = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as u32;
        let tg = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as u32;
        let tb = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as u32;

        for chunk in pixels.chunks_exact_mut(4) {
            let a = chunk[3] as u32;
            let r = (chunk[0] as u32 * tr + 127) / 255;
            let g = (chunk[1] as u32 * tg + 127) / 255;
            let b = (chunk[2] as u32 * tb + 127) / 255;
            chunk[0] = ((r * a + 127) / 255) as u8;
            chunk[1] = ((g * a + 127) / 255) as u8;
            chunk[2] = ((b * a + 127) / 255) as u8;
        }
    } else {
        // No tint — just premultiply alpha
        for chunk in pixels.chunks_exact_mut(4) {
            let a = chunk[3] as u16;
            chunk[0] = ((chunk[0] as u16 * a + 127) / 255) as u8;
            chunk[1] = ((chunk[1] as u16 * a + 127) / 255) as u8;
            chunk[2] = ((chunk[2] as u16 * a + 127) / 255) as u8;
        }
    }

    Ok((width, height, pixels))
}

/// Load a PNG file and return raw RGBA pixel data (no tinting, no premultiply).
/// Used for white-detection before choosing alpha-only vs full RGBA path.
pub fn load_png_raw(png_path: &Path) -> Result<(u32, u32, Vec<u8>), String> {
    let img = image::open(png_path)
        .map_err(|e| format!("Failed to open PNG {}: {e}", png_path.display()))?
        .into_rgba8();

    Ok((img.width(), img.height(), img.into_raw()))
}

/// Check if all visible pixels (alpha > 0) are pure white (RGB = 255,255,255).
/// If true, the frame can be stored as alpha-only (1 byte/pixel) with tint applied in shader.
pub fn is_white_only(rgba: &[u8]) -> bool {
    for chunk in rgba.chunks_exact(4) {
        if chunk[3] > 0 && (chunk[0] != 255 || chunk[1] != 255 || chunk[2] != 255) {
            return false;
        }
    }
    true
}

/// Extract just the alpha channel from RGBA data (4 bytes/pixel → 1 byte/pixel).
pub fn extract_alpha(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks_exact(4).map(|chunk| chunk[3]).collect()
}

