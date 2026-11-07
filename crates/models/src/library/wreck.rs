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
}
