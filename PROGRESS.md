# Miniko - Project Progress

## Tech Stack
- **Desktop:** Tauri v2 (Rust backend)
- **Frontend:** React 19 + TypeScript + Vite 7
- **Rendering:** Rust wgpu (native Vulkan/DX12/Metal, 3000x2320 canvas) — replaced PixiJS v8
- **State:** Zustand v5
- **Styling:** TailwindCSS v4

---

## Phase 1: Project Scaffolding & App Shell — COMPLETE

**What was built:**
- Tauri + React + TypeScript + Vite project scaffold
- Core dependencies installed (pixi.js, zustand, tailwindcss)
- `tauri.conf.json` configured (Miniko, `com.miniko.app`, 1024x768 window, 800x600 min)
- Full directory structure: `src/screens/`, `src/components/`, `src/engine/`, `src/store/`, `src/utils/`
- Skeleton screens: HomeScreen, WardrobeScreen, OverlayScreen (stub)
- Zustand store with full `MinikoState` interface
- Screen navigation (Home <-> Wardrobe)
- Color utility functions (`hexToHSL`, `hslToHex`, `computeVariants`)
- Layer definitions (all 41 layers) and category mappings (12 user-facing categories)
- Warm Miniko theme CSS variables
- Builds and runs on Windows

**Key files:**
- `src/store/minikoStore.ts` — Central state (equipped assets, colors, expressions, outfits, settings)
- `src/utils/assetUtils.ts` — LAYER_DEFS (41 layers), CATEGORIES (12 categories)
- `src/utils/colorUtils.ts` — HSL conversion and shadow/highlight computation
- `src-tauri/tauri.conf.json` — App config
- `src/App.tsx` — Screen router

---

## Phase 2: PixiJS Rendering Engine — COMPLETE

**What was built:**
- `AvatarCanvas.tsx` — PixiJS Application wrapper, auto-scales to container, transparent background
- `LayerManager.ts` — 41-layer container stack (layer 41 = backmost, layer 1 = frontmost)
- `SVGRecolorer.ts` — Sentinel color replacement (red→base, lime→shadow, blue→shadow2) + SVG→data URL conversion
- `PNGStackLoader.ts` — Loads numbered PNG sub-layers with per-layer tinting
- `PNGAnimator.ts` — Frame sequencer with preloading, configurable FPS, loop support
- `manifest.ts` — Types and loader for asset manifest
- `scripts/generate-manifest.cjs` — Scans asset folder, produces `public/manifest.json`
- Asset folder junction: `public/assets` → `~/Desktop/mini-ko assets`

**Asset manifest stats:** 483 assets across 41 layers (415 SVG, 42 PNG-stack, 23 PNG-animation)

**Lessons learned:**
- PixiJS v8 uses `Assets.load()` not `Texture.from()` for async texture loading
- SVGs must be loaded via data URLs (not blob URLs) — PixiJS needs MIME type detection
- Asset paths with spaces need `encodeURIComponent()` per path segment
- React useEffect race condition: async PixiJS init needs a `ready` state to trigger dependent effects

**Current default avatar:** Body, ears, nose, mouth, sclera, eye shapes, irises, eyebrows, front/back hair, bangs, outfit — all rendering with recolored SVGs

---

## Phase 2.5: Home Screen UI — COMPLETE

**What was built:**
- Full HomeScreen layout matching design mockup with warm Miniko theme
- Icon assets copied from miniko2 into `public/icons/` (wardrobe, hotkey, mic, volume, expression state icons)
- Reusable UI components:
  - `IconButton.tsx` — Circular icon button with configurable color/size
  - `StyledSelect.tsx` — Warm-themed dropdown select with chevron
  - `VerticalSlider.tsx` — Custom vertical range slider (rotated native input + painted track)
  - `ExpressionSlots.tsx` — 4 stacked expression slot cards with active highlighting
  - `MicDeviceSelector.tsx` — Floating popover for mic device selection (mock devices)
  - `OutfitBar.tsx` — Edit/add/delete buttons + outfit dropdown
- Layout: top bar (wardrobe, screenshot, effect dropdowns, hotkey) | left sliders (threshold + hold time, side by side) | center canvas | right expression slots | bottom bar (mic + volume with floating popups, outfit controls)
- Store additions: setters for activeExpressionSlot, volumeThreshold, holdTime, stationaryEffect, talkingEffect, currentOutfit, selectedMicId + new fields appVolume, holdTime, micEnabled
- CSS: button circle colors, expression slot active color, vertical/horizontal slider styling

