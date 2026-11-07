//! Model recipes in the original authoring coordinates (X right, Z up, -Y forward).
use super::*;
use geometry::*;
const OLIVE: [f32; 3] = [0.20, 0.28, 0.095];
const ENEMY: [f32; 3] = [0.43, 0.19, 0.10];
const SKIN: [f32; 3] = [0.63, 0.39, 0.22];
const DARK: [f32; 3] = [0.055, 0.065, 0.048];
const WOOD: [f32; 3] = [0.24, 0.13, 0.055];
const SAND: [f32; 3] = [0.42, 0.40, 0.25];
const ROOF: [f32; 3] = [0.48, 0.36, 0.13];
const METAL: [f32; 3] = [0.12, 0.15, 0.13];
const LEAF: [[f32; 3]; 4] = [
    [0.12, 0.27, 0.09],
    [0.22, 0.40, 0.12],
    [0.32, 0.46, 0.15],
    [0.08, 0.19, 0.10],
];
const JUNGLE: [[f32; 3]; 4] = [
    [0.035, 0.12, 0.025],
    [0.07, 0.21, 0.04],
    [0.12, 0.29, 0.055],
    [0.20, 0.35, 0.075],
];
fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}
struct Builder {
    model: Model,
}
impl Builder {
    fn add(
        &mut self,
        name: &str,
        mut geometry: Geometry,
        pos: Vec3,
        rotation: Quat,
        color: [f32; 3],
        joint: Option<usize>,
    ) {
        let convert = Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        geometry.transform(convert * Mat4::from_rotation_translation(rotation, pos));
        self.model.parts.push(Part {
            name: name.into(),
            geometry,
            color,
            roughness: if color == METAL { 0.45 } else { 0.85 },
            joint,
        });
    }
    fn cube(&mut self, n: &str, p: Vec3, s: Vec3, c: [f32; 3], r: f32) {
        self.add(n, cuboid(s, r), p, Quat::IDENTITY, c, None);
    }
    fn ball(&mut self, n: &str, p: Vec3, s: Vec3, c: [f32; 3], sub: u32) {
        self.add(n, ico(s, sub), p, Quat::IDENTITY, c, None);
    }
    fn rod(&mut self, n: &str, a: Vec3, b: Vec3, r: f32, c: [f32; 3]) {
        self.add(n, rod(a, b, r, 8), Vec3::ZERO, Quat::IDENTITY, c, None);
    }
    fn rotate_last(&mut self, p: Vec3, q: Quat) {
        let c = Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        self.model
            .parts
            .last_mut()
            .expect("part")
            .geometry
            .transform(
                c * Mat4::from_translation(p)
                    * Mat4::from_quat(q)
                    * Mat4::from_translation(-p)
                    * c.inverse(),
            );
    }
    fn bind_last(&mut self, j: usize) {
        self.model.parts.last_mut().expect("part").joint = Some(j);
    }
}
pub(super) fn build(name: &str, seed: u64) -> Model {
    let mut b = Builder {
        model: Model {
            name: name.into(),
            parts: vec![],
            rig: None,
        },
    };
    let mut rng = seed::SeededRandom::new(seed);
    if name == "thicket_edge" {
        for _ in 0..60 {
            rng.random();
        }
    }
    match name {
        "bridge" => {
            for i in 0..10 {
                b.cube(
                    "Bridge plank",
                    v(0., -2. + i as f32 * 0.44, 0.1),
                    v(2., 0.4, 0.2),
                    WOOD,
                    0.03,
                );
            }
            for x in [-0.9, 0.9] {
                b.rod("Rail", v(x, -2., 0.9), v(x, 2., 0.9), 0.07, WOOD);
            }
        }
        "bunker" => {
            b.cube(
                "Concrete bunker",
                v(0., 0., 0.65),
                v(3.2, 2.4, 1.3),
                SAND,
                0.15,
            );
            b.cube(
                "Bunker roof",
                v(0., 0., 1.4),
                v(3.5, 2.7, 0.25),
                OLIVE,
                0.08,
            );
            b.cube(
                "Firing slit",
                v(0., -1.21, 0.85),
                v(1.8, 0.03, 0.25),
                DARK,
                0.,
            );
        }
        "bush" => {
            for i in 0..9 {
                let a = i as f32 * 2.4;
                b.ball(
                    "Shrub",
                    v(a.cos() * 0.35, a.sin() * 0.35, 0.3 + (i % 3) as f32 * 0.12),
                    v(0.45, 0.27, 0.3),
                    LEAF[i % 4],
                    1,
                );
            }
        }
        "crate" => {
            b.cube("Ammo crate", v(0., 0., 0.25), v(0.6, 0.6, 0.5), OLIVE, 0.03);
            b.cube(
                "Marking",
                v(0., -0.31, 0.28),
                v(0.35, 0.02, 0.12),
                ROOF,
                0.03,
            );
        }
        "grass" => {
            for i in 0..7 {
                let a = i as f32 * 2.4;
                b.rod(
                    "Grass blade",
                    Vec3::ZERO,
                    v(a.cos() * 0.26, a.sin() * 0.26, 0.3 + (i % 3) as f32 * 0.1),
                    0.035,
                    LEAF[i % 4],
                );
            }
        }
        "hut" => {
            for x in [-1.5, 1.5] {
                for y in [-1.1, 1.1] {
                    b.rod("Post", v(x, y, 0.), v(x, y, 2.1), 0.13, WOOD);
                }
            }
            for x in [-1.45, 1.45] {
                b.cube("Side wall", v(x, 0., 1.), v(0.15, 2.2, 1.8), WOOD, 0.03);
            }
            b.cube("Back wall", v(0., 1.05, 1.), v(3., 0.15, 1.8), WOOD, 0.03);
            for x in [-1., 1.] {
                b.cube("Front wall", v(x, -1.05, 1.), v(0.9, 0.15, 1.8), WOOD, 0.03);
            }
            for x in [-0.9f32, 0.9] {
                let p = v(x, 0., 2.15);
                b.cube("Thatch roof", p, v(2., 2.8, 0.22), ROOF, 0.03);
                b.rotate_last(p, Quat::from_rotation_y(0.42f32.copysign(x)));
            }
            for i in 0..16 {
                let x = -1.75 + i as f32 * 0.23;
                b.cube(
                    "Roof straw",
                    v(x, 0., 2.55 - x.abs() * 0.43),
                    v(0.12, 2.85, 0.12),
                    ROOF,
                    0.03,
                );
            }
        }
        "jeep" => {
            b.cube("Jeep body", v(0., 0., 0.6), v(1.4, 2.4, 0.5), OLIVE, 0.03);
            b.cube("Bonnet", v(0., -0.7, 0.95), v(1.3, 1., 0.3), OLIVE, 0.03);
            for x in [-0.75, 0.75] {
                for y in [-0.8, 0.8] {
                    b.rod(
                        "Wheel",
                        v(x - 0.12, y, 0.4),
                        v(x + 0.12, y, 0.4),
                        0.36,
                        DARK,
                    );
                }
            }
            for x in [-0.5, 0.5] {
                b.cube("Seat", v(x, 0.5, 1.), v(0.4, 0.6, 0.45), WOOD, 0.03);
            }
        }
        "palm" => {
            for i in 0..7 {
                let i = i as f32;
                b.rod(
                    "Palm trunk",
                    v((i * 0.14).sin() * 0.3, 0., i * 0.55),
                    v(((i + 1.) * 0.14).sin() * 0.3, 0., (i + 1.) * 0.55),
                    0.17,
                    WOOD,
                );
            }
            for k in 0..9 {
                let a = k as f32 * std::f32::consts::TAU / 9.;
                for j in 0..4 {
                    let j = j as f32;
                    let r = 0.35 + j * 0.5;
                    let p = v(
                        a.cos() * r + 0.3,
                        a.sin() * r,
                        3.8 + 0.3 * j.sin() - j * 0.18,
                    );
                    b.ball("Palm frond", p, v(0.7, 0.22, 0.08), LEAF[k % 4], 1);
                    b.rotate_last(p, Quat::from_rotation_z(a));
                }
            }
        }
        "rock" => b.ball("Rock", v(0., 0., 0.35), v(0.7, 0.55, 0.5), SAND, 2),
        "tree" => {
            b.rod("Trunk", Vec3::ZERO, v(0., 0., 2.6), 0.24, WOOD);
            for i in 0..9 {
                let a = i as f32 * 2.4;
                b.ball(
                    "Broadleaf crown",
                    v(a.cos() * 0.8, a.sin() * 0.8, 2.6 + (i % 3) as f32 * 0.35),
                    v(1.05, 0.85, 0.8),
                    LEAF[i % 4],
                    2,
                );
            }
        }
        "wreck" => {
            for _ in 0..12 {
                let p = v(
                    rng.uniform(-1.5, 1.5) as f32,
                    rng.uniform(-1., 1.) as f32,
                    rng.uniform(0.05, 0.35) as f32,
                );
                let s = v(rng.uniform(0.3, 1.2) as f32, 0.15, 0.15);
                b.cube("Charred debris", p, s, DARK, 0.03);
                b.rotate_last(
                    p,
                    Quat::from_rotation_z((rng.random() * std::f64::consts::TAU) as f32),
                );
            }
        }
        "thicket" | "thicket_edge" => {
            for i in 0..20 {
                let x = rng.uniform(-0.32, 0.32) as f32;
                let y = rng.uniform(-0.32, 0.32) as f32;
                let h = 0.45 + rng.random() as f32 * 0.65;
                b.ball(
                    "Jungle leaf",
                    v(x, y, h * 0.65),
                    v(
                        (0.5 - x.abs()).min(0.20),
                        (0.5 - y.abs()).min(0.22),
                        h * 0.5,
                    ),
                    JUNGLE[i % 4],
                    1,
                );
            }
            for i in 0..5 {
                let a = i as f32 * 2.399;
                let p = v(a.cos() * 0.22, a.sin() * 0.22, 0.28);
                let mut g = rod(v(0., 0., -0.275), v(0., 0., 0.275), 0.055, 5);
                // Roots use an explicitly smaller upper radius than the shared tapered rod.
                for point in &mut g.positions {
                    if point[2] > 0. {
                        point[0] *= 0.02 / (0.055 * 0.75);
                        point[1] *= 0.02 / (0.055 * 0.75);
                    }
                }
                recompute_normals(&mut g);
                b.add(
                    "Tangled roots",
                    g,
                    p,
                    Quat::from_euler(EulerRot::XYZ, a.sin() * 0.65, a.cos() * 0.65, 0.),
                    [0.12, 0.065, 0.028],
                    None,
                );
            }
            for p in &mut b.model.parts {
                p.roughness = 0.9;
            }
        }
        "soldier" | "enemy" => soldier(&mut b, if name == "enemy" { ENEMY } else { OLIVE }),
        _ => unreachable!(),
    }
    // Joined foliage keeps instance costs comparable to the original multi-material mesh.
    if matches!(name, "thicket" | "thicket_edge") {
        let mut merged: Vec<Part> = vec![];
        for part in b.model.parts.drain(..) {
            if let Some(existing) = merged.iter_mut().find(|p| p.color == part.color) {
                let offset = existing.geometry.positions.len() as u32;
                existing.geometry.positions.extend(part.geometry.positions);
                existing.geometry.normals.extend(part.geometry.normals);
                existing
                    .geometry
                    .indices
                    .extend(part.geometry.indices.into_iter().map(|i| i + offset));
            } else {
                merged.push(part);
            }
        }
        b.model.parts = merged;
    }
    b.model
}
fn recompute_normals(g: &mut Geometry) {
    for t in g.indices.chunks_exact(3) {
        let a = Vec3::from_array(g.positions[t[0] as usize]);
        let b = Vec3::from_array(g.positions[t[1] as usize]);
        let c = Vec3::from_array(g.positions[t[2] as usize]);
        let n = (b - a).cross(c - a).normalize().to_array();
        for &i in t {
            g.normals[i as usize] = n;
        }
    }
}
fn soldier(b: &mut Builder, cloth: [f32; 3]) {
    b.model.rig = Some(Rig::new());
    b.cube("Tunic", v(0., 0., 1.02), v(0.6, 0.38, 0.52), cloth, 0.09);
    b.bind_last(0);
    b.cube("Pack", v(0., 0.25, 1.02), v(0.42, 0.24, 0.40), WOOD, 0.05);
    b.bind_last(0);
    b.ball("Face", v(0., -0.035, 1.48), v(0.25, 0.23, 0.27), SKIN, 2);
    b.bind_last(1);
    b.ball("Helmet", v(0., 0.01, 1.65), v(0.31, 0.29, 0.18), cloth, 2);
    b.bind_last(1);
    for (side, x, j) in [("Left", -0.17, 2), ("Right", 0.17, 3)] {
        b.cube(
            &format!("{side} trouser"),
            v(x, 0., 0.46),
            v(0.24, 0.27, 0.56),
            cloth,
            0.05,
        );
        b.bind_last(j);
        b.cube(
            &format!("{side} boot"),
            v(x, -0.09, 0.12),
            v(0.25, 0.42, 0.20),
            DARK,
            0.04,
        );
        b.bind_last(j);
    }
    for (side, x, j) in [("Left", -0.36, 4), ("Right", 0.36, 5)] {
        b.rod(
            &format!("{side} sleeve"),
            v(x, 0., 1.15),
            v(x, -0.23, 0.85),
            0.13,
            cloth,
        );
        b.bind_last(j);
        b.ball(
            &format!("{side} hand"),
            v(x, -0.28, 0.84),
            v(0.115, 0.12, 0.12),
            SKIN,
            2,
        );
        b.bind_last(j);
    }
    b.cube(
        "Rifle",
        v(0.3, -0.48, 0.88),
        v(0.10, 0.75, 0.12),
        WOOD,
        0.02,
    );
    b.bind_last(5);
    b.rod(
        "Barrel",
        v(0.3, -0.65, 0.89),
        v(0.3, -1.05, 0.89),
        0.035,
        METAL,
    );
    b.bind_last(5);
}
