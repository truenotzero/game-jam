use std::mem;
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::time::{Duration, Instant};

use common::AsBytes;
use entity::EntityManager;
use fireball::{Fireball, FireballManager};
use gl::{DrawContext, Shader, UniformBuffer};
use glfw::{Context, OpenGlProfileHint};
use glfw::{Key, WindowHint};
use math::ease::UnitBezier;
use math::{ease, lerp, Vec2, Vec3};
use palette::Palette;
use render::InstancedShapeManager;

use crate::math::{Mat4, Vec4};

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
mod fireball;


const WIDTH: u32 = 1200;
const HEIGHT: u32 = 1200;
// mouse to world coords
// mouse is in screen space coordinates
// normalize to [0,1] range
// remap to [-1,1] range
// multiply by the inverse of the screen matrix
// mouse is now in world coordinates

struct Game<'a> {
    f: FireballManager<'a>,

    lerping: bool,
    accum: Duration,
    bezier: UnitBezier,
    next_view: Mat4,
    current_view: Mat4,

    room: world::Room,
    man: EntityManager,
    keystroke_tx: Sender<Key>,
    palette: Palette,
    renderer: InstancedShapeManager<'a>,
    common_uniforms: UniformBuffer<'a>,
}

impl<'a> Game<'a> {
    fn new(ctx: &'a DrawContext) -> Self {
        let mut f = FireballManager::new(ctx, 16);
        f.push(Fireball {
            pos: Vec2::new(-10.0, -10.0),
            col: palette::dark_pastel().snake,
            radius: 0.5,
        });

        let normal = Mat4::screen(Vec2::default(), 50.0, 50.0);


        let renderer = InstancedShapeManager::quads(ctx, 16 * 1024);

        let (keystroke_tx, keystroke_rx) = mpsc::channel();
        let mut man = EntityManager::new(keystroke_rx);
        archetype::fruit::new(&mut man);
        archetype::snake::new(&mut man);
        let room= world::Room::main(&mut man);
        let starting_view = room.view();

        let common_uniforms = UniformBuffer::new(ctx);
        common_uniforms.bind_buffer_base(0);
        common_uniforms.set(
            unsafe { starting_view.as_bytes() },
            gl::buffer_flags::DYNAMIC_STORAGE,
        );

        Self {
            f,

            lerping: false,
            accum: Duration::ZERO,
            bezier: UnitBezier::new(0.95, 0.1, 0.1, 0.95, 128),
            current_view: room.view(),
            next_view: normal,
        
            room,
            man,
            keystroke_tx,
            palette: palette::dark_pastel(),
            renderer,
            common_uniforms,
        }
    }

    fn draw(&mut self) {
        self.man.draw(&mut self.renderer, self.palette);
        self.renderer.draw();
        self.f.draw();
    }

    fn tick(&mut self, dt: Duration) {
        let max = Duration::from_millis(1000);
        if self.lerping {
            if self.accum < max {
                let pct = self.accum.as_secs_f32() / max.as_secs_f32();
                // let p = self.bezier.apply(pct);
                let p = ease::out_expo(pct);
                let lerped_matrix = lerp(self.current_view, self.next_view, p);
                self.common_uniforms.update(0, unsafe { lerped_matrix.as_bytes() });
                self.accum += dt;
            } else {
                self.lerping = false;
                self.accum = Duration::ZERO;
                mem::swap(&mut self.current_view, &mut self.next_view);
            }
        }

        self.man.tick(dt);
    }

    fn key_press(&mut self, key: Key, is_down: bool) {
        if !is_down {
            return;
        }

        if key == Key::Space {
            self.lerping = true;
        }

        let _ = self.keystroke_tx.send(key);
    }

    fn mouse_move(&self, x: f64, y: f64) {
        let x = x as f32;
        let y = y as f32;
        println!("raw: ({x:.2},{y:.2})");

        let x = x / WIDTH as f32;
        let y = y / HEIGHT as f32;
        println!("nor: ({x:.2},{y:.2})");
        
        let x = 2.0 * x - 1.0;
        let y = 2.0 * y - 1.0;
        println!("ndc: ({x:.2},{y:.2})");

        let in_view = self.current_view.inverse();
        let Vec4{x, y, ..} = in_view * Vec4::position(Vec3::new(x,y,0.0));
        println!("wld: ({x:.2},{y:.2})");
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
        glfw.window_hint(WindowHint::Samples(Some(16)));

        let (mut window, event_pump) = glfw
            .create_window(WIDTH, HEIGHT, "Snek", glfw::WindowMode::Windowed)
            .expect("Failed to create window");

        // window setup
        window.set_resizable(false);
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        let draw_context = DrawContext::create(&mut window);

        // set up opengl stuff here
        // enable depth buffer
        gl::call!(Enable(DEPTH_TEST));
        // enable blending
        gl::call!(Enable(BLEND));
        gl::call!(BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA));
        // enable gamma correction
        gl::call!(Enable(gl::raw::FRAMEBUFFER_SRGB));
        // enable AA
        gl::call!(Enable(MULTISAMPLE));

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
                    },
                    glfw::WindowEvent::Key(key, _, glfw::Action::Release, _) => {
                        game.key_press(key, false)
                    },
                    glfw::WindowEvent::CursorPos(x, y) => {
                        game.mouse_move(x, y);
                    },
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
    let window = Window::new();
    window.run()
}
