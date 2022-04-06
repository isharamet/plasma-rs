use rand::seq::SliceRandom;
use rand::thread_rng;
use std::time::SystemTime;

pub struct Scene {
    size: (u32, u32),
    gradients: Vec<(f32, f32)>,
    start: u128,
    clock: u128,
}

impl Scene {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width, height);
        let g_size = (width * height + 1) as usize;
        let mut gradients = vec![(0f32, 0f32); g_size];
        let start = now();
        let clock = 0;
        let mut rng = thread_rng();
        let choices: [f32; 2] = [-1.0, 1.0];
        for i in 0..g_size {
            gradients[i as usize] = (
                *choices.choose(&mut rng).expect(""),
                *choices.choose(&mut rng).expect(""),
            );
        }
        Scene {
            size,
            gradients,
            start,
            clock,
        }
    }

    pub fn grad(&self, p: (f32, f32)) -> (f32, f32) {
        let width = self.size.0 as usize;
        let (x, y) = p;
        let x = x as usize;
        let y = y as usize;
        self.gradients[y * width + x]
    }

    pub fn noise(&self, p: (f32, f32)) -> f32 {
        let p0 = floor(p);
        let p1 = sum(p0, (1.0, 0.0));
        let p2 = sum(p0, (0.0, 1.0));
        let p3 = sum(p0, (1.0, 1.0));

        let g0 = self.grad(p0);
        let g1 = self.grad(p1);
        let g2 = self.grad(p2);
        let g3 = self.grad(p3);

        let t0 = p.0 - p0.0;
        let fade_t0 = fade(t0);
        let t1 = p.1 - p0.1;
        let fade_t1 = fade(t1);

        let p0p1 = (1.0 - fade_t0) * dot(g0, diff(p, p0)) + fade_t0 * dot(g1, diff(p, p1));
        let p2p3 = (1.0 - fade_t0) * dot(g2, diff(p, p2)) + fade_t0 * dot(g3, diff(p, p3));

        (1.0 - fade_t1) * p0p1 + fade_t1 * p2p3
    }

    pub fn draw(&self, frame: &mut [u8]) {
        let (w, _) = self.size;
        let width = w as usize;

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let y = i / width;
            let x = i % width;
            let p = (x as f32, y as f32);

            let n = self.noise(div(p, 64.0)) * 1.0
                + self.noise(div(p, 32.0)) * 0.5
                + self.noise(div(p, 16.0)) * 0.25
                + self.noise(div(p, 8.0)) * 0.125;

            let r = ((n * 0.5 + 0.5) * 255.0) as u8;

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

fn floor((x1, y1): (f32, f32)) -> (f32, f32) {
    (x1.floor(), y1.floor())
}

fn sum((x1, y1): (f32, f32), (x2, y2): (f32, f32)) -> (f32, f32) {
    (x1 + x2, y1 + y2)
}

fn diff((x1, y1): (f32, f32), (x2, y2): (f32, f32)) -> (f32, f32) {
    (x1 - x2, y1 - y2)
}

fn div((x, y): (f32, f32), d: f32) -> (f32, f32) {
    (x / d, y / d)
}

fn dot((x1, y1): (f32, f32), (x2, y2): (f32, f32)) -> f32 {
    x1 * x2 + y1 * y2
}
