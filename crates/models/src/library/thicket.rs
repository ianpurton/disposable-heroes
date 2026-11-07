use super::*;

pub(super) fn build(b: &mut Builder, rng: &mut seed::SeededRandom) {
    // Broad interlocking crowns above a dark, continuous understory.
    for i in 0..16 {
        let a = rng.uniform(0., std::f64::consts::TAU) as f32;
        let r = rng.uniform(0.05, 0.43) as f32;
        let upper = i >= 5;
        let h = if upper {
            rng.uniform(0.48, 0.88) as f32
        } else {
            0.22
        };
        let radius = rng.uniform(0.25, 0.40) as f32;
        b.ball(
            "Layered jungle crown",
            v(a.cos() * r, a.sin() * r, h),
            v(radius, radius * 0.92, radius * 0.72),
            JUNGLE[if upper { 1 + i % 3 } else { 0 }],
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
