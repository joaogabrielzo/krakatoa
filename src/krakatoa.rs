use crate::{
    debug::Debug,
    init_device_and_queues, init_instance, init_physical_device_and_properties,
    queue::{QueueFamilies, Queues},
    surface::Surface,
    swapchain::Swapchain,
};
use anyhow::{Ok, Result};
use ash::vk;

pub struct Krakatoa {
    pub window: winit::window::Window,
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub debug: Debug,
    pub surface: Surface,
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub queue_families: QueueFamilies,
    pub _queues: Queues,
    pub logical_device: ash::Device,
    pub swapchain: Swapchain,
}

impl Krakatoa {
    pub fn init(window: winit::window::Window) -> Result<Self> {
        let entry = ash::Entry::linked();
        let instance = init_instance(&entry)?;
        let debug = Debug::init(&entry, &instance)?;

        let (physical_device, physical_device_properties) =
            init_physical_device_and_properties(&instance)?;

        let surface = Surface::init(&window, &entry, &instance)?;

        /* Queues */

        let queue_families = QueueFamilies::init(&instance, physical_device, &surface)?;

        /* Logical Device */

        let (logical_device, _queues) =
            init_device_and_queues(&instance, physical_device, &queue_families)?;

        /* Swapchain */
        let swapchain = Swapchain::init(
            &instance,
            physical_device,
            &logical_device,
            &surface,
            &queue_families,
            &_queues,
        )?;

        Ok(Self {
            window,
            entry,
            instance,
            debug,
            surface,
            physical_device,
            physical_device_properties,
            queue_families,
            _queues,
            logical_device,
            swapchain,
        })
    }
}

impl Drop for Krakatoa {
    fn drop(&mut self) {
        unsafe {
            self.swapchain.cleanup(&self.logical_device);
            self.logical_device.destroy_device(None);
            self.surface
                .surface_loader
                .destroy_surface(self.surface.surface, None);
            self.debug
                .loader
                .destroy_debug_utils_messenger(self.debug.messenger, None);
            self.instance.destroy_instance(None);
        };
    }
}
