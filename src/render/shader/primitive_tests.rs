use super::{
    LoadedTexture, ResolvedTextureEntry, UvRemap, WowUiPipeline, WowUiPrimitive,
    apply_resolved_texture_entry, bc_texture_dimensions_fit_gpu_atlas, decode_crop_request,
    load_texture_prefer_bc, load_texture_prefer_bc_with_telemetry, remap_bc_entry_uv,
    remap_entry_uv, resolve_and_scale_quads,
};
use crate::render::BlendMode;
use crate::render::shader::QuadBatch;
use crate::render::shader::atlas::{BcFormat, BcTextureEntry};
use crate::render::shader::quad::{FLAG_COOLDOWN_SWIPE, QuadVertex};
use bytemuck::Zeroable;
use iced::widget::shader::{Pipeline, Primitive as ShaderPrimitive, Viewport};
use iced::{Point, Rectangle, Size};
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

#[test]
fn quad_shader_clamp_path_compiles_on_real_device() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let (device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("adapter");
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("device")
    });

    // Pipeline creation invokes wgpu's WGSL parser and validates both entry
    // points against the bind-group layouts, including the clamp-to-black path.
    let _pipeline = WowUiPipeline::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb);
}

#[test]
fn decode_crop_request_rejects_malformed_coords() {
    let mut mgr = crate::texture::TextureManager::new();
    assert!(decode_crop_request(&mut mgr, "foo@crop:0.1,0.2,0.3").is_none());
}

#[test]
fn decode_crop_request_uses_cached_texture_dimensions() {
    let mut mgr = crate::texture::TextureManager::new();
    mgr.insert_test_texture(
        r"Interface\Foo\Bar",
        crate::texture::TextureData {
            width: 200,
            height: 100,
            pixels: Arc::<[u8]>::from(vec![0; 200 * 100 * 4]),
        },
    );
    let decoded = decode_crop_request(
        &mut mgr,
        r"Interface\Foo\Bar@crop:0.100000,0.600000,0.200000,0.700000",
    )
    .expect("crop request should decode");
    assert_eq!(decoded.0, r"Interface\Foo\Bar");
    assert_eq!(decoded.1, 20);
    assert_eq!(decoded.2, 20);
    assert_eq!(decoded.3, 100);
    assert_eq!(decoded.4, 50);
}

#[test]
fn load_texture_prefer_bc_reuses_cached_rgba_buffer() {
    let mut mgr = crate::texture::TextureManager::new();
    let cached_pixels = Arc::<[u8]>::from(vec![0xaa; 4 * 4 * 4]);
    mgr.insert_test_texture(
        r"Interface\Foo\Cached",
        crate::texture::TextureData {
            width: 4,
            height: 4,
            pixels: Arc::clone(&cached_pixels),
        },
    );

    let loaded = load_texture_prefer_bc(&mut mgr, r"Interface\Foo\Cached")
        .expect("cached RGBA texture should load");
    let LoadedTexture::Rgba(upload) = loaded else {
        panic!("plain cached texture should stay on the RGBA upload path");
    };

    assert_eq!(
        upload.rgba.as_ptr(),
        cached_pixels.as_ptr(),
        "RGBA upload path should reuse cached pixels instead of cloning them"
    );
}

