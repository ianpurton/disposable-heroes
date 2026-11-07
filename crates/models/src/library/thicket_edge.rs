use super::*;

pub(super) fn build(b: &mut Builder, rng: &mut seed::SeededRandom) {
    for _ in 0..60 {
        rng.random();
    }
    thicket::build(b, rng);
}
