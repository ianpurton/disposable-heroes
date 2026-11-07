use super::*;

pub(super) fn build(b: &mut Builder) {
    for i in 0..7 {
        let i = i as f32;
        b.rod(
            "Palm trunk",
            v((i * 0.14).sin() * 0.3, 0., i * 0.55),
            v(((i + 1.) * 0.14).sin() * 0.3, 0., (i + 1.) * 0.55),
            0.17,
            WOOD,
        );
    }
    for k in 0..9 {
        let a = k as f32 * std::f32::consts::TAU / 9.;
        for j in 0..4 {
            let j = j as f32;
            let r = 0.35 + j * 0.5;
            let p = v(
                a.cos() * r + 0.3,
                a.sin() * r,
                3.8 + 0.3 * j.sin() - j * 0.18,
            );
            b.ball("Palm frond", p, v(0.7, 0.22, 0.08), LEAF[k % 4], 1);
            b.rotate_last(p, Quat::from_rotation_z(a));
        }
    }
}
