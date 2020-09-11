use ash::vk::*;
use ash::version::EntryV1_0;
use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::Entry;
use ash::Instance;
use ash::Device;
use ash::extensions::ext::DebugUtils;

use winit::window::Window;

use std::ffi::CString;

pub struct Implementation{
    pub instance : VKInstance,
    pub surface : VKSurface,
    pub device : VKDevice,
    pub swapchain : VKSwapchain,
    pub render_pass : VKRenderPass,
    pub framebuffers : VKFramebuffers,
    pub pipeline : VKPipeline,
    pub vertex_buffer : VertexBuffer,
    pub command_pool : VKCommandPool,
    pub synchroniser : VKSynchroniser,
    pub descriptor : VKDescriptorPool,
}
impl Implementation{
    pub fn new(validation : bool , window : &Window) -> Self{
        println!("Creating Vulkan Implementation.");
        println!("Creating Vulkan Instance.");
        let instance = VKInstance::new(validation);
        println!("Creating Vulkan Surface");
        let surface = VKSurface::new(&instance.entry , &instance.instance , &window);
        println!("Creating Vulkan Device.");
        let device = VKDevice::new(&instance.instance, &surface.surface_loader, &surface.surface);
        println!("Creating Vulkan Swapchain.");
        let swapchain = VKSwapchain::new(&instance.instance, &device.device, &surface.surface_loader, &surface.surface, device.physical_device, device.graphics_queue, device.presentation_queue);
        println!("Creating Vulkan Render Pass.");
        let render_pass = VKRenderPass::new(&device.device, swapchain.format.format);
        println!("Creating Vulkan Framebuffers.");
        let framebuffers = VKFramebuffers::new(&swapchain.swapchain_image_views, &device.device, &swapchain.extent, &render_pass.render_pass);
        println!("Creating Vulkan Descriptor Set.");
        let descriptor = VKDescriptorPool::new(&device.device , &instance.instance , &device.physical_device , swapchain.swapchain_images.len() as u32);
        println!("Creating Vulkan Pipeline.");
        let pipeline = VKPipeline::new(&device.device, &render_pass.render_pass, &descriptor.set_layout);
        println!("Creating Vulkan Command Pool.");
        let command_pool = VKCommandPool::new(&device.device,device.graphics_queue,framebuffers.framebuffers.len() as u32);
        println!("Creating Vulkan Vertex Buffer.");
        let vertices = vec!(
            Vertex{color:[1.0,1.0,1.0],pos:[-0.5,-0.5]},
            Vertex{color:[1.0,0.0,0.0],pos:[0.5,-0.5]},
            Vertex{color:[0.0,1.0,0.0],pos:[0.5,0.5]},
            Vertex{color:[0.0,1.0,0.0],pos:[-0.5,0.5]},
        );
        let indices = vec!(0,1,2,2,3,0);
        //let t_command_pool = VKCommandPool::new(&device.device, device.transfer_queue, 0);
        let vertex_buffer = VertexBuffer::new(&instance.instance, &device.device, &device.physical_device, command_pool.command_pool , device.graphics_queue_vk, vertices, indices);
        command_pool.record_command_buffers(&pipeline.pipeline, &device.device, &render_pass.render_pass, &framebuffers.framebuffers, &swapchain.extent, vertex_buffer.buffer , vertex_buffer.index_buffer , vertex_buffer.indices_count , &descriptor.descriptor_sets , &pipeline.pipeline_layout);
        println!("Creating Vulkan Synchroniser.");
        let synchroniser = VKSynchroniser::new(2, &device.device);
        return Implementation{
            instance : instance,
            surface : surface,
            device : device,
            swapchain : swapchain,
            render_pass : render_pass,
            framebuffers : framebuffers,
            pipeline : pipeline,
            vertex_buffer : vertex_buffer,
            command_pool : command_pool,
            synchroniser : synchroniser,
            descriptor : descriptor,
        };
    }
    pub fn draw_frame(&mut self , model_matrix : &cgmath::Matrix4<f32> , view_matrix : &cgmath::Matrix4<f32> , proj_matrix : &cgmath::Matrix4<f32>){
        let wait_fences = [self.synchroniser.in_flight_fences[self.synchroniser.current_frame as usize]];
        unsafe{self.device.device.wait_for_fences(&wait_fences,true,std::u64::MAX)}.expect("Failed to wait for fence.");
        let image_index = unsafe{
            let result = self.swapchain.swapchain_loader.acquire_next_image(self.swapchain.swapchain, std::u64::MAX, self.synchroniser.image_available_semaphores[self.synchroniser.current_frame as usize], Fence::null());
            match result{
                Ok(image_index) => {image_index.0}
                Err(result) => match result {
                    Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain();
                        return;
                    }
                    _ => {panic!("Failed to acquire next image.")}
                }
            }
        };
        let wait_semaphores = [self.synchroniser.image_available_semaphores[self.synchroniser.current_frame as usize]];
        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.synchroniser.render_finished_semaphores[self.synchroniser.current_frame as usize]];
        self.descriptor.update_uniform_buffer(image_index,model_matrix,view_matrix,proj_matrix,&self.device.device);
        let submit_infos = [SubmitInfo {
            s_type: StructureType::SUBMIT_INFO,
            p_next: std::ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_pool.command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];
        unsafe{self.device.device.reset_fences(&wait_fences)}.expect("Failed to reset fences.");
        unsafe{self.device.device.queue_submit(self.device.graphics_queue_vk, &submit_infos, self.synchroniser.in_flight_fences[self.synchroniser.current_frame as usize])}.expect("Failed to submit queue.");
        let swapchains = [self.swapchain.swapchain];
        let present_info = PresentInfoKHR{
            s_type: StructureType::PRESENT_INFO_KHR,
            p_next: std::ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: std::ptr::null_mut(),
        };
        let result = unsafe{self.swapchain.swapchain_loader.queue_present(self.device.presentation_queue_vk, &present_info)};
        let is_resized = match result{
            Ok(_) => false,
            Err(result) => match result{
                Result::ERROR_OUT_OF_DATE_KHR | Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to present frame.")
            }
        };
        if is_resized{
            self.recreate_swapchain();
        }
        self.synchroniser.current_frame = (self.synchroniser.current_frame+1) % self.synchroniser.max_frames_in_flight;
    }
    pub fn flush_and_refill_vertex_buffer(&mut self , vertices : Vec<Vertex> , indices : Vec<u32>){
        println!("Flushed and refilled vertex buffer, this may indicate a game transition.");
        let indices_count = indices.len() as u32;
        self.vertex_buffer.update_buffer(&self.device, &self.instance, vertices, indices, &self.command_pool.command_pool, &self.device.graphics_queue_vk);
        //unsafe{self.device.device.free_command_buffers(self.command_pool.command_pool, &self.command_pool.command_buffers)};
        if indices_count != self.vertex_buffer.indices_count{
            for &command_buffer in self.command_pool.command_buffers.iter(){
                unsafe{self.device.device.reset_command_buffer(command_buffer, CommandBufferResetFlags::empty())}.expect("Failed to flush command buffer.");
            }
            self.command_pool.record_command_buffers(&self.pipeline.pipeline, &self.device.device, &self.render_pass.render_pass, &self.framebuffers.framebuffers, &self.swapchain.extent, self.vertex_buffer.buffer, self.vertex_buffer.index_buffer, self.vertex_buffer.indices_count,&self.descriptor.descriptor_sets,&self.pipeline.pipeline_layout);
        }
    }
    fn recreate_swapchain(&mut self){
        unsafe{self.device.device.device_wait_idle()}.expect("Failed to recreate swapchain.");
        self.cleanup_swapchain();
        self.swapchain = VKSwapchain::new(&self.instance.instance, &self.device.device, &self.surface.surface_loader, &self.surface.surface, self.device.physical_device, self.device.graphics_queue, self.device.presentation_queue);
        self.render_pass = VKRenderPass::new(&self.device.device, self.swapchain.format.format);
        self.framebuffers = VKFramebuffers::new(&self.swapchain.swapchain_image_views, &self.device.device, &self.swapchain.extent, &self.render_pass.render_pass);
        let command_buffers_create_info = CommandBufferAllocateInfo{
            s_type : StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next : std::ptr::null(),
            level : CommandBufferLevel::PRIMARY,
            command_pool : self.command_pool.command_pool,
            command_buffer_count : self.framebuffers.framebuffers.len() as u32,
        };
        self.command_pool.command_buffers = unsafe{self.device.device.allocate_command_buffers(&command_buffers_create_info)}.expect("Failed to reallocate command buffers.");
        self.command_pool.record_command_buffers(&self.pipeline.pipeline, &self.device.device, &self.render_pass.render_pass, &self.framebuffers.framebuffers, &self.swapchain.extent, self.vertex_buffer.buffer , self.vertex_buffer.index_buffer, self.vertex_buffer.indices_count, &self.descriptor.descriptor_sets,&self.pipeline.pipeline_layout);
    }
    fn cleanup_swapchain(&mut self){
        unsafe{self.device.device.free_command_buffers(self.command_pool.command_pool, &self.command_pool.command_buffers)};
        for &framebuffer in self.framebuffers.framebuffers.iter(){
            unsafe{self.device.device.destroy_framebuffer(framebuffer, None)};
        }
        unsafe{self.device.device.destroy_render_pass(self.render_pass.render_pass, None)};
        for &image_view in self.swapchain.swapchain_image_views.iter(){
            unsafe{self.device.device.destroy_image_view(image_view, None)};
        }
       unsafe{self.swapchain.swapchain_loader.destroy_swapchain(self.swapchain.swapchain, None)};
    }
}
impl Drop for Implementation{
    fn drop(&mut self){
        unsafe{self.device.device.device_wait_idle()}.expect("Failed to recreate swapchain.");
        self.cleanup_swapchain();
        unsafe{self.device.device.destroy_descriptor_pool(self.descriptor.descriptor_pool, None)};
        for &buffer in self.descriptor.buffers.iter(){
            unsafe{self.device.device.destroy_buffer(buffer, None)};
        }
        for &memory in self.descriptor.buffers_memory.iter(){
            unsafe{self.device.device.free_memory(memory, None)};
        }
        unsafe{self.device.device.destroy_descriptor_set_layout(self.descriptor.set_layout, None)};
        unsafe{self.device.device.destroy_pipeline(self.pipeline.pipeline, None)};
        unsafe{self.device.device.destroy_pipeline_layout(self.pipeline.pipeline_layout, None)};
        unsafe{self.device.device.destroy_command_pool(self.command_pool.command_pool, None)};
        unsafe{self.surface.surface_loader.destroy_surface(self.surface.surface, None)};
        unsafe{self.device.device.destroy_buffer(self.vertex_buffer.buffer, None)};
        unsafe{self.device.device.destroy_buffer(self.vertex_buffer.index_buffer, None)};
        unsafe{self.device.device.free_memory(self.vertex_buffer.memory, None)};
        unsafe{self.device.device.free_memory(self.vertex_buffer.index_memory, None)};   
        for &semaphore in self.synchroniser.render_finished_semaphores.iter(){
            unsafe{self.device.device.destroy_semaphore(semaphore, None)};
        }
        for &semaphore in self.synchroniser.image_available_semaphores.iter(){
            unsafe{self.device.device.destroy_semaphore(semaphore, None)};
        }
        for &fence in self.synchroniser.in_flight_fences.iter(){
            unsafe{self.device.device.destroy_fence(fence, None)};
        }
        unsafe{self.device.device.destroy_device(None)}; 
        unsafe{self.instance.instance.destroy_instance(None)};
    }
}
pub struct VKInstance{
    pub entry : Entry,
    pub instance : Instance,
}
impl VKInstance{
    fn new(validation_enabled : bool) -> Self{
        let entry = Entry::new().expect("Failed to load vulkan.");
        let validation_layer_name = [CString::new("VK_LAYER_KHRONOS_validation").expect("Failed to create CString.")];
        let validation_layer_name_raw : Vec<*const i8> = validation_layer_name.iter().map(|name| name.as_ptr()).collect();
        let mut extensions : Vec<*const i8> = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            #[cfg(all(windows))]
            ash::extensions::khr::Win32Surface::name().as_ptr(),
            #[cfg(target_os = "macos")]
            ash::extensions::mvk::MacOSSurface::name().as_ptr(), 
            #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
            ash::extensions::khr::XlibSurface::name().as_ptr(),
        ];
        if validation_enabled {extensions.push(DebugUtils::name().as_ptr())}
        let application_info = ApplicationInfo{
            s_type : StructureType::APPLICATION_INFO,
            p_next : std::ptr::null(),
            api_version : make_version(1, 2, 0),
            application_version : make_version(0, 0, 1),
            engine_version : make_version(0, 0, 1),
            p_application_name : CString::new("VKAPP").unwrap().as_ptr(),
            p_engine_name : CString::new("VKAPP").unwrap().as_ptr(),
        };
        let create_info = InstanceCreateInfo{
            s_type : StructureType::INSTANCE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : InstanceCreateFlags::empty(),
            enabled_layer_count : validation_layer_name.len() as u32,
            enabled_extension_count : extensions.len() as u32,
            pp_enabled_extension_names : extensions.as_ptr(),
            pp_enabled_layer_names : validation_layer_name_raw.as_ptr(),
            p_application_info : &application_info
        };
        let instance = unsafe{entry.create_instance(&create_info,None)}.expect("Failed to create vulkan instance.");
        return VKInstance{
            entry : entry,
            instance : instance,
        };
    }
}
pub struct VKSurface{
    surface_loader : ash::extensions::khr::Surface,
    surface : SurfaceKHR,
}
impl VKSurface{
    pub fn new (entry : &Entry , instance : &Instance , window : &Window) -> Self{
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
        let surface = VKSurface::create_surface(entry, instance, window);
        return VKSurface{
            surface_loader : surface_loader,
            surface : surface
        };
    }
    #[cfg(all(windows))]
    pub fn create_surface(entry : &Entry , instance : &Instance , window : &Window) -> SurfaceKHR{
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winit::platform::windows::WindowExtWindows;
        let hinstance = unsafe{GetModuleHandleW(std::ptr::null())} as *const std::ffi::c_void;
        let hwnd = window.hwnd() as HWND;
        let create_info = ash::vk::Win32SurfaceCreateInfoKHR{
            s_type: ash::vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            p_next: std::ptr::null(),
            flags: Default::default(),
            hinstance,
            hwnd: hwnd as *const std::ffi::c_void,
        };
        unsafe{ash::extensions::khr::Win32Surface::new(entry, instance).create_win32_surface(&create_info, None)}.expect("Failed to create Win32 surface.")
    }
}
pub struct VKDevice{
    pub physical_device : PhysicalDevice,
    pub graphics_queue : u32,
    pub graphics_queue_vk : Queue,
    pub presentation_queue : u32,
    pub presentation_queue_vk : Queue,
    pub transfer_queue : u32,
    pub transfer_queue_vk : Queue,
    pub device : Device,
}
impl VKDevice{
    pub fn new(instance : &Instance , surface_loader : &ash::extensions::khr::Surface , surface : &SurfaceKHR) -> Self{
        let physical_devices = unsafe{instance.enumerate_physical_devices()}.expect("Failed to enumerate physical devices.");
        let physical_device = VKDevice::choose_physical_device(physical_devices , instance , surface_loader , surface);
        let mut graphics_queue = None;
        let mut present_queue = None;
        let mut transfer_queue = None;
        let queue_families = unsafe{instance.get_physical_device_queue_family_properties(physical_device)};
        for (index,queue_family) in queue_families.iter().enumerate(){
            if graphics_queue.is_none() && queue_family.queue_flags.contains(QueueFlags::GRAPHICS){graphics_queue=Some(index as u32)}
            if transfer_queue.is_none() && queue_family.queue_flags.contains(QueueFlags::TRANSFER){transfer_queue=Some(index as u32)}
            if present_queue.is_none() && unsafe{surface_loader.get_physical_device_surface_support(physical_device, index as u32, *surface)}.expect("Failed to query presentation support."){present_queue=Some(index as u32)}
            if graphics_queue.is_some() && queue_family.queue_flags.contains(QueueFlags::GRAPHICS) && !queue_family.queue_flags.contains(QueueFlags::TRANSFER){graphics_queue = Some(index as u32)}
            if transfer_queue.is_some() && queue_family.queue_flags.contains(QueueFlags::TRANSFER) && !queue_family.queue_flags.contains(QueueFlags::GRAPHICS){transfer_queue=Some(index as u32)}
        }
        let graphics_queue = graphics_queue.unwrap();
        let transfer_queue = transfer_queue.unwrap();
        let present_queue = present_queue.unwrap();
        let priorities = [1.0f32];
        let mut queues = Vec::new();
        queues.push(graphics_queue);
        if !queues.contains(&transfer_queue){queues.push(transfer_queue)}
        if !queues.contains(&present_queue){queues.push(present_queue)}
        let mut queue_infos = Vec::new();
        for queue in queues{
            queue_infos.push(DeviceQueueCreateInfo{
                s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next : std::ptr::null(),
                flags : DeviceQueueCreateFlags::empty(),
                queue_count : 1,
                queue_family_index : queue,
                p_queue_priorities : priorities.as_ptr(),
            })
        }
        let extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];
        let features = PhysicalDeviceFeatures{
            ..Default::default()
        };
        let device_create_info = DeviceCreateInfo{
            s_type : StructureType::DEVICE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : DeviceCreateFlags::empty(),
            enabled_extension_count : extensions.len() as u32,
            pp_enabled_extension_names : extensions.as_ptr(),
            enabled_layer_count : 0,
            pp_enabled_layer_names : std::ptr::null(),
            queue_create_info_count : queue_infos.len() as u32,
            p_queue_create_infos : queue_infos.as_ptr(),
            p_enabled_features : &features,
        };
        let device = unsafe{instance.create_device(physical_device, &device_create_info, None)}.expect("Failed to create device.");
        let g_queue = unsafe{device.get_device_queue(graphics_queue,0)};
        let p_queue = unsafe{device.get_device_queue(present_queue,0)};
        let t_queue = unsafe{device.get_device_queue(transfer_queue,0)};
        return VKDevice{
            physical_device : physical_device,
            device : device,
            graphics_queue : graphics_queue,
            presentation_queue : present_queue,
            transfer_queue : transfer_queue,
            graphics_queue_vk : g_queue,
            presentation_queue_vk : p_queue,
            transfer_queue_vk : t_queue
        };
    }
    fn choose_physical_device(physical_devices : Vec<PhysicalDevice> , instance : &Instance , surface_loader : &ash::extensions::khr::Surface , surface : &SurfaceKHR) -> PhysicalDevice{
        let mut prefered_device = None;
        for physical_device in physical_devices{
            let device_type = unsafe{instance.get_physical_device_properties(physical_device)}.device_type;
            for (queue_index,properties) in unsafe{instance.get_physical_device_queue_family_properties(physical_device)}.iter().enumerate(){
                if prefered_device.is_none() && unsafe{surface_loader.get_physical_device_surface_support(physical_device, queue_index as u32, *surface)}.expect("Failed to query surface support.") && properties.queue_flags.contains(QueueFlags::GRAPHICS){
                    prefered_device = Some(physical_device);
                }
                else if unsafe{surface_loader.get_physical_device_surface_support(physical_device, queue_index as u32, *surface)}.expect("Failed to query surface support.") && properties.queue_flags.contains(QueueFlags::GRAPHICS) && device_type == PhysicalDeviceType::DISCRETE_GPU{
                    prefered_device = Some(physical_device);
                }
            }
        };
        return prefered_device.expect("Failed to find suitable device.")
    }
}
pub struct VKSwapchain{
    pub extent : Extent2D,
    pub format : SurfaceFormatKHR,
    pub swapchain_loader : ash::extensions::khr::Swapchain,
    pub swapchain : SwapchainKHR,
    pub swapchain_images : Vec<Image>,
    pub swapchain_image_views : Vec<ImageView>,
}
impl VKSwapchain{
    pub fn new(instance : &Instance , device : &Device , surface_loader : &ash::extensions::khr::Surface , surface : &SurfaceKHR , physical_device : PhysicalDevice , graphics_queue : u32 , presentation_queue : u32) -> Self{
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
        let formats = unsafe{surface_loader.get_physical_device_surface_formats(physical_device, *surface)}.expect("Failed to acquire supported formats.");
        let format = VKSwapchain::choose_format(formats);
        let capabilites = unsafe{surface_loader.get_physical_device_surface_capabilities(physical_device, *surface)}.expect("Failed to acquire surface capabilities.");
        let min_image_count = if capabilites.max_image_count >= capabilites.min_image_count+1{capabilites.min_image_count+1}else{capabilites.max_image_count};
        let present_modes = unsafe{surface_loader.get_physical_device_surface_present_modes(physical_device, *surface)}.expect("Failed to acquire surface present modes.");
        let swapchain_create_info = SwapchainCreateInfoKHR{
            s_type : StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next : std::ptr::null(),
            flags : SwapchainCreateFlagsKHR::empty(),
            clipped : 1,
            image_array_layers : 1,
            image_color_space : format.color_space,
            image_format : format.format,
            image_sharing_mode : if graphics_queue == presentation_queue{SharingMode::EXCLUSIVE} else {SharingMode::CONCURRENT},
            image_usage : ImageUsageFlags::COLOR_ATTACHMENT,
            min_image_count : min_image_count,
            queue_family_index_count : if presentation_queue == graphics_queue{0} else {2},
            p_queue_family_indices : if presentation_queue == graphics_queue{std::ptr::null()}else{[graphics_queue,presentation_queue].as_ptr()},
            old_swapchain : SwapchainKHR::null(),
            composite_alpha : CompositeAlphaFlagsKHR::OPAQUE,
            present_mode : if present_modes.contains(&PresentModeKHR::MAILBOX){PresentModeKHR::MAILBOX}else{PresentModeKHR::FIFO},
            surface : *surface,
            image_extent : if capabilites.current_extent.width != std::u32::MAX{capabilites.current_extent}else{capabilites.max_image_extent},
            pre_transform : capabilites.current_transform,
        };
        let swapchain = unsafe{swapchain_loader.create_swapchain(&swapchain_create_info, None)}.expect("Failed to create swapchain.");
        let mut swapchain_image_views = Vec::new();
        let swapchain_images = unsafe{swapchain_loader.get_swapchain_images(swapchain)}.expect("Failed to acquire swapchain images.");
        for &image in swapchain_images.iter(){
            let image_view_create_info = ImageViewCreateInfo{
                s_type : StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next : std::ptr::null(),
                flags : ImageViewCreateFlags::empty(),
                image : image,
                format : format.format,
                view_type : ImageViewType::TYPE_2D,
                components : ComponentMapping{a:ComponentSwizzle::A,b:ComponentSwizzle::B,g:ComponentSwizzle::G,r:ComponentSwizzle::R},
                subresource_range : ImageSubresourceRange{
                    aspect_mask : ImageAspectFlags::COLOR,
                    layer_count : 1,
                    level_count : 1,
                    base_array_layer : 0,
                    base_mip_level : 0,
                },
            };
            let image_view = unsafe{device.create_image_view(&image_view_create_info, None)}.expect("Failed to create swapchain image view.");
            swapchain_image_views.push(image_view);
        }

        return VKSwapchain{extent : if capabilites.current_extent.width != std::u32::MAX{capabilites.current_extent}else{capabilites.max_image_extent},format,swapchain,swapchain_loader,swapchain_images,swapchain_image_views};
    }
    fn choose_format(formats : Vec<SurfaceFormatKHR>) -> SurfaceFormatKHR{
        for &format in formats.iter(){
            if format.format == Format::B8G8R8A8_SRGB && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR{ return format; }
        }
        return formats[1];
    }
}
pub struct VKRenderPass{
    render_pass : RenderPass,
}
impl VKRenderPass{
    pub fn new(device : &Device , format : Format) -> Self{
        let color_attachment = AttachmentDescription{
            flags : AttachmentDescriptionFlags::empty(),
            format : format,
            initial_layout : ImageLayout::UNDEFINED,
            final_layout : ImageLayout::PRESENT_SRC_KHR,
            load_op : AttachmentLoadOp::CLEAR,
            store_op : AttachmentStoreOp::STORE,
            stencil_load_op : AttachmentLoadOp::DONT_CARE,
            stencil_store_op : AttachmentStoreOp::DONT_CARE,
            samples : SampleCountFlags::TYPE_1,
        };
        let attachments = [color_attachment];
        let color_attachment_references = [AttachmentReference{
            attachment : 0,
            layout : ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let subpass = SubpassDescription{
            pipeline_bind_point : PipelineBindPoint::GRAPHICS,
            color_attachment_count : color_attachment_references.len() as u32,
            p_color_attachments : color_attachment_references.as_ptr(),
            p_depth_stencil_attachment : std::ptr::null(),
            input_attachment_count : 0,
            p_input_attachments : std::ptr::null(),
            preserve_attachment_count : 0,
            p_preserve_attachments : std::ptr::null(),
            p_resolve_attachments : std::ptr::null(),
            flags : SubpassDescriptionFlags::empty(),
        };
        let subpass_dependency = SubpassDependency{
            dependency_flags : DependencyFlags::empty(),
            src_subpass : SUBPASS_EXTERNAL,
            dst_subpass : 0,
            src_stage_mask : PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask : PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask : AccessFlags::empty(),
            dst_access_mask : AccessFlags::COLOR_ATTACHMENT_WRITE,
        };
        let dependencies = [subpass_dependency];
        let render_pass_create_info = RenderPassCreateInfo{
            s_type : StructureType::RENDER_PASS_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : RenderPassCreateFlags::empty(),
            subpass_count : 1,
            p_subpasses : &subpass,
            attachment_count : attachments.len() as u32,
            p_attachments : attachments.as_ptr(),
            dependency_count : dependencies.len() as u32,
            p_dependencies : dependencies.as_ptr(),
        };
        let render_pass = unsafe{device.create_render_pass(&render_pass_create_info, None)}.expect("Failed to create render pass.");
        return VKRenderPass{render_pass};
    }
}
pub struct VKFramebuffers{
    pub framebuffers : Vec<Framebuffer>
}
impl VKFramebuffers{
    pub fn new(swapchain_image_views : &Vec<ImageView> , device : &Device , extent : &Extent2D , render_pass : &RenderPass) -> Self{
        let mut framebuffers = Vec::new();
        for &image_view in swapchain_image_views.iter(){
            let attachments = [image_view];
            let framebuffer_create_info = FramebufferCreateInfo{
                s_type : StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next : std::ptr::null(),
                flags : FramebufferCreateFlags::empty(),
                width : extent.width,
                height : extent.height,
                attachment_count : attachments.len() as u32,
                p_attachments : attachments.as_ptr(),
                render_pass : *render_pass,
                layers : 1,
            };
            let framebuffer = unsafe{device.create_framebuffer(&framebuffer_create_info, None)}.expect("Failed to create framebuffer.");
            framebuffers.push(framebuffer);
        }
        return VKFramebuffers{framebuffers};
    }
}
pub struct VKPipeline{
    pub pipeline_layout : PipelineLayout,
    pub pipeline : Pipeline,
}
impl VKPipeline{
    pub fn new(device : &Device , render_pass : &RenderPass , set_layout : &DescriptorSetLayout) -> Self{
        let set_layouts = [*set_layout];
        let pipeline_layout_create_info = PipelineLayoutCreateInfo{
            s_type : StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineLayoutCreateFlags::empty(),
            set_layout_count : set_layouts.len() as u32,
            p_set_layouts : set_layouts.as_ptr(),
            push_constant_range_count : 0,
            p_push_constant_ranges : std::ptr::null(),
        };
        let pipeline_layout = unsafe{device.create_pipeline_layout(&pipeline_layout_create_info, None)}.expect("Failed to create pipeline layout.");
        let color_blend_attachments = [PipelineColorBlendAttachmentState{
            color_write_mask : ColorComponentFlags::R | ColorComponentFlags::G | ColorComponentFlags::B | ColorComponentFlags::A,
            blend_enable : 0,
            alpha_blend_op : BlendOp::ADD,
            color_blend_op : BlendOp::ADD,
            src_color_blend_factor : BlendFactor::ONE,
            dst_color_blend_factor : BlendFactor::ZERO,
            src_alpha_blend_factor : BlendFactor::ONE,
            dst_alpha_blend_factor : BlendFactor::ZERO,
        }];
        let color_blend_state = PipelineColorBlendStateCreateInfo{
            s_type : StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineColorBlendStateCreateFlags::empty(),
            blend_constants : [0.0,0.0,0.0,0.0],
            logic_op_enable : 0,
            logic_op : LogicOp::COPY,
            attachment_count : color_blend_attachments.len() as u32,
            p_attachments : color_blend_attachments.as_ptr(),
        };
        let viewport_state = PipelineViewportStateCreateInfo{
            s_type : StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineViewportStateCreateFlags::empty(),
            p_scissors : std::ptr::null(), // Dynamic
            scissor_count : 1,
            p_viewports : std::ptr::null(), // Dynamic
            viewport_count : 1,
        };
        let dynamic_states = [DynamicState::SCISSOR,DynamicState::VIEWPORT];
        let dynamic_state = PipelineDynamicStateCreateInfo{
            s_type : StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineDynamicStateCreateFlags::empty(),
            dynamic_state_count : dynamic_states.len() as u32,
            p_dynamic_states : dynamic_states.as_ptr(),
        };
        let rasterisation_state = PipelineRasterizationStateCreateInfo{
            s_type : StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            flags : PipelineRasterizationStateCreateFlags::empty(),
            p_next : std::ptr::null(),
            rasterizer_discard_enable : 0,
            polygon_mode : PolygonMode::FILL,
            front_face : FrontFace::CLOCKWISE,
            depth_clamp_enable : 0,
            cull_mode : CullModeFlags::BACK,
            line_width : 1.0,
            depth_bias_enable : 0,
            depth_bias_constant_factor : 0.0,
            depth_bias_clamp : 0.0,
            depth_bias_slope_factor : 0.0,
        };
        let multisample_state  = PipelineMultisampleStateCreateInfo{
            s_type : StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            flags : PipelineMultisampleStateCreateFlags::empty(),
            p_next : std::ptr::null(),
            min_sample_shading : 1.0,
            sample_shading_enable : 0,
            alpha_to_coverage_enable : 0,
            alpha_to_one_enable : 0,
            p_sample_mask : std::ptr::null(),
            rasterization_samples : SampleCountFlags::TYPE_1,
        };
        let input_assembly_state = PipelineInputAssemblyStateCreateInfo{
            s_type : StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            flags : PipelineInputAssemblyStateCreateFlags::empty(),
            p_next : std::ptr::null(),
            primitive_restart_enable : 0,
            topology : PrimitiveTopology::TRIANGLE_LIST,
        };
        let attribute_descriptions = Vertex::get_attribute_descriptions();
        let binding_descriptions = Vertex::get_binding_descriptions();
        let vertex_input_info = PipelineVertexInputStateCreateInfo{
            s_type : StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineVertexInputStateCreateFlags::empty(),
            vertex_attribute_description_count : 2,
            p_vertex_attribute_descriptions : attribute_descriptions.as_ptr(),
            vertex_binding_description_count : 1,
            p_vertex_binding_descriptions : binding_descriptions.as_ptr(),
        };
        let modules = [
            VKPipeline::create_shader_module(include_bytes!("./shaders/main.vert.spv").to_vec(), device),
            VKPipeline::create_shader_module(include_bytes!("./shaders/main.frag.spv").to_vec(), device),
        ];
        let p_name = CString::new("main").unwrap();
        let stages = [
            PipelineShaderStageCreateInfo{
                s_type : StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                flags : PipelineShaderStageCreateFlags::empty(),
                p_next : std::ptr::null(),
                p_name : p_name.as_ptr(),
                module : modules[0],
                p_specialization_info : &SpecializationInfo::default(),
                stage : ShaderStageFlags::VERTEX,
            },
            PipelineShaderStageCreateInfo{
                s_type : StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                flags : PipelineShaderStageCreateFlags::empty(),
                p_next : std::ptr::null(),
                p_name : p_name.as_ptr(),
                module : modules[1],
                p_specialization_info : &SpecializationInfo::default(),
                stage : ShaderStageFlags::FRAGMENT,
            },
        ];
        let pipeline_create_infos = [GraphicsPipelineCreateInfo{
            s_type : StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineCreateFlags::empty(),
            layout : pipeline_layout,
            base_pipeline_index : -1,
            base_pipeline_handle : Pipeline::null(),
            render_pass : *render_pass,
            subpass : 0,
            p_depth_stencil_state : std::ptr::null(),
            p_color_blend_state : &color_blend_state,
            p_dynamic_state : &dynamic_state,
            p_viewport_state : &viewport_state,
            p_rasterization_state : &rasterisation_state,
            p_multisample_state : &multisample_state,
            p_tessellation_state : std::ptr::null(),
            p_input_assembly_state : &input_assembly_state,
            p_vertex_input_state : &vertex_input_info,
            stage_count : stages.len() as u32,
            p_stages : stages.as_ptr(),
        }];
        let pipeline = unsafe{device.create_graphics_pipelines(PipelineCache::null(), &pipeline_create_infos, None)}.expect("Failed to create pipeline.")[0];
        for &module in modules.iter(){
            unsafe{device.destroy_shader_module(module, None)};
        }
        return VKPipeline{pipeline,pipeline_layout};
    }
    fn create_shader_module(code : Vec<u8> , device : &Device) -> ShaderModule{
        let module_create_info = ShaderModuleCreateInfo {
            s_type: StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: ShaderModuleCreateFlags::empty(),
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
        };
        let module = unsafe{device.create_shader_module(&module_create_info, None)}.expect("Failed to create shader module.");
        return module;
    }
}
pub struct VKCommandPool{
    pub command_pool : CommandPool,
    pub command_buffers : Vec<CommandBuffer>,
}
impl VKCommandPool{
    pub fn new(device : &Device , graphics_queue : u32 , count : u32) -> Self{
        let command_pool_create_info = CommandPoolCreateInfo{
            s_type : StructureType::COMMAND_POOL_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index : graphics_queue,
        };
        let command_pool = unsafe{device.create_command_pool(&command_pool_create_info, None).expect("Failed to create command pool.")};
        let command_buffers = if count != 0{
            let command_buffers_create_info = CommandBufferAllocateInfo{
                s_type : StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                p_next : std::ptr::null(),
                level : CommandBufferLevel::PRIMARY,
                command_pool : command_pool,
                command_buffer_count : count,
            };
            unsafe{device.allocate_command_buffers(&command_buffers_create_info)}.expect("Failed to allocate command buffers.")
        } else { Vec::new()};
        return VKCommandPool{command_pool,command_buffers};
    }
    pub fn begin_command_buffers(&self , device : &Device , render_pass : &RenderPass , framebuffers : &Vec<Framebuffer> , extent : &Extent2D){
        for (index,&command_buffer) in self.command_buffers.iter().enumerate(){
            let begin_info = CommandBufferBeginInfo{
                s_type : StructureType::COMMAND_BUFFER_BEGIN_INFO,
                p_next : std::ptr::null(),
                flags : CommandBufferUsageFlags::empty(),
                p_inheritance_info : std::ptr::null(),
            };
            unsafe{device.begin_command_buffer(command_buffer, &begin_info)}.expect("Failed to begin command buffer recording.");
            let render_pass_begin_info = RenderPassBeginInfo{
                s_type : StructureType::RENDER_PASS_BEGIN_INFO,
                p_next : std::ptr::null(),
                clear_value_count : 1,
                p_clear_values : &ClearValue{color:ClearColorValue{float32:[0.0,0.0,0.0,0.0]}},
                render_pass : *render_pass,
                framebuffer : framebuffers[index],
                render_area : Rect2D{extent:*extent,offset:Offset2D{x:0,y:0}},
            };
            unsafe{device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, SubpassContents::INLINE)};
        }
    }
    pub fn end_command_buffers(&self , device : &Device){
        for &command_buffer in self.command_buffers.iter(){
            unsafe{device.cmd_end_render_pass(command_buffer)};
            unsafe{device.end_command_buffer(command_buffer)}.expect("Failed to record command buffers.");
        }
    }
    pub fn record_command_buffers(&self,pipeline : &Pipeline,device : &Device,render_pass : &RenderPass , framebuffers : &Vec<Framebuffer> , extent : &Extent2D , vertex_buffer : Buffer , index_buffer : Buffer , index_count : u32 , descriptor_sets : &Vec<DescriptorSet> , pipeline_layout : &PipelineLayout){
        self.begin_command_buffers(device,render_pass,framebuffers,extent);
        for (i,&command_buffer) in self.command_buffers.iter().enumerate(){
            unsafe{device.cmd_bind_pipeline(command_buffer, PipelineBindPoint::GRAPHICS, *pipeline)};
            let vertex_buffers = [vertex_buffer];
            let offsets = [0 as DeviceSize];
            unsafe{device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets)};
            unsafe{device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, IndexType::UINT32)};
            let viewport = [Viewport{height:extent.height as f32,width : extent.width as f32,x:0.0,y:0.0,min_depth:0.0,max_depth:1.0}];
            let scissor = [Rect2D{extent:*extent,offset:Offset2D{x:0,y:0}}];
            unsafe{device.cmd_set_viewport(command_buffer, 0, &viewport)};
            unsafe{device.cmd_set_scissor(command_buffer, 0, &scissor)};
            unsafe{device.cmd_bind_descriptor_sets(command_buffer, PipelineBindPoint::GRAPHICS, *pipeline_layout, 0, &[descriptor_sets[i]], &[])}
            unsafe{device.cmd_draw_indexed(command_buffer,index_count,1,0,0,0)}
        }
        self.end_command_buffers(device);
    }
}
pub struct VKSynchroniser{
    pub image_available_semaphores : Vec<Semaphore>,
    pub render_finished_semaphores : Vec<Semaphore>,
    pub in_flight_fences : Vec<Fence>,
    pub current_frame : u32,
    pub max_frames_in_flight : u32,
}
impl VKSynchroniser{
    pub fn new(in_flight_frames : u32 , device : &Device) -> Self{
        let mut image_available_semaphores = Vec::new();
        let mut render_finished_semaphores = Vec::new();
        let mut in_flight_fences = Vec::new();
        let semaphore_create_info = SemaphoreCreateInfo{
            s_type : StructureType::SEMAPHORE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : SemaphoreCreateFlags::empty(),
        };
        let fence_create_info = FenceCreateInfo{
            s_type : StructureType::FENCE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : FenceCreateFlags::SIGNALED,
        };
        for _ in  0..in_flight_frames{
            image_available_semaphores.push(unsafe{device.create_semaphore(&semaphore_create_info, None)}.expect("Failed to create semaphore."));
            render_finished_semaphores.push(unsafe{device.create_semaphore(&semaphore_create_info, None)}.expect("Failed to create semaphore."));
            in_flight_fences.push(unsafe{device.create_fence(&fence_create_info, None)}.expect("Failed to create fence."));
        };
        return VKSynchroniser{render_finished_semaphores,image_available_semaphores,current_frame:0,max_frames_in_flight : in_flight_frames , in_flight_fences};
    }
}
pub struct VertexBuffer{
    pub buffer : Buffer,
    pub index_buffer : Buffer,
    pub memory : DeviceMemory,
    pub index_memory : DeviceMemory,
    pub vertex_count : u32,
    pub indices_count : u32,
}
impl VertexBuffer{
    pub fn new(instance : &Instance , device : &Device , physical_device : &PhysicalDevice , command_pool : CommandPool , queue : Queue , vertices : Vec<Vertex> , indices : Vec<u32>) -> Self{
        let buffer_size = (std::mem::size_of::<Vertex>() * vertices.len()) as u64;
        let device_memory_properties =unsafe{ instance.get_physical_device_memory_properties(*physical_device) };
        let (staging_buffer, staging_buffer_memory) = VertexBuffer::create_buffer(device,buffer_size,BufferUsageFlags::TRANSFER_SRC | BufferUsageFlags::TRANSFER_DST,MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,&device_memory_properties);
        let buffer_data = unsafe{device.map_memory(staging_buffer_memory,0,buffer_size,MemoryMapFlags::empty()).expect("Failed to Map Memory") as *mut Vertex};
        unsafe{buffer_data.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len())};
        unsafe{device.unmap_memory(staging_buffer_memory)};
        let (vertex_buffer, vertex_buffer_memory) = VertexBuffer::create_buffer(device,buffer_size,BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER | BufferUsageFlags::TRANSFER_SRC,MemoryPropertyFlags::DEVICE_LOCAL,&device_memory_properties);
        VertexBuffer::copy_buffer(device,queue,command_pool,staging_buffer,vertex_buffer,buffer_size);
        unsafe{device.destroy_buffer(staging_buffer, None)};
        unsafe{device.free_memory(staging_buffer_memory, None)};
        let buffer_size = (std::mem::size_of_val(&indices[0]) * indices.len()) as u64;
        let (staging_buffer,staging_buffer_memory) = VertexBuffer::create_buffer(device,buffer_size,BufferUsageFlags::TRANSFER_SRC | BufferUsageFlags::TRANSFER_DST,MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,&device_memory_properties);
        let buffer_data = unsafe{device.map_memory(staging_buffer_memory,0,buffer_size,MemoryMapFlags::empty()).expect("Failed to Map Memory") as *mut u32};
        unsafe{buffer_data.copy_from_nonoverlapping(indices.as_ptr(), indices.len())};
        unsafe{device.unmap_memory(staging_buffer_memory)};
        let (index_buffer,index_buffer_memory) = VertexBuffer::create_buffer(device, buffer_size, BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST, MemoryPropertyFlags::DEVICE_LOCAL, &device_memory_properties);
        VertexBuffer::copy_buffer(device, queue, command_pool, staging_buffer, index_buffer, buffer_size);
        unsafe{device.destroy_buffer(staging_buffer, None)};
        unsafe{device.free_memory(staging_buffer_memory, None)};
        return VertexBuffer{buffer : vertex_buffer,memory : vertex_buffer_memory, index_buffer : index_buffer , index_memory : index_buffer_memory ,vertex_count : vertices.len() as u32 , indices_count : indices.len() as u32}
    }
    fn copy_buffer(device: &ash::Device,submit_queue: Queue,command_pool: CommandPool,src_buffer: Buffer,dst_buffer: Buffer,size: DeviceSize) {
        let allocate_info = CommandBufferAllocateInfo {
            s_type: StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            command_buffer_count: 1,
            command_pool,
            level: CommandBufferLevel::PRIMARY,
        };
        let command_buffers = unsafe {device.allocate_command_buffers(&allocate_info).expect("Failed to allocate Command Buffers.")};
        let command_buffer = command_buffers[0];
        let begin_info = CommandBufferBeginInfo {
            s_type: StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: std::ptr::null(),
            flags: CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            p_inheritance_info: std::ptr::null(),
        };
        unsafe {device.begin_command_buffer(command_buffer, &begin_info).expect("Failed to begin Command Buffer.")};
        let copy_regions = [BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        }];
        unsafe{device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions)};
        unsafe{device.end_command_buffer(command_buffer).expect("Failed to end Command Buffer")};
        let submit_info = [SubmitInfo {
            s_type: StructureType::SUBMIT_INFO,
            p_next: std::ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: std::ptr::null(),
            p_wait_dst_stage_mask: std::ptr::null(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            signal_semaphore_count: 0,
            p_signal_semaphores: std::ptr::null(),
        }];
        unsafe{device.queue_submit(submit_queue, &submit_info, Fence::null()).expect("Failed to Submit Queue.")};
        unsafe{device.queue_wait_idle(submit_queue).expect("Failed to wait Queue idle")};
        unsafe{device.free_command_buffers(command_pool, &command_buffers)};
    }
    fn create_buffer(device: &ash::Device,size: DeviceSize,usage: BufferUsageFlags,required_memory_properties: MemoryPropertyFlags,device_memory_properties: &PhysicalDeviceMemoryProperties,) -> (Buffer, DeviceMemory) {
        let buffer_create_info = BufferCreateInfo {
            s_type: StructureType::BUFFER_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: BufferCreateFlags::empty(),
            size,
            usage,
            sharing_mode: SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
        };
        let buffer = unsafe{device.create_buffer(&buffer_create_info, None).expect("Failed to create Vertex Buffer")};
        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory_type = find_memory_type(
            mem_requirements.memory_type_bits,
            required_memory_properties,
            *device_memory_properties,
        );
        let allocate_info = MemoryAllocateInfo {
            s_type: StructureType::MEMORY_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            allocation_size: mem_requirements.size,
            memory_type_index: memory_type,
        };
        let buffer_memory = unsafe {device.allocate_memory(&allocate_info, None).expect("Failed to allocate vertex buffer memory!")};
        unsafe{device.bind_buffer_memory(buffer, buffer_memory, 0).expect("Failed to bind Buffer");}
        (buffer, buffer_memory)
    }
    fn update_buffer(&mut self , device : &VKDevice , instance : &VKInstance , vertices : Vec<Vertex> , indices : Vec<u32> , command_pool : &CommandPool , queue : &Queue){
        let buffer_size = (std::mem::size_of::<Vertex>() * vertices.len()) as u64;
        let device_memory_properties =unsafe{ instance.instance.get_physical_device_memory_properties(device.physical_device) };
        let (staging_buffer , staging_buffer_memory) = VertexBuffer::create_buffer(&device.device, buffer_size, BufferUsageFlags::TRANSFER_SRC, MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE, &device_memory_properties);
        let buffer_data = unsafe{device.device.map_memory(staging_buffer_memory,0,buffer_size,MemoryMapFlags::empty()).expect("Failed to Map Memory") as *mut Vertex};
        unsafe{buffer_data.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len())};
        unsafe{device.device.unmap_memory(staging_buffer_memory)};
        VertexBuffer::copy_buffer(&device.device,*queue,*command_pool,staging_buffer,self.buffer,buffer_size);
        unsafe{device.device.destroy_buffer(staging_buffer, None)};
        unsafe{device.device.free_memory(staging_buffer_memory, None)};
        let buffer_size = (std::mem::size_of_val(&indices[0]) * indices.len()) as u64;
        let (staging_buffer , staging_buffer_memory) = VertexBuffer::create_buffer(&device.device, buffer_size, BufferUsageFlags::TRANSFER_SRC, MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE, &device_memory_properties);
        let buffer_data = unsafe{device.device.map_memory(staging_buffer_memory,0,buffer_size,MemoryMapFlags::empty()).expect("Failed to Map Memory") as *mut u32};
        unsafe{buffer_data.copy_from_nonoverlapping(indices.as_ptr(), indices.len())};
        unsafe{device.device.unmap_memory(staging_buffer_memory)};
        VertexBuffer::copy_buffer(&device.device,*queue,*command_pool,staging_buffer,self.index_buffer,buffer_size);
        unsafe{device.device.destroy_buffer(staging_buffer, None)};
        unsafe{device.device.free_memory(staging_buffer_memory, None)};
        self.indices_count = indices.len() as u32;
    }
}
pub struct VKDescriptorPool{
    pub set_layout : DescriptorSetLayout,
    pub buffers : Vec<Buffer>,
    pub buffers_memory : Vec<DeviceMemory>,
    pub descriptor_pool : DescriptorPool,
    pub descriptor_sets : Vec<DescriptorSet>
}
impl VKDescriptorPool{
    pub fn new(device : &Device , instance : &Instance , physical_device : &PhysicalDevice, buffer_count : u32) -> Self{
        let ubo_layout_binding = DescriptorSetLayoutBinding{
            binding : 0,
            descriptor_type : DescriptorType::UNIFORM_BUFFER,
            descriptor_count : 1,
            stage_flags : ShaderStageFlags::VERTEX,
            p_immutable_samplers : std::ptr::null(),
        };
        let layout_bindings = [ubo_layout_binding];
        let descriptor_set_layout_create_info = DescriptorSetLayoutCreateInfo{
            s_type : StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : DescriptorSetLayoutCreateFlags::empty(),
            binding_count : layout_bindings.len() as u32,
            p_bindings : layout_bindings.as_ptr(),
        };
        let set_layout = unsafe{device.create_descriptor_set_layout(&descriptor_set_layout_create_info, None)}.expect("Failed to create desciptor set layout.");
        let buffer_size = std::mem::size_of::<UniformBufferObject>();
        let mut buffers = vec!();
        let mut buffers_memory = vec!();
        let memory_properties = unsafe{instance.get_physical_device_memory_properties(*physical_device)};
        for _ in 0..buffer_count{
            let (buffer,memory) = VertexBuffer::create_buffer(device, buffer_size as u64, BufferUsageFlags::UNIFORM_BUFFER, MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT, &memory_properties);
            buffers.push(buffer);
            buffers_memory.push(memory);
        }
        let pool_sizes = [DescriptorPoolSize{
            ty : DescriptorType::UNIFORM_BUFFER,
            descriptor_count : buffer_count,
        }];
        let pool_info = DescriptorPoolCreateInfo{
            s_type : StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : DescriptorPoolCreateFlags::empty(),
            p_pool_sizes : pool_sizes.as_ptr(),
            max_sets : buffer_count,
            pool_size_count : 1,
        };
        let descriptor_pool = unsafe{device.create_descriptor_pool(&pool_info, None)}.expect("Failed to create descriptor pool.");
        let mut layouts : Vec<DescriptorSetLayout> = vec!();
        for _ in 0..buffer_size {
            layouts.push(set_layout);
        }
        let descriptor_set_allocate_info = DescriptorSetAllocateInfo {
            s_type: StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            descriptor_pool,
            descriptor_set_count: buffer_count,
            p_set_layouts: layouts.as_ptr(),
        };
        let descriptor_sets = unsafe{device.allocate_descriptor_sets(&descriptor_set_allocate_info)}.expect("Failed to create descriptor sets.");
        for (i, &descriptor_set) in descriptor_sets.iter().enumerate(){
            let descriptor_buffer_info = [DescriptorBufferInfo {
                buffer: buffers[i],
                offset: 0,
                range: std::mem::size_of::<UniformBufferObject>() as u64,
            }];

            let descriptor_write_sets = [WriteDescriptorSet {
                s_type: StructureType::WRITE_DESCRIPTOR_SET,
                p_next: std::ptr::null(),
                dst_set: descriptor_set,
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: DescriptorType::UNIFORM_BUFFER,
                p_image_info: std::ptr::null(),
                p_buffer_info: descriptor_buffer_info.as_ptr(),
                p_texel_buffer_view: std::ptr::null(),
            }];
            unsafe{device.update_descriptor_sets(&descriptor_write_sets, &[])};
        }
        return VKDescriptorPool{set_layout,buffers,buffers_memory,descriptor_pool,descriptor_sets};
    }
    pub fn update_uniform_buffer(&self , current_index : u32 ,  model_matrix : &cgmath::Matrix4<f32> , view_matrix : &cgmath::Matrix4<f32> , proj_matrix : &cgmath::Matrix4<f32> , device : &Device){
        let ubos = [UniformBufferObject{
            model: *model_matrix,
            view: *view_matrix,
            proj: *proj_matrix,
        }];
        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;
        let buffer_data = unsafe{device.map_memory(self.buffers_memory[current_index as usize], 0, buffer_size, MemoryMapFlags::empty())}.expect("Failed to rotate camera.") as *mut UniformBufferObject;
        unsafe{buffer_data.copy_from_nonoverlapping(ubos.as_ptr(),ubos.len())};
        unsafe{device.unmap_memory(self.buffers_memory[current_index as usize])};
    }
}
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 3],
}
impl Vertex {
    fn get_binding_descriptions() -> [VertexInputBindingDescription; 1] {
        [VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: VertexInputRate::VERTEX,
        }]
    }

    fn get_attribute_descriptions() -> [VertexInputAttributeDescription; 2] {
        [
            VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: Format::R32G32_SFLOAT,
                offset: memoffset::offset_of!(Self, pos) as u32,
            },
            VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: Format::R32G32B32_SFLOAT,
                offset: memoffset::offset_of!(Self, color) as u32,
            },
        ]
    }
}
#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct UniformBufferObject{
    model : cgmath::Matrix4<f32>,
    view : cgmath::Matrix4<f32>,
    proj : cgmath::Matrix4<f32>,
}
fn find_memory_type(type_filter: u32,required_properties: MemoryPropertyFlags,mem_properties: PhysicalDeviceMemoryProperties,) -> u32 {
    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(required_properties)
        {return i as u32;}
    }

    panic!("Failed to find suitable memory type!")
}
