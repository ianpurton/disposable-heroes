//! Python-compatible MT19937 integer seeding, retained for asset reproducibility.
#[derive(Clone)]
pub struct SeededRandom {
    state: [u32; 624],
    index: usize,
}
impl SeededRandom {
    pub fn new(seed: u64) -> Self {
        let mut r = Self {
            state: [0; 624],
            index: 624,
        };
        r.state[0] = 19650218;
        for i in 1..624 {
            r.state[i] = 1812433253u32
                .wrapping_mul(r.state[i - 1] ^ (r.state[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        let keys = [seed as u32, (seed >> 32) as u32];
        let count = if keys[1] == 0 { 1 } else { 2 };
        let (mut i, mut j) = (1, 0);
        for _ in 0..624 {
            r.state[i] = (r.state[i]
                ^ ((r.state[i - 1] ^ (r.state[i - 1] >> 30)).wrapping_mul(1664525)))
            .wrapping_add(keys[j])
            .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= 624 {
                r.state[0] = r.state[623];
                i = 1;
            }
            if j >= count {
                j = 0;
            }
        }
        for _ in 0..623 {
            r.state[i] = (r.state[i]
                ^ ((r.state[i - 1] ^ (r.state[i - 1] >> 30)).wrapping_mul(1566083941)))
            .wrapping_sub(i as u32);
            i += 1;
            if i >= 624 {
                r.state[0] = r.state[623];
                i = 1;
            }
        }
        r.state[0] = 0x80000000;
        r
    }
    fn next(&mut self) -> u32 {
        if self.index >= 624 {
            for i in 0..624 {
                let y = (self.state[i] & 0x80000000) | (self.state[(i + 1) % 624] & 0x7fffffff);
                self.state[i] = self.state[(i + 397) % 624]
                    ^ (y >> 1)
                    ^ if y & 1 != 0 { 0x9908b0df } else { 0 };
            }
            self.index = 0;
        }
        let mut y = self.state[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^ (y >> 18)
    }
    pub fn random(&mut self) -> f64 {
        let a = (self.next() >> 5) as u64;
        let b = (self.next() >> 6) as u64;
        ((a << 26) + b) as f64 / 9007199254740992.
    }
    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.random()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn python_sequence() {
        let mut r = SeededRandom::new(1993);
        assert!((r.random() - 0.48068972393886344).abs() < 1e-15);
    }
}
