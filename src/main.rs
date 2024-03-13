// #![windows_subsystem = "windows"]

use std::mem::{self, swap};

use std::sync::mpsc::{self, Receiver, Sender};

use std::thread::sleep;
use std::time::{Duration, Instant};

use common::AsBytes;
use entity::EntityManager;
use gl::{DrawContext, UniformBuffer};
use glfw::{Context, OpenGlProfileHint};
use glfw::{Key, WindowHint};
use math::{ease, lerp, Vec2, Vec3};
use palette::Palette;
use render::fireball::FireballManager;
use render::instanced::InstancedShapeManager;
use render::shield::ShieldManager;
use render::swoop::SwoopManager;
use render::text::TextManager;
use render::RenderManager;
use sound::{SoundManager, Sounds};

use crate::math::{Mat4, Vec4};

mod archetype;
mod common;
mod entity;
mod gl;
mod math;
mod palette;
mod render;
mod resources;
mod sound;
mod time;
mod world;

const SCALE_FACTOR: f32 = 0.85;
// mouse to world coords
// mouse is in screen space coordinates
// normalize to [0,1] range
// remap to [-1,1] range
// multiply by the inverse of the screen matrix
// mouse is now in world coordinates

struct Game<'a> {
    pan_to_hall_trigger: Option<Receiver<()>>,
    pan_to_room_trigger: Option<Receiver<()>>,
    open_hall_trigger: Receiver<()>,

    // mouse position in world coordinates
    view_width: f32,
    view_height: f32,

    lerping: bool,
    accum: Duration,
    next_view: Mat4,
    current_view: Mat4,
    last_view: Mat4,

    last_room: Option<world::Room>,
    room: world::Room,
    room_ctr: usize,
    man: EntityManager,
    keystroke_tx: Sender<Key>,
    mouse_tx: Sender<Vec2>,
    palette: Palette,
    renderer: RenderManager<'a>,
    sound: SoundManager,
    common_uniforms: UniformBuffer<'a>,
}

impl<'a> Game<'a> {
    fn new(ctx: &'a DrawContext, view_width: f32, view_height: f32) -> Self {
        let normal = Mat4::screen(Vec2::default(), 75.0, 75.0);

        let tile_renderer = InstancedShapeManager::quads(ctx, 16 * 1024);
        let fireball_renderer = FireballManager::new(ctx, 512);

        let (keystroke_tx, keystroke_rx) = mpsc::channel();
        let (mouse_tx, mouse_rx) = mpsc::channel();
        let sound = SoundManager::new();
        let mut man = EntityManager::new(keystroke_rx, mouse_rx, sound.player());
        let (room, open_hall_trigger) = world::Room::tut_controls(&mut man);
        let starting_view = room.view();

        let common_uniforms = UniformBuffer::new(ctx);
        common_uniforms.bind_buffer_base(0);
        common_uniforms.set(
            unsafe { starting_view.as_bytes() },
            gl::buffer_flags::DYNAMIC_STORAGE,
        );

        // crt startup sequence
        sound.play(Sounds::CrtClick);
        sleep(Duration::from_millis(1250));
        sound.play(Sounds::CrtBuzz);
        sleep(Duration::from_millis(1500));
        sound.play(Sounds::CrtOn);

        let mut renderer = RenderManager::new(ctx);
        renderer.add_renderer(tile_renderer);
        renderer.add_renderer(fireball_renderer);
        renderer.add_renderer(ShieldManager::new(ctx, 512));
        renderer.add_renderer(SwoopManager::new(ctx, 16));
        renderer.add_renderer(TextManager::new(ctx));

        Self {
            pan_to_hall_trigger: None,
            pan_to_room_trigger: None,
            open_hall_trigger,

            view_width,
            view_height,

            lerping: false,
            accum: Duration::ZERO,
            current_view: room.view(),
            next_view: normal,
            last_view: normal,

            last_room: None,
            room,
            room_ctr: 0,
            man,
            keystroke_tx,
            mouse_tx,
            palette: palette::crt(),
            renderer,
            sound,
            common_uniforms,
        }
    }

    fn draw(&mut self) {
        self.man.draw(&mut self.renderer, self.palette);
        self.renderer.draw();
    }

    fn move_camera(&mut self, new_view: Mat4) {
        self.next_view = new_view;
        self.last_view = self.current_view;
        self.lerping = true;
        self.sound.play(Sounds::CameraPan);
    }

