extern crate glfw;
extern crate ash;
extern crate winapi;
extern crate memoffset;
extern crate cgmath;

use std::io::Read;
use std::mem::swap;
use std::ops::{Add, Deref};
use std::{ops::Index, sync::mpsc::Receiver};
use std::ffi::CString;
use std::os::raw::c_char;
use ash::vk::{ClearColorValue, CommandBufferUsageFlags, DescriptorSetLayoutBinding, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateFlags, PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, VertexInputAttributeDescription, VertexInputBindingDescription};
use glfw::Glfw;
use ash::{Instance, vk};
use winapi::um::libloaderapi::GetModuleHandleW;

struct UniformBufferObject {
    model : cgmath::Matrix4<f32>,
    view : cgmath::Matrix4<f32>,
    proj : cgmath::Matrix4<f32>
}

struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    fn get_binding_destcription() -> vk::VertexInputBindingDescription {
        return vk::VertexInputBindingDescription {
            binding : 0,
            stride : std::mem::size_of::<Self>() as u32,
            input_rate : vk::VertexInputRate::VERTEX
        };
    }

    fn get_attribute_descripyions() -> [vk::VertexInputAttributeDescription; 2] {
        return [
            vk::VertexInputAttributeDescription {
                binding : 0,
                location : 0,
                format : vk::Format::R32G32_SFLOAT,
                offset : memoffset::offset_of!(Self, pos) as u32,
            },
            
            vk::VertexInputAttributeDescription {
                binding : 0,
                location : 1,
                format : vk::Format::R32G32B32_SFLOAT,
                offset : memoffset::offset_of!(Self, color) as u32
            }
        ]
    }
}

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
    swap_chain_image_views : Vec<vk::ImageView>,
    render_pass : Option<vk::RenderPass>,
    descriptor_set_layout : Option<vk::DescriptorSetLayout>,
    pipeline_layout : Option<vk::PipelineLayout>,
    graphics_pipeline : Option<vk::Pipeline>,
    swap_chain_frame_buffers : Vec<vk::Framebuffer>,
    command_pool : Option<vk::CommandPool>,
    command_buffers : Option<Vec<vk::CommandBuffer>>,
    image_available_semaphores : Vec<vk::Semaphore>,
    render_finished_semaphores : Vec<vk::Semaphore>,
    in_flight_fences : Vec<vk::Fence>,
    images_in_flight : Vec<vk::Fence>,
    current_frame : usize,
    vertices : Vec<Vertex>,
    indices : Vec<u16>,
    vertex_buffer : Option<vk::Buffer>,
    vertex_buffer_memory : Option<vk::DeviceMemory>,
    index_buffer : Option<vk::Buffer>,
    index_buffer_memory : Option<vk::DeviceMemory>,
    uniform_buffers : Vec<vk::Buffer>,
    uniform_buffers_memory : Vec<vk::DeviceMemory>,
    start_time : Option<std::time::SystemTime>
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
            swap_chain_image_views : Vec::new(),
            render_pass : None,
            descriptor_set_layout : None,
            pipeline_layout : None,
            graphics_pipeline : None,
            swap_chain_frame_buffers : Vec::new(),
            command_pool : None,
            command_buffers : None,
            image_available_semaphores : Vec::new(),
            render_finished_semaphores : Vec::new(),
            in_flight_fences : Vec::new(),
            images_in_flight : Vec::new(),
            current_frame : 0,
            vertices : vec![
                Vertex{pos: [-0.5, -0.5], color: [1.0, 0.0, 0.0]},
                Vertex{pos: [0.5, -0.5], color: [0.0, 1.0, 0.0]},
                Vertex{pos: [0.5, 0.5], color: [0.0, 0.0, 1.0]},
                Vertex{pos: [-0.5, 0.5], color: [1.0, 1.0, 1.0]},
            ],
            indices : vec![0, 1, 2, 2, 3, 0],
            vertex_buffer : None,
            vertex_buffer_memory : None,
            index_buffer : None,
            index_buffer_memory : None,
            uniform_buffers : Vec::new(),
            uniform_buffers_memory : Vec::new(),
            start_time : None
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
        self.create_render_pass();
        self.create_descriptor_set_layout();
        self.create_graphics_pipeline();
        self.create_framebuffers();
        self.create_command_pool();
        self.create_vertex_buffer();
        self.create_index_buffer();
        self.create_uniform_buffers();
        self.create_command_buffers();
        self.create_sync_objects();
    }

    fn clean_swap_chain(&mut self) {
        unsafe{
            let device_ref = self.device.as_ref().unwrap();
            for swap_chain_frame_buffer in self.swap_chain_frame_buffers.drain(..) {
                device_ref.destroy_framebuffer(swap_chain_frame_buffer, None);
            }

            device_ref.free_command_buffers(
                *self.command_pool.as_ref().unwrap(), 
                self.command_buffers.as_ref().unwrap().as_slice());
            self.command_buffers = None;

            device_ref.destroy_pipeline(self.graphics_pipeline.unwrap(), None);
            self.graphics_pipeline = None;

            device_ref.destroy_pipeline_layout(*self.pipeline_layout.as_ref().unwrap(), None);
            self.graphics_pipeline = None;

            device_ref.destroy_render_pass(*self.render_pass.as_ref().unwrap(), None);
            self.render_pass = None;

            for image_view in self.swap_chain_image_views.drain(..){
                device_ref.destroy_image_view(image_view, None);
            }

            let swap_chain = ash::extensions::khr::Swapchain::new(
                self.instance.as_ref().unwrap(),
                device_ref
            );
            swap_chain.destroy_swapchain(*self.swap_chain.as_ref().unwrap(), None);
            self.swap_chain = None;

            for buffer in self.uniform_buffers.drain(..) {
                device_ref.destroy_buffer(buffer, None);
            }

            for buffer_memory in self.uniform_buffers_memory.drain(..) {
                device_ref.free_memory(buffer_memory, None);
            }
        }
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

    fn create_render_pass (&mut self){
        let color_attachment = vk::AttachmentDescription {
            format : *self.swap_chain_image_format.as_ref().unwrap(),
            samples : vk::SampleCountFlags::TYPE_1,
            load_op : vk::AttachmentLoadOp::CLEAR,
            store_op : vk::AttachmentStoreOp::STORE,
            stencil_load_op : vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op : vk::AttachmentStoreOp::DONT_CARE,
            initial_layout : vk::ImageLayout::UNDEFINED,
            final_layout : vk::ImageLayout::PRESENT_SRC_KHR,
            flags : vk::AttachmentDescriptionFlags::empty()
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment : 0,
            layout : vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        };

        let subpass = vk::SubpassDescription {
            pipeline_bind_point : vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count : 1,
            p_color_attachments : &color_attachment_ref as *const vk::AttachmentReference,
            input_attachment_count : 0,
            p_input_attachments : std::ptr::null(),
            p_resolve_attachments : std::ptr::null(),
            p_depth_stencil_attachment : std::ptr::null(),
            preserve_attachment_count : 0,
            p_preserve_attachments : std::ptr::null(),
            flags : vk::SubpassDescriptionFlags::empty()
        };

        let dependency = vk::SubpassDependency {
            src_subpass : vk::SUBPASS_EXTERNAL,
            dst_subpass : 0,
            src_stage_mask : vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask : vk::AccessFlags::empty(),
            dst_stage_mask : vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask : vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags : vk::DependencyFlags::empty()
        };

        let render_pass_info = vk::RenderPassCreateInfo {
            s_type : vk::StructureType::RENDER_PASS_CREATE_INFO,
            p_next : std::ptr::null(),
            attachment_count : 1,
            p_attachments : &color_attachment as *const vk::AttachmentDescription,
            subpass_count : 1,
            p_subpasses : &subpass as *const vk::SubpassDescription,
            flags : vk::RenderPassCreateFlags::empty(),
            dependency_count : 1,
            p_dependencies : &dependency as *const vk::SubpassDependency
        };

        self.render_pass = Some(unsafe {
            self.device.as_ref().unwrap()
            .create_render_pass(&render_pass_info, None)
            .expect("failed to create render pass")
        });
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

    fn find_memory_type(&self, type_filter : u32, properties : vk::MemoryPropertyFlags) -> Result<u32, &str> {
        let mem_properties = unsafe {
            self.instance.as_ref().unwrap()
            .get_physical_device_memory_properties(
                *self.physical_device.as_ref().unwrap()
            )
        };

        for i in 0..mem_properties.memory_type_count {
            if ((type_filter & (1 << i)) > 0)
            && (mem_properties.memory_types[i as usize].property_flags.contains(properties)) {
                return Ok(i);
            }
        }

        return Err("failed to find suitable memory type!");
    }

    fn create_buffer(
        &self, 
        size : vk::DeviceSize,
        usage : vk::BufferUsageFlags,
        properties : vk::MemoryPropertyFlags,
    ) -> (vk::Buffer, vk::DeviceMemory){
        let buffer_info = vk::BufferCreateInfo {
            s_type : vk::StructureType::BUFFER_CREATE_INFO,
            p_next : std::ptr::null(),
            size : size,
            usage : usage,
            sharing_mode : vk::SharingMode::EXCLUSIVE,
            flags : vk::BufferCreateFlags::empty(),
            queue_family_index_count : 0,
            p_queue_family_indices : std::ptr::null()
        };

        let device_ref = self.device.as_ref().unwrap();

        let buffer = unsafe{
            device_ref.create_buffer(&buffer_info, None)
            .expect("failed to create vertex buffer!")
        };

        let mem_requirements = unsafe{
            device_ref.get_buffer_memory_requirements(buffer)
        };

        let alloc_info = vk::MemoryAllocateInfo {
            s_type : vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next : std::ptr::null(),
            allocation_size : mem_requirements.size,
            memory_type_index : self.find_memory_type(mem_requirements.memory_type_bits, properties).unwrap()
        };

        let buffer_memory = unsafe {
            device_ref.allocate_memory(&alloc_info, None)
            .expect("failed to allocate vertex buffer memory")
        };

        unsafe{
            device_ref.bind_buffer_memory(buffer, buffer_memory, 0);
        }

        (buffer, buffer_memory)
    }

    fn copy_buffer(&self, src_buffer : &vk::Buffer, dst_buffer : &vk::Buffer, size : vk::DeviceSize){
        let device_ref = self.device.as_ref().unwrap();
        let alloc_info = vk::CommandBufferAllocateInfo {
            s_type : vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next : std::ptr::null(),
            level : vk::CommandBufferLevel::PRIMARY,
            command_pool : *self.command_pool.as_ref().unwrap(),
            command_buffer_count : 1
        };

        let command_buffer = unsafe{
            device_ref.allocate_command_buffers(&alloc_info)
            .unwrap()
        };

        let begin_info = vk::CommandBufferBeginInfo {
            s_type : vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next : std::ptr::null(),
            flags : vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            p_inheritance_info : std::ptr::null()
        };

        let copy_region = [vk::BufferCopy {
            src_offset : 0,
            dst_offset : 0,
            size : size
        }];

        let submit_info = [vk::SubmitInfo {
            s_type : vk::StructureType::SUBMIT_INFO,
            p_next : std::ptr::null(),
            command_buffer_count : 1,
            p_command_buffers : command_buffer.as_ptr(),
            wait_semaphore_count : 0,
            p_wait_semaphores : std::ptr::null(),
            signal_semaphore_count : 0,
            p_signal_semaphores : std::ptr::null(),
            p_wait_dst_stage_mask : std::ptr::null()
        }];

        unsafe{
            device_ref.begin_command_buffer(command_buffer[0], &begin_info);
            device_ref.cmd_copy_buffer(command_buffer[0], *src_buffer, *dst_buffer, &copy_region);
            device_ref.end_command_buffer(command_buffer[0]);
            device_ref.queue_submit(*self.graphics_queue.as_ref().unwrap(), &submit_info, vk::Fence::null());
            device_ref.queue_wait_idle(*self.graphics_queue.as_ref().unwrap());
            device_ref.free_command_buffers(*self.command_pool.as_ref().unwrap(), command_buffer.as_slice());
        }
    }

    fn create_vertex_buffer(&mut self) {
        let device_ref = self.device.as_ref().unwrap();
        let buffer_size = (std::mem::size_of::<Vertex>() * self.vertices.len()) as vk::DeviceSize;

        let staging_buffer = self.create_buffer(
            buffer_size, 
            vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, 
        );

        unsafe{
            let data = device_ref.map_memory(
                staging_buffer.1,
                0,
                buffer_size,
                vk::MemoryMapFlags::empty()
            ).expect("Failed to map mamory") as *mut Vertex;
            data.copy_from_nonoverlapping(self.vertices.as_ptr(), self.vertices.len());
            device_ref.unmap_memory(staging_buffer.1);
        }

        let buffer = self.create_buffer(
            buffer_size, 
            vk::BufferUsageFlags::TRANSFER_DST, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, 
        );

        self.copy_buffer(&staging_buffer.0, &buffer.0, buffer_size);

        self.vertex_buffer = Some(buffer.0);
        self.vertex_buffer_memory = Some(buffer.1);

        unsafe{
            device_ref.destroy_buffer(*&staging_buffer.0, None);
            device_ref.free_memory(*&staging_buffer.1, None);
        }
    }

    fn create_index_buffer(&mut self) {
        let device_ref = self.device.as_ref().unwrap();
        let buffer_size = (std::mem::size_of::<u16>() * self.indices.len()) as vk::DeviceSize;

        let staging_buffer = self.create_buffer(
            buffer_size, 
            vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, 
        );

        unsafe{
            let data = device_ref.map_memory(
                staging_buffer.1,
                0,
                buffer_size,
                vk::MemoryMapFlags::empty()
            ).expect("Failed to map mamory") as *mut u16;
            data.copy_from_nonoverlapping(self.indices.as_ptr(), self.indices.len());
            device_ref.unmap_memory(staging_buffer.1);
        }

        let buffer = self.create_buffer(
            buffer_size, 
            vk::BufferUsageFlags::TRANSFER_DST, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, 
        );

        self.copy_buffer(&staging_buffer.0, &buffer.0, buffer_size);

        self.index_buffer = Some(buffer.0);
        self.index_buffer_memory = Some(buffer.1);

        unsafe{
            device_ref.destroy_buffer(*&staging_buffer.0, None);
            device_ref.free_memory(*&staging_buffer.1, None);
        }
    }

    fn create_uniform_buffers(&mut self) {
        let buffer_size = std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;

        self.uniform_buffers.resize(
            self.swap_chain_images.as_ref().unwrap().len(), 
            vk::Buffer::null()
        );

        self.uniform_buffers_memory.resize(
            self.swap_chain_images.as_ref().unwrap().len(),
            vk::DeviceMemory::null()
        );

        for i in 1..self.swap_chain_images.as_ref().unwrap().len() {
            let buffer = unsafe{
                self.create_buffer(
                    buffer_size, 
                    vk::BufferUsageFlags::UNIFORM_BUFFER, 
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
                )
            };
            self.uniform_buffers[i] = buffer.0;
            self.uniform_buffers_memory[i] = buffer.1;
        }
    }

    fn create_descriptor_set_layout(&mut self){
        let ubo_layout_binding = vk::DescriptorSetLayoutBinding {
            binding : 0,
            descriptor_type : vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count : 1,
            stage_flags : vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers : std::ptr::null()
        };

        let layout_info = vk::DescriptorSetLayoutCreateInfo {
            s_type : vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next : std::ptr::null(),
            binding_count : 1,
            p_bindings : &ubo_layout_binding as *const DescriptorSetLayoutBinding,
            flags : vk::DescriptorSetLayoutCreateFlags::empty()
        };

        self.descriptor_set_layout = Some(unsafe {
            self.device.as_ref().unwrap()
            .create_descriptor_set_layout(&layout_info, None)
            .expect("failed to create descriptor set layout!")
        });
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

        let binding_description = Vertex::get_binding_destcription();
        let attribute_description = Vertex::get_attribute_descripyions();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            s_type : vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next : std::ptr::null(),
            vertex_binding_description_count : 1,
            p_vertex_binding_descriptions : &binding_description as *const VertexInputBindingDescription,
            vertex_attribute_description_count : attribute_description.as_ref().len() as u32,
            p_vertex_attribute_descriptions : &attribute_description as *const vk::VertexInputAttributeDescription,
            flags : vk::PipelineVertexInputStateCreateFlags::empty()
        };

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo{
            s_type : vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next : std::ptr::null(),
            topology : vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable : 0,
            flags : vk::PipelineInputAssemblyStateCreateFlags::empty()
        };

        let viewport = vk::Viewport{
            x : 0.0,
            y : 0.0,
            width : self.swap_chain_extent.as_ref().unwrap().width as f32,
            height : self.swap_chain_extent.as_ref().unwrap().height as f32,
            min_depth : 0.0,
            max_depth : 1.0
        };

        let scissor = vk::Rect2D {
            offset : vk::Offset2D { x: 0, y: 0 },
            extent : *self.swap_chain_extent.as_ref().unwrap()
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            s_type : vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next : std::ptr::null(),
            viewport_count : 1,
            p_viewports : &viewport as *const vk::Viewport,
            scissor_count : 1,
            p_scissors : &scissor as *const vk::Rect2D,
            flags : vk::PipelineViewportStateCreateFlags::empty()
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            s_type : vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next : std::ptr::null(),
            depth_clamp_enable : 0,
            rasterizer_discard_enable : 0,
            polygon_mode : vk::PolygonMode::FILL,
            line_width : 1.0,
            cull_mode : vk::CullModeFlags::BACK,
            front_face : vk::FrontFace::CLOCKWISE,
            depth_bias_enable : 0,
            depth_bias_constant_factor : 0.0,
            depth_bias_clamp : 0.0,
            depth_bias_slope_factor : 0.0,
            flags : vk::PipelineRasterizationStateCreateFlags::empty()
        };

        let multisampling = vk::PipelineMultisampleStateCreateInfo {
            s_type : vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            p_next : std::ptr::null(),
            sample_shading_enable : 0,
            rasterization_samples : vk::SampleCountFlags::TYPE_1,
            min_sample_shading : 1.0,
            p_sample_mask : std::ptr::null(),
            alpha_to_coverage_enable : 0,
            alpha_to_one_enable : 0,
            flags : vk::PipelineMultisampleStateCreateFlags::empty()
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            color_write_mask : vk::ColorComponentFlags::all(),
            blend_enable : 0,
            src_color_blend_factor : vk::BlendFactor::ONE,
            dst_color_blend_factor : vk::BlendFactor::ZERO,
            color_blend_op : vk::BlendOp::ADD,
            src_alpha_blend_factor : vk::BlendFactor::ONE,
            dst_alpha_blend_factor : vk::BlendFactor::ZERO,
            alpha_blend_op : vk::BlendOp::ADD
        };

        let blend_constants = [0.0, 0.0, 0.0, 0.0];

        let color_blending = vk::PipelineColorBlendStateCreateInfo {
            s_type : vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next : std::ptr::null(),
            logic_op_enable : 0,
            logic_op : vk::LogicOp::COPY,
            attachment_count : 1,
            p_attachments : &color_blend_attachment as *const PipelineColorBlendAttachmentState,
            blend_constants : blend_constants,
            flags : vk::PipelineColorBlendStateCreateFlags::empty()
        };

        let dynamic_states = [
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::LINE_WIDTH
        ];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            s_type : vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            p_next : std::ptr::null(),
            dynamic_state_count : 2,
            p_dynamic_states : dynamic_states.as_ptr(),
            flags : vk::PipelineDynamicStateCreateFlags::empty()
        };

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo {
            s_type : vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next : std::ptr::null(),
            set_layout_count : 1,
            p_set_layouts : self.descriptor_set_layout.as_ref().unwrap() as *const vk::DescriptorSetLayout,
            push_constant_range_count : 0,
            p_push_constant_ranges : std::ptr::null(),
            flags : vk::PipelineLayoutCreateFlags::empty()
        };

        self.pipeline_layout = Some(unsafe{
            self.device.as_ref().unwrap()
            .create_pipeline_layout(&pipeline_layout_info, None)
            .expect("failed to create pipeline layout")
        });

        let pipeline_info = [vk::GraphicsPipelineCreateInfo {
            s_type : vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next : std::ptr::null(),
            stage_count : 2,
            p_stages : &shader_stages as *const PipelineShaderStageCreateInfo,
            p_vertex_input_state : &vertex_input_info as *const PipelineVertexInputStateCreateInfo,
            p_input_assembly_state : &input_assembly as *const PipelineInputAssemblyStateCreateInfo,
            p_viewport_state : &viewport_state as *const PipelineViewportStateCreateInfo,
            p_rasterization_state : &rasterizer as *const PipelineRasterizationStateCreateInfo,
            p_multisample_state : &multisampling as *const PipelineMultisampleStateCreateInfo,
            p_depth_stencil_state : std::ptr::null(),
            p_color_blend_state : &color_blending as *const PipelineColorBlendStateCreateInfo,
            p_dynamic_state : std::ptr::null(),
            layout : *self.pipeline_layout.as_ref().unwrap(),
            render_pass : *self.render_pass.as_ref().unwrap(),
            subpass : 0,
            base_pipeline_handle : vk::Pipeline::null(),
            base_pipeline_index : -1,
            flags : vk::PipelineCreateFlags::empty(),
            p_tessellation_state : std::ptr::null()
        }];

        self.graphics_pipeline = Some(unsafe {
            self.device.as_ref().unwrap()
            .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_info, None)
            .expect("failed to create graphics pipeline!")[0]
        });
    }

    fn create_framebuffers(&mut self){
        self.swap_chain_frame_buffers
        .resize(
            self.swap_chain_image_views.len(), 
            vk::Framebuffer::null()
        );

        for (idx, image_view) in self.swap_chain_image_views.iter().enumerate(){
            let framebuffer_info = vk::FramebufferCreateInfo{
                s_type : vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next : std::ptr::null(),
                render_pass : *self.render_pass.as_ref().unwrap(),
                attachment_count : 1,
                p_attachments : image_view as *const vk::ImageView,
                width : self.swap_chain_extent.as_ref().unwrap().width,
                height : self.swap_chain_extent.as_ref().unwrap().height,
                layers : 1,
                flags : vk::FramebufferCreateFlags::empty()
            };

            self.swap_chain_frame_buffers[idx] = unsafe {
                self.device.as_ref().unwrap()
                .create_framebuffer(&framebuffer_info, None)
                .expect("failed to create framebuffer!")
            };
        }
    }

    fn create_command_pool(&mut self){
        let queue_family_indices = self.find_queue_families(self.physical_device.as_ref().unwrap());

        let pool_info = vk::CommandPoolCreateInfo {
            s_type : vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next : std::ptr::null(),
            queue_family_index : queue_family_indices.0.unwrap() as u32,
            flags : vk::CommandPoolCreateFlags::empty()
        };

        self.command_pool = Some(unsafe {
            self.device.as_ref().unwrap()
            .create_command_pool(&pool_info, None)
            .expect("failed to create command pool!")
        });
    }

    fn create_command_buffers(&mut self){
        let alloc_info = vk::CommandBufferAllocateInfo {
            s_type : vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next : std::ptr::null(),
            command_pool : *self.command_pool.as_ref().unwrap(),
            level : vk::CommandBufferLevel::PRIMARY,
            command_buffer_count : self.swap_chain_frame_buffers.len() as u32,
        };
        
        self.command_buffers = Some(unsafe {
            self.device.as_ref().unwrap()
            .allocate_command_buffers(&alloc_info)
            .expect("failed to allocate command buffers!")
        });

        let device_ref = self.device.as_ref().unwrap();

        for (idx, command_buffer) in self.command_buffers.as_ref().unwrap().iter().enumerate() {
            let begin_info = vk::CommandBufferBeginInfo {
                s_type : vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                p_next : std::ptr::null(),
                flags : CommandBufferUsageFlags::empty(),
                p_inheritance_info : std::ptr::null()
            };

            unsafe{
                device_ref.begin_command_buffer(*command_buffer, &begin_info)
                .expect("failed to begin recording command buffer");
            }

            let render_area = vk::Rect2D {
                offset : vk::Offset2D { x: 0, y: 0 },
                extent : *self.swap_chain_extent.as_ref().unwrap()
            };

            let clear_color = vk::ClearValue {
                color : ClearColorValue{ float32: [0.0f32, 0.0f32, 0.0f32, 1.0f32] },
            };
            
            let render_pass_info = vk::RenderPassBeginInfo {
                s_type : vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next : std::ptr::null(),
                render_pass : *self.render_pass.as_ref().unwrap(),
                framebuffer : self.swap_chain_frame_buffers[idx],
                render_area : render_area,
                clear_value_count : 1,
                p_clear_values : &clear_color as *const vk::ClearValue
            };

            unsafe{
                device_ref.cmd_begin_render_pass(*command_buffer, &render_pass_info, vk::SubpassContents::INLINE);
                device_ref.cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, *self.graphics_pipeline.as_ref().unwrap());

                let vertex_buffers = [*self.vertex_buffer.as_ref().unwrap()];
                let offsets = [0];
                device_ref.cmd_bind_vertex_buffers(*command_buffer, 0, &vertex_buffers, &offsets);

                device_ref.cmd_bind_index_buffer(*command_buffer, *self.index_buffer.as_ref().unwrap(), 0, vk::IndexType::UINT16);

                device_ref.cmd_draw_indexed(*command_buffer, self.indices.len() as u32, 1, 0, 0, 0);
                device_ref.cmd_end_render_pass(*command_buffer);
                device_ref.end_command_buffer(*command_buffer).expect("failed to record command buffer");
            }
        }
    }

    fn create_sync_objects(&mut self){
        self.image_available_semaphores.resize(2, vk::Semaphore::null());
        self.render_finished_semaphores.resize(2, vk::Semaphore::null());
        self.in_flight_fences.resize(2, vk::Fence::null());
        self.images_in_flight.resize(self.swap_chain_images.as_ref().unwrap().len(), vk::Fence::null());

        let semaphore_info = vk::SemaphoreCreateInfo {
            s_type : vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : vk::SemaphoreCreateFlags::empty()
        };

        let fence_info = vk::FenceCreateInfo {
            s_type : vk::StructureType::FENCE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : vk::FenceCreateFlags::SIGNALED
        };

        let device_ref = self.device.as_ref().unwrap();

        for idx in 0..2{
            self.image_available_semaphores[idx] = unsafe {
                device_ref.create_semaphore(&semaphore_info, None)
                .expect("failed to create synchronization objects for a frame!")
            };

            self.render_finished_semaphores[idx] = unsafe {
                device_ref.create_semaphore(&semaphore_info, None)
                .expect("failed to create synchronization objects for a frame!")
            };

            self.in_flight_fences[idx] = unsafe {
                device_ref.create_fence(&fence_info, None)
                .expect("failed to create synchronization objects for a frame!")
            }
        }
    }

    fn update_uniform_buffer(&mut self, current_image : u32) {
        if let None = self.start_time {
            self.start_time = Some(std::time::SystemTime::now());
        }

        let current_time = std::time::SystemTime::now();
        let time = 
            current_time.duration_since(*self.start_time.as_ref().unwrap())
            .unwrap().as_secs_f32();
    }

    fn draw_frame(&mut self){
        let fences = [*&self.in_flight_fences[self.current_frame]];
        unsafe{
            self.device.as_ref().unwrap().wait_for_fences(
                &fences, 
                true, u64::MAX
            );

            self.device.as_ref().unwrap().reset_fences(&fences);
        }
        let swapchain = ash::extensions::khr::Swapchain::new(
            self.instance.as_ref().unwrap(),
            self.device.as_ref().unwrap()
        );
        
        let draw_result = unsafe {
            swapchain.acquire_next_image(
                *self.swap_chain.as_ref().unwrap(), 
                u64::MAX, 
                *&self.image_available_semaphores[self.current_frame], 
                vk::Fence::null()
            )
            .expect("failed to load next image")
        };

        let image_index = draw_result.0;

        if self.images_in_flight[image_index as usize] != vk::Fence::null() {
            let image_fences = [*&self.images_in_flight[image_index as usize]];
            unsafe{
                self.device.as_ref().unwrap()
                .wait_for_fences(&image_fences, true, u64::MAX)
                .expect("fence not work");
            }
        }

        self.images_in_flight[image_index as usize] = self.in_flight_fences[self.current_frame];

        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        self.update_uniform_buffer(image_index);

        let submit_info = [vk::SubmitInfo {
            s_type : vk::StructureType::SUBMIT_INFO,
            p_next : std::ptr::null(),
            wait_semaphore_count : 1,
            p_wait_semaphores : &self.image_available_semaphores[self.current_frame] as *const vk::Semaphore,
            p_wait_dst_stage_mask : wait_stages.as_ptr(),
            command_buffer_count : 1,
            p_command_buffers : &self.command_buffers.as_ref().unwrap()[image_index as usize] as *const vk::CommandBuffer,
            signal_semaphore_count : 1,
            p_signal_semaphores : &self.render_finished_semaphores[self.current_frame] as *const vk::Semaphore
        }];

        unsafe{
            self.device.as_ref().unwrap()
            .reset_fences(&fences);
            
            self.device.as_ref().unwrap()
            .queue_submit(*self.graphics_queue.as_ref().unwrap(), &submit_info, self.in_flight_fences[self.current_frame])
            .expect("failed to submit draw command buffer");
        }

        let mut result = vk::Result::NOT_READY;

        let present_info = vk::PresentInfoKHR {
            s_type : vk::StructureType::PRESENT_INFO_KHR,
            p_next : std::ptr::null(),
            wait_semaphore_count : 1,
            p_wait_semaphores : &self.render_finished_semaphores[self.current_frame] as *const vk::Semaphore,
            swapchain_count : 1,
            p_swapchains : self.swap_chain.as_ref().unwrap() as *const vk::SwapchainKHR,
            p_image_indices : &image_index as *const u32,
            p_results : &mut result as *mut vk::Result
        };

        unsafe{
            swapchain
            .queue_present(*self.present_queue.as_ref().unwrap(), &present_info)
            .expect("failed to present");
        }

        self.current_frame = (self.current_frame + 1) % 2;

    }

    fn main_loop(&mut self){
        while !self.window.as_ref().unwrap().should_close(){
            self.glfw.poll_events();
            self.draw_frame();
        }

        unsafe{
            self.device.as_ref().unwrap()
            .device_wait_idle()
            .expect("");
        }
    }
}

