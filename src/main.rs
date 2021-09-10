extern crate glfw;
extern crate ash;
extern crate winapi;

use std::io::Read;
use std::mem::swap;
use std::ops::Deref;
use std::{ops::Index, sync::mpsc::Receiver};
use std::ffi::CString;
use std::os::raw::c_char;
use ash::vk::PipelineShaderStageCreateFlags;
use glfw::Glfw;
use ash::{Instance, vk};
use winapi::um::libloaderapi::GetModuleHandleW;

struct SwapChainSupportDetails{
    capabilities : vk::SurfaceCapabilitiesKHR,
    formats : Vec<vk::SurfaceFormatKHR>,
    present_modes : Vec<vk::PresentModeKHR>
}

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
    surface : Option<vk::SurfaceKHR>,
    present_queue : Option<vk::Queue>,
    swap_chain : Option<vk::SwapchainKHR>,
    swap_chain_images : Option<Vec<vk::Image>>,
    swap_chain_image_format : Option<vk::Format>,
    swap_chain_extent : Option<vk::Extent2D>,
    swap_chain_image_views : Vec<vk::ImageView>
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
            surface : None,
            present_queue : None,
            swap_chain : None,
            swap_chain_images : None,
            swap_chain_image_format : None,
            swap_chain_extent : None,
            swap_chain_image_views : Vec::new()
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
        self.create_swap_chain();
        self.create_image_views();
        self.create_graphics_pipeline();
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

        let surface_loader = ash::extensions::khr::Win32Surface::new(&self.vk_entry, self.instance.as_ref().unwrap());
        self.surface = Option::Some(
            unsafe{ surface_loader.create_win32_surface(&create_info, None).unwrap() }
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
        let queue_family = self.find_queue_families(device);
        let extension_supported = self.check_device_extension_support(device);
        let swap_chain_adequate = match extension_supported {
            false => false,
            true => {
                let swap_chain_support = self.query_swap_chain_support(device);
                !swap_chain_support.formats.is_empty() && !swap_chain_support.present_modes.is_empty()
            }
        };
        return queue_family.0.is_some() && queue_family.1.is_some() && extension_supported && swap_chain_adequate;
    }

    fn find_queue_families(&self, device : &vk::PhysicalDevice) -> (Option<usize>, Option<usize>) {
        let instance_ref = self.instance.as_ref().unwrap();
        let queue_families = unsafe{instance_ref.get_physical_device_queue_family_properties(*device)};
        let surface_loader = ash::extensions::khr::Surface::new(&self.vk_entry, &self.instance.as_ref().unwrap());
        let mut graphics_family : Option<usize> = None;
        let mut present_family : Option<usize> = None;
        for (i, queue_family) in queue_families.into_iter().enumerate(){
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS){
                graphics_family = Option::Some(i);
                
            }

            if let Ok(_) = unsafe{
                surface_loader.get_physical_device_surface_support(*device, i as u32, *self.surface.as_ref().unwrap())
            }{
                present_family = Option::Some(i);
            }
        }

        return(graphics_family, present_family)
    }

    fn query_swap_chain_support(&self, device : &vk::PhysicalDevice) -> SwapChainSupportDetails{
        let surface = ash::extensions::khr::Surface::new(&self.vk_entry, self.instance.as_ref().unwrap());
        let capabilities = unsafe{
            surface.get_physical_device_surface_capabilities(*device, *self.surface.as_ref().unwrap()).unwrap()
        };

        let formats = unsafe{
            surface.get_physical_device_surface_formats( *device, *self.surface.as_ref().unwrap()).unwrap()
        };

        let present_modes = unsafe{
            surface.get_physical_device_surface_present_modes(*device,  *self.surface.as_ref().unwrap()).unwrap()
        };

        return SwapChainSupportDetails{
            capabilities : capabilities,
            formats : formats,
            present_modes : present_modes
        };
    }

    fn check_device_extension_support(&self, device : &vk::PhysicalDevice) -> bool {
        let instance_ref = self.instance.as_ref().unwrap();
        let device_extensions = std::ffi::CString::new("VK_KHR_swapchain").unwrap();
        let available_extensions = unsafe {
            instance_ref.enumerate_device_extension_properties(*device).unwrap()
        };
        for extension in available_extensions {
            let extension_name_null_pos = extension.extension_name.iter().position(|&x| x == 0);

            let extension_name : Vec<u8> = match extension_name_null_pos{
                None => extension.extension_name.to_vec().iter().map(|&x| x as u8).collect(),
                Some(idx) => (&extension.extension_name)[0..idx].to_vec().iter().map(|&x| x as u8).collect()
            };
            
            let extension_name = CString::new(extension_name).unwrap();
            if device_extensions == extension_name{
                return true;
            }
        }
        return false;
    }



    fn create_logical_device(&mut self){
        let instance_ref = self.instance.as_ref().unwrap();
        let physical_device_ref = self.physical_device.as_ref().unwrap();
        let indices = self.find_queue_families(physical_device_ref);
        let unique_queue_families = [indices.clone().0.unwrap(), indices.clone().1.unwrap()];

        let queue_priority : f32 = 1.0;
        let mut queue_create_infos : Vec<vk::DeviceQueueCreateInfo> = Vec::new();

        for queue_family in unique_queue_families.iter(){
            let queue_create_info = vk::DeviceQueueCreateInfo {
                s_type : vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next : std::ptr::null(),
                queue_family_index : *queue_family as u32,
                queue_count : 1,
                p_queue_priorities : &queue_priority,
                flags : vk::DeviceQueueCreateFlags::empty()
            };
            queue_create_infos.push(queue_create_info);
        }

        let device_features = vk::PhysicalDeviceFeatures::default();

        let swapchain_extensions_cstring : Vec<CString> = vec![CString::new("VK_KHR_swapchain").unwrap()];
        let swapchain_extension_vec_char : Vec<*const c_char> = swapchain_extensions_cstring.iter().map(|x| x.as_ptr()).collect();

        let create_info = vk::DeviceCreateInfo {
            s_type : vk::StructureType::DEVICE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : vk::DeviceCreateFlags::empty(),
            p_queue_create_infos : queue_create_infos.as_ptr(),
            queue_create_info_count : queue_create_infos.len() as u32,
            p_enabled_features : &device_features,
            enabled_extension_count : 1,
            pp_enabled_extension_names : swapchain_extension_vec_char.as_ptr(),
            enabled_layer_count : 0,
            pp_enabled_layer_names : std::ptr::null()
        };

        self.device = Option::Some(
            unsafe{
                instance_ref.create_device(*physical_device_ref, &create_info, None).unwrap()
            }
        );

        self.graphics_queue = Option::Some(unsafe{
            self.device.as_ref().unwrap().get_device_queue(indices.0.unwrap() as u32, 0)
        });
        self.present_queue = Option::Some(unsafe{
            self.device.as_ref().unwrap().get_device_queue(indices.1.unwrap() as u32, 0)
        });
    }

    fn choose_swap_surface_format(&self, available_formats : &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR{
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR{
                return *available_format;
            }
        }
        return available_formats[0];
    }

    fn choose_swap_present_mode(&self, avaiable_present_modes : &Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR{
        for avaiable_present_mode in avaiable_present_modes {
            if *avaiable_present_mode == vk::PresentModeKHR::MAILBOX{
                return *avaiable_present_mode;
            }
        }
        return vk::PresentModeKHR::FIFO;
    }

    fn choose_swap_extent(&self, capabilities : &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        match capabilities.current_extent.width{
            u32::MAX => {
                let buffer_size = self.window.as_ref().unwrap().get_framebuffer_size();
                vk::Extent2D{
                    width : buffer_size.0.clamp(capabilities.min_image_extent.width as i32, capabilities.max_image_extent.width as i32) as u32,
                    height : buffer_size.1.clamp(capabilities.min_image_extent.height as i32, capabilities.max_image_extent.height as i32) as u32
                }
            },
            _ => capabilities.current_extent
        }
    }

    fn create_swap_chain(&mut self){
        let swap_chain_support = self.query_swap_chain_support(self.physical_device.as_ref().unwrap());
        let surface_format = self.choose_swap_surface_format(&swap_chain_support.formats);
        let present_mode = self.choose_swap_present_mode(&swap_chain_support.present_modes);
        let extent = self.choose_swap_extent(&swap_chain_support.capabilities);

        let mut image_count = swap_chain_support.capabilities.min_image_count + 1;
        if swap_chain_support.capabilities.max_image_count > 0 && image_count > swap_chain_support.capabilities.max_image_count {
            image_count = swap_chain_support.capabilities.max_image_count;
        }

        let indices = self.find_queue_families(self.physical_device.as_ref().unwrap());
        let queue_family_indices = [indices.0.unwrap(), indices.1.unwrap()];

        let image_sharing_mode;
        let queue_family_index_count;
        let p_queue_familiy_indices;

        if indices.0.unwrap() != indices.1.unwrap() {
            image_sharing_mode = vk::SharingMode::CONCURRENT;
            queue_family_index_count = 2;
            p_queue_familiy_indices = queue_family_indices.as_ptr() as *const u32;
        }else{
            image_sharing_mode = vk::SharingMode::EXCLUSIVE;
            queue_family_index_count = 0;
            p_queue_familiy_indices = std::ptr::null();
        }

        let create_info = vk::SwapchainCreateInfoKHR {
            s_type : vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next : std::ptr::null(),
            surface : *self.surface.as_ref().unwrap(),
            min_image_count : image_count,
            image_format : surface_format.format,
            image_color_space : surface_format.color_space,
            image_extent : extent,
            image_array_layers : 1,
            image_usage : vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode : image_sharing_mode,
            queue_family_index_count : queue_family_index_count,
            p_queue_family_indices : p_queue_familiy_indices,
            pre_transform : swap_chain_support.capabilities.current_transform,
            composite_alpha : vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode : present_mode,
            clipped : 1,
            old_swapchain : vk::SwapchainKHR::null(),
            flags : vk::SwapchainCreateFlagsKHR::empty()
        };

        let swap_chain = ash::extensions::khr::Swapchain::new(
            self.instance.as_ref().unwrap(),
            self.device.as_ref().unwrap()
        );

        self.swap_chain = Some(unsafe {
            swap_chain.create_swapchain(&create_info, None)
            .expect("failed to create swap chain!")
        });

        self.swap_chain_images = Some(unsafe{
            swap_chain.get_swapchain_images(*self.swap_chain.as_ref().unwrap()).unwrap()
        });

        self.swap_chain_image_format = Some(surface_format.format);

        self.swap_chain_extent = Some(extent);
    }

    fn create_image_views(&mut self){
        self.swap_chain_image_views.resize(self.swap_chain_images.as_ref().unwrap().len(), vk::ImageView::null());
        let images_len = self.swap_chain_image_views.len();

        
        for idx in 0 .. images_len {
            let component_mapping = vk::ComponentMapping {
                r : vk::ComponentSwizzle::IDENTITY,
                g : vk::ComponentSwizzle::IDENTITY,
                b : vk::ComponentSwizzle::IDENTITY,
                a : vk::ComponentSwizzle::IDENTITY,
            };

            let subresource_range = vk::ImageSubresourceRange {
                aspect_mask : vk::ImageAspectFlags::COLOR,
                base_mip_level : 0,
                level_count : 1,
                base_array_layer : 0,
                layer_count : 1
            };

            let create_info = vk::ImageViewCreateInfo{
                s_type : vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next : std::ptr::null(),
                image : self.swap_chain_images.as_ref().unwrap()[idx],
                view_type : vk::ImageViewType::TYPE_2D,
                format : *self.swap_chain_image_format.as_ref().unwrap(),
                components : component_mapping,
                subresource_range : subresource_range,
                flags : vk::ImageViewCreateFlags::empty(),
            };

            self.swap_chain_image_views[idx] = unsafe{
                self.device.as_ref().unwrap().create_image_view(&create_info, None)
                .expect("failed to create image views!")
            };
        }
    }

    fn create_shader_module(&self, code : Vec<u8>) -> vk::ShaderModule{
        let create_info = vk::ShaderModuleCreateInfo {
            s_type : vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next : std::ptr::null(),
            code_size : code.len(),
            p_code : code.as_ptr() as *const u32,
            flags : vk::ShaderModuleCreateFlags::empty()
        };

        return unsafe{self.device.as_ref().unwrap().create_shader_module(&create_info, None).expect("failed to create shader module!")};
    }

    fn create_graphics_pipeline(&mut self){
        let vert_shader_code = read_file(std::path::Path::new("shaders/vert.spv"));
        let frag_shader_code = read_file(std::path::Path::new("shaders/frag.spv"));

        let vert_shader_module = self.create_shader_module(vert_shader_code);
        let frag_shader_module = self.create_shader_module(frag_shader_code);
        let name = std::ffi::CString::new("main").unwrap();

        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            s_type : vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next : std::ptr::null(),
            stage : vk::ShaderStageFlags::VERTEX,
            module : vert_shader_module,
            p_name : name.as_ptr() as *const i8,
            flags : PipelineShaderStageCreateFlags::empty(),
            p_specialization_info : std::ptr::null()
        };

        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            s_type : vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next : std::ptr::null(),
            stage : vk::ShaderStageFlags::FRAGMENT,
            module : frag_shader_module,
            p_name : name.as_ptr() as *const i8,
            flags : PipelineShaderStageCreateFlags::empty(),
            p_specialization_info : std::ptr::null()
        };

        let shader_stages = [vert_shader_stage_info, frag_shader_stage_info];
    }

    fn main_loop(&mut self){
        let window_ref = self.window.as_ref().unwrap();
        while !window_ref.should_close(){
            self.glfw.poll_events();
        }
    }
}

fn read_file(file_name : &std::path::Path) -> Vec<u8>{
    let mut file = std::fs::File::open(file_name).expect("failed to open file");
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer).expect("failed to open file");
    return buffer;

}

fn main() {
    let mut app = HelloTriangleApplication::new();
    app.run();
}
