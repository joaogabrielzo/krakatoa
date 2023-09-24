use crate::create_command_buffers;
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
use ash::vk::{self, Buffer};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, Allocator, AllocatorCreateDesc};

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
    pub allocator: Allocator,
    pub buffer: vk::Buffer,
    pub allocation: Allocation,
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

        let (logical_device, queues) =
            init_device_and_queues(&instance, physical_device, &queue_families)?;

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
        let mut allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: logical_device.clone(),
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: false,
        })?;
        let buffer = unsafe {
            logical_device.create_buffer(
                &vk::BufferCreateInfo::builder()
                    .size(36)
                    .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                    .build(),
                None,
            )?
        };
        let requirements = unsafe { logical_device.get_buffer_memory_requirements(buffer) };
        let allocation = allocator.allocate(&AllocationCreateDesc {
            name: "Memory Allocation",
            requirements,
            location: gpu_allocator::MemoryLocation::CpuToGpu,
            linear: true,
            allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
        })?;
        unsafe {
            logical_device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())?
        };

        let data_ptr = allocation.mapped_ptr().unwrap().as_ptr() as *mut f32;
        let data = [
            0.1f32, -0.3f32, 0.0f32, 1.0f32, // Position
            5.0f32, // Size
            1.0f32, 1.0f32, 0.0f32, 1.0f32, // Colour
        ];
        unsafe { data_ptr.copy_from_nonoverlapping(data.as_ptr(), 9) };

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
            &buffer,
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
            queues,
            logical_device,
            swapchain,
            renderpass,
            pipeline,
            pools,
            command_buffers,
            allocator,
            buffer,
            allocation,
        })
    }

    fn fill_command_buffers(
        command_buffers: &Vec<vk::CommandBuffer>,
        logical_device: &ash::Device,
        renderpass: &vk::RenderPass,
        swapchain: &Swapchain,
        pipeline: &Pipeline,
        vb: &Buffer,
    ) -> Result<()> {
        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();
            unsafe {
                logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info)?
            };

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.08, 1.0],
                },
            }];
            let renderpass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(*renderpass)
                .framebuffer(swapchain.framebuffers[i])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: swapchain.extent,
                })
                .clear_values(&clear_values);

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

                logical_device.cmd_bind_vertex_buffers(command_buffer, 0, &[*vb], &[0]);
                logical_device.cmd_draw(command_buffer, 1, 1, 0, 0);
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
            self.logical_device.destroy_buffer(self.buffer, None);
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
