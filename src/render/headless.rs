//! Headless GPU rendering for producing screenshots.
//!
//! Uses the same wgpu shader pipeline as the iced GUI but drives it
//! without a window. Produces pixel-identical output to the live renderer.

use iced::widget::shader::{Pipeline, Primitive};
use image::RgbaImage;
use std::sync::Arc;

use super::shader::primitive::{LoadedTexture, load_texture_prefer_bc};
use super::shader::{GpuBcTextureData, GpuTextureData, QuadBatch, WowUiPrimitive};
use crate::texture::TextureManager;
use crate::widget::FrameStrata;

const BYTES_PER_PIXEL: u32 = 4;
const READ_BACK_ROW_ALIGNMENT: u32 = 256;
const HEADLESS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Load unique textures for batches whose paths are not already resident.
fn load_batches_textures<'a>(
    batches: impl IntoIterator<Item = &'a QuadBatch>,
    tex_mgr: &mut TextureManager,
    pipeline: &super::shader::WowUiPipeline,
) -> (Vec<GpuTextureData>, Vec<GpuBcTextureData>) {
    let mut textures = Vec::new();
    let mut bc_textures = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for batch in batches {
        for request in batch
            .texture_requests
            .iter()
            .chain(&batch.mask_texture_requests)
        {
            if !seen.insert(request.path.clone()) || pipeline.has_texture(&request.path) {
                continue;
            }
            if let Some(loaded) = load_texture_prefer_bc(tex_mgr, &request.path) {
                match loaded {
                    LoadedTexture::Rgba(data) => textures.push(data),
                    LoadedTexture::Bc(data) => bc_textures.push(data),
                }
            }
        }
    }
    (textures, bc_textures)
}

fn load_batch_textures(
    batch: &QuadBatch,
    tex_mgr: &mut TextureManager,
    pipeline: &super::shader::WowUiPipeline,
) -> (Vec<GpuTextureData>, Vec<GpuBcTextureData>) {
    load_batches_textures(std::iter::once(batch), tex_mgr, pipeline)
}

/// Create a headless wgpu device and queue.
fn create_headless_device() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: std::env::var("WOW_SIM_SOFTWARE_RENDER")
                    .is_ok_and(|v| v == "1" || v == "true"),
            })
            .await
            .expect("Failed to find GPU adapter");

        // Request BC texture compression if available (for direct BLP upload)
        let features = if adapter
            .features()
            .contains(wgpu::Features::TEXTURE_COMPRESSION_BC)
        {
            wgpu::Features::TEXTURE_COMPRESSION_BC
        } else {
            wgpu::Features::empty()
        };
        adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: features,
                ..Default::default()
            })
            .await
            .expect("Failed to create GPU device")
    })
}

/// Create a render target texture and its view.
fn create_render_target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Screenshot Render Target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[derive(Debug, Clone, Copy)]
struct ReadBackBufferLayout {
    bytes_per_row: u32,
    size_bytes: u64,
}

fn read_back_buffer_layout(width: u32, height: u32) -> ReadBackBufferLayout {
    let row_bytes = width * BYTES_PER_PIXEL;
    let bytes_per_row = (row_bytes + READ_BACK_ROW_ALIGNMENT - 1) & !(READ_BACK_ROW_ALIGNMENT - 1);
    ReadBackBufferLayout {
        bytes_per_row,
        size_bytes: (bytes_per_row * height) as u64,
    }
}

fn create_read_back_buffer(device: &wgpu::Device, layout: &ReadBackBufferLayout) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenshot Output Buffer"),
        size: layout.size_bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn copy_render_texture_to_read_back_buffer(
    encoder: &mut wgpu::CommandEncoder,
    render_texture: &wgpu::Texture,
    output_buffer: &wgpu::Buffer,
    width: u32,
    height: u32,
    bytes_per_row: u32,
) {
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: render_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

fn map_read_back_buffer(device: &wgpu::Device, output_buffer: &wgpu::Buffer) -> wgpu::BufferView {
    let buffer_slice = output_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(std::time::Duration::from_secs(10)),
    });
    receiver.recv().unwrap().expect("Failed to map buffer");
    buffer_slice.get_mapped_range()
}

