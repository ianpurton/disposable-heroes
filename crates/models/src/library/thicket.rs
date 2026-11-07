use super::*;

pub(super) fn build(b: &mut Builder, rng: &mut seed::SeededRandom) {
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
    // Joined foliage keeps instance costs comparable to the original multi-material mesh.
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
