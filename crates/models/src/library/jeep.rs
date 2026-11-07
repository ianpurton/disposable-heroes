use super::*;

pub(super) fn build(b: &mut Builder) {
    b.cube("Jeep body", v(0., 0., 0.6), v(1.4, 2.4, 0.5), OLIVE, 0.03);
    b.cube("Bonnet", v(0., -0.7, 0.95), v(1.3, 1., 0.3), OLIVE, 0.03);
    for x in [-0.75, 0.75] {
        for y in [-0.8, 0.8] {
            b.rod(
                "Wheel",
                v(x - 0.12, y, 0.4),
                v(x + 0.12, y, 0.4),
                0.36,
                DARK,
            );
        }
    }
    for x in [-0.5, 0.5] {
        b.cube("Seat", v(x, 0.5, 1.), v(0.4, 0.6, 0.45), WOOD, 0.03);
    }
}
