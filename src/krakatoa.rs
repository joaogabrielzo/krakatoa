use crate::buffer::Buffer;
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
use nalgebra::{Matrix4, Vector3};

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
    pub uniform_buffer: Buffer,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

impl Krakatoa {
    pub fn init(window: winit::window::Window) -> Result<Self> {
        let entry = ash::Entry::linked();
        let instance = init_instance(&entry)?;
        let debug = Debug::init(&entry, &instance)?;

        let (physical_device, physical_device_properties, physical_device_features) =
            init_physical_device_and_properties(&instance)?;

        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

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
            memory_properties,
        )?;
        swapchain.create_framebuffers(&logical_device, renderpass)?;

        /* Pipeline */
        let pipeline = Pipeline::init(&logical_device, &swapchain, &renderpass)?;

        /* Mem Allocation */
        let mut cube = Model::cube();
        let angle = 0.2;
        cube.insert_visibly(InstanceData {
            model_matrix: (Matrix4::from_scaled_axis(Vector3::new(0.0, 0.0, angle))
                * Matrix4::new_translation(&Vector3::new(0.0, 0.5, 0.0))
                * Matrix4::new_scaling(0.1))
            .into(),
            colour: [0.0, 0.5, 0.0],
        });
        cube.update_vertex_buffer(&logical_device, memory_properties)?;
        cube.update_instance_buffer(&logical_device, memory_properties)?;

        let models = vec![cube];

        /* Command Buffers */
        let pools = Pools::init(&logical_device, &queue_families)?;
        let command_buffers =
            create_command_buffers(&logical_device, &pools, swapchain.framebuffers.len())?;

        /* Uniform Buffers */
        let mut uniform_buffer = Buffer::init(
            128,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            memory_properties,
            &logical_device,
        )?;
        let camera_transforms: [[[f32; 4]; 4]; 2] =
            [Matrix4::identity().into(), Matrix4::identity().into()];
        uniform_buffer.fill(&logical_device, &camera_transforms, memory_properties)?;

        /* Descriptor Pool */
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: swapchain.amount_of_images as u32,
        }];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(swapchain.amount_of_images as u32)
            .pool_sizes(&pool_sizes);
        let descriptor_pool =
            unsafe { logical_device.create_descriptor_pool(&descriptor_pool_info, None) }?;

        let desc_layouts = vec![pipeline.descriptor_set_layouts[0]; swapchain.amount_of_images];
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts);
        let descriptor_sets =
            unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info) }?;

        descriptor_sets.iter().for_each(|descset| {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: uniform_buffer.buffer,
                offset: 0,
                range: 128,
            }];
            let desc_sets_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe { logical_device.update_descriptor_sets(&desc_sets_write, &[]) };
        });

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
            uniform_buffer,
            descriptor_pool,
            descriptor_sets,
        })
    }

    pub fn update(&mut self, index: usize) -> Result<()> {
        let command_buffer = self.command_buffers[index];
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();
        unsafe {
            self.logical_device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
        }?;

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.4, 0.5, 0.6, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let renderpass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass)
            .framebuffer(self.swapchain.framebuffers[index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clear_values);
        unsafe {
            self.logical_device.cmd_begin_render_pass(
                command_buffer,
                &renderpass_begin_info,
                vk::SubpassContents::INLINE,
            );
            self.logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            self.logical_device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[self.descriptor_sets[index]],
                &[],
            );
            self.models
                .iter()
                .for_each(|m| m.draw(&self.logical_device, command_buffer));
            self.logical_device.cmd_end_render_pass(command_buffer);
            self.logical_device.end_command_buffer(command_buffer)?;
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
            self.logical_device
                .destroy_buffer(self.uniform_buffer.buffer, None);
            self.logical_device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            for m in &self.models {
                if let Some(vb) = &m.vertex_buffer {
                    self.logical_device.destroy_buffer(vb.buffer, None);
                }
                if let Some(ib) = &m.instance_buffer {
                    self.logical_device.destroy_buffer(ib.buffer, None);
                }
                if let Some(ib) = &m.index_buffer {
                    self.logical_device.destroy_buffer(ib.buffer, None);
                };
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
