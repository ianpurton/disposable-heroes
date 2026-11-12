use super::*;

pub(super) fn build(b: &mut Builder) {
    b.ball("Rock", v(0., 0., 0.35), v(0.7, 0.55, 0.5), SAND, 2);
    for i in 0..5 {
        let a = i as f32 * 2.399;
        b.ball(
            "Stone fracture",
            v(a.cos() * 0.31, a.sin() * 0.22, 0.60),
            v(0.23, 0.19, 0.14),
            SAND,
            1,
        );
    }
}
