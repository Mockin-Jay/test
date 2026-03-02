/// Animation controller — manages frame timing and frame data access.
/// Supports two storage modes:
///   1. BC-compressed contiguous buffer (fast path): frames are slices of BC4/BC7 data
///   2. RAM-cached Vec (fallback): frames stored as Vec<u8> per frame

use std::time::{Duration, Instant};

/// Frame data storage — either BC-compressed contiguous buffer or individual Vec buffers.
pub enum FrameStorage {
    /// BC-compressed frames in a contiguous buffer.
    /// frame_data(idx) returns a slice of compressed BC blocks.
    Compressed {
        data: Vec<u8>,
        frame_bytes: usize,
        frame_count: usize,
    },
    /// Individual frame buffers in RAM (fallback for non-BC sublayers).
    Frames(Vec<Vec<u8>>),
}

impl FrameStorage {
    pub fn frame_data(&self, idx: usize) -> &[u8] {
        match self {
            FrameStorage::Compressed {
                data, frame_bytes, ..
            } => {
                let start = idx * frame_bytes;
                &data[start..start + frame_bytes]
            }
            FrameStorage::Frames(frames) => &frames[idx],
        }
    }

    pub fn frame_count(&self) -> usize {
        match self {
            FrameStorage::Compressed { frame_count, .. } => *frame_count,
            FrameStorage::Frames(frames) => frames.len(),
        }
    }

    /// Returns true if this storage is BC-compressed data.
    pub fn is_compressed(&self) -> bool {
        matches!(self, FrameStorage::Compressed { .. })
    }
}

/// One sublayer of an animation (e.g., one color channel).
pub struct AnimSubLayer {
    pub storage: FrameStorage,
    pub offset: (f32, f32),
    pub size: (u32, u32),
    /// Single persistent GPU texture (BC4/R8Unorm for alpha-only, BC7/Rgba8Unorm for full RGBA)
    pub gpu_texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
    /// Which frame index is currently uploaded to the GPU texture
    pub uploaded_frame_idx: Option<usize>,
    /// If true, frames store alpha-only data and shader applies tint
    pub alpha_only: bool,
    /// Tint color for sublayers [r, g, b] in 0.0..1.0
    pub tint_rgb: [f32; 3],
}

/// Full animation state for a layer.
pub struct AnimationState {
    pub sub_layers: Vec<AnimSubLayer>,
    pub current_frame: usize,
    pub max_frame_count: usize,
    pub last_advance: Instant,
    pub frame_duration: Duration,
}

impl AnimationState {
    pub fn new(sub_layers: Vec<AnimSubLayer>, fps: f32) -> Self {
        let max_frame_count = sub_layers
            .iter()
            .map(|s| s.storage.frame_count())
            .max()
            .unwrap_or(1);

        Self {
            sub_layers,
            current_frame: 0,
            max_frame_count,
            last_advance: Instant::now(),
            frame_duration: Duration::from_secs_f32(1.0 / fps),
        }
    }

    /// Advance the animation, returns true if frame changed.
    pub fn tick(&mut self) -> bool {
        if self.max_frame_count <= 1 {
            return false;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_advance);

        if elapsed >= self.frame_duration {
            let frames_to_advance =
                (elapsed.as_secs_f32() / self.frame_duration.as_secs_f32()) as usize;
            self.current_frame =
                (self.current_frame + frames_to_advance) % self.max_frame_count;
            self.last_advance = now;
            true
        } else {
            false
        }
    }
}
