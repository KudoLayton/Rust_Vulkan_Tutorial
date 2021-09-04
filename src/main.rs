extern crate glfw;

use std::sync::mpsc::Receiver;
use glfw::Glfw;

struct HelloTriangleApplication {
    glfw : Glfw,
    window : Option<glfw::Window>,
    event : Option<Receiver<(f64, glfw::WindowEvent)>>,
    width : u32,
    height : u32
}

impl HelloTriangleApplication {
    pub fn run(&mut self) {
        self.init_window();
        self.init_vulkan();
        self.main_loop();
    }

    fn new() -> HelloTriangleApplication {
        HelloTriangleApplication {
            glfw : glfw::init(glfw::FAIL_ON_ERRORS).unwrap(),
            window : None,
            event : None,
            width : 800,
            height : 600
        }
    }

    fn init_window(&mut self){
        self.glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        self.glfw.window_hint(glfw::WindowHint::Resizable(false));
        let window = self.glfw.create_window(self.width, self.height, "Vulkan", glfw::WindowMode::Windowed).unwrap();
        self.window = Option::Some(window.0);
        self.event = Option::Some(window.1);
    }

    fn init_vulkan(&mut self){
    }

    fn main_loop(&mut self){
        let window_ref = self.window.as_ref().unwrap();
        while !window_ref.should_close(){
            self.glfw.poll_events();
        }
    }
}

fn main() {
    let mut app = HelloTriangleApplication::new();
    app.run();
}
