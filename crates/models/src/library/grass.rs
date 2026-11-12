use super::*;
pub(super) fn build(b: &mut Builder) {
    for i in 0..13 {
        let a = i as f32 * 2.399;
        b.leaf(
            "Arching grass",
            v(a.cos() * 0.025, a.sin() * 0.025, 0.),
            v(
                a.cos() * 0.28,
                a.sin() * 0.28,
                0.24 + (i % 4) as f32 * 0.055,
            ),
            0.035,
            LEAF[i % 4],
        );
    }
}