fn image_from_read_back_buffer(
    data: &[u8],
    width: u32,
    height: u32,
    bytes_per_row: u32,
) -> RgbaImage {
    let mut image = RgbaImage::new(width, height);
    for y in 0..height {
        let src_offset = (y * bytes_per_row) as usize;
        let row = &data[src_offset..src_offset + (width * BYTES_PER_PIXEL) as usize];
        for x in 0..width {
            let i = (x * BYTES_PER_PIXEL) as usize;
            image.put_pixel(
                x,
                y,
                image::Rgba([row[i], row[i + 1], row[i + 2], row[i + 3]]),
            );
        }
    }
    image
}

fn queue_read_back_copy(
    queue: &wgpu::Queue,
    encoder: wgpu::CommandEncoder,
    render_texture: &wgpu::Texture,
    output_buffer: &wgpu::Buffer,
    width: u32,
    height: u32,
    bytes_per_row: u32,
) {
    let mut encoder = encoder;
    copy_render_texture_to_read_back_buffer(
        &mut encoder,
        render_texture,
        output_buffer,
        width,
        height,
        bytes_per_row,
    );
    queue.submit(std::iter::once(encoder.finish()));
}

fn read_back_image(
    device: &wgpu::Device,
    output_buffer: &wgpu::Buffer,
    width: u32,
    height: u32,
    bytes_per_row: u32,
) -> RgbaImage {
    let image = {
        let data = map_read_back_buffer(device, output_buffer);
        image_from_read_back_buffer(&data, width, height, bytes_per_row)
    };
    output_buffer.unmap();
    image
}

fn build_headless_primitive(
    batch: &QuadBatch,
    tex_mgr: &mut TextureManager,
    pipeline: &super::shader::WowUiPipeline,
    glyph_atlas_data: Option<(&[u8], u32)>,
) -> WowUiPrimitive {
    let (textures, bc_textures) = load_batch_textures(batch, tex_mgr, pipeline);
    let mut primitive = WowUiPrimitive::new_merged_with_textures(
        std::sync::Arc::new(batch.clone()),
        textures,
        bc_textures,
    );
    install_glyph_atlas_data(&mut primitive, glyph_atlas_data);
    primitive
}

fn install_glyph_atlas_data(
    primitive: &mut WowUiPrimitive,
    glyph_atlas_data: Option<(&[u8], u32)>,
) {
    let Some((data, size)) = glyph_atlas_data else {
        return;
    };
    primitive.glyph_atlas_data = Some(data.to_vec());
    primitive.glyph_atlas_size = size;
}

#[allow(clippy::too_many_arguments)]
fn prepare_headless_primitive(
    primitive: &mut WowUiPrimitive,
    pipeline: &mut super::shader::WowUiPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    stage_width: u32,
    stage_height: u32,
    target_width: u32,
    target_height: u32,
) {
    let bounds = iced::Rectangle::new(
        iced::Point::ORIGIN,
        iced::Size::new(stage_width as f32, stage_height as f32),
    );
    let viewport = iced::widget::shader::Viewport::with_physical_size(
        iced::Size::new(target_width, target_height),
        1.0,
    );
    primitive.prepare(pipeline, device, queue, &bounds, &viewport);
}

fn clear_headless_render_target(
    device: &wgpu::Device,
    pipeline: &mut super::shader::WowUiPipeline,
    render_view: &wgpu::TextureView,
    width: u32,
    height: u32,
) -> wgpu::CommandEncoder {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenshot Encoder"),
    });
    let clip_bounds_u32 = iced::Rectangle {
        x: 0u32,
        y: 0u32,
        width,
        height,
    };
    // The Visualizer stage is an opaque, neutral black canvas.  Keeping the
    // clear colour here (rather than relying on the shell CSS) makes native
    // screenshots and live frames identical and removes the old tiled/marble
    // simulator backdrop from every render path.
    pipeline.render_clear(
        &mut encoder,
        render_view,
        &clip_bounds_u32,
        [0.0, 0.0, 0.0, 1.0],
    );
    encoder
}

/// Reusable target/readback allocation for one stream size.
struct HeadlessTarget {
    width: u32,
    height: u32,
    render_texture: wgpu::Texture,
    render_view: wgpu::TextureView,
    output_buffer: wgpu::Buffer,
    layout: ReadBackBufferLayout,
}

impl HeadlessTarget {
    fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let (render_texture, render_view) =
            create_render_target(device, width, height, HEADLESS_FORMAT);
        let layout = read_back_buffer_layout(width, height);
        let output_buffer = create_read_back_buffer(device, &layout);
        Self {
            width,
            height,
            render_texture,
            render_view,
            output_buffer,
            layout,
        }
    }
}

