use rand::{Rng, SeedableRng, rngs::StdRng};

pub struct PerlinNoise {
    perm: [u8; 512],
    grad: [[f32; 2]; 4],
}

impl PerlinNoise {
    pub fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut perm = [0u8; 512];

        for i in 0..256 {
            perm[i] = i as u8;
        }

        for i in (1..256).rev() {
            let j = rng.random_range(0..=i);
            perm.swap(i, j);
        }

        for i in 0..256 {
            perm[256 + i] = perm[i];
        }

        Self {
            perm,
            grad: [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]],
        }
    }

    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    fn grad_dot(&self, hash: u8, x: f32, y: f32) -> f32 {
        let g = self.grad[(hash % 4) as usize];
        g[0] * x + g[1] * y
    }

    pub fn noise_2d(&self, x: f32, y: f32) -> f32 {
        let xi = x.floor() as i32 & 255;
        let yi = y.floor() as i32 & 255;

        let xf = x - x.floor();
        let yf = y - y.floor();

        let u = Self::fade(xf);
        let v = Self::fade(yf);

        let aa = self.perm[self.perm[xi as usize] as usize + yi as usize];
        let ab = self.perm[self.perm[xi as usize] as usize + yi as usize + 1];
        let ba = self.perm[self.perm[xi as usize + 1] as usize + yi as usize];
        let bb = self.perm[self.perm[xi as usize + 1] as usize + yi as usize + 1];

        let x1 = Self::lerp(self.grad_dot(aa, xf, yf), self.grad_dot(ba, xf - 1.0, yf), u);
        let x2 = Self::lerp(
            self.grad_dot(ab, xf, yf - 1.0),
            self.grad_dot(bb, xf - 1.0, yf - 1.0),
            u,
        );

        Self::lerp(x1, x2, v)
    }

    pub fn fbm(&self, x: f32, y: f32, octaves: u32, persistence: f32) -> f32 {
        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            total += self.noise_2d(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        total / max_value
    }
}

