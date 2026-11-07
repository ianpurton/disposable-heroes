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
        "rock" => rock::build(&mut b),
        "soldier" => soldier::build(&mut b),
        "thicket" => thicket::build(&mut b, &mut rng),
        "thicket_edge" => thicket_edge::build(&mut b, &mut rng),
        "tree" => tree::build(&mut b),
        "wreck" => wreck::build(&mut b, &mut rng),
        _ => unreachable!(),
    }
    b.model
}