#[test]
fn aura9_tga_alpha_survives_decode_request_and_rgba_atlas_upload() {
    let root = TempDir::new().expect("temporary addon root");
    let texture_dir = root.path().join("WeakAuras");
    std::fs::create_dir_all(&texture_dir).expect("addon texture directory");
    let texture_path = texture_dir.join("Aura9.tga");
    let mut image = image::RgbaImage::from_pixel(128, 128, image::Rgba([32, 64, 96, 255]));
    for pixel in image.pixels_mut().take(10_301) {
        pixel.0[3] = 0;
    }
    image.save(&texture_path).expect("write Aura9 TGA fixture");

    let request_path = r"Interface\AddOns\WeakAuras\Aura9";
    let mut texture_manager = crate::texture::TextureManager::new().with_addons_path(root.path());
    let loaded = load_texture_prefer_bc(&mut texture_manager, request_path)
        .expect("Aura9 request should decode");
    let LoadedTexture::Rgba(upload) = loaded else {
        panic!("TGA Aura9 must use the RGBA renderer upload path");
    };
    assert_eq!((upload.width, upload.height), (128, 128));
    assert_eq!(
        upload
            .rgba
            .chunks_exact(4)
            .filter(|pixel| pixel[3] == 0)
            .count(),
        10_301,
        "decoded Aura9 alpha must retain the transparent source pixels"
    );

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let (device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("adapter");
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("device")
    });
    let mut pipeline = WowUiPipeline::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb);
    let entry = pipeline
        .texture_atlas_mut()
        .upload(
            &queue,
            request_path,
            upload.width,
            upload.height,
            &upload.rgba,
        )
        .expect("Aura9 RGBA data should upload into the atlas");
    assert_eq!(entry.original_width, 128);
    assert_eq!(entry.original_height, 128);

    let mut batch = QuadBatch::default();
    batch.push_textured_path(
        Rectangle::new(Point::ORIGIN, Size::new(128.0, 128.0)),
        request_path,
        [1.0, 1.0, 1.0, 1.0],
        BlendMode::Alpha,
    );
    let resolved = resolve_and_scale_quads(&mut pipeline, &batch, 1.0);
    assert_eq!(resolved.vertices.len(), 4);
    for vertex in &resolved.vertices {
        assert!(
            vertex.tex_coords[0] >= entry.uv_x
                && vertex.tex_coords[0] <= entry.uv_x + entry.uv_width
                && vertex.tex_coords[1] >= entry.uv_y
                && vertex.tex_coords[1] <= entry.uv_y + entry.uv_height,
            "request UV must resolve inside the Aura9 atlas slot"
        );
    }
}
#[test]
fn load_texture_prefer_bc_reuses_cached_crop_buffer() {
    let mut mgr = crate::texture::TextureManager::new();
    mgr.insert_test_texture(
        r"Interface\Foo\CropSource",
        crate::texture::TextureData {
            width: 8,
            height: 8,
            pixels: Arc::<[u8]>::from(vec![0xbb; 8 * 8 * 4]),
        },
    );

    let crop_path = r"Interface\Foo\CropSource@crop:0.250000,0.750000,0.250000,0.750000";
    let _ = mgr
        .load_sub_region(r"Interface\Foo\CropSource", 2, 2, 4, 4)
        .expect("crop should populate the sub-region cache");
    let cached_crop_ptr = mgr
        .load_sub_region(r"Interface\Foo\CropSource", 2, 2, 4, 4)
        .expect("crop should stay cached");
    let cached_crop_ptr = cached_crop_ptr.pixels.as_ptr();

    let loaded =
        load_texture_prefer_bc(&mut mgr, crop_path).expect("cached crop texture should load");
    let LoadedTexture::Rgba(upload) = loaded else {
        panic!("crop requests should stay on the RGBA upload path");
    };

    assert_eq!(
        upload.rgba.as_ptr(),
        cached_crop_ptr,
        "crop upload path should reuse cached crop pixels instead of cloning them"
    );
}

#[test]
fn load_texture_prefer_bc_cached_crop_request_skips_crop_decode_work() {
    let mut mgr = crate::texture::TextureManager::new();
    mgr.insert_test_texture(
        r"Interface\Foo\CropSource",
        crate::texture::TextureData {
            width: 8,
            height: 8,
            pixels: Arc::<[u8]>::from(vec![0xbb; 8 * 8 * 4]),
        },
    );

    let crop_path = r"Interface\Foo\CropSource@crop:0.250000,0.750000,0.250000,0.750000";
    let _ = load_texture_prefer_bc_with_telemetry(&mut mgr, crop_path)
        .0
        .expect("first crop request should populate the crop-request cache");

    let (loaded, telemetry) = load_texture_prefer_bc_with_telemetry(&mut mgr, crop_path);
    let LoadedTexture::Rgba(_) = loaded.expect("cached crop request should still load") else {
        panic!("crop requests should stay on the RGBA upload path");
    };

    assert_eq!(
        telemetry.crop_decode_elapsed,
        std::time::Duration::ZERO,
        "cached crop requests should bypass crop decoding entirely"
    );
    assert_eq!(
        telemetry.crop_extract_elapsed,
        std::time::Duration::ZERO,
        "cached crop requests should bypass sub-region extraction entirely"
    );
}

