use super::*;

pub(super) fn build(b: &mut Builder) {
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
