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
    pub framebuffers: Vec<vk::Framebuffer>,
    pub surface_format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub image_available: Vec<vk::Semaphore>,
    pub rendering_finished: Vec<vk::Semaphore>,
    pub may_begin_drawing: Vec<vk::Fence>,
    pub amount_of_images: u32,
    pub current_image: usize,
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
        let extent = surface_capabilities.current_extent;
        let _surface_present_modes = surface.get_present_modes(physical_device)?;
        let surface_format = *surface.get_formats(physical_device)?.first().unwrap();

        let queue_families = [queue_families.graphics_q_index.unwrap()];
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .min_image_count(
                3.max(surface_capabilities.min_image_count)
                    .min(surface_capabilities.max_image_count),
            )
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_families)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO);
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, logical_device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }?;

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;
        let amount_of_images = images.len();
        let mut image_views = Vec::with_capacity(amount_of_images);
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

        let mut image_available = vec![];
        let mut rendering_finished = vec![];
        let mut may_begin_drawing = vec![];

        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        for _ in 0..amount_of_images {
            let semaphore_available =
                unsafe { logical_device.create_semaphore(&semaphore_info, None)? };
            let semaphore_finished =
                unsafe { logical_device.create_semaphore(&semaphore_info, None)? };

            image_available.push(semaphore_available);
            rendering_finished.push(semaphore_finished);

            let fence = unsafe { logical_device.create_fence(&fence_info, None)? };
            may_begin_drawing.push(fence);
        }

        Ok(Swapchain {
            swapchain_loader,
            swapchain,
            images,
            image_views,
            framebuffers: vec![],
            surface_format,
            extent,
            amount_of_images: amount_of_images as u32,
            current_image: 0,
            image_available,
            rendering_finished,
            may_begin_drawing,
        })
    }

    pub fn create_framebuffers(
        &mut self,
        logical_device: &ash::Device,
        renderpass: vk::RenderPass,
    ) -> Result<()> {
        for iv in &self.image_views {
            let iview = [*iv];
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&iview)
                .width(self.extent.width)
                .height(self.extent.height)
                .layers(1);
            let fb = unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }?;
            self.framebuffers.push(fb);
        }

        Ok(())
    }

    pub unsafe fn cleanup(&self, logical_device: &ash::Device) {
        for fb in &self.framebuffers {
            logical_device.destroy_framebuffer(*fb, None);
        }
        for iv in &self.image_views {
            unsafe { logical_device.destroy_image_view(*iv, None) }
        }
        for semaphore in &self.image_available {
            logical_device.destroy_semaphore(*semaphore, None);
        }
        for semaphore in &self.rendering_finished {
            logical_device.destroy_semaphore(*semaphore, None);
        }
        for fence in &self.may_begin_drawing {
            logical_device.destroy_fence(*fence, None);
        }

        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None);
    }
}
