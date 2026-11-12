use super::*;

pub(super) fn build(b: &mut Builder) {
    for x in [-1.5, 1.5] {
        for y in [-1.1, 1.1] {
            b.rod("Post", v(x, y, 0.), v(x, y, 2.1), 0.13, WOOD);
        }
    }
    for x in [-1.45, 1.45] {
        b.cube("Side wall", v(x, 0., 1.), v(0.15, 2.2, 1.8), WOOD, 0.03);
    }
    b.cube("Back wall", v(0., 1.05, 1.), v(3., 0.15, 1.8), WOOD, 0.03);
    for x in [-1., 1.] {
        b.cube("Front wall", v(x, -1.05, 1.), v(0.9, 0.15, 1.8), WOOD, 0.03);
    }
    for x in [-0.9f32, 0.9] {
        let p = v(x, 0., 2.15);
        b.cube("Thatch roof", p, v(2., 2.8, 0.22), ROOF, 0.03);
        b.rotate_last(p, Quat::from_rotation_y(0.42f32.copysign(x)));
    }
    for i in 0..16 {
        let x = -1.75 + i as f32 * 0.23;
        b.cube(
            "Roof straw",
            v(x, 0., 2.55 - x.abs() * 0.43),
            v(0.12, 2.85, 0.12),
            ROOF,
            0.03,
        );
    }

    for side in [-1., 1.] {
        for i in 0..11 {
            let y = -0.95 + i as f32 * 0.19;
            b.cube(
                "Wall plank",
                v(side * 1.54, y, 1.03),
                v(0.035, 0.17, 1.72),
                if i % 3 == 0 { ROOF } else { WOOD },
                0.008,
            );
        }
        for z in [0.24, 1.72] {
            b.cube(
                "Wall brace",
                v(side * 1.57, 0., z),
                v(0.06, 2.12, 0.11),
                ROOF,
                0.012,
            );
        }
        let window = v(side * 1.59, -0.1, 1.22);
        b.cube("Window recess", window, v(0.015, 0.60, 0.48), DARK, 0.);
        for z in [0.95, 1.49] {
            b.cube(
                "Window frame",
                v(side * 1.61, -0.1, z),
                v(0.06, 0.72, 0.065),
                ROOF,
                0.01,
            );
        }
        for y in [-0.43, 0.23] {
            b.cube(
                "Window jamb",
                v(side * 1.61, y, 1.22),
                v(0.06, 0.06, 0.54),
                ROOF,
                0.01,
            );
        }
        b.cube(
            "Window mullion",
            v(side * 1.63, -0.1, 1.22),
            v(0.04, 0.04, 0.48),
            WOOD,
            0.,
        );
    }
    for x in [-0.51, 0.51] {
        b.cube(
            "Door jamb",
            v(x, -1.14, 0.92),
            v(0.09, 0.11, 1.72),
            ROOF,
            0.015,
        );
    }
    b.cube(
        "Door lintel",
        v(0., -1.14, 1.80),
        v(1.12, 0.11, 0.10),
        ROOF,
        0.015,
    );
    b.cube(
        "Threshold",
        v(0., -1.15, 0.12),
        v(1.12, 0.35, 0.13),
        SAND,
        0.02,
    );
    b.rod(
        "Roof ridge",
        v(0., -1.43, 2.61),
        v(0., 1.43, 2.61),
        0.065,
        WOOD,
    );
    for x in [-1., 1.] {
        for i in 0..14 {
            let y = -1.29 + i as f32 * 0.20;
            b.rod(
                "Thatch binding",
                v(0., y, 2.65),
                v(x * 1.75, y, 1.88),
                0.018,
                WOOD,
            );
        }
    }
}