impl Drop for HelloTriangleApplication {
    fn drop(&mut self) {
        self.clean_swap_chain();

        unsafe{
            let device_ref = self.device.as_ref().unwrap();

            device_ref.destroy_descriptor_set_layout(*self.descriptor_set_layout.as_ref().unwrap(), None);
            self.descriptor_set_layout = None;

            device_ref.destroy_buffer(*self.vertex_buffer.as_ref().unwrap(), None);
            self.vertex_buffer = None;
            device_ref.free_memory(*self.vertex_buffer_memory.as_ref().unwrap(), None);
            self.vertex_buffer_memory = None;

            device_ref.destroy_buffer(*self.index_buffer.as_ref().unwrap(), None);
            self.index_buffer = None;
            device_ref.free_memory(*self.index_buffer_memory.as_ref().unwrap(), None);
            self.index_buffer_memory = None;

            for semaphore in self.render_finished_semaphores.drain(..){
                device_ref.destroy_semaphore(semaphore, None);
            }

            for semaphore in self.image_available_semaphores.drain(..){
                device_ref.destroy_semaphore(semaphore, None);
            }

            for fence in self.in_flight_fences.drain(..) {
                device_ref.destroy_fence(fence, None);
            }

            device_ref.destroy_command_pool(*self.command_pool.as_ref().unwrap(), None);
            self.command_pool = None;

            device_ref.destroy_device(None);

            let surface = ash::extensions::khr::Surface::new(&self.vk_entry, self.instance.as_ref().unwrap());
            surface.destroy_surface(*self.surface.as_ref().unwrap(), None);
            self.surface = None;

            self.instance.as_ref().unwrap().destroy_instance(None);
            self.instance = None;
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
