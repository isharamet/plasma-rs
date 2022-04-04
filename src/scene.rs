use rand::seq::SliceRandom;
use rand::thread_rng;
use std::time::SystemTime;

pub struct Scene {
    size: (u32, u32),
    gradients: Vec<i8>,
    start: u128,
    clock: u128,
}

impl Scene {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width, height);
        let mut gradients = vec![0i8; (width + 1) as usize];
        let start = now();
        let clock = 0;
        let mut rng = thread_rng();
        let choices: [i8; 2] = [-1, 1];
        for i in 0..(width + 1) {
            gradients[i as usize] = *choices.choose(&mut rng).expect("");
        }
        Scene {
            size,
            gradients,
            start,
            clock,
        }
    }

    pub fn grad(&self, p: f32) -> i8 {
        self.gradients[p as usize]
    }

    pub fn noise(&self, p: f32) -> f32 {
        let p = p % self.size.0 as f32;
        let p0 = p.floor();
        let p1 = p0 + 1.0;
        let t = p - p0;
        let fade_t = fade(t);
        let g0 = self.grad(p0) as f32;
        let g1 = self.grad(p1) as f32;
        (1.0 - fade_t) * g0 * (p - p0) + fade_t * g1 * (p - p1)
    }

    pub fn draw(&self, frame: &mut [u8]) {
        let (w, h) = self.size;
        let width = w as usize;
        let height = h as usize;
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let y = i / width;
            let x = i % width;
            let pos = (self.clock as f64 / 10.0 + x as f64) as f32;
            let n = self.noise(pos as f32 * (1.0 / 300.0)) * 1.0
                + self.noise(pos as f32 * (1.0 / 150.0)) * 0.5
                + self.noise(pos as f32 * (1.0 / 75.0)) * 0.25
                + self.noise(pos as f32 * (1.0 / 37.5)) * 0.125;
            let y = 2.0 * (y as f32 / height as f32) - 1.0;
            let r = if n < y { 255 } else { 0 };
            let rgba = [r, 0, 0, 0xff];
            pixel.copy_from_slice(&rgba);
        }
    }

    pub fn update(&mut self) {
        self.clock = now() - self.start;
    }
}

fn now() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards!")
        .as_millis()
}

fn fade(t: f32) -> f32 {
    t.powi(3) * (t * (t * 6.0 - 15.0) + 10.0)
}
