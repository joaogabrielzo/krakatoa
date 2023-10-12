use crate::create_command_buffers;
use crate::pipeline::Pipeline;
use crate::pools::Pools;
use crate::{
    buffer::Buffer,
    debug::Debug,
    init_device_and_queues, init_instance, init_physical_device_and_properties, init_renderpass,
    queue::{QueueFamilies, Queues},
    surface::Surface,
    swapchain::Swapchain,
};
use anyhow::{Ok, Result};
use ash::vk::{self};

pub struct Krakatoa {
    pub window: winit::window::Window,
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub debug: Debug,
    pub surface: Surface,
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub queue_families: QueueFamilies,
    pub queues: Queues,
    pub logical_device: ash::Device,
    pub swapchain: Swapchain,
    pub renderpass: vk::RenderPass,
    pub pipeline: Pipeline,
    pub pools: Pools,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub buffers: Vec<Buffer>,
}

impl Krakatoa {
    pub fn init(window: winit::window::Window) -> Result<Self> {
        let entry = ash::Entry::linked();
        let instance = init_instance(&entry)?;
        let debug = Debug::init(&entry, &instance)?;

        let (physical_device, physical_device_properties, physical_device_features) =
            init_physical_device_and_properties(&instance)?;

        let surface = Surface::init(&window, &entry, &instance)?;

        /* Queues */

        let queue_families = QueueFamilies::init(&instance, physical_device, &surface)?;

        /* Logical Device */

        let (logical_device, queues) = init_device_and_queues(
            &instance,
            physical_device,
            physical_device_features,
            &queue_families,
        )?;

        /* Renderpass */
        let renderpass = init_renderpass(&logical_device, physical_device, &surface)?;

        /* Swapchain */
        let mut swapchain = Swapchain::init(
            &instance,
            physical_device,
            &logical_device,
            &surface,
            &queue_families,
            &queues,
        )?;
        swapchain.create_framebuffers(&logical_device, renderpass)?;

        /* Pipeline */
        let pipeline = Pipeline::init(&logical_device, &swapchain, &renderpass)?;

        /* Mem Allocation */
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let data = [
            0.5f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.2f32, 0.0f32, 1.0f32, -0.5f32, 0.0f32,
            0.0f32, 1.0f32, -0.9f32, -0.9f32, 0.0f32, 1.0f32, 0.3f32, -0.8f32, 0.0f32, 1.0f32,
            0.0f32, -0.6f32, 0.0f32, 1.0f32,
        ];

        let buffer = Buffer::init(
            data.len() * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            memory_properties,
            &logical_device,
            &data,
        )?;

        let data2 = [
            15.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, 15.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32,
            15.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, 1.0f32, 0.8f32, 0.7f32, 0.0f32, 1.0f32,
            1.0f32, 0.8f32, 0.7f32, 0.0f32, 1.0f32, 1.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32,
        ];

        let buffer2 = Buffer::init(
            data2.len() * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            memory_properties,
            &logical_device,
            &data2,
        )?;

        /* Command Buffers */
        let pools = Pools::init(&logical_device, &queue_families)?;
        let command_buffers =
            create_command_buffers(&logical_device, &pools, swapchain.framebuffers.len())?;

        Self::fill_command_buffers(
            &command_buffers,
            &logical_device,
            &renderpass,
            &swapchain,
            &pipeline,
            &buffer.buffer,
            &buffer2.buffer,
        )?;

        let buffers = vec![buffer, buffer2];

        Ok(Self {
            window,
            entry,
            instance,
            debug,
            surface,
            physical_device,
            physical_device_properties,
            queue_families,
            queues,
            logical_device,
            swapchain,
            renderpass,
            pipeline,
            pools,
            command_buffers,
            buffers,
        })
    }

    fn fill_command_buffers(
        command_buffers: &Vec<vk::CommandBuffer>,
        logical_device: &ash::Device,
        renderpass: &vk::RenderPass,
        swapchain: &Swapchain,
        pipeline: &Pipeline,
        vb: &vk::Buffer,
        vb2: &vk::Buffer,
    ) -> Result<()> {
        for (i, &commandbuffer) in command_buffers.iter().enumerate() {
            let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
            unsafe {
                logical_device.begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
            }
            let clearvalues = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.08, 1.0],
                },
            }];
            let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
                .render_pass(*renderpass)
                .framebuffer(swapchain.framebuffers[i])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: swapchain.extent,
                })
                .clear_values(&clearvalues);
            unsafe {
                logical_device.cmd_begin_render_pass(
                    commandbuffer,
                    &renderpass_begininfo,
                    vk::SubpassContents::INLINE,
                );
                logical_device.cmd_bind_pipeline(
                    commandbuffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.pipeline,
                );
                logical_device.cmd_bind_vertex_buffers(commandbuffer, 0, &[*vb], &[0]);
                logical_device.cmd_bind_vertex_buffers(commandbuffer, 1, &[*vb2], &[0]);
                logical_device.cmd_draw(commandbuffer, 6, 1, 0, 0);
                logical_device.cmd_end_render_pass(commandbuffer);
                logical_device.end_command_buffer(commandbuffer)?;
            }
        }
        Ok(())
    }
}

impl Drop for Krakatoa {
    fn drop(&mut self) {
        unsafe {
            self.logical_device
                .device_wait_idle()
                .expect("Something wrong while waiting.");
            self.buffers.iter().for_each(|buffer| {
                self.logical_device.destroy_buffer(buffer.buffer, None);
            });
            self.pools.cleanup(&self.logical_device);
            self.pipeline.cleanup(&self.logical_device);
            self.swapchain.cleanup(&self.logical_device);
            self.logical_device
                .destroy_render_pass(self.renderpass, None);
            self.surface
                .surface_loader
                .destroy_surface(self.surface.surface, None);
            self.debug
                .loader
                .destroy_debug_utils_messenger(self.debug.messenger, None);
            self.logical_device.destroy_device(None);
            self.instance.destroy_instance(None);
        };
    }
}
