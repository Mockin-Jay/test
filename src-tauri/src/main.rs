// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tauri::{Manager, RunEvent, WindowEvent};

use miniko_renderer::{AvatarRenderer, RawWindowHandles};

#[derive(serde::Deserialize)]
struct LayerUpdate {
    layer_num: u8,
    asset_id: String,
    colors: HashMap<String, String>,
    offset: (f32, f32),
}

#[tauri::command]
fn update_layer(
    state: tauri::State<'_, Arc<AvatarRenderer<'static>>>,
    layer_num: u8,
    asset_id: String,
    colors: HashMap<String, String>,
    offset: (f32, f32),
) -> Result<(), String> {
    state.load_layer(layer_num, &asset_id, &colors, offset);
    Ok(())
}

#[tauri::command]
fn clear_layer(
    state: tauri::State<'_, Arc<AvatarRenderer<'static>>>,
    layer_num: u8,
) -> Result<(), String> {
    state.clear_layer(layer_num);
    Ok(())
}

#[tauri::command]
fn update_all_layers(
    state: tauri::State<'_, Arc<AvatarRenderer<'static>>>,
    updates: Vec<LayerUpdate>,
) -> Result<(), String> {
    for update in updates {
        state.load_layer(
            update.layer_num,
            &update.asset_id,
            &update.colors,
            update.offset,
        );
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            let resource_dir = app
                .path()
                .resource_dir()
                .expect("Failed to get resource dir");

            let manifest_path = resource_dir.join("manifest.json");
            let assets_dir = resource_dir.join("assets");

            let (manifest_path, assets_dir) = if manifest_path.exists() {
                // Bundled resources (tauri build with resources config)
                (manifest_path, assets_dir)
            } else {
                // Fallback: look next to the executable
                let exe_dir = std::env::current_exe()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_path_buf();
                let exe_manifest = exe_dir.join("manifest.json");
                if exe_manifest.exists() {
                    (exe_manifest, exe_dir.join("assets"))
                } else {
                    // Dev mode fallback
                    let dev_public = std::env::current_dir()
                        .unwrap()
                        .join("../public");
                    (
                        dev_public.join("manifest.json"),
                        dev_public.join("assets"),
                    )
                }
            };

            println!("[renderer] Manifest: {}", manifest_path.display());
            println!("[renderer] Assets dir: {}", assets_dir.display());

            let size = window.inner_size().unwrap();
            let handles = RawWindowHandles {
                window: window.window_handle().unwrap().as_raw(),
                display: window.display_handle().unwrap().as_raw(),
            };

            let renderer = tauri::async_runtime::block_on(AvatarRenderer::new(
                handles,
                size.width,
                size.height,
                &manifest_path,
                assets_dir,
            ));

            // Load default equipped assets on startup
            let default_assets: Vec<(u8, &str, HashMap<String, String>)> = vec![
                (32, "body", HashMap::new()),
                (8, "animal-ear_front_1", HashMap::new()),
                (38, "animal-ear_back_1", HashMap::new()),
                (17, "hair_front_28", HashMap::new()),
                (36, "hair_back_28", HashMap::new()),
                (9, "glasses_1", HashMap::new()),
            ];

            for (layer_num, asset_id, colors) in &default_assets {
                renderer.load_layer(*layer_num, asset_id, colors, (0.0, 0.0));
                println!("[renderer] Queued: layer {layer_num} = {asset_id}");
            }

            let renderer = Arc::new(renderer);

            // Spawn a dedicated render thread at ~60fps.
            {
                let r = Arc::clone(&renderer);
                std::thread::spawn(move || {
                    const INTERVAL: Duration = Duration::from_millis(16);
                    println!("[renderer] Render thread started (~60fps)");
                    loop {
                        let t0 = Instant::now();
                        if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            r.render_frame();
                        })) {
                            eprintln!("[renderer] render_frame panicked: {:?}", e);
                        }
                        let elapsed = t0.elapsed();
                        if elapsed < INTERVAL {
                            std::thread::sleep(INTERVAL - elapsed);
                        }
                    }
                });
            }

            app.manage(renderer);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            update_layer,
            clear_layer,
            update_all_layers,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            RunEvent::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let state = app_handle.state::<Arc<AvatarRenderer<'static>>>();
                state.resize(size.width, size.height);
            }
            _ => {}
        });
}