**Key files:**
- `src/screens/HomeScreen.tsx` — Full home screen layout
- `src/components/IconButton.tsx` — Reusable circular button
- `src/components/StyledSelect.tsx` — Themed dropdown
- `src/components/VerticalSlider.tsx` — Vertical range slider
- `src/components/ExpressionSlots.tsx` — Expression slot cards
- `src/components/MicDeviceSelector.tsx` — Floating mic device picker
- `src/components/OutfitBar.tsx` — Outfit management controls
- `public/icons/` — UI icon assets

**Notes:**
- Mic device list uses mock data — real `navigator.mediaDevices.enumerateDevices()` comes in Phase 5
- Screenshot button (green, top-left) has no handler yet — functionality TBD
- Hotkey button (blue, top-right) has no handler yet
- Outfit edit/add/delete buttons are visual only — wiring comes in Phase 4
- Volume slider controls `appVolume` state but no audio playback system exists yet
- Floating popups (mic devices, volume) are absolutely positioned, dismiss each other

---

## Phase 2.6: Rendering Quality & Layout Improvements — COMPLETE

**What was built:**
- Fixed pixelated/choppy avatar rendering by implementing proper resolution and texture filtering
- Repositioned avatar canvas to fill entire window and anchor to bottom center
- Implemented UI overlay architecture with pointer-events management for transparent controls

**Rendering quality fixes:**
- `AvatarCanvas.tsx`: Added `resolution: window.devicePixelRatio` to match display pixel density (was hardcoded to 1)
- `AvatarCanvas.tsx`: Added `autoDensity: true` for automatic CSS canvas size adjustment
- `LayerManager.ts`: Set `texture.source.scaleMode = "linear"` for SVG textures (smooth bilinear filtering)
- `PNGStackLoader.ts`: Set `texture.source.scaleMode = "linear"` for PNG stack textures
- `PNGAnimator.ts`: Set `texture.source.scaleMode = "linear"` for PNG animation frames
- Moved scaling from CSS (`object-fit: contain`) to PixiJS GPU pipeline via `stage.scale` and `renderer.resize()`

**Layout improvements:**
- `HomeScreen.tsx`: Avatar canvas now positioned absolutely at `inset-0` behind all UI
- `HomeScreen.tsx`: UI controls wrapped in `pointer-events-none` overlay with `pointer-events-auto` on interactive elements
- `AvatarCanvas.tsx`: Canvas scales based on window height (fills vertically), centers horizontally via `stage.x`, anchors to bottom via `stage.y = 0`
- Added `ResizeObserver` to dynamically reposition avatar on window resize

**Key technical decisions:**
- PixiJS by default can use nearest-neighbor filtering → changed to linear for smooth texture scaling
- CSS canvas downscaling at 5:1+ ratio produces rough edges → moved scaling into WebGL for proper GPU filtering
- Avatar fills window height and centers horizontally (not width-based scaling) for PNGTuber use case

**Lessons learned:**
- PixiJS `resolution` setting affects backing store density but SVG/PNG textures must also set `scaleMode` explicitly
- CSS `object-fit` on canvas elements may not use optimal filtering algorithms
- `pointer-events-none` + selective `pointer-events-auto` enables layered UI over full-screen canvas

---

## Phase 2.7: Animation Performance & Memory Optimization — COMPLETE

**What was built:**
- Fixed WebGL context loss crashes when loading large PNG animations (105+ frames)
- Implemented sliding window texture management to prevent GPU memory exhaustion
- Fixed avatar sequential rendering issue (layers painting in one-by-one)
- Optimized thumbnail loading in wardrobe asset grid

**Avatar loading optimization:**
- `AvatarCanvas.tsx`: Changed from sequential to parallel asset loading with `Promise.all()`
- `AvatarCanvas.tsx`: Hide stage during loading (`app.stage.visible = false`), reveal when complete for instant pop-in
- Reduced load time from ~4 seconds (sequential) to <1 second (parallel)

