/// Rust types for manifest.json — mirrors src/engine/manifest.ts

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Bounds {
    pub width: u32,
    pub height: u32,
    #[serde(rename = "offsetX")]
    pub offset_x: i32,
    #[serde(rename = "offsetY")]
    pub offset_y: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SubLayerDef {
    pub name: String,
    #[serde(rename = "frameCount")]
    pub frame_count: u32,
    pub prefix: String,
    pub bounds: Option<Bounds>,
    /// True if all visible pixels are white — store alpha-only, tint in shader
    #[serde(rename = "alphaOnly")]
    pub alpha_only: Option<bool>,
    /// Path to pre-baked BC-compressed file (relative to assets dir)
    #[serde(rename = "bcFile")]
    pub bc_file: Option<String>,
    /// Frame width in the BC file (padded to multiple of 4)
    #[serde(rename = "bcWidth")]
    pub bc_width: Option<u32>,
    /// Frame height in the BC file (padded to multiple of 4)
    #[serde(rename = "bcHeight")]
    pub bc_height: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AssetDef {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub path: String,
    #[serde(rename = "subLayerCount")]
    pub sub_layer_count: Option<u32>,
    #[serde(rename = "subLayers")]
    pub sub_layers: Option<Vec<SubLayerDef>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LayerManifest {
    #[serde(rename = "layerNum")]
    pub layer_num: u8,
    #[serde(rename = "folderName")]
    pub folder_name: String,
    pub assets: Vec<AssetDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Manifest {
    pub layers: Vec<LayerManifest>,
}

impl Manifest {
    pub fn load(manifest_path: &Path) -> Result<Self, String> {
        let data = std::fs::read_to_string(manifest_path)
            .map_err(|e| format!("Failed to read manifest: {e}"))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse manifest: {e}"))
    }

    pub fn get_asset(&self, layer_num: u8, asset_id: &str) -> Option<&AssetDef> {
        self.layers
            .iter()
            .find(|l| l.layer_num == layer_num)
            .and_then(|l| l.assets.iter().find(|a| a.id == asset_id))
    }
}