#[test]
fn remap_entry_uv_insets_slot_edges_by_half_texel() {
    let remap = UvRemap::entry_axis(0.25, 32.0 / 4096.0, 32, 0);
    let left = remap_entry_uv(0.0, remap);
    let right = remap_entry_uv(1.0, remap);

    assert!((left - (0.25 + 0.5 / 4096.0)).abs() < 1e-6);
    assert!((right - (0.25 + 31.5 / 4096.0)).abs() < 1e-6);
}

#[test]
fn tab_crop_texture_requests_disable_half_texel_inset() {
    let request = crate::render::TextureRequest::new(
        r"Interface\FrameGeneral\UIFrameTabs@crop:0.015625,0.593750,0.324219,0.488281",
        0,
        4,
    );

    assert!(
        !request.use_uv_inset,
        "tab atlas crops should preserve exact UV edges"
    );
}

#[test]
fn non_tab_texture_requests_keep_half_texel_inset() {
    let request = crate::render::TextureRequest::new(
        r"Interface\Buttons\UI-Panel-Button-Up@crop:0.000000,0.500000,0.000000,1.000000",
        0,
        4,
    );

    assert!(
        request.use_uv_inset,
        "non-tab texture crops should keep bleed protection"
    );
}

#[test]
fn texture_request_inset_policy_can_be_overridden() {
    let request = crate::render::TextureRequest::new_with_uv_inset(
        r"Interface\Buttons\UI-Panel-Button-Up@crop:0.000000,0.500000,0.000000,1.000000",
        0,
        4,
        false,
    );

    assert!(!request.use_uv_inset);
}

#[test]
fn resolved_tab_crop_uses_exact_atlas_slot_edges() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let (device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("adapter");
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("device")
    });

    let mut pipeline = WowUiPipeline::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb);
    let path = r"Interface\FrameGeneral\UIFrameTabs@crop:0.015625,0.593750,0.324219,0.488281";
    let entry = pipeline
        .texture_atlas_mut()
        .upload(&queue, path, 37, 42, &[0xff; 37 * 42 * 4])
        .expect("tab crop should upload into the atlas");

    let mut batch = QuadBatch::default();
    batch.push_textured_path(
        Rectangle::new(Point::ORIGIN, Size::new(37.0, 42.0)),
        path,
        [1.0, 1.0, 1.0, 1.0],
        BlendMode::Alpha,
    );

    let resolved = resolve_and_scale_quads(&mut pipeline, &batch, 1.0);
    let left = &resolved.vertices[0];
    let right = &resolved.vertices[1];

    assert_eq!(left.tex_coords[0], entry.uv_x);
    assert_eq!(right.tex_coords[0], entry.uv_x + entry.uv_width);
}

#[test]
fn bc_texture_dimensions_must_fit_bc_gpu_cell() {
    assert!(bc_texture_dimensions_fit_gpu_atlas(4, 4));
    assert!(bc_texture_dimensions_fit_gpu_atlas(
        crate::render::shader::atlas::BC_CELL_SIZE,
        crate::render::shader::atlas::BC_CELL_SIZE,
    ));
    assert!(!bc_texture_dimensions_fit_gpu_atlas(2, 4));
    assert!(!bc_texture_dimensions_fit_gpu_atlas(
        crate::render::shader::atlas::BC_CELL_SIZE + 4,
        4,
    ));
}

