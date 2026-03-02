/// compress-animations — Pre-bake animation PNGs into BC4/BC7 GPU-compressed .bcf files.
///
/// For each animation sublayer:
///   1. Decode first frame, detect alpha-only (all visible pixels white)
///   2. For each frame: pad to multiple-of-4 dims, encode BC4 (alpha) or BC7 (RGBA premultiplied)
///   3. Write .bcf file (20-byte header + concatenated compressed frames)
///   4. Write bc-metadata.json for generate-manifest.cjs
///
/// Usage: cargo run --manifest-path tools/compress-animations/Cargo.toml

use image::GenericImageView;
use intel_tex_2::RSurface;
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

const ASSETS_DIR: &str = "C:/Users/derec/Desktop/mini-ko assets";

// BCF file format magic
const BCF_MAGIC: &[u8; 4] = b"BCF\0";

#[derive(Serialize)]
struct BcMetadataEntry {
    #[serde(rename = "animRelPath")]
    anim_rel_path: String,
    #[serde(rename = "subName")]
    sub_name: String,
    #[serde(rename = "bcFile")]
    bc_file: String,
    #[serde(rename = "alphaOnly")]
    alpha_only: bool,
    #[serde(rename = "frameCount")]
    frame_count: u32,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct BcMetadata {
    #[serde(rename = "processedAt")]
    processed_at: String,
    frames: Vec<BcMetadataEntry>,
}

/// Info about a discovered animation sublayer.
struct SublayerInfo {
    anim_rel_path: String,
    sub_name: String,
    source_path: PathBuf,
    frame_count: u32,
    prefix: String,
    bcf_abs_path: PathBuf,
    bcf_rel_path: String,
}

/// Check if all visible pixels (alpha > 0) are pure white (R=G=B=255).
fn is_white_only(rgba: &[u8]) -> bool {
    for chunk in rgba.chunks_exact(4) {
        if chunk[3] > 0 && (chunk[0] != 255 || chunk[1] != 255 || chunk[2] != 255) {
            return false;
        }
    }
    true
}

/// Pad image data to dimensions that are multiples of 4.
/// Returns (padded_width, padded_height, padded_data).
fn pad_to_mult4(width: u32, height: u32, data: &[u8], bpp: u32) -> (u32, u32, Vec<u8>) {
    let pad_w = (4 - (width % 4)) % 4;
    let pad_h = (4 - (height % 4)) % 4;
    let new_w = width + pad_w;
    let new_h = height + pad_h;

    if pad_w == 0 && pad_h == 0 {
        return (new_w, new_h, data.to_vec());
    }

    let row_bytes = (width * bpp) as usize;
    let new_row_bytes = (new_w * bpp) as usize;
    let mut padded = vec![0u8; (new_w * new_h * bpp) as usize];

    for y in 0..height as usize {
        let src_start = y * row_bytes;
        let dst_start = y * new_row_bytes;
        padded[dst_start..dst_start + row_bytes].copy_from_slice(&data[src_start..src_start + row_bytes]);
    }

    (new_w, new_h, padded)
}

/// Premultiply alpha in RGBA data in-place.
fn premultiply_alpha(rgba: &mut [u8]) {
    for chunk in rgba.chunks_exact_mut(4) {
        let a = chunk[3] as u16;
        chunk[0] = ((chunk[0] as u16 * a + 127) / 255) as u8;
        chunk[1] = ((chunk[1] as u16 * a + 127) / 255) as u8;
        chunk[2] = ((chunk[2] as u16 * a + 127) / 255) as u8;
    }
}

/// Find all animation sublayer folders.
fn find_animation_sublayers(assets_dir: &Path) -> Vec<SublayerInfo> {
    let mut sublayers = Vec::new();

    let layer_folders: Vec<_> = fs::read_dir(assets_dir)
        .expect("Cannot read assets directory")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && e.file_name()
                    .to_str()
                    .map(|n| n.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
                    .unwrap_or(false)
        })
        .collect();

    for layer_dir in &layer_folders {
        let layer_path = layer_dir.path();
        let entries: Vec<_> = fs::read_dir(&layer_path)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();

        for entry in &entries {
            let asset_path = entry.path();
            let asset_contents: Vec<_> = fs::read_dir(&asset_path)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                        && e.file_name().to_str() != Some("optimized")
                })
                .collect();

            if asset_contents.is_empty() {
                continue;
            }

