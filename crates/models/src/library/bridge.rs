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
}