#[test]
fn unresolved_pending_textures_become_transparent() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let (device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("adapter");
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("device")
    });

    let mut pipeline = WowUiPipeline::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb);
    let mut batch = QuadBatch::default();
    batch.push_textured_path(
        Rectangle::new(Point::ORIGIN, Size::new(16.0, 16.0)),
        r"Interface\Missing\Pending",
        [1.0, 1.0, 1.0, 1.0],
        BlendMode::Alpha,
    );

    let resolved = resolve_and_scale_quads(&mut pipeline, &batch, 1.0);
    assert!(
        resolved
            .vertices
            .iter()
            .all(|vertex| vertex.tex_index == -1)
    );
    assert!(
        resolved
            .vertices
            .iter()
            .all(|vertex| vertex.color[3] == 0.0)
    );
}

#[test]
fn prepare_bc_upload_failure_keeps_path_unready_and_requests_rgba_retry() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let (device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("adapter");
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("device")
    });

    let mut pipeline = WowUiPipeline::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb);
    if crate::render::shader::atlas::is_bc_supported() {
        let bc_blocks = vec![
            0u8;
            ((crate::render::shader::atlas::BC_CELL_SIZE / 4)
                * (crate::render::shader::atlas::BC_CELL_SIZE / 4)
                * 8) as usize
        ];
        let mut slot = 0usize;
        while pipeline
            .texture_atlas_mut()
            .upload_bc(
                &queue,
                &format!("filled-bc-slot-{slot}"),
                crate::render::shader::atlas::BC_CELL_SIZE,
                crate::render::shader::atlas::BC_CELL_SIZE,
                &bc_blocks,
                BcFormat::Bc1,
            )
            .is_some()
        {
            slot += 1;
        }
    }

    let path = r"Interface\WorldMap\Test\Tile01".to_string();
    let request = crate::render::TextureRequest::new(path.clone(), 0, 4);

    let mut primitive = WowUiPrimitive::empty();
    primitive.bc_textures.push(crate::render::GpuBcTextureData {
        path: path.clone(),
        width: crate::render::shader::atlas::BC_CELL_SIZE,
        height: crate::render::shader::atlas::BC_CELL_SIZE,
        bc_data: Arc::<[u8]>::from(vec![
            0u8;
            ((crate::render::shader::atlas::BC_CELL_SIZE / 4)
                * (crate::render::shader::atlas::BC_CELL_SIZE / 4)
                * 8) as usize
        ]),
        bc_format: BcFormat::Bc1,
    });
    let tracker = Arc::new(Mutex::new(super::TextureRequestTracker::default()));
    tracker.lock().unwrap().register_request(&request);
    primitive.texture_requests = Some(Arc::clone(&tracker));

    <WowUiPrimitive as ShaderPrimitive>::prepare(
        &primitive,
        &mut pipeline,
        &device,
        &queue,
        &Rectangle::new(Point::ORIGIN, Size::new(64.0, 64.0)),
        &Viewport::with_physical_size(Size::new(64, 64), 1.0),
    );

    assert!(request.handle.is_pending(), "failed uploads stay pending");
    assert!(
        request.handle.needs_force_rgba(),
        "failed BC atlas uploads should retry through the RGBA atlas"
    );
    assert!(
        !request.handle.is_staged(),
        "failed BC atlas uploads must clear staged state so draw can retry"
    );
    assert!(
        !request.handle.is_ready(),
        "failed BC atlas uploads must not be marked ready"
    );
    assert!(
        pipeline.texture_atlas_mut().get(&path).is_none()
            && pipeline.texture_atlas_mut().get_bc(&path).is_none(),
        "rejected BC uploads must not appear in the atlas"
    );
}