**Thumbnail loading fixes:**
- `AssetGrid.tsx`: Removed artificial 20-asset limit, now loads all thumbnails in category
- `AssetGrid.tsx`: Changed from sequential to parallel loading with `Promise.all()`
- `AssetGrid.tsx`: Added error state tracking and red error icon for failed thumbnails
- `AssetGrid.tsx`: Added proper HTTP response checking for SVG fetches

**PNG animation memory management:**
- `PNGAnimator.ts`: Completely rewrote texture management from array to Map-based sliding window
- `PNGAnimator.ts`: Implemented 40-frame sliding window (keeps only nearby frames in memory)
- `PNGAnimator.ts`: Added active texture unloading with `texture.destroy(true)` to free GPU memory
- `PNGAnimator.ts`: Fixed wraparound handling for looping animations (circular distance calculation)
- `PNGAnimator.ts`: Switched from `setInterval` to PixiJS `Ticker.shared` for animation playback
- `PNGAnimator.ts`: Added lazy loading flag to prevent multiple simultaneous loads
- Reduced peak memory from ~8.7GB (315 frames × 28MB) to ~3.4GB (120 frames × 28MB)

**PNG animation auto-crop optimization (Phase 2.8):**
- `scripts/optimize-animations.cjs`: Auto-crops transparent borders from all PNG animation frames using sharp
- Analyzes ALL frames to compute union bounding box (not just first frame — different frames have content in different areas)
- Only targets animation folders (files matching `prefix_00000.png` pattern), skips png-stacks
- Outputs cropped frames to `optimized/` subfolders, writes `animation-bounds.json` metadata
- `scripts/generate-manifest.cjs`: Loads bounds data and merges into manifest sublayer definitions; filters `optimized/` from directory scanning
- `src/engine/manifest.ts`: Added `bounds` field to `SubLayerDef` (width, height, offsetX, offsetY)
- `src/engine/PNGAnimator.ts`: Uses bounds for sprite positioning/sizing; skips sliding window for optimized frames (small enough to preload all)
- `src/engine/LayerManager.ts`: Routes to `optimized/` subfolder when bounds exist, passes bounds to PNGAnimator
- 51 animation sublayers optimized, typical savings 80-98% memory per frame
- Example: sfx_tail_1 outline went from 3000×2320 (~28MB/frame) to 762×752 (~2.3MB/frame) — 12× reduction
- Optimized animations preload all frames (no sliding window churn on loop), non-optimized still use sliding window as fallback

**Key technical decisions:**
- Each PNG frame at 3000×2320 = ~28MB GPU memory → need aggressive memory management
- Sliding window keeps frames within ±20 of current position, destroys others
- Wraparound logic for looping: calculate circular distance to avoid unloading frames near loop boundary
- Ticker callback receives `Ticker` object (not `deltaTime` number) in PixiJS v8

**Lessons learned:**
- WebGL has hard limits on texture memory (~4-6GB typical) → can't keep 315 full-res textures loaded
- Lazy loading must prevent race conditions with `isLoadingFrames` flag
- Looping animations need circular distance math: frame 5 is only 10 frames from frame 95 (when total=105)
- PixiJS v8 Ticker signature: `(ticker: Ticker) => void`, access `ticker.deltaTime`
- Parallel Promise.all() loading dramatically improves perceived performance vs sequential await

**Files modified:**
- [src/components/AvatarCanvas.tsx](src/components/AvatarCanvas.tsx) — Parallel loading, stage visibility control
- [src/components/AssetGrid.tsx](src/components/AssetGrid.tsx) — Parallel thumbnails, error handling, no artificial limits
- [src/engine/PNGAnimator.ts](src/engine/PNGAnimator.ts) — Sliding window, texture destruction, wraparound handling, Ticker.shared
- [src/engine/LayerManager.ts](src/engine/LayerManager.ts) — Parallel layer loading
- [src/main.tsx](src/main.tsx) — Global error handlers for debugging

**Performance improvements:**
- Avatar screen switching: instant pop-in (was slow layer-by-layer)
- Large animation playback: stable with no crashes (was immediate WebGL context loss)
- Wardrobe thumbnails: all load correctly (was infinite spinners for 20+)
- Memory usage: capped at ~120 frames in memory max (was unbounded growth to 315+)

---

## Phase 3: Wardrobe UI — COMPLETE

**Goal:** Build the asset selection and color customization interface.

