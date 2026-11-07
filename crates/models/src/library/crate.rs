use super::*;

pub(super) fn build(b: &mut Builder) {
    b.cube("Ammo crate", v(0., 0., 0.25), v(0.6, 0.6, 0.5), OLIVE, 0.03);
    b.cube(
        "Marking",
        v(0., -0.31, 0.28),
        v(0.35, 0.02, 0.12),
        ROOF,
        0.03,
    );
}
