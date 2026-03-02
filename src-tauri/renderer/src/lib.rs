pub mod animation;
pub mod color;
pub mod layer_stack;
pub mod manifest;
pub mod png_loader;
pub mod svg_loader;
pub mod wgpu_state;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;

use rayon::prelude::*;

use animation::{AnimSubLayer, AnimationState, FrameStorage};
use layer_stack::{LayerStack, TextureLayer, CANVAS_HEIGHT, CANVAS_WIDTH};
use manifest::Manifest;
use wgpu_state::{DrawCommand, TextureUploader, WgpuState};

/// Raw window/display handles for cross-crate surface creation.
/// Avoids the main crate needing wgpu as a dependency.
#[derive(Copy, Clone)]
pub struct RawWindowHandles {
    pub window: raw_window_handle::RawWindowHandle,
    pub display: raw_window_handle::RawDisplayHandle,
}

// Safety: handles are Copy enums of raw pointers; the window (owned by Tauri)
// is guaranteed to outlive the renderer.
unsafe impl Send for RawWindowHandles {}
unsafe impl Sync for RawWindowHandles {}

const ANIMATION_FPS: f32 = 24.0;

/// Dedicated rayon thread pool for animation decoding (PNG fallback only).
/// Uses 75% of CPU cores.
static ANIM_POOL: LazyLock<rayon::ThreadPool> = LazyLock::new(|| {
    let threads = (std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        * 3
        / 4)
    .max(2);
    println!("[renderer] Animation thread pool: {threads} threads");
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap()
});

/// Parse a hex color string into [r, g, b] floats in 0.0..1.0.
fn parse_tint_rgb(hex: &str) -> [f32; 3] {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

/// Full state key for dirty checking a layer.
#[derive(Clone, PartialEq)]
struct LayerStateKey {
    asset_id: String,
    colors: HashMap<String, String>,
    offset: (i32, i32),
}

impl LayerStateKey {
    fn new(asset_id: &str, colors: &HashMap<String, String>, offset: (f32, f32)) -> Self {
        Self {
            asset_id: asset_id.to_string(),
            colors: colors.clone(),
            offset: ((offset.0 * 100.0) as i32, (offset.1 * 100.0) as i32),
        }
    }
}

/// A completed static layer upload (SVG, PNG, PNG-stack) ready to install.
struct CompletedLayerUpload {
    layer_num: u8,
    asset_id: String,
    texture_layers: Vec<TextureLayer>,
}

// TextureLayer contains wgpu types that are Send+Sync
unsafe impl Send for CompletedLayerUpload {}

/// Decoded animation sublayer data ready to install (no GPU resources yet).
struct AnimSubLayerData {
    storage: FrameStorage,
    offset: (f32, f32),
    size: (u32, u32),
    alpha_only: bool,
    tint_rgb: [f32; 3],
}

/// A completed animation load ready to install.
struct CompletedAnimUpload {
    layer_num: u8,
    asset_id: String,
    sub_layers: Vec<AnimSubLayerData>,
}

// FrameStorage contains Vec<u8> or Vec<Vec<u8>>, both are Send+Sync
unsafe impl Send for CompletedAnimUpload {}

/// The main avatar renderer — owns GPU state, layer stack, and animations.
pub struct AvatarRenderer<'win> {
    pub gpu: WgpuState<'win>,
    uploader: TextureUploader,
    layers: Mutex<LayerStack>,
    animations: Mutex<HashMap<u8, AnimationState>>,
    manifest: Arc<Manifest>,
    assets_dir: Arc<PathBuf>,
    loaded_states: Mutex<HashMap<u8, LayerStateKey>>,
    dirty: AtomicBool,
    completed_layers: Arc<Mutex<Vec<CompletedLayerUpload>>>,
    completed_anims: Arc<Mutex<Vec<CompletedAnimUpload>>>,
}

