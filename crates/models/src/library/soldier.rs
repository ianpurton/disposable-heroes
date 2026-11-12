use super::*;

pub(super) fn build(b: &mut Builder) {
    build_with_cloth(b, OLIVE);
}

pub(super) fn build_with_cloth(b: &mut Builder, cloth: [f32; 3]) {
    b.model.rig = Some(Rig::new());
    b.cube("Tunic", v(0., 0., 1.02), v(0.6, 0.38, 0.52), cloth, 0.09);
    b.bind_last(0);
    b.cube("Pack", v(0., 0.25, 1.02), v(0.42, 0.24, 0.40), WOOD, 0.05);
    b.bind_last(0);
    b.ball("Face", v(0., -0.035, 1.48), v(0.25, 0.23, 0.27), SKIN, 2);
    b.bind_last(1);
    b.ball("Helmet", v(0., 0.01, 1.65), v(0.31, 0.29, 0.18), cloth, 2);
    b.bind_last(1);
    for (side, x, j) in [("Left", -0.17, 2), ("Right", 0.17, 3)] {
        b.cube(
            &format!("{side} trouser"),
            v(x, 0., 0.46),
            v(0.24, 0.27, 0.56),
            cloth,
            0.05,
        );
        b.bind_last(j);
        b.cube(
            &format!("{side} boot"),
            v(x, -0.09, 0.12),
            v(0.25, 0.42, 0.20),
            DARK,
            0.04,
        );
        b.bind_last(j);
    }
    for (side, x, j) in [("Left", -0.36, 4), ("Right", 0.36, 5)] {
        b.rod(
            &format!("{side} sleeve"),
            v(x, 0., 1.15),
            v(x, -0.23, 0.85),
            0.13,
            cloth,
        );
        b.bind_last(j);
        b.ball(
            &format!("{side} hand"),
            v(x, -0.28, 0.84),
            v(0.115, 0.12, 0.12),
            SKIN,
            2,
        );
        b.bind_last(j);
    }
    b.cube(
        "Rifle",
        v(0.3, -0.48, 0.88),
        v(0.10, 0.75, 0.12),
        WOOD,
        0.02,
    );
    b.bind_last(5);
    b.rod(
        "Barrel",
        v(0.3, -0.65, 0.89),
        v(0.3, -1.05, 0.89),
        0.035,
        METAL,
    );
    b.bind_last(5);

    for x in [-0.19, 0.19] {
        b.cube(
            "Webbing strap",
            v(x, -0.199, 1.05),
            v(0.065, 0.035, 0.43),
            WOOD,
            0.01,
        );
        b.bind_last(0);
        b.cube(
            "Ammo pouch",
            v(x, -0.23, 0.90),
            v(0.14, 0.09, 0.16),
            OLIVE,
            0.02,
        );
        b.bind_last(0);
        b.cube(
            "Pack strap",
            v(x * 0.7, 0.379, 1.04),
            v(0.045, 0.025, 0.33),
            DARK,
            0.005,
        );
        b.bind_last(0);
    }
    b.cube("Belt", v(0., -0.01, 0.81), v(0.59, 0.41, 0.07), WOOD, 0.02);
    b.bind_last(0);
    b.cube(
        "Buckle",
        v(0., -0.225, 0.81),
        v(0.08, 0.025, 0.065),
        METAL,
        0.01,
    );
    b.bind_last(0);
    b.ball(
        "Helmet rim",
        v(0., 0., 1.60),
        v(0.315, 0.295, 0.045),
        cloth,
        2,
    );
    b.bind_last(1);
    b.cube(
        "Helmet band",
        v(0., -0.26, 1.65),
        v(0.29, 0.035, 0.04),
        WOOD,
        0.01,
    );
    b.bind_last(1);
    b.cube(
        "Rifle receiver",
        v(0.3, -0.43, 0.96),
        v(0.105, 0.29, 0.09),
        METAL,
        0.015,
    );
    b.bind_last(5);
    b.cube(
        "Magazine",
        v(0.3, -0.51, 0.79),
        v(0.075, 0.14, 0.17),
        METAL,
        0.012,
    );
    b.bind_last(5);
    b.cube(
        "Front sight",
        v(0.3, -0.97, 0.94),
        v(0.035, 0.045, 0.09),
        DARK,
        0.005,
    );
    b.bind_last(5);
}
