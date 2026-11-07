use super::*;

pub(super) fn build(b: &mut Builder) {
    for i in 0..9 {
        let a = i as f32 * 2.4;
        b.ball(
            "Shrub",
            v(a.cos() * 0.35, a.sin() * 0.35, 0.3 + (i % 3) as f32 * 0.12),
            v(0.45, 0.27, 0.3),
            LEAF[i % 4],
            1,
        );
    }
}