**What was built:**
- Asset category icon bar with 12 categories (copied 10 PNG icons, created 2 SVG placeholders for mouth/eyebrows)
- Asset selection grid (4-column scrollable layout, dynamic thumbnail loading, "None" option, selection states)
- Color palette with circular swatches and left/right navigation arrows
- Full-featured color picker popup (gradient square, hue slider, hex input, preset swatches, reset button)
- D-pad positioning control (4 arrows + center reset, 5px nudge increments)
- Expression management buttons (duplicate/reset with dropdown menu)
- Integrated wardrobe screen layout with avatar preview, expression slots, outfit controls
- Store extensions: selectedCategory, selectedAssetForColor, userOffsets setter
- LayerManager updated to apply userOffsets for asset positioning
- AvatarCanvas wired to subscribe to userOffsets for real-time position updates

**Key files:**
- [src/components/AssetCategoryBar.tsx](src/components/AssetCategoryBar.tsx) — Category icon strip (93 lines)
- [src/components/AssetGrid.tsx](src/components/AssetGrid.tsx) — Asset thumbnail grid with filtering (206 lines)
- [src/components/ColorPalette.tsx](src/components/ColorPalette.tsx) — Color swatch navigation (79 lines)
- [src/components/ColorPicker.tsx](src/components/ColorPicker.tsx) — HSL gradient picker (220 lines)
- [src/components/DPadControl.tsx](src/components/DPadControl.tsx) — 4-directional positioning (98 lines)
- [src/components/ExpressionButtons.tsx](src/components/ExpressionButtons.tsx) — Duplicate/reset buttons (83 lines)
- [src/screens/WardrobeScreen.tsx](src/screens/WardrobeScreen.tsx) — Complete wardrobe layout (132 lines)
- [src/store/minikoStore.ts](src/store/minikoStore.ts) — Extended with wardrobe state
- [src/engine/LayerManager.ts](src/engine/LayerManager.ts) — Added userOffset support

**Lessons learned:**
- Dynamic asset thumbnail loading from manifest works well for 20+ assets at once
- HSL color picker requires careful conversion between hsl object (0-100, 0-100) and hslToHex parameters (0-1, 0-1, 0-1)
- TypeScript needs explicit React.JSX.Element type for function components used as icon alternatives
- userOffset positioning works cleanly at the container level in LayerManager
- Color picker gradient uses CSS linear-gradient overlay trick for saturation/brightness square

**Tasks completed:**
- [x] Asset Category Bar — horizontal icon strip (Hair, Face, Mouth, Eyebrows, Hands, Outfit, Glasses, Earrings, Hats, Bows, Wings/Tails, SFX)
- [x] Asset Selection Grid — thumbnails with none/empty first slot, equip on click
- [x] Color Picker — HSL picker (saturation/brightness square + hue slider), hex input, preset palette
- [x] Color swatches — per-sentinel-group for SVG, per-sub-layer for PNG
- [x] D-Pad Position Control — 4 arrows + center reset, nudge asset
- [x] Wire everything to Zustand — equip/deselect triggers canvas re-render
- [x] Avatar preview in wardrobe (reuse AvatarCanvas)

**Known limitations (deferred to Phase 4):**
- Expression duplicate/reset buttons have placeholder implementations
- Color reset doesn't properly restore default colors (needs default color tracking)
- Asset grid loads only first 20 thumbnails for performance (virtualization needed for full list)
- No thumbnail caching between category switches

---

## wgpu Migration: PixiJS → Rust Native Rendering — COMPLETE

**Why:** WebGL context loss crashes persisted despite extensive optimization (sliding window, auto-crop, memory caps). Browser-imposed GPU memory limits (~4-6GB) can't be circumvented from JavaScript. Native wgpu has no such limits.

**Architecture:**
- wgpu creates a surface on the Tauri window via `instance.create_surface(window)`
- React webview sits on top with transparent background (`"transparent": true` in tauri.conf.json)
- wgpu renders the 41-layer alpha-blended avatar behind the webview
- React UI overlays on top — single window, zero IPC for frame display

