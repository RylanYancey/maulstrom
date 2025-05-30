use std::ops::Range;

#[cfg(not(target_arch = "wasm32"))]
pub fn entropy() -> u64 {
    match getrandom::u64() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[E444] 'rng::entropy()' Failed to generate seed from system source with err: '{e:?}'.");
            38938593895389
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn entropy() -> u64 {
    let now = web_time::Instant::now().elapsed().subsec_millis() as u64;
    now.wrapping_mul(0xa076_1d64_78bd_642f)
}

/// Wasm-compatible Wyrand.
#[derive(Copy, Clone)]
pub struct WyRand {
    pub seed: u64,
}

impl WyRand {
    pub fn from_entropy() -> Self {
        Self { seed: entropy() }
    }

    pub fn next(&mut self) -> u64 {
        const P0: u64 = 0xa076_1d64_78bd_642f;
        const P1: u64 = 0xe703_7ed1_a0b4_28db;
        self.seed = self.seed.wrapping_add(P0);
        let r = u128::from(self.seed) * u128::from(self.seed ^ P1);
        ((r >> 64) ^ r) as u64
    }

    pub fn range(&mut self, range: Range<usize>) -> u64 {
        (self.next() % (range.end as u64 - range.start as u64)) + range.start as u64
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in 0..slice.len() {
            slice.swap(i, self.range(i..slice.len()) as usize)
        }
    }
}