            for sub_dir in &asset_contents {
                let sub_path = sub_dir.path();
                let sub_name = sub_dir.file_name().to_string_lossy().to_string();

                // Check for optimized folder first
                let optimized_dir = sub_path.join("optimized");
                let (source_path, frame_pngs) = if optimized_dir.exists() {
                    let pngs = list_frame_pngs(&optimized_dir);
                    if pngs.is_empty() {
                        let pngs = list_frame_pngs(&sub_path);
                        (sub_path.clone(), pngs)
                    } else {
                        (optimized_dir, pngs)
                    }
                } else {
                    let pngs = list_frame_pngs(&sub_path);
                    (sub_path.clone(), pngs)
                };

                if frame_pngs.is_empty() {
                    continue;
                }

                // Extract prefix from first frame
                let first_frame = &frame_pngs[0];
                let prefix = first_frame
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|s| s.rfind('_').map(|i| &s[..i]))
                    .unwrap_or(&sub_name)
                    .to_string();

                let anim_rel_path = asset_path
                    .strip_prefix(assets_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");

                let bcf_filename = format!("{}.bcf", sub_name);
                let bcf_abs_path = asset_path.join(&bcf_filename);
                let bcf_rel_path = bcf_abs_path
                    .strip_prefix(assets_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");

                sublayers.push(SublayerInfo {
                    anim_rel_path,
                    sub_name,
                    source_path,
                    frame_count: frame_pngs.len() as u32,
                    prefix,
                    bcf_abs_path,
                    bcf_rel_path,
                });
            }
        }
    }

    sublayers
}

/// List animation frame PNGs in a directory, sorted.
fn list_frame_pngs(dir: &Path) -> Vec<PathBuf> {
    let mut pngs: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| {
                        // Match pattern: prefix_NNNNN.png
                        n.contains('_')
                            && n.rsplit('_')
                                .next()
                                .map(|s| s.trim_end_matches(".png").trim_end_matches(".PNG").chars().all(|c| c.is_ascii_digit()))
                                .unwrap_or(false)
                    })
                    .unwrap_or(false)
        })
        .collect();
    pngs.sort();
    pngs
}