**What was built:**
- `src-tauri/src/renderer/mod.rs` — AvatarRenderer: orchestration, non-blocking IPC, background asset loading
- `src-tauri/src/renderer/wgpu_state.rs` — Device, queue, surface, render pipeline, WGSL shader, TextureUploader
- `src-tauri/src/renderer/svg_loader.rs` — Sentinel color replacement via regex + resvg rasterization to RGBA
- `src-tauri/src/renderer/png_loader.rs` — PNG decode (image crate) + tint + premultiply alpha
- `src-tauri/src/renderer/animation.rs` — AnimationState: frame timing at 24fps, full preload
- `src-tauri/src/renderer/color.rs` — Port of colorUtils.ts: hex↔HSL, compute shadow/shadow2 variants
- `src-tauri/src/renderer/manifest.rs` — Rust types for manifest.json (serde deserialization)
- `src-tauri/src/renderer/layer_stack.rs` — 41 layer slots with TextureLayer (texture + bind group + offset)
- `src-tauri/src/renderer/shader.wgsl` — Textured quad vertex/fragment shader with per-instance transform
- `src/hooks/useRendererSync.ts` — Syncs Zustand store → Rust renderer via Tauri IPC with dirty checking

**Rendering pipeline:**
- Premultiplied alpha throughout: resvg outputs premultiplied, PNG loader premultiplies in tint pass
- Blend state: `PREMULTIPLIED_ALPHA_BLENDING`, `CompositeAlphaMode::Opaque`
- Non-sRGB swapchain (avoids double encoding), `Rgba8Unorm` textures
- Clear color: `#f5efe6` (warm background)

**Non-blocking asset loading:**
- `load_layer()` returns immediately (microseconds), spawns `std::thread` for heavy work
- `TextureUploader` (clone of `Arc<Device>` + `Arc<Queue>`) enables GPU uploads from any thread
- Completed uploads queued in `completed_layers` / `completed_anims`, installed by render thread

**Dedicated render thread (the key fix):**
- Tauri/tao uses `ControlFlow::Wait` → `GetMessageW()` blocks the event loop when idle
- Multiple attempts to wake the event loop from background threads all failed:
  - `PostMessageW(hwnd, WM_NULL)` — no-op message, doesn't advance tao's runner state
  - `SetTimer(hwnd, ...)` — targeted WebView2 child HWND, not tao's top-level window
  - `PostThreadMessageW(tid, WM_APP)` — thread messages with hwnd=0; DispatchMessageW ignores them
  - `GetAncestor(hwnd, GA_ROOT)` — still didn't reliably find the correct tao HWND
- **Solution:** Spawn a dedicated `std::thread` in `lib.rs` setup that calls `render_frame()` every 16ms (~60fps), completely independent of the event loop
- Window resize still handled via `RunEvent::WindowEvent::Resized` in the event loop callback

**Tauri IPC commands:**
- `update_layer(layer_num, asset_id, colors, offset)` — load/replace a layer
- `clear_layer(layer_num)` — remove a layer
- `update_all_layers(updates)` — batch update for expression switching

**Frontend changes:**
- Removed pixi.js dependency and PixiJS engine files
- `AvatarCanvas.tsx` → transparent div placeholder (wgpu renders behind it)
- Added `useRendererSync` hook: watches Zustand state, calls `invoke()` on changes
- Kept `manifest.ts` for wardrobe UI asset browsing

**Key files:**
- [src-tauri/src/renderer/](src-tauri/src/renderer/) — Full Rust rendering engine (8 modules)
- [src-tauri/src/lib.rs](src-tauri/src/lib.rs) — Tauri setup, render thread spawn, IPC commands
- [src/hooks/useRendererSync.ts](src/hooks/useRendererSync.ts) — State → Rust IPC bridge
- [src-tauri/Cargo.toml](src-tauri/Cargo.toml) — wgpu, resvg, image, etc.

---

## PNG Loading Performance Optimization — COMPLETE

**Why:** PNG-stack assets took ~1-2s to load, and PNG animations took ~5.7s for 315 frames. All decoding happened sequentially in a single thread, and the premultiply-alpha pass used slow scalar f32 math.

**Before (from timing logs):**
- `body` (png-stack): 1.97s
- `hand_right_1` (png-stack): 1.24s
- `sfx_tail_1` (animation, 315 frames): 5.69s

**What was built:**

1. **Rayon parallel decoding** — Added `rayon` crate for work-stealing parallelism
   - Animation frames decoded in parallel across all CPU cores via `par_iter()` (was sequential single-thread loop)
   - PNG-stack sub-layers decoded in parallel via `par_iter()` (was sequential loop)
   - `TextureUploader` is already thread-safe (`Arc<Device>`, `Arc<Queue>`) — GPU uploads work from rayon threads
   - `par_iter().collect()` preserves frame ordering automatically

