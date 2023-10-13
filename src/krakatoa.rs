use crate::create_command_buffers;
use crate::model::{InstanceData, Model};
use crate::pipeline::Pipeline;
use crate::pools::Pools;
use crate::{
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
    pub physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_families: QueueFamilies,
    pub queues: Queues,
    pub logical_device: ash::Device,
    pub swapchain: Swapchain,
    pub renderpass: vk::RenderPass,
    pub pipeline: Pipeline,
    pub pools: Pools,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub models: Vec<Model<[f32; 3], InstanceData>>,
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

        let mut cube = Model::cube();
        cube.insert_visibly(InstanceData {
            position: [0.0, 0.0, 0.0],
            colour: [1.0, 0.0, 0.0],
        });
        cube.update_vertex_buffer(&logical_device, memory_properties)?;
        cube.update_instance_buffer(&logical_device, memory_properties)?;

        let models = vec![cube];

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
            &models,
        )?;

        Ok(Self {
            window,
            entry,
            instance,
            debug,
            surface,
            physical_device,
            physical_device_properties,
            physical_device_memory_properties: memory_properties,
            queue_families,
            queues,
            logical_device,
            swapchain,
            renderpass,
            pipeline,
            pools,
            command_buffers,
            models,
        })
    }

    fn fill_command_buffers(
        command_buffers: &[vk::CommandBuffer],
        logical_device: &ash::Device,
        renderpass: &vk::RenderPass,
        swapchain: &Swapchain,
        pipeline: &Pipeline,
        models: &[Model<[f32; 3], InstanceData>],
    ) -> Result<()> {
        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();
            unsafe {
                logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info)?;
            }
            let clearvalues = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.4, 0.5, 0.6, 1.0],
                },
            }];
            let renderpass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(*renderpass)
                .framebuffer(swapchain.framebuffers[i])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: swapchain.extent,
                })
                .clear_values(&clearvalues);
            unsafe {
                logical_device.cmd_begin_render_pass(
                    command_buffer,
                    &renderpass_begin_info,
                    vk::SubpassContents::INLINE,
                );
                logical_device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.pipeline,
                );
                models
                    .iter()
                    .for_each(|m| m.draw(logical_device, command_buffer));
                logical_device.cmd_draw(command_buffer, 6, 1, 0, 0);
                logical_device.cmd_end_render_pass(command_buffer);
                logical_device.end_command_buffer(command_buffer)?;
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
            for m in &self.models {
                if let Some(vb) = &m.vertex_buffer {
                    self.logical_device.destroy_buffer(vb.buffer, None);
                }
                if let Some(ib) = &m.instance_buffer {
                    self.logical_device.destroy_buffer(ib.buffer, None);
                }
            }
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
