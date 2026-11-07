//! Deterministic original synthesized effects. No filesystem or audio device needed to generate.
use bevy::prelude::*;
#[path = "../../seed.rs"]
mod seed;
pub const SAMPLE_RATE: u32 = 22050;
pub const NAMES: [&str; 3] = ["rifle", "explosion", "jungle"];
pub const DEFAULT_SEED: u64 = 1993;
#[derive(Clone, Debug, PartialEq)]
pub struct Sound {
    pub name: &'static str,
    pub samples: Vec<i16>,
}
impl Sound {
    pub fn wav(&self) -> Vec<u8> {
        let size = (self.samples.len() * 2) as u32;
        let mut b = Vec::with_capacity(size as usize + 44);
        b.extend(b"RIFF");
        b.extend((size + 36).to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(SAMPLE_RATE.to_le_bytes());
        b.extend((SAMPLE_RATE * 2).to_le_bytes());
        b.extend(2u16.to_le_bytes());
        b.extend(16u16.to_le_bytes());
        b.extend(b"data");
        b.extend(size.to_le_bytes());
        for s in &self.samples {
            b.extend(s.to_le_bytes());
        }
        b
    }
    pub fn audio_source(&self) -> AudioSource {
        AudioSource {
            bytes: self.wav().into(),
        }
    }
}
pub fn generate(seed: u64) -> Vec<Sound> {
    let mut rng = seed::SeededRandom::new(seed);
    NAMES
        .into_iter()
        .zip([0.13, 1.1, 12.])
        .map(|(name, duration)| {
            let mut previous = 0.;
            let samples = (0..(SAMPLE_RATE as f64 * duration) as usize)
                .map(|i| {
                    let t = i as f64 / SAMPLE_RATE as f64;
                    let noise = rng.uniform(-1., 1.);
                    previous = 0.94 * previous + 0.06 * noise;
                    let v = match name {
                        "rifle" => (noise * 0.6 + (t * 900.).sin() * 0.25) * (-t * 40.).exp(),
                        "explosion" => {
                            (previous * 3. + noise * 0.18 + (t * 70.).sin() * 0.25)
                                * (-t * 4.).exp()
                        }
                        _ => {
                            previous * 0.12
                                + (t * 2400. + (t * 2.).sin() * 40.).sin()
                                    * (t * 1.8).sin().max(0.).powi(18)
                                    * 0.025
                        }
                    };
                    (v.clamp(-1., 1.) * 0.65 * 32767.) as i16
                })
                .collect();
            Sound { name, samples }
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_and_bounded() {
        let a = generate(DEFAULT_SEED);
        assert_eq!(a, generate(DEFAULT_SEED));
        assert_ne!(a, generate(5));
        assert_eq!(
            a.iter().map(|s| s.samples.len()).collect::<Vec<_>>(),
            [2866, 24255, 264600]
        );
        for s in a {
            assert_eq!(s.wav().len(), 44 + s.samples.len() * 2);
            assert!(s.samples.iter().all(|v| v.unsigned_abs() < 32767));
        }
    }
}
#[cfg(test)]
mod legacy_tests {
    use super::*;
    #[test]
    fn matches_original_pcm() {
        let expected = [
            0xa4c3d3833a4b9c8cu64,
            0x2c5d22ef4ece4b39,
            0x1b60e5ff5e83ede5,
        ];
        for (sound, expected) in generate(DEFAULT_SEED).iter().zip(expected) {
            let mut hash = 0xcbf29ce484222325u64;
            for b in sound.samples.iter().flat_map(|v| v.to_le_bytes()) {
                hash = (hash ^ (b as u64)).wrapping_mul(0x100000001b3);
            }
            assert_eq!(hash, expected, "{}", sound.name);
        }
    }
}