2. **Integer premultiply-alpha** — Replaced f32 per-pixel math with integer operations
   - Untinted path: `u16` intermediates — `((pixel as u16 * alpha as u16 + 127) / 255) as u8`
   - Tinted path: `u32` intermediates — two-step multiply (tint then alpha)
   - Auto-vectorizes with LLVM SIMD (f32 `.round()` calls prevented vectorization before)

**Files modified:**
- [src-tauri/Cargo.toml](src-tauri/Cargo.toml) — Added `rayon = "1.10"`
- [src-tauri/src/renderer/png_loader.rs](src-tauri/src/renderer/png_loader.rs) — Integer premultiply-alpha
- [src-tauri/src/renderer/mod.rs](src-tauri/src/renderer/mod.rs) — Rayon `par_iter()` in `queue_animation()` and png-stack loading

**Lessons learned:**
- `(x + 127) / 255` is the integer equivalent of `(x as f32 / 255.0).round()` — avoids f32 entirely
- `u16` suffices for untinted premultiply (max 255×255 = 65025 < 65535), `u32` needed for tinted (two multiplies)
- Rayon's `par_iter().collect()` on indexed collections preserves order — critical for animation frame sequences
- `wgpu::Queue::write_texture()` is thread-safe — multiple rayon threads can upload textures concurrently

---

## Window Resize Crash Fix — COMPLETE

**Bug:** Resizing the window caused a wgpu panic ("Surface is not configured for presentation") followed by `PoisonError` cascade that killed the app.

**Root cause:** Race condition between two threads accessing the wgpu surface concurrently:
- Main thread (Tauri event loop): `WindowEvent::Resized` → `surface.configure()`
- Render thread (~60fps loop): `render_frame()` → `surface.get_current_texture()`

When these overlapped, wgpu's validation failed. The render thread panic poisoned the `animations` Mutex (held during `render_frame`), cascading into `PoisonError` on subsequent IPC calls.

**Fix: Deferred resize via atomics**
- `WgpuState::resize()` now stores dimensions in `AtomicU32` fields (lock-free, no surface access)
- New `apply_pending_resize()` method reconfigures the surface, called only from the render thread
- `render_frame()` calls `apply_pending_resize()` at the top of each frame before any surface operations
- All `surface.configure()` calls now happen exclusively on the render thread — no race condition
- Added `SurfaceError::Timeout` handling (skip frame gracefully)
- Render loop wrapped in `std::panic::catch_unwind` as defense-in-depth against future panics

**Files modified:**
- [src-tauri/src/renderer/wgpu_state.rs](src-tauri/src/renderer/wgpu_state.rs) — Atomic pending resize, deferred `surface.configure()`
- [src-tauri/src/renderer/mod.rs](src-tauri/src/renderer/mod.rs) — `apply_pending_resize()` call in `render_frame()`
- [src-tauri/src/lib.rs](src-tauri/src/lib.rs) — `catch_unwind` in render loop

**Lesson learned:** When a dedicated render thread and the main event loop both touch a GPU surface, all surface operations (configure + get_current_texture) must happen on the same thread. Use atomics to pass resize dimensions cross-thread instead of calling `surface.configure()` directly from the event handler.

---

## BC4/BC7 GPU Texture Compression — COMPLETE

**Why:** Raw blob files totaled 6.72 GB (too large for CDN), wasted VRAM (uncompressed textures), and RGBA blobs required CPU-side `tint_and_premultiply` every frame.

**What was built:**

1. **Pre-bake tool** (`tools/compress-animations/`) — Standalone Rust binary
   - Scans animation folders, detects alpha-only sublayers (white pixels only)
   - Pads frame dimensions to multiples of 4 (BC block alignment requirement)
   - Compresses to BC4 (alpha-only, 0.5 bytes/pixel) or BC7 (RGBA premultiplied, 1 byte/pixel) via `intel_tex_2`
   - Writes `.bcf` files: 20-byte header (magic, format, dims, frame count) + concatenated compressed frames
   - Writes `bc-metadata.json` consumed by `generate-manifest.cjs`
   - BC7 uses `alpha_slow_settings()` for maximum quality (offline tool)