#[test]
fn resolved_textures_remap_quad_uvs_into_atlas_slot() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let (device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("adapter");
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("device")
    });

    let mut pipeline = WowUiPipeline::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb);
    let path = r"Interface\Foo\Resolved";
    let entry = pipeline
        .texture_atlas_mut()
        .upload(&queue, path, 16, 16, &[0xff; 16 * 16 * 4])
        .expect("texture should upload into the atlas");

    let mut batch = QuadBatch::default();
    batch.push_textured_path(
        Rectangle::new(Point::ORIGIN, Size::new(16.0, 16.0)),
        path,
        [1.0, 1.0, 1.0, 1.0],
        BlendMode::Alpha,
    );

    let resolved = resolve_and_scale_quads(&mut pipeline, &batch, 1.0);
    for vertex in &resolved.vertices {
        let expected_u = remap_entry_uv(
            vertex.local_uv[0],
            UvRemap::entry_axis(entry.uv_x, entry.uv_width, entry.original_width, entry.tier),
        );
        let expected_v = remap_entry_uv(
            vertex.local_uv[1],
            UvRemap::entry_axis(
                entry.uv_y,
                entry.uv_height,
                entry.original_height,
                entry.tier,
            ),
        );
        assert_eq!(vertex.tex_index, entry.tex_index());
        assert!((vertex.tex_coords[0] - expected_u).abs() < 1e-6);
        assert!((vertex.tex_coords[1] - expected_v).abs() < 1e-6);
    }
}

#[test]
fn resolved_cooldown_swipe_preserves_progress_and_remaps_sample_uvs() {
    let entry = crate::render::shader::atlas::TextureEntry {
        tier: 0,
        grid_x: 2,
        grid_y: 3,
        original_width: 64,
        original_height: 32,
        uv_x: 0.25,
        uv_y: 0.5,
        uv_width: 0.125,
        uv_height: 0.0625,
    };
    let flags = BlendMode::Alpha as u32 | FLAG_COOLDOWN_SWIPE;
    let mut vertices = [
        QuadVertex {
            tex_coords: [0.25, 0.0],
            flags,
            mask_tex_coords: [0.2, 0.3],
            tex_index: -2,
            ..QuadVertex::zeroed()
        },
        QuadVertex {
            tex_coords: [0.25, 0.0],
            flags,
            mask_tex_coords: [0.8, 0.9],
            tex_index: -2,
            ..QuadVertex::zeroed()
        },
    ];

    apply_resolved_texture_entry(&mut vertices, ResolvedTextureEntry::Rgba(entry), true);

    assert_eq!(vertices[0].tex_index, entry.tex_index());
    assert_eq!(vertices[1].tex_index, entry.tex_index());
    assert_eq!(
        vertices[0].tex_coords,
        [0.25, 0.0],
        "atlas resolution must not reinterpret cooldown progress as a texture U"
    );
    assert_eq!(
        vertices[1].tex_coords,
        [0.25, 0.0],
        "all cooldown vertices must retain the same radial progress"
    );
    assert!(
        (vertices[0].mask_tex_coords[0]
            - remap_entry_uv(
                0.2,
                UvRemap::entry_axis(entry.uv_x, entry.uv_width, entry.original_width, entry.tier),
            ))
        .abs()
            < 1e-6
    );
    assert!(
        (vertices[0].mask_tex_coords[1]
            - remap_entry_uv(
                0.3,
                UvRemap::entry_axis(
                    entry.uv_y,
                    entry.uv_height,
                    entry.original_height,
                    entry.tier
                ),
            ))
        .abs()
            < 1e-6
    );
    assert!(
        (vertices[1].mask_tex_coords[0]
            - remap_entry_uv(
                0.8,
                UvRemap::entry_axis(entry.uv_x, entry.uv_width, entry.original_width, entry.tier),
            ))
        .abs()
            < 1e-6
    );
    assert!(
        (vertices[1].mask_tex_coords[1]
            - remap_entry_uv(
                0.9,
                UvRemap::entry_axis(
                    entry.uv_y,
                    entry.uv_height,
                    entry.original_height,
                    entry.tier
                ),
            ))
        .abs()
            < 1e-6
    );
}

