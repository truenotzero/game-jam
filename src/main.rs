use std::sync::mpsc::{self, Sender};
use std::time::{Duration, Instant};

use common::AsBytes;
use entity::EntityManager;
use gl::{DrawContext, UniformBuffer};
use glfw::{Context, OpenGlProfileHint};
use glfw::{Key, WindowHint};
use math::{ease, Vec3};
use palette::Palette;
use render::InstancedShapeManager;

use crate::math::Mat4;

mod archetype;
mod common;
mod entity;
mod gl;
mod math;
mod noise;
mod palette;
mod render;
mod time;
mod world;

struct Game<'a> {
    room: world::Room,
    man: EntityManager,
    keystroke_tx: Sender<Key>,
    palette: Palette,
    renderer: InstancedShapeManager<'a>,
    common_uniforms: UniformBuffer<'a>,
}

impl<'a> Game<'a> {
    fn new(ctx: &'a DrawContext) -> Self {
        let width = 50;
        let height = 50;
        let normal = Mat4::screen(width as f32, height as f32);

        let common_uniforms = UniformBuffer::new(ctx);
        common_uniforms.bind_buffer_base(0);
        common_uniforms.set(
            unsafe { normal.as_bytes() },
            gl::buffer_flags::DYNAMIC_STORAGE,
        );

        let renderer = InstancedShapeManager::quads(ctx, 16 * 1024);

        let (keystroke_tx, keystroke_rx) = mpsc::channel();
        let mut man = EntityManager::new(keystroke_rx);
        archetype::fruit::new(&mut man);
        archetype::snake::new(&mut man);

        Self {
            room: world::Room::main(&mut man),
            man,
            keystroke_tx,
            palette: palette::aperture(),
            renderer,
            common_uniforms,
        }
    }

    fn draw(&mut self) {
        self.man.draw(&mut self.renderer, self.palette);
        self.renderer.draw();
    }

    fn tick(&mut self, dt: Duration) {
        self.man.tick(dt);
    }

    fn key_press(&mut self, key: Key, is_down: bool) {
        if !is_down {
            return;
        }
        let _ = self.keystroke_tx.send(key);
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
        // gl::call!(Enable(gl::raw::FRAMEBUFFER_SRGB));

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