pub fn generate_wind_texture(width: u32, height: u32, seed: u64) -> Vec<[f32; 4]> {
    let noise = PerlinNoise::new(seed);
    let mut data = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let fx = x as f32 / width as f32 * 4.0;
            let fy = y as f32 / height as f32 * 4.0;

            let wind_x = noise.fbm(fx, fy, 4, 0.5);
            let wind_y = noise.fbm(fx + 100.0, fy + 100.0, 4, 0.5);

            data.push([wind_x, wind_y, 0.0, 1.0]);
        }
    }

    data
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_perlin_deterministic() {
        let noise1 = PerlinNoise::new(42);
        let noise2 = PerlinNoise::new(42);

        for x in 0..10 {
            for y in 0..10 {
                let v1 = noise1.noise_2d(x as f32 * 0.1, y as f32 * 0.1);
                let v2 = noise2.noise_2d(x as f32 * 0.1, y as f32 * 0.1);
                assert!((v1 - v2).abs() < f32::EPSILON);
            }
        }
    }

    #[test]
    fn test_perlin_different_seeds_produce_different_values() {
        let noise1 = PerlinNoise::new(42);
        let noise2 = PerlinNoise::new(123);

        let v1 = noise1.noise_2d(0.5, 0.5);
        let v2 = noise2.noise_2d(0.5, 0.5);

        assert!((v1 - v2).abs() > 0.01);
    }

    #[test]
    fn test_perlin_fade_edge_zero() {
        let result = PerlinNoise::fade(0.0);
        assert!((result - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_perlin_fade_edge_one() {
        let result = PerlinNoise::fade(1.0);
        assert!((result - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_perlin_fade_midpoint() {
        let result = PerlinNoise::fade(0.5);
        assert!(result > 0.0 && result < 1.0);
    }

    #[test]
    fn test_perlin_lerp_at_zero() {
        let result = PerlinNoise::lerp(10.0, 20.0, 0.0);
        assert!((result - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_perlin_lerp_at_one() {
        let result = PerlinNoise::lerp(10.0, 20.0, 1.0);
        assert!((result - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_perlin_lerp_midpoint() {
        let result = PerlinNoise::lerp(10.0, 20.0, 0.5);
        assert!((result - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_perlin_noise_2d_bounded() {
        let noise = PerlinNoise::new(42);
        for x in 0..20 {
            for y in 0..20 {
                let value = noise.noise_2d(x as f32, y as f32);
                assert!(
                    value >= -2.0 && value <= 2.0,
                    "noise_2d returned {} which is out of bounds",
                    value
                );
            }
        }
    }

    #[test]
    fn test_perlin_fbm_bounded() {
        let noise = PerlinNoise::new(42);
        for x in 0..10 {
            for y in 0..10 {
                let value = noise.fbm(x as f32, y as f32, 4, 0.5);
                assert!(
                    value >= -1.0 && value <= 1.0,
                    "fbm returned {} which is out of bounds",
                    value
                );
            }
        }
    }

    #[test]
    fn test_generate_wind_texture_dimensions() {
        let texture = generate_wind_texture(10, 5, 42);
        assert_eq!(texture.len(), 50);
    }

    #[test]
    fn test_generate_wind_texture_deterministic() {
        let t1 = generate_wind_texture(10, 10, 42);
        let t2 = generate_wind_texture(10, 10, 42);
        for (a, b) in t1.iter().zip(t2.iter()) {
            assert!((a[0] - b[0]).abs() < f32::EPSILON);
            assert!((a[1] - b[1]).abs() < f32::EPSILON);
        }
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn proptest_fade_bounds(t in 0.0f32..1.0) {
            let result = PerlinNoise::fade(t);
            prop_assert!(result >= -0.0001 && result <= 1.0001);
        }

        #[test]
        fn proptest_fade_monotonic(t1 in 0.0f32..0.99, t2 in 0.01f32..1.0) {
            prop_assume!(t1 < t2);
            let r1 = PerlinNoise::fade(t1);
            let r2 = PerlinNoise::fade(t2);
            prop_assert!(r1 <= r2);
        }

        #[test]
        fn proptest_lerp_bounds(a in -100.0f32..100.0, b in -100.0f32..100.0, t in 0.0f32..1.0) {
            let result = PerlinNoise::lerp(a, b, t);
            let min_val = a.min(b);
            let max_val = a.max(b);
            prop_assert!(result >= min_val - 0.001 && result <= max_val + 0.001);
        }

        #[test]
        fn proptest_lerp_at_zero_returns_a(a in -1000.0f32..1000.0, b in -1000.0f32..1000.0) {
            let result = PerlinNoise::lerp(a, b, 0.0);
            prop_assert!((result - a).abs() < 0.001);
        }

        #[test]
        fn proptest_lerp_at_one_returns_b(a in -1000.0f32..1000.0, b in -1000.0f32..1000.0) {
            let result = PerlinNoise::lerp(a, b, 1.0);
            prop_assert!((result - b).abs() < 0.001);
        }

        #[test]
        fn proptest_noise_2d_bounds(x in -100.0f32..100.0, y in -100.0f32..100.0) {
            let noise = PerlinNoise::new(42);
            let result = noise.noise_2d(x, y);
            prop_assert!(result >= -2.0 && result <= 2.0);
        }

        #[test]
        fn proptest_fbm_bounds(x in -50.0f32..50.0, y in -50.0f32..50.0, octaves in 1u32..8, persistence in 0.1f32..0.9) {
            let noise = PerlinNoise::new(42);
            let result = noise.fbm(x, y, octaves, persistence);
            prop_assert!(result >= -1.0 && result <= 1.0);
        }

        #[test]
        fn proptest_noise_continuous(x in 0.0f32..10.0, y in 0.0f32..10.0, delta in 0.001f32..0.1) {
            let noise = PerlinNoise::new(42);
            let v1 = noise.noise_2d(x, y);
            let v2 = noise.noise_2d(x + delta, y);
            let diff = (v1 - v2).abs();
            prop_assert!(diff < 1.0, "Small delta should produce small change, got diff={}", diff);
        }
    }
}
