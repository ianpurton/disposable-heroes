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

    for x in [-0.58, 0.58] {
        b.rod(
            "Windscreen frame",
            v(x, -0.15, 0.98),
            v(x, -0.05, 1.55),
            0.035,
            METAL,
        );
    }
    b.rod(
        "Windscreen top",
        v(-0.58, -0.05, 1.55),
        v(0.58, -0.05, 1.55),
        0.035,
        METAL,
    );
    b.cube(
        "Front bumper",
        v(0., -1.22, 0.55),
        v(1.5, 0.12, 0.12),
        METAL,
        0.02,
    );
    for x in [-0.47, 0.47] {
        b.ball(
            "Headlamp",
            v(x, -1.215, 0.88),
            v(0.11, 0.045, 0.11),
            SAND,
            2,
        );
    }
    for i in 0..8 {
        b.cube(
            "Radiator grille",
            v(-0.30 + i as f32 * 0.085, -1.22, 0.84),
            v(0.035, 0.025, 0.24),
            DARK,
            0.005,
        );
    }
    for x in [-0.87, 0.87] {
        for y in [-0.8, 0.8] {
            b.ball("Wheel hub", v(x, y, 0.4), v(0.035, 0.13, 0.13), METAL, 2);
        }
    }
}
