//! Authoritative procedural models, shared by the game, gallery and optional exporter.
use bevy::{
    animation::{
        AnimatedBy, AnimationTargetId, animated_field,
        animation_curves::{AnimatableCurve, AnimatableKeyframeCurve},
    },
    mesh::{
        VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
    prelude::*,
};
use std::collections::HashMap;
#[cfg(all(feature = "export", not(target_arch = "wasm32")))]
pub mod export;
pub mod geometry;
mod library;
#[path = "../../seed.rs"]
mod seed;
pub const DEFAULT_SEED: u64 = 1993;
pub const NAMES: [&str; 15] = [
    "bridge",
    "bunker",
    "bush",
    "crate",
    "enemy",
    "grass",
    "hut",
    "jeep",
    "palm",
    "rock",
    "soldier",
    "thicket",
    "thicket_edge",
    "tree",
    "wreck",
];
pub const CLIPS: [&str; 7] = ["Idle", "Run", "Fire", "Throw", "Rocket", "Swim", "Death"];
pub const BONES: [&str; 6] = ["Root", "Head", "LeftLeg", "RightLeg", "LeftArm", "RightArm"];
#[derive(Clone, Copy, Debug)]
pub struct GenerationOptions {
    pub seed: u64,
}
impl Default for GenerationOptions {
    fn default() -> Self {
        Self { seed: DEFAULT_SEED }
    }
}
#[derive(Clone, Debug)]
pub struct Part {
    pub name: String,
    pub geometry: geometry::Geometry,
    pub color: [f32; 3],
    pub roughness: f32,
    pub joint: Option<usize>,
}
#[derive(Clone, Debug)]
pub struct Model {
    pub name: String,
    pub parts: Vec<Part>,
    pub rig: Option<Rig>,
}
#[derive(Clone, Debug)]
pub struct Rig {
    pub rest: [Transform; 6],
}
impl Rig {
    pub fn new() -> Self {
        Self {
            rest: [
                Transform::from_xyz(0., 0.75, 0.),
                Transform::from_xyz(0., 0.45, 0.),
                Transform::from_xyz(-0.17, 0.03, 0.)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
                Transform::from_xyz(0.17, 0.03, 0.)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
                Transform::from_xyz(-0.32, 0.4, 0.).with_rotation(Quat::from_rotation_x(2.5213432)),
                Transform::from_xyz(0.32, 0.4, 0.).with_rotation(Quat::from_rotation_x(2.5213432)),
            ],
        }
    }
    pub fn inverse_bind(&self) -> Vec<Mat4> {
        self.rest
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if i == 0 {
                    t.to_matrix().inverse()
                } else {
                    (self.rest[0].to_matrix() * t.to_matrix()).inverse()
                }
            })
            .collect()
    }
    pub fn pose(&self, clip: &str, frame: usize) -> [Transform; 6] {
        let mut pose = self.rest;
        let phase = frame as f32 / 24. * std::f32::consts::TAU;
        for (i, t) in pose.iter_mut().enumerate() {
            let n = BONES[i];
            let mut r = Vec3::ZERO;
            let mut y = 0.;
            match clip {
                "Run" => {
                    if n.contains("Leg") {
                        r.x = phase.sin() * 0.65 * if n.contains("Left") { 1. } else { -1. };
                    }
                    if n.contains("Arm") {
                        r.x = phase.sin() * 0.3 * if n.contains("Right") { 1. } else { -1. };
                    }
                    if i == 0 {
                        y = phase.sin().abs() * 0.055;
                    }
                }
                "Idle" if i == 0 => y = phase.sin() * 0.012,
                "Fire" | "Rocket" if n.contains("Arm") => r.x = -0.6 + phase.sin() * 0.06,
                "Throw" if n == "RightArm" => {
                    r.x = -(frame as f32 / 24. * std::f32::consts::PI).sin() * 2.
                }
                "Swim" if n.contains("Arm") => r.z = phase.sin() * 0.8,
                "Death" if i == 0 => {
                    let v = (frame as f32 / 16.).min(1.);
                    r.x = v * 1.5;
                    y = -v * 0.6;
                }
                _ => {}
            }
            // Blender pose channels are in each bone's local coordinates.
            t.translation += t.rotation * Vec3::new(0., y, 0.);
            t.rotation *= Quat::from_euler(EulerRot::XYZ, r.x, r.y, r.z);
        }
        pose
    }
}
impl Default for Rig {
    fn default() -> Self {
        Self::new()
    }
}
pub fn target(index: usize) -> AnimationTargetId {
    let mut path = vec!["SoldierRig", "Root"];
    if index > 0 {
        path.push(BONES[index]);
    }
    AnimationTargetId::from_iter(path)
}
impl Model {
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = -lo;
        for p in &self.parts {
            for v in &p.geometry.positions {
                let v = Vec3::from_array(*v);
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        (lo, hi)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.parts.is_empty() {
            return Err(format!("{}: empty model", self.name));
        }
        for p in &self.parts {
            let g = &p.geometry;
            if g.positions.is_empty()
                || g.positions.len() != g.normals.len()
                || g.indices.len() % 3 != 0
                || g.positions.iter().flatten().any(|v| !v.is_finite())
                || g.indices.iter().any(|i| *i as usize >= g.positions.len())
                || p.joint.is_some_and(|i| i >= 6 || self.rig.is_none())
            {
                return Err(format!("{}: invalid part {}", self.name, p.name));
            }
        }
        Ok(())
    }
}
pub fn generate(name: &str, options: GenerationOptions) -> Result<Model, String> {
    if !NAMES.contains(&name) {
        return Err(format!("unknown model {name}"));
    }
    let model = library::build(name, options.seed);
    model.validate()?;
    Ok(model)
}
pub struct PreparedPart {
    name: String,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}
pub struct PreparedModel {
    pub definition: Model,
    parts: Vec<PreparedPart>,
    bind: Option<Handle<SkinnedMeshInverseBindposes>>,
    pub clips: HashMap<String, Handle<AnimationClip>>,
}
pub struct SpawnedModel {
    pub root: Entity,
    pub player: Option<Entity>,
}
impl PreparedModel {
    pub fn new(
        definition: Model,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        binds: &mut Assets<SkinnedMeshInverseBindposes>,
        clips: &mut Assets<AnimationClip>,
    ) -> Self {
        let parts = definition
            .parts
            .iter()
            .map(|p| {
                let mut mesh = p.geometry.mesh();
                if let Some(j) = p.joint {
                    mesh.insert_attribute(
                        Mesh::ATTRIBUTE_JOINT_INDEX,
                        VertexAttributeValues::Uint16x4(vec![
                            [j as u16, 0, 0, 0];
                            p.geometry.positions.len()
                        ]),
                    );
                    mesh.insert_attribute(
                        Mesh::ATTRIBUTE_JOINT_WEIGHT,
                        vec![[1., 0., 0., 0.]; p.geometry.positions.len()],
                    );
                }
                PreparedPart {
                    name: p.name.clone(),
                    mesh: meshes.add(mesh),
                    material: materials.add(StandardMaterial {
                        base_color: Color::linear_rgb(p.color[0], p.color[1], p.color[2]),
                        perceptual_roughness: p.roughness,
                        ..default()
                    }),
                }
            })
            .collect();
        let mut named = HashMap::new();
        let bind = definition.rig.as_ref().map(|rig| {
            for name in CLIPS {
                let mut clip = AnimationClip::default();
                for i in 0..6 {
                    let samples: Vec<_> = (0..25)
                        .map(|f| ((f + 1) as f32 / 24., rig.pose(name, f)[i]))
                        .collect();
                    clip.add_curve_to_target(
                        target(i),
                        AnimatableCurve::new(
                            animated_field!(Transform::translation),
                            AnimatableKeyframeCurve::new(
                                samples.iter().map(|(t, p)| (*t, p.translation)),
                            )
                            .expect("increasing sample times"),
                        ),
                    );
                    clip.add_curve_to_target(
                        target(i),
                        AnimatableCurve::new(
                            animated_field!(Transform::rotation),
                            AnimatableKeyframeCurve::new(
                                samples.iter().map(|(t, p)| (*t, p.rotation)),
                            )
                            .expect("increasing sample times"),
                        ),
                    );
                }
                named.insert(name.into(), clips.add(clip));
            }
            binds.add(SkinnedMeshInverseBindposes::from(rig.inverse_bind()))
        });
        Self {
            definition,
            parts,
            bind,
            clips: named,
        }
    }
    pub fn spawn(&self, commands: &mut Commands, transform: Transform) -> SpawnedModel {
        let root = commands
            .spawn((
                Name::new(self.definition.name.clone()),
                transform,
                Visibility::default(),
            ))
            .id();
        let mut joints = vec![];
        let mut player = None;
        if let Some(rig) = &self.definition.rig {
            let p = commands
                .spawn((
                    Name::new("SoldierRig"),
                    Transform::default(),
                    Visibility::default(),
                    AnimationPlayer::default(),
                ))
                .id();
            commands.entity(root).add_child(p);
            player = Some(p);
            for (i, rest) in rig.rest.iter().enumerate() {
                let e = commands
                    .spawn((
                        Name::new(BONES[i]),
                        *rest,
                        Visibility::default(),
                        target(i),
                        AnimatedBy(p),
                    ))
                    .id();
                commands
                    .entity(if i == 0 { p } else { joints[0] })
                    .add_child(e);
                joints.push(e);
            }
        }
        for (i, p) in self.parts.iter().enumerate() {
            let e = commands
                .spawn((
                    Name::new(p.name.clone()),
                    Mesh3d(p.mesh.clone()),
                    MeshMaterial3d(p.material.clone()),
                    Transform::default(),
                ))
                .id();
            commands.entity(root).add_child(e);
            if self.definition.parts[i].joint.is_some() {
                commands.entity(e).insert(SkinnedMesh {
                    inverse_bindposes: self.bind.clone().expect("rig bind poses"),
                    joints: joints.clone(),
                });
            }
        }
        SpawnedModel { root, player }
    }
    pub fn animation_graph(
        &self,
    ) -> (
        AnimationGraph,
        HashMap<String, bevy::animation::graph::AnimationNodeIndex>,
    ) {
        let mut graph = AnimationGraph::new();
        for i in 0..6 {
            graph.add_target_to_mask_group(
                target(i),
                if i == 0 || BONES[i].contains("Leg") {
                    0
                } else {
                    1
                },
            );
        }
        let mut nodes = HashMap::new();
        for name in CLIPS.into_iter().chain(["__lower"]) {
            let source = if name == "__lower" { "Idle" } else { name };
            if let Some(clip) = self.clips.get(source) {
                let mask = if name == "__lower" {
                    2
                } else if matches!(name, "Fire" | "Throw" | "Rocket") {
                    1
                } else {
                    0
                };
                let node = graph.add_clip_with_mask(clip.clone(), mask, 1., graph.root);
                nodes.insert(name.into(), node);
            }
        }
        (graph, nodes)
    }
}
pub fn ground_texture(seed: u64) -> Image {
    use bevy::{
        asset::RenderAssetUsages,
        image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
        render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    };
    let mut rng = seed::SeededRandom::new(seed);
    for _ in 0..120 {
        rng.random();
    }
    let mut data = Vec::with_capacity(256 * 256 * 4);
    for y in 0..256 {
        for x in 0..256 {
            let grain = rng.uniform(-0.035, 0.035);
            let wave = (x as f64 / 256. * std::f64::consts::TAU * 7.).sin()
                * (y as f64 / 256. * std::f64::consts::TAU * 5.).sin()
                * 0.025;
            let blade = if rng.random() < 0.025 { 0.1 } else { 0. };
            for v in [
                0.82 + grain + wave + blade,
                0.87 + grain + wave + blade,
                0.67 + grain + wave,
            ] {
                // Legacy image pixels were written as byte values, without a transfer curve.
                data.push(((v as f32).clamp(0., 1.) * 255. + 0.5) as u8);
            }
            data.push(255);
        }
    }
    // Image pixels in Blender start at the bottom-left; Bevy images start at the top-left.
    let data: Vec<_> = data
        .chunks_exact(256 * 4)
        .rev()
        .flatten()
        .copied()
        .collect();
    let mut image = Image::new(
        Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..default()
    });
    image
}
