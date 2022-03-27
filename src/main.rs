use pixels::{Error, Pixels, SurfaceTexture};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::time::SystemTime;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct Scene {
    size: (u32, u32),
    gradients: Vec<i8>,
    clock: u128,
}

impl Scene {
    fn new(width: u32, height: u32) -> Self {
        let size = (width, height);
        let mut gradients = vec![0i8; (WIDTH + 1) as usize];
        let clock = now();

        let mut rng = thread_rng();
        let choices: [i8; 2] = [-1, 1];

        for i in 0..(WIDTH + 1) {
            gradients[i as usize] = *choices.choose(&mut rng).expect("");
        }

        Scene {
            size,
            gradients,
            clock,
        }
    }

    fn grad(&self, p: f32) -> i8 {
        self.gradients[p as usize]
    }

    fn noise(&self, p: f32) -> f32 {
        let p0 = p.floor();
        let p1 = p0 + 1.0;

        let t = p - p0;
        let fade_t = fade(t);

        let g0 = self.grad(p0) as f32;
        let g1 = self.grad(p1) as f32;

        (1.0 - fade_t) * g0 * (p - p0) + fade_t * g1 * (p - p1)
    }

    fn draw(&self, frame: &mut [u8]) {
        let (w, h) = self.size;
        let width = w as usize;
        let height = h as usize;
        let mut levels = vec![0f32; width];

        let frequency = 1.0 / 20.0;
        let amplitude = 1.0 / 5.0;

        for i in 0..width {
            levels[i] = self.noise(i as f32 * frequency) * amplitude;
        }

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let y = i / width;
            let x = i % width;

            let n = levels[x];
            let y = 2.0 * (y as f32 / height as f32) - 1.0;

            let r = if n < y { 255 } else { 0 };
            let rgba = [r, 0, 0, 0xff];

            pixel.copy_from_slice(&rgba);
        }
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

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("plasma-rs")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let scene = Scene::new(WIDTH, HEIGHT);

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            scene.draw(pixels.get_frame());

            if pixels
                .render()
                .map_err(|e| println!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            // world.update();
            window.request_redraw();
        }
    });
}