2. **Manifest pipeline** — Updated for BC metadata
   - `scripts/generate-manifest.cjs`: reads `bc-metadata.json` instead of `blob-metadata.json`
   - `manifest.rs`: `blobFile`/`blobWidth`/`blobHeight` → `bcFile`/`bcWidth`/`bcHeight`

3. **Renderer changes**
   - `wgpu_state.rs`: Requests `TEXTURE_COMPRESSION_BC` device feature; added `upload_bc4_texture`, `upload_bc7_texture`, `write_bc4`, `write_bc7` methods
   - `animation.rs`: `FrameStorage::Blob` (mmap) → `FrameStorage::Compressed` (Vec<u8> contiguous buffer)
   - `shader.wgsl`: RGBA path now applies shader tint (`sample.rgb * quad.tint.rgb`); static layers pass white tint `[1,1,1,0]`
   - `mod.rs`: BCF file loading (header parse, validation), BC4/BC7 dispatch in install + render, tint for all sublayers
   - `png_loader.rs`: Removed `tint_and_premultiply()` (no longer needed — BC7 pre-baked, tint in shader)
   - `Cargo.toml`: Removed `memmap2` (not CDN-compatible)

4. **Deleted** `scripts/generate-blobs.cjs` (replaced by Rust tool)

**Results:**
- Total compressed size: **2.76 GB** (down from 6.72 GB raw blobs) — 59% reduction
- GPU memory: BC textures stay compressed in VRAM (4-8x smaller footprint)
- Zero CPU processing at runtime — no `tint_and_premultiply`, GPU decodes BC natively
- BC4 encoding: ~0.1s per sublayer; BC7 encoding: ~10-25s per sublayer (offline, one-time)

**Build pipeline order:**
```
1. node scripts/optimize-animations.cjs        (crop animation frames)
2. cargo run --release --manifest-path tools/compress-animations/Cargo.toml
                                                (PNG → BC4/BC7 .bcf files + bc-metadata.json)
3. node scripts/generate-manifest.cjs          (reads bc-metadata.json → manifest.json)
4. cargo tauri dev                             (run the app)
```

**Key files:**
- `tools/compress-animations/src/main.rs` — BC compression tool
- `src-tauri/src/renderer/wgpu_state.rs` — BC upload/write methods
- `src-tauri/src/renderer/animation.rs` — FrameStorage::Compressed
- `src-tauri/src/renderer/shader.wgsl` — Tint for all texture types

---

## Phase 4: Expression System & Outfit Management — TODO

**Goal:** Add expression slots and outfit save/load.

**Tasks:**
- [ ] Expression slot UI (4 numbered slots)
- [ ] Per-slot independent selections for expression-driven layers (eyes, mouth, eyebrows, hands)
- [ ] Duplicate/Reset expression buttons
- [ ] Visibility toggle (eye icon) for expression layers
- [ ] Outfit dropdown — save/load/rename/delete snapshots
- [ ] State persistence — JSON save on change (debounced 500ms), load on startup
- [ ] Layer dependency logic (sclera-eye binding, dizzy iris override, elf/human ears, hat masking, etc.)

---

## Phase 5: Voice Reactivity — TODO

**Goal:** Mic-driven mouth animation and effects.

**Tasks:**
- [ ] Microphone capture (`getUserMedia`, device enumeration)
- [ ] Audio pipeline (`AudioContext` + `AnalyserNode`, NOT connected to output)
- [ ] Volume detection (~30Hz polling, RMS, normalize 0-1)
- [ ] Volume UI (level indicator + sensitivity slider)
- [ ] Expression switching (above threshold → slot 2/talking, below → slot 1/idle with 150ms hold)
- [ ] Effects system (breathing, bounce, shake, scale pulse)

---

## Phase 6: OBS Integration (Focus-Toggle Overlay) — TODO

**Goal:** Single-window OBS overlay — unfocus the window to enter transparent overlay mode, focus to edit.

**Architecture:** No second window needed. The existing window toggles between edit mode and overlay mode based on focus:
- **Focused (edit mode):** UI visible, `CompositeAlphaMode::Opaque`, clear color `#f5efe6`
- **Unfocused (overlay mode):** UI hidden, `CompositeAlphaMode::PreMultiplied`, clear color `(0,0,0,0)`, always-on-top, click-through

