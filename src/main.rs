extern crate glfw;
extern crate ash;
extern crate winapi;

use std::sync::mpsc::Receiver;
use std::ffi::CString;
use std::os::raw::c_char;
use glfw::Glfw;
use ash::{Instance, vk};
use winapi::um::libloaderapi::GetModuleHandleW;

struct HelloTriangleApplication {
    glfw : Glfw,
    window : Option<glfw::Window>,
    event : Option<Receiver<(f64, glfw::WindowEvent)>>,
    width : u32,
    height : u32,
    vk_entry : ash::Entry,
    instance : Option<Instance>,
    physical_device : Option<vk::PhysicalDevice>,
    device : Option<ash::Device>,
    graphics_queue : Option<vk::Queue>,
    surface : Option<vk::SurfaceKHR>
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
            instance : None,
            physical_device : None,
            device : None,
            graphics_queue : None,
            surface : None
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
        self.create_surface();
        self.pick_physical_device();
        self.create_logical_device();
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


    fn create_surface(&mut self) {
        let hinstance = unsafe{ GetModuleHandleW(std::ptr::null()) as *const std::ffi::c_void };
        let hwnd = self.window.as_ref().unwrap().get_win32_window();
        let create_info = vk::Win32SurfaceCreateInfoKHR{
            s_type : vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            p_next : std::ptr::null(),
            flags : Default::default(),
            hinstance : hinstance,
            hwnd : hwnd,
        };

        let win32_surface_loader = ash::extensions::khr::Win32Surface::new(&self.vk_entry, self.instance.as_ref().unwrap());
        self.surface = Option::Some(
            unsafe{ win32_surface_loader.create_win32_surface(&create_info, None).unwrap() }
        );
    }

    fn pick_physical_device(&mut self){
        let instance_ref = self.instance.as_ref().unwrap();
        let devices = unsafe{instance_ref.enumerate_physical_devices().unwrap()};
        for device in devices {
            if self.is_device_suitable(&device){
                self.physical_device = Option::Some(device);
                break;
            }
        }

        if let None = self.physical_device {
            panic!("failed to find GPUs with Vulkan support!");
        }
    }

    fn is_device_suitable(&self, device : &vk::PhysicalDevice) -> bool{
        let queue_family_index = self.find_queue_families(device);
        return queue_family_index.is_some();
    }

    fn find_queue_families(&self, device : &vk::PhysicalDevice) -> Option<usize> {
        let instance_ref = self.instance.as_ref().unwrap();
        let queue_families = unsafe{instance_ref.get_physical_device_queue_family_properties(*device)};
        for (i, queue_family) in queue_families.into_iter().enumerate(){
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS){
                return Option::Some(i);
            }
        }

        None
    }

    fn create_logical_device(&mut self){
        let instance_ref = self.instance.as_ref().unwrap();
        let physical_device_ref = self.physical_device.as_ref().unwrap();
        let indices = self.find_queue_families(physical_device_ref);

        let queue_priority : f32 = 1.0;

        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type : vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next : std::ptr::null(),
            queue_family_index : indices.clone().unwrap() as u32,
            queue_count : 1,
            p_queue_priorities : &queue_priority,
            flags : vk::DeviceQueueCreateFlags::empty()
        };

        let device_features = vk::PhysicalDeviceFeatures::default();

        let create_info = vk::DeviceCreateInfo {
            s_type : vk::StructureType::DEVICE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : vk::DeviceCreateFlags::empty(),
            p_queue_create_infos : &queue_create_info,
            queue_create_info_count : 1,
            p_enabled_features : &device_features,
            enabled_extension_count : 0,
            pp_enabled_extension_names : std::ptr::null(),
            enabled_layer_count : 0,
            pp_enabled_layer_names : std::ptr::null()
        };

        self.device = Option::Some(
            unsafe{
                instance_ref.create_device(*physical_device_ref, &create_info, None).unwrap()
            }
        );

        self.graphics_queue = Option::Some(unsafe{
            self.device.as_ref().unwrap().get_device_queue(indices.unwrap() as u32, 0)
        });
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
