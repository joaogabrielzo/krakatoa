use std::mem::align_of;

use anyhow::{Ok, Result};
use ash::{util::Align, vk};

use crate::find_memorytype_index;

pub struct Buffer {
    pub buffer: vk::Buffer,
}

impl Buffer {
    pub fn init(
        size_in_bytes: usize,
        usage: vk::BufferUsageFlags,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
        logical_device: &ash::Device,
        data: &[f32],
    ) -> Result<Self> {
        let buffer = unsafe {
            logical_device.create_buffer(
                &vk::BufferCreateInfo::builder()
                    .size(size_in_bytes as u64)
                    .usage(usage)
                    .build(),
                None,
            )?
        };
        let requirements = unsafe { logical_device.get_buffer_memory_requirements(buffer) };
        let memory_index = find_memorytype_index(
            &requirements,
            &memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("Unable to find suitable memorytype for the vertex buffer.");

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(memory_index);
        let memory = unsafe { logical_device.allocate_memory(&allocate_info, None) }?;

        let data_ptr = unsafe {
            logical_device.map_memory(memory, 0, requirements.size, vk::MemoryMapFlags::empty())
        }?;
        let mut align =
            unsafe { Align::new(data_ptr, align_of::<f32>() as u64, requirements.size) };
        align.copy_from_slice(data);

        unsafe { logical_device.unmap_memory(memory) };
        unsafe { logical_device.bind_buffer_memory(buffer, memory, 0) }?;

        Ok(Self { buffer })
    }
}
