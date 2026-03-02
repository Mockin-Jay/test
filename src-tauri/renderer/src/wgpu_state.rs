use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;

use crate::layer_stack::{CANVAS_HEIGHT, CANVAS_WIDTH};

/// A single draw command for the compositor.
pub struct DrawCommand<'a> {
    pub bind_group: &'a wgpu::BindGroup,
    pub offset: (f32, f32),
    pub size: (u32, u32),
    /// [r, g, b, 0.0] for full RGBA textures, [r, g, b, 1.0] for alpha-only textures
    pub tint: [f32; 4],
}

/// Thread-safe handle for uploading textures from any thread.
/// Clone this and send to background threads for off-main-thread GPU uploads.
#[derive(Clone)]
pub struct TextureUploader {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    bind_group_layout: Arc<wgpu::BindGroupLayout>,
    sampler: Arc<wgpu::Sampler>,
}

impl TextureUploader {
    pub fn upload_texture(&self, width: u32, height: u32, rgba: &[u8]) -> wgpu::Texture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        texture
    }

    /// Upload alpha-only data to a R8Unorm GPU texture.
    pub fn upload_alpha_texture(
        &self,
        width: u32,
        height: u32,
        alpha: &[u8],
    ) -> wgpu::Texture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            alpha,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width), // 1 byte per pixel
                rows_per_image: Some(height),
            },
            size,
        );

        texture
    }

    /// Write RGBA data to an existing Rgba8Unorm texture (for frame updates).
    pub fn write_rgba(&self, texture: &wgpu::Texture, width: u32, height: u32, rgba: &[u8]) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
    }

    /// Write alpha-only data to an existing R8Unorm texture (for frame updates).
    pub fn write_alpha(
        &self,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        alpha: &[u8],
    ) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            alpha,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width), // 1 byte per pixel
                rows_per_image: Some(height),
            },
            size,
        );
    }

    /// Upload BC4-compressed data to a Bc4RUnorm GPU texture.
    pub fn upload_bc4_texture(&self, width: u32, height: u32, data: &[u8]) -> wgpu::Texture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bc4RUnorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((width / 4) * 8), // 8 bytes per 4x4 block
                rows_per_image: Some(height / 4),
            },
            size,
        );

        texture
    }

    /// Upload BC7-compressed data to a Bc7RgbaUnorm GPU texture.
    pub fn upload_bc7_texture(&self, width: u32, height: u32, data: &[u8]) -> wgpu::Texture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bc7RgbaUnorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((width / 4) * 16), // 16 bytes per 4x4 block
                rows_per_image: Some(height / 4),
            },
            size,
        );

        texture
    }

    /// Write BC4 data to an existing Bc4RUnorm texture (for frame updates).
    pub fn write_bc4(&self, texture: &wgpu::Texture, width: u32, height: u32, data: &[u8]) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((width / 4) * 8),
                rows_per_image: Some(height / 4),
            },
            size,
        );
    }

    /// Write BC7 data to an existing Bc7RgbaUnorm texture (for frame updates).
    pub fn write_bc7(&self, texture: &wgpu::Texture, width: u32, height: u32, data: &[u8]) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((width / 4) * 16),
                rows_per_image: Some(height / 4),
            },
            size,
        );
    }

    pub fn create_texture_bind_group(&self, texture: &wgpu::Texture) -> wgpu::BindGroup {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }
}

pub struct WgpuState<'win> {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pub surface: wgpu::Surface<'win>,
    pub config: Mutex<wgpu::SurfaceConfiguration>,
    pub render_pipeline: wgpu::RenderPipeline,
    sampler: Arc<wgpu::Sampler>,
    texture_bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pub quad_bind_group_layout: wgpu::BindGroupLayout,
    /// Pending resize dimensions (set by main thread, consumed by render thread).
    /// 0 = no pending resize (resize clamps to .max(1)).
    pending_width: AtomicU32,
    pending_height: AtomicU32,
}

impl<'win> WgpuState<'win> {
    pub async fn new(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'win>,
        width: u32,
        height: u32,
    ) -> Self {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate GPU adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("miniko_device"),
                    required_features: wgpu::Features::TEXTURE_COMPRESSION_BC,
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("Failed to create GPU device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let sampler = Arc::new(device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        }));

        // Group 0: texture + sampler
        let texture_bind_group_layout =
            Arc::new(
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("texture_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(
                                wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }),
            );

        // Group 1: per-quad uniform (transform + tint = 32 bytes)
        let quad_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("quad_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &quad_bind_group_layout],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);

        // Prefer non-sRGB swapchain format so our sRGB source textures aren't double-encoded.
        // Our textures are Rgba8Unorm (no hardware sRGB decode), and we composite in sRGB space
        // like Photoshop/PixiJS. Non-sRGB swapchain means no encoding on output either.
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|f| !f.is_srgb())
            .copied()
            .unwrap_or(swapchain_capabilities.formats[0]);

        // Use Opaque alpha mode: the wgpu surface provides the window background.
        // This avoids white fringing from DWM compositing semi-transparent edge
        // pixels against the desktop. The webview overlays transparently on top.
        let alpha_mode = if swapchain_capabilities
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::Opaque)
        {
            wgpu::CompositeAlphaMode::Opaque
        } else {
            swapchain_capabilities.alpha_modes[0]
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        println!(
            "[wgpu] Swapchain format: {:?}, alpha mode: {:?}",
            swapchain_format, alpha_mode
        );

        Self {
            device,
            queue,
            surface,
            config: Mutex::new(config),
            render_pipeline,
            sampler,
            texture_bind_group_layout,
            quad_bind_group_layout,
            pending_width: AtomicU32::new(0),
            pending_height: AtomicU32::new(0),
        }
    }

