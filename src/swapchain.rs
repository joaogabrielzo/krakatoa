use anyhow::Result;
use ash::vk;

use crate::{
    queue::{QueueFamilies, Queues},
    surface::Surface,
};

pub struct Swapchain {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        surface: &Surface,
        queue_families: &QueueFamilies,
        _queues: &Queues,
    ) -> Result<Self> {
        let surface_capabilities = surface.get_capabilities(physical_device)?;
        let _surface_present_modes = surface.get_present_modes(physical_device)?;
        let surface_formats = surface.get_formats(physical_device)?;

        let queue_families = [queue_families.graphics_q_index.unwrap()];
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
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
        let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &logical_device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }?;

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;
        let mut image_views = Vec::with_capacity(images.len());
        images.iter().for_each(|image| {
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
            let image_view = unsafe {
                logical_device
                    .create_image_view(&imageview_create_info, None)
                    .expect("Creating image view.")
            };
            image_views.push(image_view);
        });

        Ok(Swapchain {
            swapchain_loader,
            swapchain,
            images,
            image_views,
        })
    }

    pub unsafe fn cleanup(&self, logical_device: &ash::Device) {
        for iv in &self.image_views {
            unsafe { logical_device.destroy_image_view(*iv, None) }
        }

        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None);
    }
}