**Why this works:** `transparent: true` is already set in `tauri.conf.json`, so DWM supports alpha. Surface reconfiguration uses the same atomic pattern as window resize. OBS Window Capture respects alpha with "Allow Transparency" enabled.

**Tasks:**
- [ ] Add `AtomicBool` overlay mode flag to `WgpuState`, check each frame in render loop
- [ ] Reconfigure surface on mode change (`CompositeAlphaMode` + clear color swap)
- [ ] Listen for window focus/blur events in Tauri, toggle overlay mode flag
- [ ] Hide/show React UI on focus change (`window.blur`/`focus` events)
- [ ] Set always-on-top + `set_ignore_cursor_events(true)` on blur, revert on focus
- [ ] Verify transparency with OBS Window Capture ("Allow Transparency" checkbox)
- [ ] Browser source (localhost HTTP server serving avatar) — stretch goal

---

## Phase 7: Face Tracking — TODO

**Goal:** Webcam-based expression detection.

**Tasks:**
- [ ] MediaPipe Face Mesh integration
- [ ] Expression detection (mouth open, smile, eyebrow raise, eye blink, head tilt)
- [ ] Expression mapping to slots 3-4
- [ ] Priority system (voice > face tracking > idle)
- [ ] 10fps inference rate

---

## Phase 8: Licensing & Asset Delivery — TODO

**Goal:** License key activation and encrypted CDN delivery.

**Tasks:**
- [ ] LemonSqueezy/Gumroad integration
- [ ] Machine fingerprinting (Rust)
- [ ] AES-256-GCM asset encryption
- [ ] S3 + CloudFront CDN with signed URLs
- [ ] Manifest-based delta sync
- [ ] SVG obfuscation
- [ ] 30-day offline grace period

---

## Phase 9: Build, Polish & Ship — TODO

**Goal:** Production-ready installers and UX polish.

**Tasks:**
- [ ] Windows NSIS installer, macOS DMG
- [ ] Code signing
- [ ] Auto-update system (Tauri updater)
- [ ] CI/CD (GitHub Actions)
- [ ] UX polish (loading states, error handling, onboarding)
- [ ] Performance targets (<60MB RAM, <2% CPU idle, <3s startup)

---

## Project Structure

```
src/
  App.tsx                       # Screen router + useRendererSync
  main.tsx                      # React entry point
  index.css                     # Tailwind + theme variables
  screens/
    HomeScreen.tsx               # Avatar preview + controls
    WardrobeScreen.tsx           # Asset selection UI (Phase 3)
    OverlayScreen.tsx            # OBS overlay (Phase 6)
  components/
    AvatarCanvas.tsx             # Transparent div placeholder (wgpu renders behind)
  hooks/
    useRendererSync.ts           # Zustand → Rust wgpu IPC bridge
  engine/
    manifest.ts                  # Asset manifest types + loader (used by wardrobe UI)
  store/
    minikoStore.ts               # Zustand state
  utils/
    colorUtils.ts                # HSL conversion + shadow computation
    assetUtils.ts                # Layer defs + categories
scripts/
  generate-manifest.cjs          # Asset folder → manifest.json
  optimize-animations.cjs        # Auto-crop PNG animations
tools/
  compress-animations/           # Standalone Rust binary: PNG → BC4/BC7 .bcf files
public/
  manifest.json                  # Generated asset manifest (483 assets)
  assets/                        # Junction → ~/Desktop/mini-ko assets
src-tauri/
  src/main.rs                    # Tauri entry point
  src/lib.rs                     # Tauri commands + render thread spawn
  src/renderer/
    mod.rs                       # AvatarRenderer: orchestration, non-blocking IPC
    wgpu_state.rs                # GPU device, surface, pipeline, TextureUploader
    svg_loader.rs                # Sentinel color replacement + resvg rasterization
    png_loader.rs                # PNG decode + tint + premultiply
    animation.rs                 # Frame timing, AnimationState
    color.rs                     # hex↔HSL, shadow/highlight computation
    manifest.rs                  # Rust types for manifest.json
    layer_stack.rs               # 41 layer slots (TextureLayer)
    shader.wgsl                  # Textured quad vertex/fragment shader
  Cargo.toml                     # wgpu, resvg, image, etc.
  tauri.conf.json                # Tauri app config (transparent: true)
```
