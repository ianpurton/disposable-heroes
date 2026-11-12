use super::*;

pub(super) fn build(b: &mut Builder) {
    for i in 0..10 {
        b.cube(
            "Bridge plank",
            v(0., -2. + i as f32 * 0.44, 0.1),
            v(2., 0.4, 0.2),
            WOOD,
            0.03,
        );
    }
    for x in [-0.9, 0.9] {
        b.rod("Rail", v(x, -2., 0.9), v(x, 2., 0.9), 0.07, WOOD);
    }

    for x in [-0.9, 0.9] {
        for i in 0..5 {
            let y = -1.9 + i as f32 * 0.95;
            b.rod("Railing post", v(x, y, 0.13), v(x, y, 0.9), 0.045, WOOD);
            b.ball(
                "Iron nail",
                v(x * 0.85, y, 0.207),
                v(0.035, 0.035, 0.012),
                METAL,
                1,
            );
        }
    }
}
