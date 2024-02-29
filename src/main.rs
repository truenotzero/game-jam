use gl::DrawContext;
use glfw::Context;

mod gl;

struct Window {
    draw_context: DrawContext,
    window: glfw::PWindow,
    event_pump: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    glfw: glfw::Glfw,
}

impl Window {
    fn new() -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init GLFW");
        let (mut window, event_pump) = glfw
            .create_window(1200, 1200, "Breakout", glfw::WindowMode::Windowed)
            .expect("Failed to create window");

        // window setup
        let draw_context = DrawContext::create(&mut window);

        Self {
            glfw,
            window,
            event_pump,
            draw_context,
        }
    }
}

fn main() {
    println!("Hello, world!");

    let mut window = Window::new();
    window.window.show();
    while !window.window.should_close() {
        window.glfw.poll_events();
        for _ in glfw::flush_messages(&window.event_pump) {}
        window.window.swap_buffers();
    }
}
