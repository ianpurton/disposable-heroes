use super::*;

pub(super) fn build(b: &mut Builder) {
    b.ball("Rock", v(0., 0., 0.35), v(0.7, 0.55, 0.5), SAND, 2);
}
