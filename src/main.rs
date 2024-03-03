use std::{
    ops::{Add, Mul}, path::Path, time::{Duration, Instant}
};

use common::AsBytes;
use gl::{DrawContext, Shader, Uniform, UniformBuffer};
use glfw::WindowHint;
use glfw::{Context, OpenGlProfileHint};
use math::ease;
use palette::Palette;
use render::{Instance, InstancedShapeManager};
use scene::Scene;
use snake::Snake;

use crate::math::Mat4;

mod common;
mod gl;
mod math;
mod render;
mod scene;
mod snake;
mod palette;

struct Lerp {
    accum: Duration,
    duration: Duration,
}

impl Lerp {
    fn new(duration: Duration) -> Self {
        Self {
            accum: duration,
            duration,
        }
    }

    fn reset(&mut self) {
        self.accum = Duration::ZERO;
    }

    fn tick(&mut self, dt: Duration) -> bool {
        self.accum += dt;

        self.accum < self.duration
    }


    fn progress(&self) -> f32 {
        self.accum.as_secs_f32() / self.duration.as_secs_f32()
    }
}

struct Game<'a> {
    snake: Snake<'a>,
    scene: Scene<'a>,

    lerp: Lerp,
    normal: Mat4,
    zoom: Mat4,
    zoomed: bool,

    common_uniforms: UniformBuffer<'a>,
    basic_shader: Shader<'a>,
    instanced_shader: Shader<'a>,
}

impl<'a> Game<'a> {
    fn new(ctx: &'a DrawContext) -> Self {
        let palette = palette::aperture();

        let width = 50;
        let height = 50;
        let zoom = Mat4::screen(width as f32 / 4.0, height as f32 / 4.0);
        let normal = Mat4::screen(width as f32, height as f32);
        let common_uniforms = UniformBuffer::new(ctx);
        common_uniforms.bind_buffer_base(0);
        common_uniforms.set(unsafe { normal.as_bytes() }, gl::buffer_flags::DYNAMIC_STORAGE);

        let basic_shader = Shader::from_file(ctx, Path::new("res/shaders/basic"))
            .expect("Failed to compile basic shader");

        let instanced_shader = Shader::from_file(ctx, Path::new("res/shaders/instanced"))
            .expect("Failed to compile instanced shader");

        let scene = Scene::new(ctx, palette, width, height);

        let snake = Snake::new(ctx, (0.0).into(), palette);

        Self {
            snake,
            scene,

            lerp: Lerp::new(Duration::from_millis(500)),
            normal,
            zoom,
            zoomed: true,

            common_uniforms,
            basic_shader,
            instanced_shader,
        }
    }

    fn draw(&self) {
        self.snake.draw(&self.basic_shader);
        self.scene.draw(&self.instanced_shader);
    }

    fn tick(&mut self, dt: Duration) {
        if self.lerp.tick(dt) {
            // lerp active 
            let (to, from) = if !self.zoomed {
                (self.normal, self.zoom)
            } else {
                (self.zoom, self.normal)
            };

            let p = ease::out_back(self.lerp.progress());
            let screen = math::lerp(to, from, p);
            self.common_uniforms.update(0, unsafe { screen.as_bytes() })
        }


        self.snake.tick(dt);
    }

    fn key_press(&mut self, key: glfw::Key, is_down: bool) {
        use glfw::Key as K;
        if is_down {
            match key {
                K::W => self.snake.handle_move(snake::Direction::Up),
                K::A => self.snake.handle_move(snake::Direction::Left),
                K::S => self.snake.handle_move(snake::Direction::Down),
                K::D => self.snake.handle_move(snake::Direction::Right),
                K::Space => {
                    self.zoomed = !self.zoomed;
                    self.lerp.reset();
                },
                _ => (),
            }
        }
    }
}

struct Window {
    draw_context: DrawContext,
    window: glfw::PWindow,
    event_pump: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    glfw: glfw::Glfw,
}

impl Window {
    fn new() -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init GLFW");

        // window hints
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        glfw.window_hint(WindowHint::ContextVersion(4, 5));

        let (mut window, event_pump) = glfw
            .create_window(1200, 1200, "Snek", glfw::WindowMode::Windowed)
            .expect("Failed to create window");

        // window setup
        window.set_resizable(false);
        window.set_key_polling(true);
        let draw_context = DrawContext::create(&mut window);

        // set up opengl stuff here
        // enable depth buffer
        gl::call!(Enable(gl::raw::DEPTH_TEST));
        // enable blending
        gl::call!(Enable(gl::raw::BLEND));
        gl::call!(BlendFunc(gl::raw::SRC_ALPHA, gl::raw::ONE_MINUS_SRC_ALPHA));
        // enable gamma correction
        gl::call!(Enable(gl::raw::FRAMEBUFFER_SRGB));

        Self {
            glfw,
            window,
            event_pump,
            draw_context,
        }
    }

    fn run(mut self) {
        self.window.show();

        let mut game = Game::new(&self.draw_context);

        let mut last = Instant::now();
        while !self.window.should_close() {
            self.glfw.poll_events();
            for (_, e) in glfw::flush_messages(&self.event_pump) {
                match e {
                    glfw::WindowEvent::Key(key, _, glfw::Action::Press, _) => {
                        game.key_press(key, true)
                    }
                    glfw::WindowEvent::Key(key, _, glfw::Action::Release, _) => {
                        game.key_press(key, false)
                    }
                    _ => (),
                }
            }

            let now = Instant::now();
            let dt = now - last;
            game.tick(dt);
            last = now;

            unsafe {
                gl::raw::Clear(gl::raw::COLOR_BUFFER_BIT | gl::raw::DEPTH_BUFFER_BIT);
            }
            game.draw();
            self.window.swap_buffers();
        }
    }
}

fn main() {
    println!("Hello, world!");

    let window = Window::new();
    window.run()
}