/// Process one sublayer: encode all frames to BC4 or BC7, write .bcf file.
fn process_sublayer(sub: &SublayerInfo) -> Option<BcMetadataEntry> {
    let t_start = Instant::now();

    // Decode first frame to detect alpha-only and get dimensions
    let first_frame_path = sub
        .source_path
        .join(format!("{}_{:05}.png", sub.prefix, 0));

    if !first_frame_path.exists() {
        eprintln!("  [SKIP] First frame not found: {}", first_frame_path.display());
        return None;
    }

    let first_img = image::open(&first_frame_path)
        .map_err(|e| eprintln!("  [ERR] Failed to open first frame: {e}"))
        .ok()?;

    let (orig_w, orig_h) = first_img.dimensions();
    let first_rgba = first_img.into_rgba8().into_raw();
    let alpha_only = is_white_only(&first_rgba);

    // Compute padded dimensions
    let pad_w = (4 - (orig_w % 4)) % 4;
    let pad_h = (4 - (orig_h % 4)) % 4;
    let padded_w = orig_w + pad_w;
    let padded_h = orig_h + pad_h;

    let block_size: usize = if alpha_only { 8 } else { 16 }; // BC4 = 8, BC7 = 16
    let blocks_wide = (padded_w / 4) as usize;
    let blocks_high = (padded_h / 4) as usize;
    let frame_compressed_bytes = blocks_wide * blocks_high * block_size;

    let label = if alpha_only { "BC4 (alpha-only)" } else { "BC7 (RGBA)" };
    println!(
        "  {}: {} frames, {}x{} -> {}x{} padded, {}",
        sub.sub_name, sub.frame_count, orig_w, orig_h, padded_w, padded_h, label
    );

    // Encode all frames in parallel
    let frame_indices: Vec<u32> = (0..sub.frame_count).collect();

    let encoded_frames: Vec<Option<Vec<u8>>> = frame_indices
        .par_iter()
        .map(|&frame_idx| {
            let frame_path = sub
                .source_path
                .join(format!("{}_{:05}.png", sub.prefix, frame_idx));

            let img = image::open(&frame_path)
                .map_err(|e| eprintln!("  [ERR] Frame {}: {e}", frame_idx))
                .ok()?;

            let mut rgba = img.into_rgba8().into_raw();

            if alpha_only {
                // Extract alpha, pad, compress BC4
                let alpha: Vec<u8> = rgba.iter().skip(3).step_by(4).copied().collect();
                let (_pw, _ph, padded) = pad_to_mult4(orig_w, orig_h, &alpha, 1);

                let surface = RSurface {
                    data: &padded,
                    width: padded_w,
                    height: padded_h,
                    stride: padded_w, // 1 bpp, tightly packed
                };
                let mut compressed = vec![0u8; frame_compressed_bytes];
                intel_tex_2::bc4::compress_blocks_into(&surface, &mut compressed);
                Some(compressed)
            } else {
                // Premultiply alpha, pad, compress BC7
                premultiply_alpha(&mut rgba);
                let (_pw, _ph, padded) = pad_to_mult4(orig_w, orig_h, &rgba, 4);

                let surface = intel_tex_2::RgbaSurface {
                    data: &padded,
                    width: padded_w,
                    height: padded_h,
                    stride: padded_w * 4, // 4 bpp, tightly packed
                };
                let settings = intel_tex_2::bc7::alpha_slow_settings();
                let mut compressed = vec![0u8; frame_compressed_bytes];
                intel_tex_2::bc7::compress_blocks_into(&settings, &surface, &mut compressed);
                Some(compressed)
            }
        })
        .collect();

    // Check all frames succeeded
    let mut all_compressed: Vec<Vec<u8>> = Vec::with_capacity(sub.frame_count as usize);
    for (i, frame) in encoded_frames.into_iter().enumerate() {
        match frame {
            Some(data) => all_compressed.push(data),
            None => {
                eprintln!("  [ERR] Failed to encode frame {i}, skipping sublayer");
                return None;
            }
        }
    }

    // Write .bcf file
    let total_data_bytes: usize = all_compressed.iter().map(|f| f.len()).sum();
    let file_size = 20 + total_data_bytes;
    let mut bcf_data = Vec::with_capacity(file_size);

    // 20-byte header
    bcf_data.extend_from_slice(BCF_MAGIC);                     // 0..4: magic
    bcf_data.push(if alpha_only { 4 } else { 7 });             // 4: format (4=BC4, 7=BC7)
    bcf_data.extend_from_slice(&[0, 0, 0]);                     // 5..8: padding
    bcf_data.extend_from_slice(&padded_w.to_le_bytes());        // 8..12: width
    bcf_data.extend_from_slice(&padded_h.to_le_bytes());        // 12..16: height
    bcf_data.extend_from_slice(&sub.frame_count.to_le_bytes()); // 16..20: frame_count

    // Frame data
    for frame in &all_compressed {
        bcf_data.extend_from_slice(frame);
    }

    fs::write(&sub.bcf_abs_path, &bcf_data)
        .map_err(|e| eprintln!("  [ERR] Failed to write BCF: {e}"))
        .ok()?;

    let size_mb = file_size as f64 / (1024.0 * 1024.0);
    println!(
        "    -> {} ({:.1} MB, {:.1}s)",
        sub.bcf_rel_path,
        size_mb,
        t_start.elapsed().as_secs_f32()
    );

    Some(BcMetadataEntry {
        anim_rel_path: sub.anim_rel_path.clone(),
        sub_name: sub.sub_name.clone(),
        bc_file: sub.bcf_rel_path.clone(),
        alpha_only,
        frame_count: sub.frame_count,
        width: padded_w,
        height: padded_h,
    })
}

fn main() {
    let t_total = Instant::now();
    let assets_dir = Path::new(ASSETS_DIR);

    println!("Scanning for animation sublayers in {}...", ASSETS_DIR);
    let sublayers = find_animation_sublayers(assets_dir);
    println!("Found {} animation sublayers\n", sublayers.len());

    let mut metadata = BcMetadata {
        processed_at: String::new(), // filled below
        frames: Vec::new(),
    };

    for sub in &sublayers {
        println!("Processing: {}/{}", sub.anim_rel_path, sub.sub_name);
        if let Some(entry) = process_sublayer(sub) {
            metadata.frames.push(entry);
        }
    }

    // Write bc-metadata.json
    metadata.processed_at = chrono_free_timestamp();
    let metadata_path = assets_dir.join("bc-metadata.json");
    let json = serde_json::to_string_pretty(&metadata).expect("Failed to serialize metadata");
    fs::write(&metadata_path, &json).expect("Failed to write bc-metadata.json");

    println!("\nDone! {} BCF files generated in {:.1}s", metadata.frames.len(), t_total.elapsed().as_secs_f32());
    println!("Metadata: {}", metadata_path.display());

    // Summary
    let total_size: u64 = metadata.frames.iter().map(|f| {
        let block_size: u64 = if f.alpha_only { 8 } else { 16 };
        let blocks = (f.width as u64 / 4) * (f.height as u64 / 4);
        20 + blocks * block_size * f.frame_count as u64
    }).sum();
    println!("Total BCF size: {:.2} GB", total_size as f64 / (1024.0 * 1024.0 * 1024.0));
}

/// Simple timestamp without chrono dependency.
fn chrono_free_timestamp() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    format!("unix:{}", duration.as_secs())
}
