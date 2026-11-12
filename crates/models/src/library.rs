//! Model recipes in the original authoring coordinates (X right, Z up, -Y forward).
use super::*;
use geometry::*;

#[path = "library/crate.rs"]
mod ammo_crate;
mod bridge;
mod bunker;
mod bush;
mod enemy;
mod grass;
mod hill;
mod hut;
mod jeep;
mod palm;
mod rock;
mod soldier;
mod thicket;
mod thicket_edge;
mod tree;
mod wreck;

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
    fn leaf(&mut self, name: &str, start: Vec3, end: Vec3, width: f32, color: [f32; 3]) {
        let d = end - start;
        let side = d.cross(Vec3::Z).normalize_or_zero() * width;
        let mid = start.lerp(end, 0.48) + Vec3::Z * width * 0.7;
        let mut g = Geometry::default();
        for (a, c) in [(mid - side, mid), (mid, mid + side)] {
            g.triangle(start, a, c);
            g.triangle(a, end, c);
            g.triangle(start, c, a);
            g.triangle(a, c, end);
        }
        self.add(name, g, Vec3::ZERO, Quat::IDENTITY, color, None);
    }
    fn spray(&mut self, center: Vec3, radius: f32, phase: usize) {
        for j in 0..40 {
            let a = j as f32 * 2.399 + phase as f32;
            let r = radius * (0.35 + (j % 5) as f32 * 0.14);
            let p = center
                + v(
                    a.cos() * r,
                    a.sin() * r,
                    radius * (0.82 - 0.40 * (r / radius).powi(2)),
                );
            self.leaf(
                "Sunlit foliage",
                p,
                p + v(
                    a.cos() * radius * 0.40,
                    a.sin() * radius * 0.40,
                    -radius * 0.15,
                ),
                radius * 0.19,
                LEAF[(j + phase) % 4],
            );
        }
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
    match name {
        "bridge" => bridge::build(&mut b),
        "bunker" => bunker::build(&mut b),
        "bush" => bush::build(&mut b),
        "crate" => ammo_crate::build(&mut b),
        "enemy" => enemy::build(&mut b),
        "grass" => grass::build(&mut b),
        "hut" => hut::build(&mut b),
        "jeep" => jeep::build(&mut b),
        "palm" => palm::build(&mut b),
        "hill" => hill::build(&mut b),
        "gravestone" => hill::grave(&mut b),
        "rock" => rock::build(&mut b),
        "soldier" => soldier::build(&mut b),
        "thicket" => thicket::build(&mut b, &mut rng),
        "thicket_edge" => thicket_edge::build(&mut b, &mut rng),
        "tree" => tree::build(&mut b),
        "wreck" => wreck::build(&mut b, &mut rng),
        _ => unreachable!(),
    }
    let mut merged: Vec<Part> = vec![];
    for part in b.model.parts.drain(..) {
        if let Some(p) = merged.iter_mut().find(|p| {
            p.color == part.color && p.joint == part.joint && p.roughness == part.roughness
        }) {
            let offset = p.geometry.positions.len() as u32;
            p.geometry.positions.extend(part.geometry.positions);
            p.geometry.normals.extend(part.geometry.normals);
            p.geometry
                .indices
                .extend(part.geometry.indices.into_iter().map(|i| i + offset));
        } else {
            merged.push(part);
        }
    }
    b.model.parts = merged;
    b.model
}
