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
}
