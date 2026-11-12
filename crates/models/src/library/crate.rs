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

    for x in [-0.22, 0.22] {
        b.cube(
            "Crate band",
            v(x, 0., 0.508),
            v(0.045, 0.59, 0.015),
            METAL,
            0.,
        );
        b.cube(
            "Crate strap",
            v(x, -0.307, 0.25),
            v(0.045, 0.016, 0.48),
            METAL,
            0.,
        );
    }
    for z in [0.10, 0.40] {
        b.cube("Lid seam", v(0., -0.305, z), v(0.58, 0.01, 0.012), DARK, 0.);
    }
}
