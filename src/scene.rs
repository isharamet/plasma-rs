use rand::seq::SliceRandom;
use rand::thread_rng;
use std::time::SystemTime;

pub struct Scene {
    size: (u32, u32),
    gradients: Vec<(f32, f32, f32)>,
    start: u128,
    clock: u128,
}

impl Scene {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width, height);
        let g_size = (width * height) as usize;
        let mut gradients = vec![(0f32, 0f32, 0f32); g_size];
        let start = now();
        let clock = 0;
        let mut rng = thread_rng();
        let choices: [f32; 2] = [-1.0, 1.0];
        for i in 0..g_size {
            gradients[i as usize] = (
                *choices.choose(&mut rng).expect(""),
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

    pub fn grad(&self, p: (f32, f32, f32)) -> (f32, f32, f32) {
        let width = self.size.0 as usize;
        let (x, y, z) = p;
        let x = x as usize;
        let y = y as usize;
        let z = (z as usize) % 10;

        let i = y * width + x * 10 + z;

        self.gradients[i]
    }

    pub fn noise(&self, p: (f32, f32, f32)) -> f32 {
        let p0 = floor(p);

        let p1 = sum(p0, (1.0, 0.0, 0.0));
        let p2 = sum(p0, (0.0, 1.0, 0.0));
        let p3 = sum(p0, (1.0, 1.0, 0.0));
        let p4 = sum(p0, (0.0, 0.0, 1.0));
        let p5 = sum(p4, (1.0, 0.0, 0.0));
        let p6 = sum(p4, (0.0, 1.0, 0.0));
        let p7 = sum(p4, (1.0, 1.0, 0.0));

        let g0 = self.grad(p0);
        let g1 = self.grad(p1);
        let g2 = self.grad(p2);
        let g3 = self.grad(p3);
        let g4 = self.grad(p4);
        let g5 = self.grad(p5);
        let g6 = self.grad(p6);
        let g7 = self.grad(p7);

        let t0 = p.0 - p0.0;
        let fade_t0 = fade(t0);
        let t1 = p.1 - p0.1;
        let fade_t1 = fade(t1);
        let t2 = p.2 - p0.2;
        let fade_t2 = fade(t2);

        let p0p1 = (1.0 - fade_t0) * dot(g0, diff(p, p0)) + fade_t0 * dot(g1, diff(p, p1));
        let p2p3 = (1.0 - fade_t0) * dot(g2, diff(p, p2)) + fade_t0 * dot(g3, diff(p, p3));

        let p4p5 = (1.0 - fade_t0) * dot(g4, diff(p, p4)) + fade_t0 * dot(g5, diff(p, p5));
        let p6p7 = (1.0 - fade_t0) * dot(g6, diff(p, p6)) + fade_t0 * dot(g7, diff(p, p7));

        let y1 = (1.0 - fade_t1) * p0p1 + fade_t1 * p2p3;
        let y2 = (1.0 - fade_t1) * p4p5 + fade_t1 * p6p7;

        (1.0 - fade_t2) * y1 + fade_t2 * y2
    }

    pub fn draw(&self, frame: &mut [u8]) {
        let (w, h) = self.size;
        let width = w as usize;
        let height = h as usize;

        let threads = 40;
        let rows_per_band = height / threads + 1;

        let band_size = rows_per_band * width * 4;
        let bands: Vec<&mut [u8]> = frame.chunks_mut(band_size).collect();

        fn render_band(band: &mut [u8], width: usize, offset: usize, scene: &Scene) {
            for (i, pixel) in band.chunks_exact_mut(4).enumerate() {
                let j = i + offset;
                let x = (j % width) as f32;
                let y = (j / width) as f32;
                let shift = scene.clock as f32 / 40.0;

                let n = scene.noise(div((x, y, shift), 256.0)) * 1.0
                    + scene.noise(div((x, y, shift), 128.0)) * 0.5
                    + scene.noise(div((x, y, shift), 64.0)) * 0.25;

                let c = ((n * 0.5 + 0.5) * 255.0) as u8;

                let rgba = if c < 64 {
                    [0, 0, c * 4, 0xff]
                } else if c >= 64 && c < 128 {
                    [0, (c - 64) * 4, (127 - c) * 4, 0xff]
                } else if c >= 128 && c < 192 {
                    [(c - 128) * 4, (191 - c) * 4, 0, 0xff]
                } else {
                    [(255 - c) * 4, 0, 0, 0xff]
                };

                pixel.copy_from_slice(&rgba);
            }
        }

        crossbeam::scope(|spawner| {
            for (i, band) in bands.into_iter().enumerate() {
                let offset = i * rows_per_band * width;

                spawner.spawn(move |_| {
                    render_band(band, width, offset, self);
                });
            }
        })
        .unwrap();
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

fn floor((x, y, z): (f32, f32, f32)) -> (f32, f32, f32) {
    (x.floor(), y.floor(), z.floor())
}

fn sum((x1, y1, z1): (f32, f32, f32), (x2, y2, z2): (f32, f32, f32)) -> (f32, f32, f32) {
    (x1 + x2, y1 + y2, z1 + z2)
}

fn diff((x1, y1, z1): (f32, f32, f32), (x2, y2, z2): (f32, f32, f32)) -> (f32, f32, f32) {
    (x1 - x2, y1 - y2, z1 - z2)
}

fn div((x, y, z): (f32, f32, f32), d: f32) -> (f32, f32, f32) {
    (x / d, y / d, z / d)
}

fn dot((x1, y1, z1): (f32, f32, f32), (x2, y2, z2): (f32, f32, f32)) -> f32 {
    x1 * x2 + y1 * y2 + z1 * z2
}