/// Session-owned headless GPU context.
///
/// Device, pipeline, texture atlas, render target, and readback buffer persist
/// across frames. This keeps interactive native capture proportional to frame
/// work instead of repeating GPU initialization and shader compilation.
pub struct HeadlessRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: super::shader::WowUiPipeline,
    target: Option<HeadlessTarget>,
    strata_cache: [Option<Arc<QuadBatch>>; FrameStrata::COUNT],
    last_glyph_revision: Option<u64>,
}

impl HeadlessRenderer {
    pub fn new() -> Self {
        let (device, queue) = create_headless_device();
        let pipeline = super::shader::WowUiPipeline::new(&device, &queue, HEADLESS_FORMAT);
        Self {
            device,
            queue,
            pipeline,
            target: None,
            strata_cache: std::array::from_fn(|_| None),
            last_glyph_revision: None,
        }
    }

    fn ensure_target(&mut self, width: u32, height: u32) {
        if self
            .target
            .as_ref()
            .is_some_and(|target| target.width == width && target.height == height)
        {
            return;
        }
        self.target = Some(HeadlessTarget::new(&self.device, width, height));
    }

    /// Whether this independent GPU atlas needs the current glyph pixels.
    pub fn needs_glyph_upload(&self, revision: u64) -> bool {
        self.last_glyph_revision != Some(revision)
    }

    fn record_glyph_upload(&mut self, revision: u64, uploaded: bool) {
        if uploaded {
            self.last_glyph_revision = Some(revision);
        }
    }

    /// Render one batch at matching stage and target dimensions.
    pub fn render_to_image(
        &mut self,
        batch: &QuadBatch,
        tex_mgr: &mut TextureManager,
        width: u32,
        height: u32,
        glyph_atlas_data: Option<(&[u8], u32)>,
    ) -> RgbaImage {
        self.pipeline.clear_all_strata();
        self.strata_cache.fill(None);
        self.render_scaled_to_image(
            batch,
            tex_mgr,
            width,
            height,
            width,
            height,
            glyph_atlas_data,
        )
    }

    /// Exact render with a shared glyph-atlas revision.
    #[allow(clippy::too_many_arguments)]
    pub fn render_to_image_with_glyph_revision(
        &mut self,
        batch: &QuadBatch,
        tex_mgr: &mut TextureManager,
        width: u32,
        height: u32,
        glyph_atlas_data: Option<(&[u8], u32)>,
        glyph_revision: u64,
    ) -> RgbaImage {
        let uploaded = glyph_atlas_data.is_some();
        let image = self.render_to_image(batch, tex_mgr, width, height, glyph_atlas_data);
        self.record_glyph_upload(glyph_revision, uploaded);
        image
    }

    /// Render an authoritative stage into a reduced output target.
    ///
    /// Vertex geometry and projection remain in stage pixels while the GPU
    /// viewport scales the result into the stream target.
    #[allow(clippy::too_many_arguments)]
    pub fn render_scaled_to_image(
        &mut self,
        batch: &QuadBatch,
        tex_mgr: &mut TextureManager,
        stage_width: u32,
        stage_height: u32,
        target_width: u32,
        target_height: u32,
        glyph_atlas_data: Option<(&[u8], u32)>,
    ) -> RgbaImage {
        self.ensure_target(target_width, target_height);
        let mut primitive =
            build_headless_primitive(batch, tex_mgr, &self.pipeline, glyph_atlas_data);
        let Self {
            device,
            queue,
            pipeline,
            target,
            ..
        } = self;
        let target = target.as_ref().expect("headless target initialized");
        prepare_headless_primitive(
            &mut primitive,
            pipeline,
            device,
            queue,
            stage_width,
            stage_height,
            target_width,
            target_height,
        );
        let encoder = clear_headless_render_target(
            device,
            pipeline,
            &target.render_view,
            target_width,
            target_height,
        );
        queue_read_back_copy(
            queue,
            encoder,
            &target.render_texture,
            &target.output_buffer,
            target_width,
            target_height,
            target.layout.bytes_per_row,
        );
        read_back_image(
            device,
            &target.output_buffer,
            target_width,
            target_height,
            target.layout.bytes_per_row,
        )
    }

