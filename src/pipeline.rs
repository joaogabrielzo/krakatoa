use crate::swapchain::Swapchain;
use anyhow::{Ok, Result};
use ash::vk;

pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
}

impl Pipeline {
    pub fn init(
        logical_device: &ash::Device,
        swapchain: &Swapchain,
        renderpass: &vk::RenderPass,
    ) -> Result<Self> {
        /* Shaders */
        let vertex_info = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!("shaders/shader.vert", kind: vert));
        let vertex_module = unsafe { logical_device.create_shader_module(&vertex_info, None) }?;

        let fragment_info = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!("shaders/shader.frag", kind: frag));
        let fragment_module = unsafe { logical_device.create_shader_module(&fragment_info, None) }?;

        let main_function_name = std::ffi::CString::new("main").unwrap();
        let vertex_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_module)
            .name(&main_function_name);
        let fragment_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_module)
            .name(&main_function_name);
        let shader_stages = vec![vertex_stage.build(), fragment_stage.build()];

        let vertex_attrib_descs = [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                offset: 0,
                format: vk::Format::R32G32B32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 1,
                offset: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 2,
                offset: 16,
                format: vk::Format::R32G32B32A32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 3,
                offset: 32,
                format: vk::Format::R32G32B32A32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 4,
                offset: 48,
                format: vk::Format::R32G32B32A32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 5,
                offset: 64,
                format: vk::Format::R32G32B32_SFLOAT,
            },
        ];
        let vertex_binding_descs = [
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: 12,
                input_rate: vk::VertexInputRate::VERTEX,
            },
            vk::VertexInputBindingDescription {
                binding: 1,
                stride: 76,
                input_rate: vk::VertexInputRate::INSTANCE,
            },
        ];
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_attrib_descs)
            .vertex_binding_descriptions(&vertex_binding_descs);
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        /* Rasterization */

        let viewports = [vk::Viewport {
            x: 0.,
            y: 0.,
            width: swapchain.extent.width as f32,
            height: swapchain.extent.height as f32,
            min_depth: 0.,
            max_depth: 1.,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        }];
        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .line_width(1.0)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .cull_mode(vk::CullModeFlags::NONE)
            .polygon_mode(vk::PolygonMode::FILL);

        let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let colourblend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .build()];
        let colourblend_info =
            vk::PipelineColorBlendStateCreateInfo::builder().attachments(&colourblend_attachments);
        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);

        /* Descriptor Set Layout */
        let descriptorset_layout_binding_descs = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build()];
        let descriptorset_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&descriptorset_layout_binding_descs);
        let descriptorset_layout = unsafe {
            logical_device.create_descriptor_set_layout(&descriptorset_layout_info, None)
        }?;
        let descriptor_layouts = vec![descriptorset_layout];
        let _pipeline_layout_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_layouts);

        /* Pipeline */
        let pipeline_layout_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_layouts);
        let pipeline_layout =
            unsafe { logical_device.create_pipeline_layout(&pipeline_layout_info, None) }?;

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampler_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&colourblend_info)
            .layout(pipeline_layout)
            .render_pass(*renderpass)
            .subpass(0);
        let graphics_pipeline = unsafe {
            logical_device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipeline_info.build()],
                    None,
                )
                .expect("A problem with the pipeline creation")
        }[0];

        unsafe {
            logical_device.destroy_shader_module(fragment_module, None);
            logical_device.destroy_shader_module(vertex_module, None)
        }

        Ok(Pipeline {
            pipeline: graphics_pipeline,
            layout: pipeline_layout,
            descriptor_set_layouts: descriptor_layouts,
        })
    }

    pub fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            for dsl in &self.descriptor_set_layouts {
                logical_device.destroy_descriptor_set_layout(*dsl, None);
            }
            logical_device.destroy_pipeline(self.pipeline, None);
            logical_device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
