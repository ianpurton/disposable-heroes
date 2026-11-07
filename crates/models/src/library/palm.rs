use super::*;

pub(super) fn build(b: &mut Builder) {
    for i in 0..7 {
        let i = i as f32;
        b.rod(
            "Palm trunk",
            v((i * 0.14).sin() * 0.3, 0., i * 0.36),
            v(((i + 1.) * 0.14).sin() * 0.3, 0., (i + 1.) * 0.36),
            0.11,
            WOOD,
        );
    }
    for k in 0..9 {
        let a = k as f32 * std::f32::consts::TAU / 9.;
        for j in 0..4 {
            let j = j as f32;
            let r = 0.22 + j * 0.34;
            let p = v(
                a.cos() * r + 0.3,
                a.sin() * r,
                2.52 + 0.25 * j.sin() - j * j * 0.055,
            );
            b.ball("Palm frond", p, v(0.43, 0.16, 0.065), LEAF[k % 4], 1);
            b.rotate_last(p, Quat::from_rotation_z(a));
        }
    }
}
