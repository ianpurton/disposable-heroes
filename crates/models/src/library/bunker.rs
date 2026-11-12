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

    for row in 0..4 {
        for col in 0..7 {
            b.cube(
                "Concrete joint",
                v(-1.32 + col as f32 * 0.44, -1.208, 0.15 + row as f32 * 0.29),
                v(0.42, 0.025, 0.018),
                WOOD,
                0.,
            );
        }
    }
    for x in [-1.25, 1.25] {
        b.cube(
            "Roof reinforcement",
            v(x, 0., 1.55),
            v(0.13, 2.4, 0.055),
            METAL,
            0.015,
        );
    }
    for i in 0..9 {
        b.cube(
            "Sandbag parapet",
            v(-1.4 + i as f32 * 0.35, 0.91, 1.61),
            v(0.37, 0.30, 0.15),
            SAND,
            0.06,
        );
    }
}