    /// Create a thread-safe TextureUploader for off-main-thread GPU uploads.
    pub fn create_uploader(&self) -> TextureUploader {
        TextureUploader {
            device: Arc::clone(&self.device),
            queue: Arc::clone(&self.queue),
            bind_group_layout: Arc::clone(&self.texture_bind_group_layout),
            sampler: Arc::clone(&self.sampler),
        }
    }

    /// Upload RGBA pixel data to a GPU texture.
    pub fn upload_texture(&self, width: u32, height: u32, rgba: &[u8]) -> wgpu::Texture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        texture
    }

    /// Create a bind group for a texture (group 0).
    pub fn create_texture_bind_group(&self, texture: &wgpu::Texture) -> wgpu::BindGroup {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    /// Called from the main thread. Stores pending dimensions for the render thread to apply.
    pub fn resize(&self, width: u32, height: u32) {
        self.pending_width.store(width.max(1), Ordering::Relaxed);
        self.pending_height.store(height.max(1), Ordering::Relaxed);
    }

    /// Apply any pending resize. Called from the render thread only.
    pub fn apply_pending_resize(&self) -> bool {
        let w = self.pending_width.swap(0, Ordering::Relaxed);
        let h = self.pending_height.swap(0, Ordering::Relaxed);

        if w == 0 || h == 0 {
            return false;
        }

        let mut config = self.config.lock().unwrap();
        if config.width == w && config.height == h {
            return false;
        }

        config.width = w;
        config.height = h;
        self.surface.configure(&self.device, &config);
        true
    }

    /// Compute the NDC transform for a layer texture.
    fn compute_quad_transform(
        &self,
        tex_width: u32,
        tex_height: u32,
        offset: (f32, f32),
    ) -> [f32; 4] {
        let config = self.config.lock().unwrap();
        let win_w = config.width as f32;
        let win_h = config.height as f32;

        let scale = win_h / CANVAS_HEIGHT as f32;

        let scaled_w = tex_width as f32 * scale;
        let scaled_h = tex_height as f32 * scale;

        let ndc_scale_x = scaled_w / win_w;
        let ndc_scale_y = scaled_h / win_h;

        let canvas_center_x = CANVAS_WIDTH as f32 / 2.0;
        let tex_center_x = tex_width as f32 / 2.0 + offset.0;
        let tex_center_y = tex_height as f32 / 2.0 + offset.1;

        let ndc_offset_x = ((tex_center_x - canvas_center_x) * scale) / (win_w / 2.0);
        let ndc_offset_y =
            -((tex_center_y - CANVAS_HEIGHT as f32 / 2.0) * scale) / (win_h / 2.0);

        [ndc_offset_x, ndc_offset_y, ndc_scale_x, ndc_scale_y]
    }

    /// Render a list of draw commands to the surface.
    pub fn render_draw_list(&self, commands: &[DrawCommand]) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                let config = self.config.lock().unwrap();
                self.surface.configure(&self.device, &config);
                return;
            }
            Err(wgpu::SurfaceError::Timeout) => {
                return;
            }
            Err(e) => {
                eprintln!("[wgpu] Surface error: {e:?}");
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Clear with the app's warm background color (#f5efe6).
                        // With Opaque alpha mode, this becomes the window background.
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0xf5 as f64 / 255.0,
                            g: 0xef as f64 / 255.0,
                            b: 0xe6 as f64 / 255.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.render_pipeline);

            for cmd in commands {
                let transform = self.compute_quad_transform(cmd.size.0, cmd.size.1, cmd.offset);

                // 8 floats: 4 for transform + 4 for tint
                let uniforms: [f32; 8] = [
                    transform[0],
                    transform[1],
                    transform[2],
                    transform[3],
                    cmd.tint[0],
                    cmd.tint[1],
                    cmd.tint[2],
                    cmd.tint[3],
                ];

                let uniform_buf =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&uniforms),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                let quad_bind_group =
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &self.quad_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: uniform_buf.as_entire_binding(),
                        }],
                    });

                rpass.set_bind_group(0, cmd.bind_group, &[]);
                rpass.set_bind_group(1, &quad_bind_group, &[]);
                rpass.draw(0..6, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
