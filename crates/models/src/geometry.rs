use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
#[derive(Clone, Debug, Default)]
pub struct Geometry {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}
impl Geometry {
    pub fn triangle(&mut self, a: Vec3, b: Vec3, c: Vec3) {
        if (b - a).cross(c - a).length_squared() < 1e-16 {
            return;
        }
        let n = (b - a).cross(c - a).normalize().to_array();
        let i = self.positions.len() as u32;
        self.positions
            .extend([a.to_array(), b.to_array(), c.to_array()]);
        self.normals.extend([n; 3]);
        self.indices.extend([i, i + 1, i + 2]);
    }
    /// Planar projection per flat face avoids stretched material detail on walls and roofs.
    pub fn surface_uvs(&self) -> Vec<[f32; 2]> {
        self.positions
            .iter()
            .zip(&self.normals)
            .map(|(p, n)| {
                let n = Vec3::from_array(*n).abs();
                if n.y >= n.x && n.y >= n.z {
                    [p[0] * 1.7, p[2] * 1.7]
                } else if n.x >= n.z {
                    [p[2] * 1.7, p[1] * 1.7]
                } else {
                    [p[0] * 1.7, p[1] * 1.7]
                }
            })
            .collect()
    }
    pub fn mesh(&self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.surface_uvs())
        .with_inserted_indices(Indices::U32(self.indices.clone()))
    }
    pub fn transform(&mut self, m: Mat4) {
        let normal = m.inverse().transpose();
        for p in &mut self.positions {
            *p = m.transform_point3(Vec3::from_array(*p)).to_array();
        }
        for n in &mut self.normals {
            *n = normal
                .transform_vector3(Vec3::from_array(*n))
                .normalize()
                .to_array();
        }
    }
}
pub fn cuboid(size: Vec3, radius: f32) -> Geometry {
    let h = size * 0.5;
    let r = radius.min(h.min_element());
    let inner = h - Vec3::splat(r);
    let mut g = Geometry::default();
    // Surface patches retain flat faces and a two-segment rounded bevel.
    for axis in 0..3 {
        for sign in [-1., 1.] {
            let u = (axis + 1) % 3;
            let v = (axis + 2) % 3;
            let levels = |a: usize| {
                let k = std::f32::consts::FRAC_1_SQRT_2;
                [
                    -h[a],
                    -inner[a] - r * k,
                    -inner[a],
                    inner[a],
                    inner[a] + r * k,
                    h[a],
                ]
            };
            let us = levels(u);
            let vs = levels(v);
            let point = |x: f32, y: f32| {
                let mut p = Vec3::ZERO;
                p[axis] = sign * h[axis];
                p[u] = x;
                p[v] = y;
                let core = p.clamp(-inner, inner);
                if r > 0. {
                    core + (p - core).normalize() * r
                } else {
                    p
                }
            };
            for i in 0..5 {
                for j in 0..5 {
                    let a = point(us[i], vs[j]);
                    let b = point(us[i + 1], vs[j]);
                    let c = point(us[i + 1], vs[j + 1]);
                    let d = point(us[i], vs[j + 1]);
                    if sign > 0. {
                        g.triangle(a, b, c);
                        g.triangle(a, c, d);
                    } else {
                        g.triangle(a, c, b);
                        g.triangle(a, d, c);
                    }
                }
            }
        }
    }
    g
}
pub fn ico(size: Vec3, sub: u32) -> Geometry {
    // Blender's pole-oriented icosahedron; using Bevy's differently oriented
    // primitive changes the silhouettes of flattened foliage and helmets.
    let mut points = vec![Vec3::NEG_Z, Vec3::Z];
    for (height, offset) in [(-0.447215, std::f32::consts::PI), (0.447215, 0.)] {
        for i in 0..5 {
            let a = offset + i as f32 * std::f32::consts::TAU / 5.;
            points.push(Vec3::new(a.cos() * 0.894425, a.sin() * 0.894425, height));
        }
    }
    let mut faces = vec![];
    for i in 0..12 {
        for j in i + 1..12 {
            for k in j + 1..12 {
                let (a, b, c) = (points[i], points[j], points[k]);
                if a.distance(b) < 1.06 && b.distance(c) < 1.06 && c.distance(a) < 1.06 {
                    if (b - a).cross(c - a).dot(a) > 0. {
                        faces.push([a, b, c]);
                    } else {
                        faces.push([a, c, b]);
                    }
                }
            }
        }
    }
    for _ in 1..sub {
        let mut next = vec![];
        for [a, b, c] in faces {
            let ab = (a + b).normalize();
            let bc = (b + c).normalize();
            let ca = (c + a).normalize();
            next.extend([[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]]);
        }
        faces = next;
    }
    let mut g = Geometry::default();
    for [a, b, c] in faces {
        g.triangle(a * size, b * size, c * size);
    }
    g
}
pub fn rod(a: Vec3, b: Vec3, r: f32, vertices: usize) -> Geometry {
    let mut g = Geometry::default();
    let d = b - a;
    let q = Quat::from_rotation_arc(Vec3::Z, d.normalize());
    let point = |i: usize, top: bool| {
        let angle = i as f32 * std::f32::consts::TAU / vertices as f32;
        let radius = if top { r * 0.75 } else { r };
        q * Vec3::new(
            angle.cos() * radius,
            angle.sin() * radius,
            if top {
                d.length() * 0.5
            } else {
                -d.length() * 0.5
            },
        ) + (a + b) * 0.5
    };
    for i in 0..vertices {
        let j = (i + 1) % vertices;
        g.triangle(point(i, false), point(j, false), point(j, true));
        g.triangle(point(i, false), point(j, true), point(i, true));
        g.triangle(a, point(j, false), point(i, false));
        g.triangle(b, point(i, true), point(j, true));
    }
    g
}
