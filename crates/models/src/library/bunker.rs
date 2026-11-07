use super::*;

pub(super) fn build(b: &mut Builder) {
    b.cube(
        "Concrete bunker",
        v(0., 0., 0.65),
        v(3.2, 2.4, 1.3),
        SAND,
        0.15,
    );
    b.cube(
        "Bunker roof",
        v(0., 0., 1.4),
        v(3.5, 2.7, 0.25),
        OLIVE,
        0.08,
    );
    b.cube(
        "Firing slit",
        v(0., -1.21, 0.85),
        v(1.8, 0.03, 0.25),
        DARK,
        0.,
    );
}
