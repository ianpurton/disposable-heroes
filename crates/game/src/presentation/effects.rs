//! Bounded, camera-facing combat effects. Materials are shared by lifetime stage.
use super::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Clone, Copy)]
pub(super) enum Style {
    Flame,
    Smoke,
    Spark,
    Debris,
    Scorch,
}
#[derive(Component)]
pub(super) struct Particle {
    pub velocity: Vec3,
    pub remaining: f32,
    pub total: f32,
    style: Style,
    size: f32,
    spin: f32,
}
pub(super) struct EffectAssets {
    quad: Handle<Mesh>,
    flame: Vec<Handle<StandardMaterial>>,
    smoke: Vec<Handle<StandardMaterial>>,
    scorch: Vec<Handle<StandardMaterial>>,
    spark: Handle<StandardMaterial>,
    debris: Handle<StandardMaterial>,
    fragment: Handle<Mesh>,
}
fn texture(smoke: bool, scorch: bool) -> Image {
    let size = 128;
    let mut pixels = vec![];
    for y in 0..size {
        for x in 0..size {
            let u = x as f32 / (size - 1) as f32 * 2. - 1.;
            let v = y as f32 / (size - 1) as f32 * 2. - 1.;
            let r = (u * u + v * v).sqrt();
            let noise = (u * 17. + (v * 13.).sin()).sin() * (v * 19. + (u * 11.).cos()).cos();
            let edge = (1. - r + noise * if scorch { 0.06 } else { 0.13 }).clamp(0., 1.);
            let alpha = edge.powf(if smoke { 0.85 } else { 1.4 }) * ((1. - r) * 12.).clamp(0., 1.);
            let rgb = if scorch {
                [28., 23., 17.]
            } else if smoke {
                let light = 105. + v * 25. + noise * 23.;
                [light, light * 0.98, light * 0.94]
            } else {
                [255., 85. + edge * 170., 12. + edge.powi(3) * 220.]
            };
            pixels.extend([
                rgb[0] as u8,
                rgb[1] as u8,
                rgb[2] as u8,
                (alpha * 255.) as u8,
            ]);
        }
    }
    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
impl EffectAssets {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
    ) -> Self {
        let mut stages = |smoke, scorch| {
            let image = images.add(texture(smoke, scorch));
            (0..16)
                .map(|i| {
                    materials.add(StandardMaterial {
                        base_color: Color::srgba(
                            1.,
                            1.,
                            1.,
                            (1. - i as f32 / 15.).powf(0.7)
                                * if smoke { (i as f32 / 3.).min(1.) } else { 1. },
                        ),
                        base_color_texture: Some(image.clone()),
                        emissive_texture: if smoke || scorch {
                            None
                        } else {
                            Some(image.clone())
                        },
                        emissive: if smoke || scorch {
                            LinearRgba::BLACK
                        } else {
                            LinearRgba::rgb(12., 7., 2.)
                        },
                        alpha_mode: AlphaMode::Blend,
                        unlit: smoke || scorch,
                        cull_mode: None,
                        depth_bias: if scorch { 1. } else { 0. },
                        ..default()
                    })
                })
                .collect()
        };
        let flame = stages(false, false);
        let smoke = stages(true, false);
        let scorch = stages(false, true);
        Self {
            quad: meshes.add(Rectangle::new(1., 1.)),
            fragment: meshes.add(Cuboid::new(1., 0.4, 0.6)),
            debris: materials.add(StandardMaterial {
                base_color: Color::srgb(0.13, 0.10, 0.07),
                perceptual_roughness: 0.9,
                ..default()
            }),
            flame,
            smoke,
            scorch,
            spark: materials.add(StandardMaterial {
                base_color: Color::srgb(1., 0.58, 0.08),
                emissive: LinearRgba::rgb(9., 3., 0.1),
                unlit: false,
                ..default()
            }),
        }
    }
    fn material(&self, style: Style, age: f32) -> Handle<StandardMaterial> {
        let i = (age.clamp(0., 1.) * 15.) as usize;
        match style {
            Style::Flame => &self.flame[i],
            Style::Smoke => &self.smoke[i],
            Style::Scorch => &self.scorch[i],
            Style::Spark => &self.spark,
            Style::Debris => &self.debris,
        }
        .clone()
    }
    fn spawn(
        &self,
        commands: &mut Commands,
        lib: &Library,
        p: Vec3,
        velocity: Vec3,
        style: Style,
        size: f32,
        life: f32,
        spin: f32,
    ) {
        let rotation = match style {
            Style::Scorch => Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            _ => {
                Quat::from_rotation_arc(Vec3::Z, Vec3::new(0., 40., 28.).normalize())
                    * Quat::from_rotation_z(spin)
            }
        };
        commands.spawn((
            LevelEntity,
            Mesh3d(if matches!(style, Style::Debris) {
                self.fragment.clone()
            } else if matches!(style, Style::Spark) {
                lib.sphere.clone()
            } else {
                self.quad.clone()
            }),
            MeshMaterial3d(self.material(style, 0.)),
            Transform::from_translation(p)
                .with_rotation(rotation)
                .with_scale(Vec3::splat(size)),
            bevy::light::NotShadowCaster,
            bevy::light::NotShadowReceiver,
            Particle {
                velocity,
                remaining: life,
                total: life,
                style,
                size,
                spin,
            },
        ));
    }
    pub fn muzzle(&self, commands: &mut Commands, lib: &Library, p: Vec3, large: bool) {
        self.spawn(
            commands,
            lib,
            p,
            Vec3::ZERO,
            Style::Flame,
            if large { 0.75 } else { 0.46 },
            0.09,
            0.,
        );
    }
    pub fn burst(
        &self,
        commands: &mut Commands,
        lib: &Library,
        p: Vec3,
        explosion: bool,
        budget: &mut usize,
    ) {
        let count = if explosion {
            if cfg!(target_arch = "wasm32") { 26 } else { 48 }
        } else {
            4
        };
        for i in 0..count {
            if *budget == 0 {
                break;
            }
            *budget -= 1;
            let a = i as f32 * 2.399;
            let (style, size, life, speed, rise) = if !explosion {
                (Style::Smoke, 0.22, 0.45, 0.45, 0.5)
            } else if i == 0 {
                (Style::Scorch, 3.8, 14., 0., 0.)
            } else if i < 16 {
                (
                    Style::Flame,
                    2.0 + (i % 4) as f32 * 0.28,
                    0.70 + (i % 4) as f32 * 0.15,
                    1.4 + (i % 3) as f32,
                    1.0 + (i % 5) as f32 * 0.4,
                )
            } else if i < 34 {
                (
                    Style::Smoke,
                    1.6 + (i % 4) as f32 * 0.25,
                    3.0 + (i % 5) as f32 * 0.24,
                    0.6 + (i % 3) as f32 * 0.35,
                    1.4 + (i % 4) as f32 * 0.4,
                )
            } else if i >= 40 {
                (Style::Debris, 0.14, 1.2, 3.5, 3.5)
            } else {
                (
                    Style::Spark,
                    0.045 + (i % 3) as f32 * 0.015,
                    0.8 + (i % 4) as f32 * 0.15,
                    2.5 + (i % 5) as f32,
                    2. + (i % 4) as f32,
                )
            };
            self.spawn(
                commands,
                lib,
                p + Vec3::Y * if i == 0 && explosion { 0.018 } else { 0.4 },
                Vec3::new(a.cos() * speed, rise, a.sin() * speed),
                style,
                size,
                life,
                a,
            );
        }
    }
}
pub(super) fn update(
    mut commands: Commands,
    time: Res<Time>,
    lib: Res<Library>,
    mut q: Query<(
        Entity,
        &mut Particle,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let dt = time.delta_secs();
    for (e, mut p, mut t, mut material) in &mut q {
        p.remaining -= dt;
        if p.remaining <= 0. {
            commands.entity(e).despawn();
            continue;
        }
        let age = 1. - p.remaining / p.total;
        t.translation += p.velocity * dt;
        match p.style {
            Style::Spark | Style::Debris => {
                p.velocity.y -= 7. * dt;
                t.rotate_local_x(dt * 4.);
                t.rotate_local_z(dt * p.spin);
                t.scale = Vec3::splat(p.size * (1. - age));
            }
            Style::Scorch => {}
            Style::Smoke => {
                p.velocity *= (-0.65 * dt).exp();
                t.scale = Vec3::splat(p.size * (0.5 + age * 2.3));
                t.rotate_local_z(dt * p.spin.sin() * 0.15);
            }
            Style::Flame => {
                p.velocity *= (-1.8 * dt).exp();
                t.scale = Vec3::splat(p.size * (0.6 + age * 0.8));
            }
        }
        material.0 = lib.effects.material(p.style, age);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_textures_have_clear_borders() {
        for (smoke, scorch) in [(false, false), (true, false), (false, true)] {
            let image = texture(smoke, scorch);
            let pixels = image.data.unwrap();
            for i in 0..128 {
                for (x, y) in [(i, 0), (i, 127), (0, i), (127, i)] {
                    assert_eq!(pixels[(y * 128 + x) * 4 + 3], 0);
                }
            }
            assert!(pixels.chunks_exact(4).any(|p| p[3] > 200));
        }
    }

    #[test]
    fn bursts_respect_budget_and_all_particles_expire() {
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let mut images = Assets::<Image>::default();
        let effects = EffectAssets::new(&mut meshes, &mut materials, &mut images);
        let library = Library {
            effects,
            sphere: meshes.add(Sphere::new(1.)),
            ground: Handle::default(),
            ground_normal: Handle::default(),
            water: Handle::default(),
            models: HashMap::new(),
            animations: HashMap::new(),
            shot: Handle::default(),
            boom: Handle::default(),
        };
        let mut app = App::new();
        app.insert_resource(library)
            .insert_resource(Time::<()>::default())
            .add_systems(Update, update);
        let material_count = materials.len();
        app.world_mut().resource_scope(|world, lib: Mut<Library>| {
            let mut budget = 48;
            lib.effects
                .burst(&mut world.commands(), &lib, Vec3::ZERO, true, &mut budget);
            assert_eq!(budget, 0);
            lib.effects
                .burst(&mut world.commands(), &lib, Vec3::ZERO, true, &mut budget);
        });
        app.world_mut().flush();
        assert_eq!(
            app.world_mut()
                .query::<&Particle>()
                .iter(app.world())
                .count(),
            48
        );
        for _ in 0..960 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(1. / 60.));
            app.update();
        }
        assert_eq!(
            app.world_mut()
                .query::<&Particle>()
                .iter(app.world())
                .count(),
            0
        );
        assert_eq!(materials.len(), material_count);
    }
}
