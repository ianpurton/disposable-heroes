//! Small seamless material maps generated in memory, shared by runtime and export.
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// Neutral weathering: preserves the linear material palette while breaking up flat surfaces.
pub fn surface_texture() -> Image {
    let size = 256;
    let mut pixels = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let u = x as f32 * std::f32::consts::TAU / size as f32;
            let v = y as f32 * std::f32::consts::TAU / size as f32;
            let h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
            let h = (h ^ (h >> 13)).wrapping_mul(1274126177);
            let grain = (h & 255) as f32 / 255.;
            let stain = (u * 3. + (v * 2.).sin()).sin() * (v * 5. + u.cos()).cos();
            let fiber = (u * 31. + (v * 3.).sin() * 2.).sin().abs();
            let value = (0.82 + stain * 0.10 + fiber * 0.04 + grain * 0.035).clamp(0., 1.);
            let c = (value * 255.) as u8;
            pixels.extend([c, c, c, 255]);
        }
    }
    repeat_image(size as u32, pixels, true)
}

pub(crate) fn repeat_image(size: u32, data: Vec<u8>, srgb: bool) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        if srgb {
            TextureFormat::Rgba8UnormSrgb
        } else {
            TextureFormat::Rgba8Unorm
        },
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..default()
    });
    image
}

/// Derive shallow, wrapping soil relief from the generated albedo, without changing collision.
pub fn ground_normal_map(albedo: &Image) -> Image {
    let size = albedo.width() as usize;
    let Some(data) = albedo.data.as_ref() else {
        return repeat_image(1, vec![128, 128, 255, 255], false);
    };
    let height = |x: usize, y: usize| {
        let p = ((y % size) * size + x % size) * 4;
        (data[p] as f32 + data[p + 1] as f32 + data[p + 2] as f32) / 765.
    };
    let mut pixels = Vec::with_capacity(data.len());
    for y in 0..size {
        for x in 0..size {
            let dx = height(x + 1, y) - height(x + size - 1, y);
            let dy = height(x, y + 1) - height(x, y + size - 1);
            let n = Vec3::new(-dx * 0.6, -dy * 0.6, 1.).normalize();
            pixels.extend(n.to_array().map(|v| ((v * 0.5 + 0.5) * 255.) as u8));
            pixels.push(255);
        }
    }
    repeat_image(size as u32, pixels, false)
}
