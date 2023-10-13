use std::mem::align_of;

use anyhow::{Ok, Result};
use ash::{
    util::Align,
    vk::{self, DeviceMemory, MemoryRequirements},
};

use crate::find_memorytype_index;

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub size_in_bytes: usize,
    pub usage: vk::BufferUsageFlags,
    pub memory: DeviceMemory,
    pub requirements: MemoryRequirements,
}

impl Buffer {
    pub fn init(
        size_in_bytes: usize,
        usage: vk::BufferUsageFlags,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
        logical_device: &ash::Device,
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

        Ok(Self {
            buffer,
            size_in_bytes,
            usage,
            memory,
            requirements,
        })
    }

    pub fn fill<T>(
        &mut self,
        logical_device: &ash::Device,
        data: &[T],
        memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> Result<()>
    where
        T: Copy,
    {
        let bytes_to_write = std::mem::size_of_val(data);
        if bytes_to_write > self.size_in_bytes {
            unsafe { logical_device.destroy_buffer(self.buffer, None) };
            let new_buffer = Buffer::init(
                bytes_to_write,
                self.usage,
                memory_properties,
                logical_device,
            )?;
            *self = new_buffer;
        }

        let data_ptr = unsafe {
            logical_device.map_memory(
                self.memory,
                0,
                self.requirements.size,
                vk::MemoryMapFlags::empty(),
            )
        }?;

        let mut align =
            unsafe { Align::new(data_ptr, align_of::<T>() as u64, self.requirements.size) };
        align.copy_from_slice(data);

        unsafe { logical_device.unmap_memory(self.memory) };
        unsafe { logical_device.bind_buffer_memory(self.buffer, self.memory, 0) }?;

        Ok(())
    }
}
