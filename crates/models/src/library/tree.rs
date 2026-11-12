use super::*;
pub(super) fn build(b: &mut Builder) {
    b.rod("Trunk", Vec3::ZERO, v(0., 0., 2.6), 0.24, WOOD);
    for k in 0..7 {
        let a = k as f32 * 2.399;
        b.rod(
            "Buttress root",
            v(a.cos() * 0.62, a.sin() * 0.62, 0.03),
            v(0., 0., 0.65),
            0.09,
            WOOD,
        );
    }
    for i in 0..12 {
        let a = i as f32 * 2.4;
        let p = v(a.cos() * 0.8, a.sin() * 0.8, 2.6 + (i % 3) as f32 * 0.24);
        b.rod("Spreading branch", v(0., 0., 1.9), p, 0.065, WOOD);
        b.ball("Broadleaf crown", p, v(0.82, 0.70, 0.50), LEAF[i % 4], 2);
        b.spray(p, 0.84, i);
    }
}
