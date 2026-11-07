use super::*;
pub(super) fn build(b: &mut Builder) {
    b.ball("Recruitment hill", v(0., 0., -0.4), v(7., 5., 2.), OLIVE, 3);
    b.ball("Far slope", v(-4., 2., -0.7), v(4., 4., 1.8), LEAF[0], 2);
}
pub(super) fn grave(b: &mut Builder) {
    b.cube("Memorial stone", v(0., 0., 0.5), v(0.45, 0.2, 1.), SAND, 0.);
    b.cube(
        "Cross vertical",
        v(0., -0.105, 0.65),
        v(0.06, 0.015, 0.4),
        DARK,
        0.,
    );
    b.cube(
        "Cross horizontal",
        v(0., -0.105, 0.7),
        v(0.25, 0.015, 0.06),
        DARK,
        0.,
    );
}
