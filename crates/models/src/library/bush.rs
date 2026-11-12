use super::*;
pub(super) fn build(b: &mut Builder) {
    for i in 0..7 {
        let a = i as f32 * 2.4;
        let p = v(a.cos() * 0.30, a.sin() * 0.30, 0.28 + (i % 3) as f32 * 0.12);
        b.rod("Shrub branch", Vec3::ZERO, p, 0.025, WOOD);
        b.ball("Shrub core", p, v(0.29, 0.25, 0.20), LEAF[i % 4], 2);
        b.spray(p, 0.36, i);
    }
}