#[test]
fn resolved_bc_entries_remap_quad_uvs_into_bc_slot() {
    let bc_entry = BcTextureEntry {
        format: BcFormat::Bc3,
        grid_x: 0,
        grid_y: 0,
        original_width: 128,
        original_height: 64,
        uv_x: 0.25,
        uv_y: 0.5,
        uv_width: 0.125,
        uv_height: 0.25,
    };
    let mut vertices = [
        QuadVertex {
            tex_coords: [0.0, 0.0],
            local_uv: [0.0, 0.0],
            tex_index: -2,
            ..QuadVertex::zeroed()
        },
        QuadVertex {
            tex_coords: [1.0, 1.0],
            local_uv: [1.0, 1.0],
            tex_index: -2,
            ..QuadVertex::zeroed()
        },
    ];

    apply_resolved_texture_entry(&mut vertices, ResolvedTextureEntry::Bc(bc_entry), true);

    assert_eq!(vertices[0].tex_index, bc_entry.tex_index());
    assert_eq!(vertices[1].tex_index, bc_entry.tex_index());
    assert!((vertices[0].tex_coords[0] - remap_bc_entry_uv(0.0, 0.25, 0.125, 128)).abs() < 1e-6);
    assert!((vertices[0].tex_coords[1] - remap_bc_entry_uv(0.0, 0.5, 0.25, 64)).abs() < 1e-6);
    assert!((vertices[1].tex_coords[0] - remap_bc_entry_uv(1.0, 0.25, 0.125, 128)).abs() < 1e-6);
    assert!((vertices[1].tex_coords[1] - remap_bc_entry_uv(1.0, 0.5, 0.25, 64)).abs() < 1e-6);
}

#[test]
fn resolved_cooldown_swipe_preserves_progress_and_remaps_bc_sample_uvs() {
    let entry = BcTextureEntry {
        format: BcFormat::Bc3,
        grid_x: 1,
        grid_y: 2,
        original_width: 64,
        original_height: 32,
        uv_x: 0.25,
        uv_y: 0.5,
        uv_width: 0.125,
        uv_height: 0.0625,
    };
    let flags = BlendMode::Alpha as u32 | FLAG_COOLDOWN_SWIPE;
    let mut vertices = [
        QuadVertex {
            tex_coords: [0.75, 0.0],
            flags,
            mask_tex_coords: [0.2, 0.3],
            tex_index: -2,
            ..QuadVertex::zeroed()
        },
        QuadVertex {
            tex_coords: [0.75, 0.0],
            flags,
            mask_tex_coords: [0.8, 0.9],
            tex_index: -2,
            ..QuadVertex::zeroed()
        },
    ];

    apply_resolved_texture_entry(&mut vertices, ResolvedTextureEntry::Bc(entry), true);

    assert_eq!(vertices[0].tex_index, entry.tex_index());
    assert_eq!(vertices[1].tex_index, entry.tex_index());
    assert_eq!(
        vertices[0].tex_coords,
        [0.75, 0.0],
        "BC atlas resolution must not reinterpret cooldown progress as a texture U"
    );
    assert_eq!(
        vertices[1].tex_coords,
        [0.75, 0.0],
        "all BC cooldown vertices must retain the same radial progress"
    );
    assert!(
        (vertices[0].mask_tex_coords[0]
            - remap_bc_entry_uv(0.2, entry.uv_x, entry.uv_width, entry.original_width))
        .abs()
            < 1e-6
    );
    assert!(
        (vertices[0].mask_tex_coords[1]
            - remap_bc_entry_uv(0.3, entry.uv_y, entry.uv_height, entry.original_height))
        .abs()
            < 1e-6
    );
    assert!(
        (vertices[1].mask_tex_coords[0]
            - remap_bc_entry_uv(0.8, entry.uv_x, entry.uv_width, entry.original_width))
        .abs()
            < 1e-6
    );
    assert!(
        (vertices[1].mask_tex_coords[1]
            - remap_bc_entry_uv(0.9, entry.uv_y, entry.uv_height, entry.original_height))
        .abs()
            < 1e-6
    );
}

