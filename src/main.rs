use std::{
    path::Path,
    time::{Duration, Instant},
};

use gl::{DrawContext, Shader};
use glfw::{Context, OpenGlProfileHint};
use math::{Mat3, Vec3};
use render::{Instance, InstancedShapeManager};
use glfw::WindowHint;

use crate::math::Mat4;

mod common;
mod gl;
mod math;
mod render;
mod scene;

struct Game<'a> {
    quads: InstancedShapeManager<'a>,
    shader: Shader<'a>,
}

impl<'a> Game<'a> {
    fn new(ctx: &'a DrawContext) -> Self {
        let shader = Shader::from_file(ctx, Path::new("res/shaders/basic"))
            .expect("Failed to compile shader");

        let screen = Mat4::screen(2.0, 2.0);

        let mut quads= InstancedShapeManager::quads(ctx, 1);
        quads.new_instance(Some(Instance {
            // transform: Mat3::translate(translate)
            col: (0.0, 1.0, 0.0).into(),
            ..Default::default()
        })).expect("Bad alloc of instance");
        Self {
            quads,
            shader,
        }
    }

    fn draw(&self) {
        self.quads.draw(&self.shader);
    }

    fn tick(&mut self, dt: Duration) {}
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
            for _ in glfw::flush_messages(&self.event_pump) {}

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
