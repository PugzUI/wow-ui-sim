//! Headless GPU rendering for producing screenshots.
//!
//! Uses the same wgpu shader pipeline as the iced GUI but drives it
//! without a window. Produces pixel-identical output to the live renderer.

use iced::widget::shader::{Pipeline, Primitive};
use image::RgbaImage;

use super::shader::primitive::{LoadedTexture, load_texture_prefer_bc};
use super::shader::{GpuBcTextureData, GpuTextureData, QuadBatch, WowUiPrimitive};
use crate::texture::TextureManager;

const BYTES_PER_PIXEL: u32 = 4;
const READ_BACK_ROW_ALIGNMENT: u32 = 256;

/// Load unique textures for all batch texture requests.
fn load_batch_textures(
    batch: &QuadBatch,
    tex_mgr: &mut TextureManager,
) -> (Vec<GpuTextureData>, Vec<GpuBcTextureData>) {
    let mut textures = Vec::new();
    let mut bc_textures = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for request in batch
        .texture_requests
        .iter()
        .chain(&batch.mask_texture_requests)
    {
        if seen.contains(&request.path) {
            continue;
        }
        if let Some(loaded) = load_texture_prefer_bc(tex_mgr, &request.path) {
            seen.insert(request.path.clone());
            match loaded {
                LoadedTexture::Rgba(data) => textures.push(data),
                LoadedTexture::Bc(data) => bc_textures.push(data),
            }
        }
    }
    (textures, bc_textures)
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

/// Copy render target to a readable buffer and read back pixels into an image.
fn read_back_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: wgpu::CommandEncoder,
    render_texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> RgbaImage {
    let layout = read_back_buffer_layout(width, height);
    let output_buffer = create_read_back_buffer(device, &layout);
    queue_read_back_copy(
        queue,
        encoder,
        render_texture,
        &output_buffer,
        width,
        height,
        layout.bytes_per_row,
    );
    read_back_image(device, &output_buffer, width, height, layout.bytes_per_row)
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
    let data = map_read_back_buffer(device, output_buffer);
    image_from_read_back_buffer(&data, width, height, bytes_per_row)
}

fn build_headless_primitive(
    batch: &QuadBatch,
    tex_mgr: &mut TextureManager,
    glyph_atlas_data: Option<(&[u8], u32)>,
) -> WowUiPrimitive {
    let (textures, bc_textures) = load_batch_textures(batch, tex_mgr);
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

fn create_headless_pipeline_and_target(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
) -> (
    super::shader::WowUiPipeline,
    wgpu::Texture,
    wgpu::TextureView,
) {
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let pipeline = super::shader::WowUiPipeline::new(device, queue, format);
    let (render_texture, render_view) = create_render_target(device, width, height, format);
    (pipeline, render_texture, render_view)
}

fn prepare_headless_primitive(
    primitive: &mut WowUiPrimitive,
    pipeline: &mut super::shader::WowUiPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
) {
    let bounds = iced::Rectangle::new(
        iced::Point::ORIGIN,
        iced::Size::new(width as f32, height as f32),
    );
    let viewport =
        iced::widget::shader::Viewport::with_physical_size(iced::Size::new(width, height), 1.0);
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

/// Render a QuadBatch to an RGBA image using headless wgpu.
///
/// Creates a headless GPU device, sets up the same WowUiPipeline used by
/// the iced GUI, and renders to an offscreen texture. The result is read
/// back to CPU memory as an RgbaImage.
///
/// When `glyph_atlas_data` is provided, text glyphs are rendered using the
/// glyph atlas texture.
pub fn render_to_image(
    batch: &QuadBatch,
    tex_mgr: &mut TextureManager,
    width: u32,
    height: u32,
    glyph_atlas_data: Option<(&[u8], u32)>,
) -> RgbaImage {
    let mut primitive = build_headless_primitive(batch, tex_mgr, glyph_atlas_data);
    let (device, queue) = create_headless_device();
    render_headless_primitive_to_image(&mut primitive, &device, &queue, width, height)
}

fn render_headless_primitive_to_image(
    primitive: &mut WowUiPrimitive,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
) -> RgbaImage {
    let (mut pipeline, render_texture, render_view) =
        create_headless_pipeline_and_target(device, queue, width, height);
    prepare_headless_primitive(primitive, &mut pipeline, device, queue, width, height);
    let encoder = clear_headless_render_target(device, &mut pipeline, &render_view, width, height);
    read_back_pixels(device, queue, encoder, &render_texture, width, height)
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
