use super::*;

pub(super) fn build(b: &mut Builder) {
    for i in 0..7 {
        let a = i as f32 * 2.4;
        b.rod(
            "Grass blade",
            Vec3::ZERO,
            v(a.cos() * 0.26, a.sin() * 0.26, 0.3 + (i % 3) as f32 * 0.1),
            0.035,
            LEAF[i % 4],
        );
    }
}
