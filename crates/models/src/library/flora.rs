//! Forest recipes shared by the battlefield and Flora gallery.
use bevy::prelude::*;
use bevy_flora::{
    FoliagePlacement, LodDistances, LodSettings, RenderStyle, TreeParams, TreePool, TreePreset,
};

pub const NAMES: [&str; 3] = ["flora_tree", "flora_palm", "flora_shrub"];
pub const VARIANTS: u32 = 6;

pub fn recipe(name: &str, seed: u64) -> TreeParams {
    let mut params = match name {
        "flora_tree" => TreePreset::Oak.params(),
        "flora_palm" => TreePreset::Palm.params(),
        "flora_shrub" => TreePreset::Shrub.params(),
        _ => panic!("unknown Flora recipe {name}"),
    };
    if name == "flora_palm" {
        // Leaflets along the fronds read as a palm at the battlefield's overhead angle.
        // The stock preset puts a few large oval cards only at branch tips.
        params.branches.radius_ratio = 0.15;
        params.branches.recursion = 0;
        params.branches.sub_branch_count = 0;
        params.foliage.placement = FoliagePlacement::AllBranches;
        params.foliage.density = 24.;
        params.foliage.leaf_size = 0.3;
        params.foliage.radius_threshold = 1.;
    }
    if name == "flora_tree" {
        params.foliage.density *= 3.;
    }
    params.seed = seed as i32;
    params.style = RenderStyle::Painterly;
    params.foliage.color = Color::linear_rgb(0.075, 0.19, 0.035);
    params.trunk_color = Color::linear_rgb(0.16, 0.10, 0.045);
    params
}

pub fn pool(name: &str, seed: u64, meshes: &mut Assets<Mesh>) -> TreePool {
    let lod = LodSettings {
        distances: LodDistances {
            lod1: 20.,
            lod2: 35.,
            impostor: if cfg!(target_arch = "wasm32") {
                f32::INFINITY
            } else {
                55.
            },
        },
        impostor_elevation: (40_f32 / 28.).atan().to_degrees(),
        impostor_shadows: true,
        lod2_leaf_keep: if name == "flora_shrub" { 8 } else { 3 },
        ..default()
    };
    bevy_flora::build_pool(&recipe(name, seed), VARIANTS, f32::INFINITY, &lod, meshes)
}

/// Keep the trunk at its placement anchor and fit crowns to the existing scenery envelope.
/// A child transform performs this fit so visibility logic can retain a uniform root scale.
pub fn fitted_transform(pool: &TreePool, variant: u32, envelope: Vec3) -> Transform {
    let bounds = pool.variants[variant as usize % pool.variants.len()].bounds[0];
    let center: Vec3 = bounds.center.into();
    let half: Vec3 = bounds.half_extents.into();
    let horizontal = envelope.x.min(envelope.z) / (2. * half.x.max(half.z)).max(0.001);
    // Preserve proportions: stretching after baking would invalidate the atlas's view angle.
    let scale = Vec3::splat(horizontal.min(envelope.y / (2. * half.y).max(0.001)));
    Transform::from_translation(-Vec3::new(center.x, center.y - half.y, center.z) * scale)
        .with_scale(scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recipes_are_repeatable_and_fit_their_envelopes() {
        for name in NAMES {
            let params = recipe(name, 1993);
            let a = bevy_flora::build_tree(&params, f32::INFINITY);
            let b = bevy_flora::build_tree(&recipe(name, 1993), f32::INFINITY);
            assert_eq!(a.trunk, b.trunk);
            assert_eq!(a.foliage, b.foliage);
            let mut meshes = Assets::<Mesh>::default();
            let pool = pool(name, 1993, &mut meshes);
            for k in 0..VARIANTS {
                let bounds = pool.variants[k as usize].bounds[0];
                let fit = fitted_transform(&pool, k, Vec3::new(1.5, 1.2, 1.5));
                let lo = fit.transform_point((bounds.center - bounds.half_extents).into());
                let hi = fit.transform_point((bounds.center + bounds.half_extents).into());
                assert!(lo.y.abs() < 1e-4);
                assert!(hi.y <= 1.201 && hi.y > 0.);
                assert!(lo.x >= -0.751 && hi.x <= 0.751);
                assert!(lo.z >= -0.751 && hi.z <= 0.751);
            }
        }
    }
}