impl AvatarRenderer<'static> {
    pub async fn new(
        handles: RawWindowHandles,
        width: u32,
        height: u32,
        manifest_path: &Path,
        assets_dir: PathBuf,
    ) -> Self {
        let instance = wgpu::Instance::default();
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_window_handle: handles.window,
                raw_display_handle: handles.display,
            })
        }
        .expect("Failed to create wgpu surface");
        let gpu = WgpuState::new(instance, surface, width, height).await;
        let uploader = gpu.create_uploader();
        let manifest =
            Manifest::load(manifest_path).expect("Failed to load manifest.json");

        println!("[renderer] ===== Renderer initialized (non-blocking IPC) =====");

        Self {
            gpu,
            uploader,
            layers: Mutex::new(LayerStack::new()),
            animations: Mutex::new(HashMap::new()),
            manifest: Arc::new(manifest),
            assets_dir: Arc::new(assets_dir),
            loaded_states: Mutex::new(HashMap::new()),
            dirty: AtomicBool::new(true),
            completed_layers: Arc::new(Mutex::new(Vec::new())),
            completed_anims: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Load an asset into a layer. Returns immediately; heavy work happens in background.
    pub fn load_layer(
        &self,
        layer_num: u8,
        asset_id: &str,
        colors: &HashMap<String, String>,
        offset: (f32, f32),
    ) {
        let t_start = Instant::now();

        // Dirty checking: skip if identical to what's already loaded
        let new_key = LayerStateKey::new(asset_id, colors, offset);
        {
            let loaded = self.loaded_states.lock().unwrap();
            if let Some(existing) = loaded.get(&layer_num) {
                if *existing == new_key {
                    return;
                }
            }
        }

        let asset = match self.manifest.get_asset(layer_num, asset_id) {
            Some(a) => a.clone(),
            None => {
                eprintln!("[renderer] Asset not found: layer={layer_num}, id={asset_id}");
                return;
            }
        };

        // Record this as the desired state immediately (prevents duplicate loads)
        self.loaded_states
            .lock()
            .unwrap()
            .insert(layer_num, new_key);

        // Clear any existing animation for this layer
        self.animations.lock().unwrap().remove(&layer_num);
        self.completed_anims
            .lock()
            .unwrap()
            .retain(|d| d.layer_num != layer_num);
        self.completed_layers
            .lock()
            .unwrap()
            .retain(|d| d.layer_num != layer_num);

        let uploader = self.uploader.clone();
        let assets_dir = Arc::clone(&self.assets_dir);
        let completed_layers = Arc::clone(&self.completed_layers);
        let colors = colors.clone();
        let asset_id_owned = asset_id.to_string();

        let asset_type = asset.asset_type.clone();
        match asset_type.as_str() {
            "svg" | "png-stack" | "png" => {
                let asset_id_for_log = asset_id_owned.clone();
                std::thread::spawn(move || {
                    let t_bg = Instant::now();
                    let mut texture_layers = Vec::new();

                    match asset.asset_type.as_str() {
                        "svg" => {
                            let svg_path = assets_dir.join(&asset.path);
                            let base_color = colors.get("base").map(|s| s.as_str());
                            match svg_loader::load_and_rasterize_svg(
                                &svg_path,
                                base_color,
                                CANVAS_WIDTH,
                                CANVAS_HEIGHT,
                            ) {
                                Ok((w, h, rgba)) => {
                                    let texture = uploader.upload_texture(w, h, &rgba);
                                    let bind_group =
                                        uploader.create_texture_bind_group(&texture);
                                    texture_layers.push(TextureLayer {
                                        texture,
                                        bind_group,
                                        offset,
                                        size: (CANVAS_WIDTH, CANVAS_HEIGHT),
                                    });
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[renderer] SVG load error layer={} id={}: {e}",
                                        layer_num, asset_id_owned
                                    );
                                    return;
                                }
                            }
                        }
                        "png-stack" => {
                            let count = asset.sub_layer_count.unwrap_or(0);
                            let base_path = assets_dir.join(&asset.path);
                            let indices: Vec<u32> = (1..=count).collect();

                            texture_layers = indices
                                .par_iter()
                                .map(|&i| {
                                    let png_path = base_path.join(format!("{i}.png"));
                                    let tint = colors
                                        .get(&(i - 1).to_string())
                                        .map(|s| s.as_str());
                                    match png_loader::load_png(&png_path, tint) {
                                        Ok((w, h, rgba)) => {
                                            let texture =
                                                uploader.upload_texture(w, h, &rgba);
                                            let bind_group = uploader
                                                .create_texture_bind_group(&texture);
                                            Some(TextureLayer {
                                                texture,
                                                bind_group,
                                                offset,
                                                size: (w, h),
                                            })
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "[renderer] PNG-stack load error layer={} sub={}: {e}",
                                                layer_num, i
                                            );
                                            None
                                        }
                                    }
                                })
                                .collect::<Vec<_>>()
                                .into_iter()
                                .flatten()
                                .collect();
                        }
                        "png" => {
                            let png_path = assets_dir.join(&asset.path);
                            match png_loader::load_png(&png_path, None) {
                                Ok((w, h, rgba)) => {
                                    let texture = uploader.upload_texture(w, h, &rgba);
                                    let bind_group =
                                        uploader.create_texture_bind_group(&texture);
                                    texture_layers.push(TextureLayer {
                                        texture,
                                        bind_group,
                                        offset,
                                        size: (w, h),
                                    });
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[renderer] PNG load error layer={}: {e}",
                                        layer_num
                                    );
                                    return;
                                }
                            }
                        }
                        _ => unreachable!(),
                    }

                    if !texture_layers.is_empty() {
                        println!(
                            "[TIMING] bg load layer={} id={} type={}: {:?}",
                            layer_num, asset_id_owned, asset.asset_type, t_bg.elapsed()
                        );
                        completed_layers.lock().unwrap().push(CompletedLayerUpload {
                            layer_num,
                            asset_id: asset_id_owned,
                            texture_layers,
                        });
                    }
                });

                println!(
                    "[TIMING] load_layer QUEUED layer={} id={}: {:?}",
                    layer_num, asset_id_for_log, t_start.elapsed()
                );
            }
            "png-animation" => {
                self.queue_animation(layer_num, &asset, &colors, offset);
                println!(
                    "[TIMING] load_layer QUEUED anim layer={} id={}: {:?}",
                    layer_num, asset_id_owned, t_start.elapsed()
                );
            }
            other => {
                eprintln!("[renderer] Unknown asset type: {other}");
            }
        }
    }

    /// Queue an animation for background loading.
    /// Uses pre-baked blob files when available (instant mmap), falls back to PNG decode.
    fn queue_animation(
        &self,
        layer_num: u8,
        asset: &manifest::AssetDef,
        colors: &HashMap<String, String>,
        offset: (f32, f32),
    ) {
        let sub_layers_def = match &asset.sub_layers {
            Some(subs) => subs,
            None => return,
        };

        let base_path = self.assets_dir.join(&asset.path);
        let assets_dir = Arc::clone(&self.assets_dir);

        struct SubInfo {
            png_path: PathBuf,
            prefix: String,
            frame_count: u32,
            tint: Option<String>,
            frame_offset: (f32, f32),
            frame_size: (u32, u32),
            /// Pre-baked BC-compressed file path (absolute)
            bc_path: Option<PathBuf>,
            /// From manifest metadata
            alpha_only: bool,
        }
        let mut sub_infos = Vec::new();

        for (i, sub) in sub_layers_def.iter().enumerate() {
            let use_optimized = sub.bounds.is_some();
            let png_path = if use_optimized {
                base_path.join(&sub.name).join("optimized")
            } else {
                base_path.join(&sub.name)
            };

            let tint = colors.get(&i.to_string()).cloned();

            let (frame_offset, frame_size) = if let Some(bounds) = &sub.bounds {
                (
                    (
                        offset.0 + bounds.offset_x as f32,
                        offset.1 + bounds.offset_y as f32,
                    ),
                    (bounds.width, bounds.height),
                )
            } else if let (Some(bw), Some(bh)) = (sub.bc_width, sub.bc_height) {
                // Use BC file dimensions (padded to multiple of 4)
                (offset, (bw, bh))
            } else {
                (offset, (CANVAS_WIDTH, CANVAS_HEIGHT))
            };

            // Check for pre-baked BC-compressed file
            let bc_path = sub.bc_file.as_ref().map(|bf| assets_dir.join(bf));
            let alpha_only = sub.alpha_only.unwrap_or(false);

            sub_infos.push(SubInfo {
                png_path,
                prefix: sub.prefix.clone(),
                frame_count: sub.frame_count,
                tint,
                frame_offset,
                frame_size,
                bc_path,
                alpha_only,
            });
        }

        let completed_anims = Arc::clone(&self.completed_anims);
        let asset_id = asset.id.clone();

        std::thread::spawn(move || {
            let t_thread = Instant::now();
            let mut all_subs: Vec<AnimSubLayerData> = Vec::new();

            for (sub_idx, sub_info) in sub_infos.iter().enumerate() {
                // Tint applies to all sublayers via shader
                let tint_rgb = sub_info
                    .tint
                    .as_deref()
                    .map(parse_tint_rgb)
                    .unwrap_or([1.0, 1.0, 1.0]);

                // --- Try BC-compressed file first ---
                if let Some(bc_path) = &sub_info.bc_path {
                    if bc_path.exists() {
                        match std::fs::read(bc_path) {
                            Ok(file_data) => {
                                if file_data.len() < 20 {
                                    eprintln!(
                                        "[renderer] BCF file too small for sub {sub_idx}. Falling back to PNG."
                                    );
                                } else if &file_data[0..4] != b"BCF\0" {
                                    eprintln!(
                                        "[renderer] Invalid BCF magic for sub {sub_idx}. Falling back to PNG."
                                    );
                                } else {
                                    let format = file_data[4];
                                    let bc_w = u32::from_le_bytes([
                                        file_data[8], file_data[9], file_data[10], file_data[11],
                                    ]);
                                    let bc_h = u32::from_le_bytes([
                                        file_data[12], file_data[13], file_data[14], file_data[15],
                                    ]);
                                    let frame_count = u32::from_le_bytes([
                                        file_data[16], file_data[17], file_data[18], file_data[19],
                                    ]) as usize;

                                    let block_size: usize = if format == 4 { 8 } else { 16 };
                                    let frame_bytes = (bc_w as usize / 4) * (bc_h as usize / 4) * block_size;
                                    let expected_data = frame_count * frame_bytes;

                                    if file_data.len() - 20 >= expected_data {
                                        let label = if format == 4 { "BC4" } else { "BC7" };
                                        let total_mb = file_data.len() as f64 / (1024.0 * 1024.0);
                                        println!(
                                            "[renderer] Sub {} ({} frames, {label}, bcf): {:.1}MB loaded",
                                            sub_idx, frame_count, total_mb,
                                        );

                                        // Strip header, keep only frame data
                                        let data = file_data[20..20 + expected_data].to_vec();

                                        all_subs.push(AnimSubLayerData {
                                            storage: FrameStorage::Compressed {
                                                data,
                                                frame_bytes,
                                                frame_count,
                                            },
                                            offset: sub_info.frame_offset,
                                            size: (bc_w, bc_h),
                                            alpha_only: sub_info.alpha_only,
                                            tint_rgb,
                                        });
                                        continue;
                                    } else {
                                        eprintln!(
                                            "[renderer] BCF size mismatch for sub {}: expected {expected_data} data bytes, got {}. Falling back to PNG.",
                                            sub_idx, file_data.len() - 20
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "[renderer] Failed to read BCF for sub {sub_idx}: {e}. Falling back to PNG."
                                );
                            }
                        }
                    }
                }

                // --- PNG fallback path ---
                // Detect alpha-only from first frame
                let first_path = sub_info
                    .png_path
                    .join(format!("{}_{:05}.png", sub_info.prefix, 0));

                let (is_alpha_only, frame_0_pixels) =
                    match png_loader::load_png_raw(&first_path) {
                        Ok((_w, _h, raw_rgba)) => {
                            let is_alpha = png_loader::is_white_only(&raw_rgba);
                            if is_alpha {
                                (true, png_loader::extract_alpha(&raw_rgba))
                            } else {
                                match png_loader::load_png(
                                    &first_path,
                                    sub_info.tint.as_deref(),
                                ) {
                                    Ok((_w, _h, rgba)) => (false, rgba),
                                    Err(e) => {
                                        eprintln!(
                                            "[renderer] Anim first frame err: {e}"
                                        );
                                        continue;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[renderer] Anim first frame load err: {e}"
                            );
                            continue;
                        }
                    };

                // Decode remaining frames in parallel
                let remaining: Vec<u32> = (1..sub_info.frame_count).collect();
                let prefix = &sub_info.prefix;
                let png_path = &sub_info.png_path;
                let tint_ref = &sub_info.tint;

                let decoded: Vec<Vec<u8>> = ANIM_POOL.install(|| {
                    remaining
                        .par_iter()
                        .filter_map(|&frame_idx| {
                            let frame_path = png_path.join(format!(
                                "{}_{:05}.png",
                                prefix, frame_idx
                            ));

                            if is_alpha_only {
                                png_loader::load_png_raw(&frame_path)
                                    .ok()
                                    .map(|(_, _, raw)| {
                                        png_loader::extract_alpha(&raw)
                                    })
                            } else {
                                png_loader::load_png(
                                    &frame_path,
                                    tint_ref.as_deref(),
                                )
                                .ok()
                                .map(|(_, _, rgba)| rgba)
                            }
                        })
                        .collect()
                });

                let mut frames: Vec<Vec<u8>> =
                    Vec::with_capacity(sub_info.frame_count as usize);
                frames.push(frame_0_pixels);
                frames.extend(decoded);

                let label = if is_alpha_only { "alpha-only" } else { "rgba" };
                let bpp: usize = if is_alpha_only { 1 } else { 4 };
                let frame_bytes = sub_info.frame_size.0 as usize
                    * sub_info.frame_size.1 as usize
                    * bpp;
                let total_mb =
                    (frames.len() * frame_bytes) as f64 / (1024.0 * 1024.0);
                println!(
                    "[renderer] Sub {} ({} frames, {label}, png-fallback): {:.1}MB RAM",
                    sub_idx,
                    frames.len(),
                    total_mb
                );

                all_subs.push(AnimSubLayerData {
                    storage: FrameStorage::Frames(frames),
                    offset: sub_info.frame_offset,
                    size: sub_info.frame_size,
                    alpha_only: is_alpha_only,
                    tint_rgb,
                });
            }

            if !all_subs.is_empty() {
                let total: usize =
                    all_subs.iter().map(|s| s.storage.frame_count()).sum();
                println!(
                    "[TIMING] anim bg thread layer={}: {} frames in {:?}",
                    layer_num, total, t_thread.elapsed()
                );
                completed_anims.lock().unwrap().push(CompletedAnimUpload {
                    layer_num,
                    asset_id,
                    sub_layers: all_subs,
                });
            }
        });
    }

    /// Install completed uploads from background threads. Called every frame.
    fn install_completed_work(&self) {
        // Install static layers
        let layers: Vec<CompletedLayerUpload> = {
            let mut queue = self.completed_layers.lock().unwrap();
            if queue.is_empty() {
                // skip
            }
            std::mem::take(&mut *queue)
        };

        for upload in layers {
            let wanted = {
                let loaded = self.loaded_states.lock().unwrap();
                loaded
                    .get(&upload.layer_num)
                    .map(|s| s.asset_id == upload.asset_id)
                    .unwrap_or(false)
            };
            if !wanted {
                println!(
                    "[renderer] Discarding stale layer upload: layer={} id={}",
                    upload.layer_num, upload.asset_id
                );
                continue;
            }

            self.layers
                .lock()
                .unwrap()
                .set_layer(upload.layer_num, upload.texture_layers);
            self.dirty.store(true, Ordering::Relaxed);
            println!(
                "[renderer] Installed layer {} ({})",
                upload.layer_num, upload.asset_id
            );
        }

        // Install animations — create GPU textures from frame data
        let anims: Vec<CompletedAnimUpload> = {
            let mut queue = self.completed_anims.lock().unwrap();
            if queue.is_empty() {
                return;
            }
            std::mem::take(&mut *queue)
        };

        for upload in anims {
            let wanted = {
                let loaded = self.loaded_states.lock().unwrap();
                loaded
                    .get(&upload.layer_num)
                    .map(|s| s.asset_id == upload.asset_id)
                    .unwrap_or(false)
            };
            if !wanted {
                continue;
            }

            let mut anim_sub_layers: Vec<AnimSubLayer> = Vec::new();

            for sub_data in upload.sub_layers {
                if sub_data.storage.frame_count() == 0 {
                    continue;
                }

                let frame_0 = sub_data.storage.frame_data(0);

                // Create ONE persistent GPU texture per sublayer and upload frame 0
                let (gpu_texture, bind_group) = match (&sub_data.storage, sub_data.alpha_only) {
                    (FrameStorage::Compressed { .. }, true) => {
                        // BC4: GPU-compressed alpha-only
                        let tex = self.uploader.upload_bc4_texture(
                            sub_data.size.0,
                            sub_data.size.1,
                            frame_0,
                        );
                        let bg = self.uploader.create_texture_bind_group(&tex);
                        (tex, bg)
                    }
                    (FrameStorage::Compressed { .. }, false) => {
                        // BC7: GPU-compressed RGBA (pre-baked premultiplied)
                        let tex = self.uploader.upload_bc7_texture(
                            sub_data.size.0,
                            sub_data.size.1,
                            frame_0,
                        );
                        let bg = self.uploader.create_texture_bind_group(&tex);
                        (tex, bg)
                    }
                    (FrameStorage::Frames(_), true) => {
                        // PNG fallback alpha-only: R8Unorm
                        let tex = self.uploader.upload_alpha_texture(
                            sub_data.size.0,
                            sub_data.size.1,
                            frame_0,
                        );
                        let bg = self.uploader.create_texture_bind_group(&tex);
                        (tex, bg)
                    }
                    (FrameStorage::Frames(_), false) => {
                        // PNG fallback RGBA: already tinted + premultiplied
                        let tex = self.uploader.upload_texture(
                            sub_data.size.0,
                            sub_data.size.1,
                            frame_0,
                        );
                        let bg = self.uploader.create_texture_bind_group(&tex);
                        (tex, bg)
                    }
                };

                anim_sub_layers.push(AnimSubLayer {
                    storage: sub_data.storage,
                    offset: sub_data.offset,
                    size: sub_data.size,
                    gpu_texture,
                    bind_group,
                    uploaded_frame_idx: Some(0),
                    alpha_only: sub_data.alpha_only,
                    tint_rgb: sub_data.tint_rgb,
                });
            }

            if !anim_sub_layers.is_empty() {
                let total: usize = anim_sub_layers
                    .iter()
                    .map(|s| s.storage.frame_count())
                    .sum();
                let gpu_textures = anim_sub_layers.len();
                self.layers.lock().unwrap().clear_layer(upload.layer_num);
                let anim = AnimationState::new(anim_sub_layers, ANIMATION_FPS);
                self.animations
                    .lock()
                    .unwrap()
                    .insert(upload.layer_num, anim);
                self.dirty.store(true, Ordering::Relaxed);
                println!(
                    "[renderer] Installed anim layer {} ({}) with {total} frames, {gpu_textures} GPU textures",
                    upload.layer_num, upload.asset_id
                );
            }
        }
    }

    /// Clear a layer (both static and animated).
    pub fn clear_layer(&self, layer_num: u8) {
        self.layers.lock().unwrap().clear_layer(layer_num);
        self.animations.lock().unwrap().remove(&layer_num);
        self.loaded_states.lock().unwrap().remove(&layer_num);
        self.completed_anims
            .lock()
            .unwrap()
            .retain(|d| d.layer_num != layer_num);
        self.completed_layers
            .lock()
            .unwrap()
            .retain(|d| d.layer_num != layer_num);
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Render all layers to the surface. Called every frame from the render thread.
    pub fn render_frame(&self) {
        // Apply any pending resize from the main thread
        if self.gpu.apply_pending_resize() {
            self.dirty.store(true, Ordering::Relaxed);
        }

        // Install any completed background work
        self.install_completed_work();

        // Advance animations and upload changed frames
        let mut any_anim_ticked = false;
        {
            let mut anims = self.animations.lock().unwrap();
            for anim in anims.values_mut() {
                if anim.tick() {
                    any_anim_ticked = true;
                }

                let current_frame = anim.current_frame;
                for sub in &mut anim.sub_layers {
                    let idx = current_frame
                        .min(sub.storage.frame_count().saturating_sub(1));
                    if sub.uploaded_frame_idx != Some(idx) {
                        let data = sub.storage.frame_data(idx);
                        match (&sub.storage, sub.alpha_only) {
                            (FrameStorage::Compressed { .. }, true) => {
                                self.uploader.write_bc4(
                                    &sub.gpu_texture,
                                    sub.size.0,
                                    sub.size.1,
                                    data,
                                );
                            }
                            (FrameStorage::Compressed { .. }, false) => {
                                self.uploader.write_bc7(
                                    &sub.gpu_texture,
                                    sub.size.0,
                                    sub.size.1,
                                    data,
                                );
                            }
                            (FrameStorage::Frames(_), true) => {
                                self.uploader.write_alpha(
                                    &sub.gpu_texture,
                                    sub.size.0,
                                    sub.size.1,
                                    data,
                                );
                            }
                            (FrameStorage::Frames(_), false) => {
                                self.uploader.write_rgba(
                                    &sub.gpu_texture,
                                    sub.size.0,
                                    sub.size.1,
                                    data,
                                );
                            }
                        }
                        sub.uploaded_frame_idx = Some(idx);
                    }
                }
            }
        }

        // Only re-render if something changed
        let was_dirty = self.dirty.swap(false, Ordering::Relaxed);
        if !was_dirty && !any_anim_ticked {
            return;
        }

        // Build draw list
        let layers = self.layers.lock().unwrap();
        let anims = self.animations.lock().unwrap();

        let mut active_layers: Vec<u8> = Vec::new();
        for &num in layers.layer_nums() {
            active_layers.push(num);
        }
        for &num in anims.keys() {
            if !active_layers.contains(&num) {
                active_layers.push(num);
            }
        }
        active_layers.sort_by(|a, b| b.cmp(a));

        let mut draw_commands: Vec<DrawCommand> = Vec::new();

        for layer_num in active_layers {
            if let Some(anim) = anims.get(&layer_num) {
                for sub in &anim.sub_layers {
                    let tint = if sub.alpha_only {
                        // Alpha-only: tint.a > 0.5 triggers alpha-only shader path
                        [sub.tint_rgb[0], sub.tint_rgb[1], sub.tint_rgb[2], 1.0]
                    } else {
                        // RGBA: tint.a = 0.0, shader multiplies RGB by tint
                        [sub.tint_rgb[0], sub.tint_rgb[1], sub.tint_rgb[2], 0.0]
                    };
                    draw_commands.push(DrawCommand {
                        bind_group: &sub.bind_group,
                        offset: sub.offset,
                        size: sub.size,
                        tint,
                    });
                }
            } else if let Some(tex_layers) = layers.get_layer(layer_num) {
                for tex_layer in tex_layers {
                    // Static layers: white tint (no color change)
                    draw_commands.push(DrawCommand {
                        bind_group: &tex_layer.bind_group,
                        offset: tex_layer.offset,
                        size: tex_layer.size,
                        tint: [1.0, 1.0, 1.0, 0.0],
                    });
                }
            }
        }

        self.gpu.render_draw_list(&draw_commands);
    }

    /// Handle window resize.
    pub fn resize(&self, width: u32, height: u32) {
        self.gpu.resize(width, height);
        self.dirty.store(true, Ordering::Relaxed);
    }
}
