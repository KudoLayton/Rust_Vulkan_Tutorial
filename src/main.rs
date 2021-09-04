extern crate glfw;
extern crate ash;

use std::sync::mpsc::Receiver;
use std::ffi::CString;
use std::os::raw::c_char;
use glfw::Glfw;
use ash::{Instance, vk};

struct HelloTriangleApplication {
    glfw : Glfw,
    window : Option<glfw::Window>,
    event : Option<Receiver<(f64, glfw::WindowEvent)>>,
    width : u32,
    height : u32,
    vk_entry : ash::Entry,
    instance : Option<Instance>
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
            height : 600,
            vk_entry : unsafe{ash::Entry::new().unwrap()},
            instance : None
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
        self.create_instance();
    }

    fn create_instance(&mut self){
        let name = CString::new("Hello Triangle").unwrap();
        let engine_name = CString::new("No Engine").unwrap();
        let app_info = vk::ApplicationInfo {
            s_type : vk::StructureType::APPLICATION_INFO,
            p_next : std::ptr::null(),
            p_application_name : name.as_ptr(),
            application_version : vk::make_api_version(1, 0, 0, 0),
            p_engine_name : engine_name.as_ptr(),
            engine_version : vk::make_api_version(1, 0, 0, 0),
            api_version : vk::API_VERSION_1_0
        };

        let glfw_extensions = self.glfw.get_required_instance_extensions().unwrap();
        let glfw_extensions_cstring : Vec<CString> = glfw_extensions.into_iter().map(|x| CString::new(x).unwrap()).collect();
        let glfw_extension_vec_char : Vec<*const c_char> = glfw_extensions_cstring.iter().map(|x| x.as_ptr()).collect();

        let create_info = vk::InstanceCreateInfo{
            s_type : vk::StructureType::INSTANCE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : vk::InstanceCreateFlags::empty(),
            p_application_info : &app_info,
            pp_enabled_layer_names : std::ptr::null(),
            enabled_layer_count : 0,
            enabled_extension_count : glfw_extension_vec_char.len() as u32,
            pp_enabled_extension_names : glfw_extension_vec_char.as_ptr()
        };

        self.instance = Option::Some(unsafe{self.vk_entry.create_instance(&create_info, None).unwrap()});
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
