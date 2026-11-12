use super::*;

pub(super) fn build(b: &mut Builder, rng: &mut seed::SeededRandom) {
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

    for i in 0..8 {
        let a = i as f32 * 2.399;
        let p = v(a.cos() * 0.9, a.sin() * 0.7, 0.09);
        b.cube("Broken masonry", p, v(0.28, 0.21, 0.15), SAND, 0.025);
        b.rotate_last(p, Quat::from_rotation_z(a));
    }
    for i in 0..3 {
        let p = v(-0.75 + i as f32 * 0.65, 0.15, 0.15);
        b.cube("Twisted roof sheet", p, v(0.44, 0.65, 0.035), METAL, 0.01);
        b.rotate_last(
            p,
            Quat::from_euler(EulerRot::XYZ, 0.23, -0.17, i as f32 * 1.2),
        );
    }
}
