#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
pub(super) struct Particle {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) vx: f64,
    pub(super) vy: f64,
    pub(super) life: f64,
    pub(super) color: usize,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
pub(super) struct Node {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) vx: f64,
    pub(super) vy: f64,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
pub(super) struct Glyph {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) speed: f64,
    pub(super) value: u32,
}

#[cfg(target_arch = "wasm32")]
pub(super) struct Rng {
    state: u32,
}

#[cfg(target_arch = "wasm32")]
impl Rng {
    pub(super) fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    pub(super) fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    pub(super) fn next_f64(&mut self) -> f64 {
        f64::from(self.next_u32()) / f64::from(u32::MAX)
    }

    pub(super) fn range(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min).max(0.0)
    }
}
