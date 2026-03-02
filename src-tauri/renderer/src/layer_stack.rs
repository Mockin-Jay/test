/// Layer Stack — Manages 41 texture layers for compositing.
/// Layer 41 = backmost, Layer 1 = frontmost.

use std::collections::HashMap;

pub const LAYER_COUNT: u8 = 41;
pub const CANVAS_WIDTH: u32 = 3000;
pub const CANVAS_HEIGHT: u32 = 2320;

/// A single texture layer that can be drawn.
pub struct TextureLayer {
    pub texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
    /// Position offset in canvas pixels
    pub offset: (f32, f32),
    /// Size in canvas pixels (for cropped animation frames)
    pub size: (u32, u32),
}

/// Manages all 41 layers.
pub struct LayerStack {
    /// layerNum (1-41) → list of TextureLayers (most layers have 1, png-stacks can have multiple sublayers)
    layers: HashMap<u8, Vec<TextureLayer>>,
}

impl LayerStack {
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
        }
    }

    /// Set the texture(s) for a layer, replacing any existing ones.
    pub fn set_layer(&mut self, layer_num: u8, textures: Vec<TextureLayer>) {
        self.layers.insert(layer_num, textures);
    }

    /// Clear a layer (remove all its textures).
    pub fn clear_layer(&mut self, layer_num: u8) {
        self.layers.remove(&layer_num);
    }

    /// Iterate layers in draw order (41 first/backmost → 1 last/frontmost).
    pub fn iter_draw_order(&self) -> impl Iterator<Item = (u8, &Vec<TextureLayer>)> {
        let mut entries: Vec<_> = self.layers.iter().map(|(&k, v)| (k, v)).collect();
        entries.sort_by(|a, b| b.0.cmp(&a.0)); // 41, 40, ..., 2, 1
        entries.into_iter()
    }

    pub fn clear_all(&mut self) {
        self.layers.clear();
    }

    /// Get all active layer numbers.
    pub fn layer_nums(&self) -> Vec<&u8> {
        self.layers.keys().collect()
    }

    /// Get the textures for a specific layer.
    pub fn get_layer(&self, layer_num: u8) -> Option<&Vec<TextureLayer>> {
        self.layers.get(&layer_num)
    }
}
