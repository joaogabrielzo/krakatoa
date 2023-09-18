use anyhow::Result;
use ash::vk;

use crate::surface::Surface;

pub struct QueueFamilies {
    pub graphics_q_index: Option<u32>,
    pub transfer_q_index: Option<u32>,
}
impl QueueFamilies {
    pub fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> Result<QueueFamilies> {
        let queuefamilyproperties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut found_graphics_q_index = None;
        let mut found_transfer_q_index = None;

        for (index, qfam) in queuefamilyproperties.iter().enumerate() {
            if qfam.queue_count > 0
                && qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && unsafe {
                    surface.surface_loader.get_physical_device_surface_support(
                        physical_device,
                        index as u32,
                        surface.surface,
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

        Ok(QueueFamilies {
            graphics_q_index: found_graphics_q_index,
            transfer_q_index: found_transfer_q_index,
        })
    }
}

pub struct Queues {
    pub graphics_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
}