    fn tick(&mut self, dt: Duration) {
        let max = Duration::from_millis(1000);
        if self.lerping {
            if self.accum < max {
                let pct = self.accum.as_secs_f32() / max.as_secs_f32();
                // let p = self.bezier.apply(pct);
                let p = ease::out_expo(pct);
                self.current_view = lerp(self.last_view, self.next_view, p);
                self.common_uniforms
                    .update(0, unsafe { self.current_view.as_bytes() });
                self.accum += dt;
            } else {
                self.lerping = false;
                self.accum = Duration::ZERO;
                swap(&mut self.current_view, &mut self.next_view);

                // now that the last room is out of view get rid of it
                if let Some(mut room) = self.last_room.take() {
                    room.destroy(&mut self.man);
                }
            }
        }

        self.man.tick(dt);

        // hall enter trigger
        if self
            .pan_to_hall_trigger
            .as_ref()
            .map(|rx| rx.try_recv().is_ok())
            .unwrap_or_default()
        {
            // pan to hall
            self.move_camera(self.room.view_hall());

            // close hall entrance off
            //self.current_room.close_hall_entrance(&mut self.man);

            // prepare next room
            // it's okay to reset open_hall_trigger here
            // since if it must be that the hall is already open
            let (mut next_room, next_trigger) =
                world::next_room(&mut self.room_ctr)(&mut self.man, &self.room);
            self.open_hall_trigger = next_trigger;

            self.room.swap(&mut next_room);
            self.last_room = Some(next_room);
        }

        // hall leave trigger
        if self
            .pan_to_room_trigger
            .as_ref()
            .map(|rx| rx.try_recv().is_ok())
            .unwrap_or_default()
        {
            // pan to new room
            self.move_camera(self.room.view());
        }

        // hall open trigger
        if self.open_hall_trigger.try_recv().is_ok() {
            if let Some((hall, room)) = self.room.open_hallway(&mut self.man) {
                self.pan_to_hall_trigger = Some(hall);
                self.pan_to_room_trigger = Some(room);
            }
        }
    }

    fn key_press(&mut self, key: Key, is_down: bool) {
        if !is_down {
            return;
        }

        match key {
            Key::G => {
                let view = self.current_view;
                self.move_camera(Mat4::scale(0.25.into()) * view);
            }
            Key::B => {
                if let Some((hall, room)) = self.room.open_hallway(&mut self.man) {
                    self.pan_to_hall_trigger = Some(hall);
                    self.pan_to_room_trigger = Some(room);
                }
            }
            _ => (),
        }

        let _ = self.keystroke_tx.send(key);
    }

    fn mouse_move(&mut self, screen_x: f64, screen_y: f64) {
        // screen coords
        // normalized [0,1]
        let nx = screen_x as f32 / self.view_width;
        let ny = screen_y as f32 / self.view_height;

        // normalized [-1,1]
        let ndc_x =   2.0 * nx - 1.0;
        let ndc_y = -(2.0 * ny - 1.0);

        // world coords
        let in_view = self.current_view.invert_screem();
        // println!("view:\n{}", self.current_view);
        // println!("inverse:\n{}", in_view);
        // println!("unit?:\n{}", self.current_view * in_view);
        let Vec4 { x, y, .. } = in_view * Vec4::position(Vec3::new(ndc_x, ndc_y, 0.0));

        let pos = Vec2::new(x, y);
        // println!("mouse: {pos:?}");
        let _ = self.mouse_tx.send(pos);
    }
}

struct Window {
    width: f32,
    height: f32,
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

        let (screen_width, screen_height) = glfw.with_primary_monitor(|_, m| {
            let m = m.expect("Can't get primary monitor");
            let mode = m.get_video_mode().expect("Can't get video mode");
            (mode.width as f32, mode.height as f32)
        });

        // aspect ratio 1:1
        let dim = screen_height.min(screen_width);
        let width = SCALE_FACTOR * dim as f32;
        let height = SCALE_FACTOR * dim as f32;

        let (mut window, event_pump) = glfw
            .create_window(
                width as u32,
                height as u32,
                "snek?",
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create window");

        // window setup
        window.set_resizable(false);
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        let favicon = image::load_from_memory(resources::textures::ICON).unwrap();
        window.set_icon(vec![favicon.into()]);

        // center the window
        let pos_x = (screen_width - width) / 2.0;
        let pos_y = (screen_height - height) / 2.0;
        window.set_pos(pos_x as i32, pos_y as i32);

        let draw_context = DrawContext::create(&mut window);

        // set up opengl stuff here
        // backface culling & apparently I can't specify vertices
        gl::call!(FrontFace(CW));
        gl::call!(Enable(CULL_FACE));
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
            width,
            height,

            glfw,
            window,
            event_pump,
            draw_context,
        }
    }

    fn run(mut self) {
        self.window.show();
        gl::call!(Clear(COLOR_BUFFER_BIT));
        self.window.swap_buffers();
        let mut game = Game::new(&self.draw_context, self.width, self.height);

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
                    glfw::WindowEvent::CursorPos(x, y) => {
                        game.mouse_move(x, y);
                    }
                    _ => (),
                }
            }

            let now = Instant::now();
            let dt = now - last;
            game.tick(dt);
            last = now;

            game.draw();
            self.window.swap_buffers();
        }
    }
}

fn main() {
    let window = Window::new();
    window.run()
}