#[test]
fn pending_transition_reuploads_strata_vertices_with_resolved_tex_indices_after_ready() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let (device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("adapter");
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("device")
    });

    let mut pipeline = WowUiPipeline::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb);
    let path = r"Interface\WorldMap\Test\TileReady".to_string();
    let mut batch = QuadBatch::default();
    batch.push_textured_path(
        Rectangle::new(Point::ORIGIN, Size::new(16.0, 16.0)),
        &path,
        [1.0, 1.0, 1.0, 1.0],
        BlendMode::Alpha,
    );
    let request = batch.texture_requests[0].clone();
    let batch = Arc::new(batch);

    let mut first = WowUiPrimitive::empty();
    first.strata_batches[0] = Some(Arc::clone(&batch));
    <WowUiPrimitive as ShaderPrimitive>::prepare(
        &first,
        &mut pipeline,
        &device,
        &queue,
        &Rectangle::new(Point::ORIGIN, Size::new(64.0, 64.0)),
        &Viewport::with_physical_size(Size::new(64, 64), 1.0),
    );

    let pending_vertices = pipeline.uploaded_vertices(0).to_vec();
    assert_eq!(pending_vertices.len(), batch.vertices.len());
    assert!(
        pending_vertices.iter().all(|vertex| vertex.tex_index == -1),
        "unresolved first-open vertices should upload as hidden placeholders first"
    );
    assert!(
        pending_vertices.iter().all(|vertex| vertex.color[3] == 0.0),
        "pending first-open vertices should be transparent until the atlas entry exists"
    );

    let mut second = WowUiPrimitive::empty();
    second.strata_batches[0] = Some(Arc::clone(&batch));
    second.textures.push(crate::render::GpuTextureData {
        path: path.clone(),
        width: 16,
        height: 16,
        rgba: Arc::<[u8]>::from(vec![0xff; 16 * 16 * 4]),
    });
    let tracker = Arc::new(Mutex::new(super::TextureRequestTracker::default()));
    tracker.lock().unwrap().register_request(&request);
    second.texture_requests = Some(Arc::clone(&tracker));
    <WowUiPrimitive as ShaderPrimitive>::prepare(
        &second,
        &mut pipeline,
        &device,
        &queue,
        &Rectangle::new(Point::ORIGIN, Size::new(64.0, 64.0)),
        &Viewport::with_physical_size(Size::new(64, 64), 1.0),
    );

    let entry = pipeline
        .texture_atlas_mut()
        .get(&path)
        .copied()
        .expect("second prepare should upload the pending texture into the atlas");
    let uploaded_vertices = pipeline.uploaded_vertices(0);
    assert_eq!(uploaded_vertices.len(), batch.vertices.len());
    assert!(
        request.handle.is_ready(),
        "successful prepare should mark the path atlas-ready"
    );
    assert!(
        !request.handle.needs_force_rgba(),
        "plain RGBA uploads should not request BC fallback handling"
    );

    for (resolved, pending) in uploaded_vertices.iter().zip(&pending_vertices) {
        let expected_u = remap_entry_uv(
            resolved.local_uv[0],
            UvRemap::entry_axis(entry.uv_x, entry.uv_width, entry.original_width, entry.tier),
        );
        let expected_v = remap_entry_uv(
            resolved.local_uv[1],
            UvRemap::entry_axis(
                entry.uv_y,
                entry.uv_height,
                entry.original_height,
                entry.tier,
            ),
        );
        assert_eq!(resolved.tex_index, entry.tex_index());
        assert!((resolved.tex_coords[0] - expected_u).abs() < 1e-6);
        assert!((resolved.tex_coords[1] - expected_v).abs() < 1e-6);
        assert!(
            resolved.color[3] > 0.0,
            "resolved upload should restore visible vertex alpha"
        );
        assert_ne!(
            resolved.tex_index, pending.tex_index,
            "the retained strata slot must be rewritten, not left on the hidden placeholder state"
        );
    }
}