    /// Render the app's cached native strata, uploading only replaced batches.
    #[allow(clippy::too_many_arguments)]
    pub fn render_cached_strata_to_image(
        &mut self,
        cached: &[Option<Arc<QuadBatch>>; FrameStrata::COUNT],
        tex_mgr: &mut TextureManager,
        stage_width: u32,
        stage_height: u32,
        target_width: u32,
        target_height: u32,
        glyph_atlas_data: Option<(&[u8], u32)>,
        glyph_revision: u64,
    ) -> RgbaImage {
        self.ensure_target(target_width, target_height);
        let mut dirty: [Option<Arc<QuadBatch>>; FrameStrata::COUNT] = std::array::from_fn(|_| None);
        for (index, current) in cached.iter().enumerate() {
            let unchanged = match (&self.strata_cache[index], current) {
                (Some(previous), Some(current)) => Arc::ptr_eq(previous, current),
                (None, None) => true,
                _ => false,
            };
            if unchanged {
                continue;
            }
            self.strata_cache[index] = current.clone();
            dirty[index] = Some(
                current
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| Arc::new(QuadBatch::new())),
            );
        }
        let (textures, bc_textures) = load_batches_textures(
            dirty.iter().flatten().map(Arc::as_ref),
            tex_mgr,
            &self.pipeline,
        );
        let uploaded_glyphs = glyph_atlas_data.is_some();
        let mut primitive = WowUiPrimitive {
            strata_batches: dirty,
            overlay: QuadBatch::new(),
            clear_color: [0.0, 0.0, 0.0, 1.0],
            textures,
            bc_textures,
            glyph_atlas_data: glyph_atlas_data.map(|(data, _)| data.to_vec()),
            glyph_atlas_size: glyph_atlas_data.map_or(0, |(_, size)| size),
            texture_requests: None,
        };
        let Self {
            device,
            queue,
            pipeline,
            target,
            ..
        } = self;
        let target = target.as_ref().expect("headless target initialized");
        prepare_headless_primitive(
            &mut primitive,
            pipeline,
            device,
            queue,
            stage_width,
            stage_height,
            target_width,
            target_height,
        );
        let encoder = clear_headless_render_target(
            device,
            pipeline,
            &target.render_view,
            target_width,
            target_height,
        );
        queue_read_back_copy(
            queue,
            encoder,
            &target.render_texture,
            &target.output_buffer,
            target_width,
            target_height,
            target.layout.bytes_per_row,
        );
        let image = read_back_image(
            device,
            &target.output_buffer,
            target_width,
            target_height,
            target.layout.bytes_per_row,
        );
        self.record_glyph_upload(glyph_revision, uploaded_glyphs);
        image
    }
}

impl Default for HeadlessRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a QuadBatch to an RGBA image using a disposable headless context.
///
/// One-shot and test callers retain the historical isolation contract. Warm
/// Visualizer sessions keep a [`HeadlessRenderer`] in the application state.
pub fn render_to_image(
    batch: &QuadBatch,
    tex_mgr: &mut TextureManager,
    width: u32,
    height: u32,
    glyph_atlas_data: Option<(&[u8], u32)>,
) -> RgbaImage {
    HeadlessRenderer::new().render_to_image(batch, tex_mgr, width, height, glyph_atlas_data)
}

#[cfg(test)]
mod tests {
    use super::{image_from_read_back_buffer, install_glyph_atlas_data, read_back_buffer_layout};
    use crate::render::shader::WowUiPrimitive;

    #[test]
    fn read_back_buffer_layout_aligns_rows_to_256_bytes() {
        let layout = read_back_buffer_layout(3, 2);
        assert_eq!(layout.bytes_per_row, 256);
        assert_eq!(layout.size_bytes, 512);
    }

    #[test]
    fn image_from_read_back_buffer_ignores_row_padding() {
        let data = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 99, 99, 99, 99, 9, 10, 11, 12, 13, 14, 15, 16, 88, 88, 88, 88,
        ];
        let image = image_from_read_back_buffer(&data, 2, 2, 12);

        assert_eq!(image.get_pixel(0, 0).0, [1, 2, 3, 4]);
        assert_eq!(image.get_pixel(1, 0).0, [5, 6, 7, 8]);
        assert_eq!(image.get_pixel(0, 1).0, [9, 10, 11, 12]);
        assert_eq!(image.get_pixel(1, 1).0, [13, 14, 15, 16]);
    }

    #[test]
    fn install_glyph_atlas_data_copies_pixels_and_size() {
        let mut primitive = WowUiPrimitive::empty();
        install_glyph_atlas_data(&mut primitive, Some((&[1, 2, 3, 4], 64)));

        assert_eq!(primitive.glyph_atlas_data, Some(vec![1, 2, 3, 4]));
        assert_eq!(primitive.glyph_atlas_size, 64);
    }
}
