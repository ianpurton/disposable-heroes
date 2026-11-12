use super::*;
pub(super) fn build(b: &mut Builder) {
    for i in 0..14 {
        let z = i as f32 * 0.18;
        let p = v(0.08 * z * z, 0.025 * z, z);
        let q = v(0.08 * (z + 0.18).powi(2), 0.025 * (z + 0.18), z + 0.18);
        b.rod("Ringed palm trunk", p, q, 0.13 - z * 0.018, WOOD);
        b.ball(
            "Old leaf collar",
            p,
            v(0.14 - z * 0.018, 0.14 - z * 0.018, 0.045),
            ROOF,
            1,
        );
    }
    let crown = v(0.50, 0.063, 2.52);
    for k in 0..11 {
        let a = k as f32 * 2.399;
        let dir = v(a.cos(), a.sin(), 0.);
        let side = v(-a.sin(), a.cos(), 0.);
        let length = 1.45 + (k % 3) as f32 * 0.13;
        let point = |t: f32| crown + dir * (length * t) + Vec3::Z * (0.65 * t - 1.05 * t * t);
        for j in 0..10 {
            let t = j as f32 / 10.;
            let p = point(t);
            b.rod(
                "Frond rib",
                p,
                point(t + 0.1),
                0.016 * (1. - t * 0.7),
                LEAF[2],
            );
            for sign in [-1., 1.] {
                let width = (t * std::f32::consts::PI).sin() * 0.36 + 0.035;
                let tip = p + side * width * sign + dir * 0.16 - Vec3::Z * (0.07 + t * 0.12);
                b.leaf(
                    "Palm leaflet",
                    p,
                    tip,
                    0.055 * (1. - t * 0.5),
                    LEAF[(k + j) % 4],
                );
            }
        }
    }
    for i in 0..3 {
        b.ball(
            "Coconut",
            crown + v(i as f32 * 0.10 - 0.1, -0.08, -0.12),
            v(0.095, 0.08, 0.10),
            WOOD,
            2,
        );
    }
}
