pub mod buffer;
pub mod debug;
pub mod krakatoa;
pub mod pipeline;
pub mod pools;
pub mod queue;
pub mod surface;
pub mod swapchain;

use anyhow::{Ok, Result};
use ash::extensions::ext::DebugUtils;
use ash::vk::{self, ApplicationInfo, ExtMetalSurfaceFn, InstanceCreateFlags, InstanceCreateInfo};
use ash::{Entry, Instance};
use pools::Pools;
use queue::{QueueFamilies, Queues};
use surface::Surface;

pub unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}

pub fn init_instance(entry: &Entry) -> Result<Instance, ash::vk::Result> {
    /* App Info */
    let engine_name = std::ffi::CString::new("UnknownGameEngine").unwrap();
    let app_name = std::ffi::CString::new("Learn Vulkan").unwrap();
    let app_info = ApplicationInfo::builder()
        .application_name(&app_name)
        .engine_name(&engine_name)
        .api_version(vk::API_VERSION_1_2)
        .build();

    /* Debug Info */
    let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
        .build();

    /* Instance Create Info */
    let layer_names: Vec<std::ffi::CString> =
        vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
    let layer_name_pointers: Vec<*const i8> = layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    let mut extension_names = vec![
        DebugUtils::name().as_ptr(),
        ash::extensions::khr::Surface::name().as_ptr(),
    ];
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        extension_names.push(vk::KhrPortabilityEnumerationFn::name().as_ptr());
        // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
        extension_names.push(vk::KhrGetPhysicalDeviceProperties2Fn::name().as_ptr());
        extension_names.push(ExtMetalSurfaceFn::name().as_ptr());
    }
    let create_info = InstanceCreateInfo::builder()
        .push_next(&mut debug_create_info)
        .application_info(&app_info)
        .enabled_layer_names(&layer_name_pointers)
        .flags(InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR)
        .enabled_extension_names(&extension_names)
        .build();

    /* Setup */
    unsafe { entry.create_instance(&create_info, None) }
}

pub fn init_device_and_queues(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    physical_device_features: vk::PhysicalDeviceFeatures,
    queue_families: &QueueFamilies,
) -> Result<(ash::Device, Queues)> {
    let priorities = [1.0f32];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_families.graphics_q_index.unwrap())
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_families.transfer_q_index.unwrap())
            .queue_priorities(&priorities)
            .build(),
    ];
    let device_extension_name_pointers: Vec<*const i8> =
        vec![ash::extensions::khr::Swapchain::name().as_ptr()];
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extension_name_pointers)
        .enabled_features(&physical_device_features);

    let logical_device =
        unsafe { instance.create_device(physical_device, &device_create_info, None)? };
    let graphics_queue =
        unsafe { logical_device.get_device_queue(queue_families.graphics_q_index.unwrap(), 0) };
    let transfer_queue =
        unsafe { logical_device.get_device_queue(queue_families.transfer_q_index.unwrap(), 0) };
    Ok((
        logical_device,
        Queues {
            graphics_queue,
            transfer_queue,
        },
    ))
}

pub fn init_physical_device_and_properties(
    instance: &ash::Instance,
) -> Result<(
    vk::PhysicalDevice,
    vk::PhysicalDeviceProperties,
    vk::PhysicalDeviceFeatures,
)> {
    let phys_devs = unsafe { instance.enumerate_physical_devices()? };
    let mut chosen = None;
    for p in phys_devs {
        let properties = unsafe { instance.get_physical_device_properties(p) };
        let features = unsafe { instance.get_physical_device_features(p) };
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            chosen = Some((p, properties, features));
        }
    }

    Ok(chosen.unwrap())
}

pub fn init_renderpass(
    logical_device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    surface: &Surface,
) -> Result<vk::RenderPass> {
    let attachments = [vk::AttachmentDescription::builder()
        .format(
            surface
                .get_formats(physical_device)?
                .first()
                .unwrap()
                .format,
        )
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .samples(vk::SampleCountFlags::TYPE_1)
        .build()];

    let color_attachment_refs = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];

    let subpasses = [vk::SubpassDescription::builder()
        .color_attachments(&color_attachment_refs)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .build()];

    let subspass_dependencies = [vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_subpass(0)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        )
        .build()];

    let renderpass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&subspass_dependencies);

    let renderpass = unsafe { logical_device.create_render_pass(&renderpass_info, None) }?;

    Ok(renderpass)
}

pub fn create_command_buffers(
    logical_device: &ash::Device,
    pools: &Pools,
    amount: usize,
) -> Result<Vec<vk::CommandBuffer>> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pools.graphics_command_pool)
        .command_buffer_count(amount as u32);

    Ok(unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info)? })
}

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
