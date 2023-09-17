use anyhow::Result;
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk::{self, ApplicationInfo, ExtMetalSurfaceFn, InstanceCreateFlags, InstanceCreateInfo};
use learn_vulkan::vulkan_debug_utils_callback;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::event_loop::EventLoop;
use winit::window::Window;

fn main() -> Result<()> {
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
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
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
    let entry = ash::Entry::linked();
    let instance = unsafe { entry.create_instance(&create_info, None) }?;

    let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &instance);

    let utils_messenger =
        unsafe { debug_utils.create_debug_utils_messenger(&debug_create_info, None)? };

    let physical_device = unsafe { instance.enumerate_physical_devices() }?
        .into_iter()
        .next()
        .unwrap();

    /* Window */
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop)?;

    let surface = unsafe {
        ash_window::create_surface(
            &entry,
            &instance,
            window.raw_display_handle(),
            window.raw_window_handle(),
            None,
        )
    }?;
    let surface_loader = Surface::new(&entry, &instance);

    /* Queues */
    let queue_family_props =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let qfam_indices = {
        let mut found_graphics_q_index = None;
        let mut found_transfer_q_index = None;
        for (index, qfam) in queue_family_props.iter().enumerate() {
            if qfam.queue_count > 0
                && qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && unsafe {
                    surface_loader.get_physical_device_surface_support(
                        physical_device,
                        index as u32,
                        surface,
                    )?
                }
            {
                found_graphics_q_index = Some(index as u32);
            }
            if qfam.queue_count > 0 && qfam.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                if found_transfer_q_index.is_none()
                    || !qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                {
                    found_transfer_q_index = Some(index as u32);
                }
            }
        }
        (
            found_graphics_q_index.unwrap(),
            found_transfer_q_index.unwrap(),
        )
    };

    /* Logical Device */
    let priorities = [1.0f32];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(qfam_indices.0)
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(qfam_indices.1)
            .queue_priorities(&priorities)
            .build(),
    ];

    let device_extension_name_pointers = vec![ash::extensions::khr::Swapchain::name().as_ptr()];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extension_name_pointers);

    let logical_device =
        unsafe { instance.create_device(physical_device, &device_create_info, None) }?;
    let graphics_queue = unsafe { logical_device.get_device_queue(qfam_indices.0, 0) };
    let transfer_queue = unsafe { logical_device.get_device_queue(qfam_indices.1, 0) };

    /* Swapchain */
    let surface_capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, surface)
    }?;
    let surface_present_modes = unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
    }?;
    let surface_formats =
        unsafe { surface_loader.get_physical_device_surface_formats(physical_device, surface) }?;

    let queue_families = [qfam_indices.0];
    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(
            3.max(surface_capabilities.min_image_count)
                .min(surface_capabilities.max_image_count),
        )
        .image_format(surface_formats.first().unwrap().format)
        .image_color_space(surface_formats.first().unwrap().color_space)
        .image_extent(surface_capabilities.current_extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .queue_family_indices(&queue_families)
        .pre_transform(surface_capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(vk::PresentModeKHR::FIFO);
    let swapchain_loader = Swapchain::new(&instance, &logical_device);
    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }?;

    let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;
    let mut swapchain_image_views = Vec::with_capacity(swapchain_images.len());
    swapchain_images.iter().for_each(|image| {
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(*image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::B8G8R8A8_UNORM)
            .subresource_range(*subresource_range);
        let imageview = unsafe {
            logical_device
                .create_image_view(&imageview_create_info, None)
                .expect("Creating image view.")
        };
        swapchain_image_views.push(imageview);
    });

    /* Destroy */
    unsafe {
        swapchain_image_views.iter().for_each(|iv| {
            logical_device.destroy_image_view(*iv, None);
        });
        swapchain_loader.destroy_swapchain(swapchain, None);
        surface_loader.destroy_surface(surface, None);
        logical_device.destroy_device(None);
        debug_utils.destroy_debug_utils_messenger(utils_messenger, None);
        instance.destroy_instance(None)
    };

    Ok(())
}
