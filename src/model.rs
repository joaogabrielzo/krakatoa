use crate::buffer::Buffer;
use ash::vk;

pub struct Model<V, I>
where
    V: Copy,
    I: Copy,
{
    pub vertex_data: Vec<V>,
    pub index_data: Vec<u32>,
    pub handle_to_index: std::collections::HashMap<usize, usize>,
    pub handles: Vec<usize>,
    pub instances: Vec<I>,
    pub first_invisible: usize,
    pub next_handle: usize,
    pub vertex_buffer: Option<Buffer>,
    pub index_buffer: Option<Buffer>,
    pub instance_buffer: Option<Buffer>,
}

impl<V: Copy, I: Copy> Model<V, I> {
    pub fn get(&self, handle: usize) -> Option<&I> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            self.instances.get(index)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, handle: usize) -> Option<&mut I> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            self.instances.get_mut(index)
        } else {
            None
        }
    }

    pub fn swap_by_handle(&mut self, handle1: usize, handle2: usize) -> Result<(), InvalidHandle> {
        if handle1 == handle2 {
            return Ok(());
        }
        if let (Some(&index1), Some(&index2)) = (
            self.handle_to_index.get(&handle1),
            self.handle_to_index.get(&handle2),
        ) {
            self.handles.swap(index1, index2);
            self.instances.swap(index1, index2);

            self.handle_to_index.insert(index1, handle1);
            self.handle_to_index.insert(index2, handle2);

            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }

    pub fn swap_by_index(&mut self, index1: usize, index2: usize) {
        if index1 == index2 {
            return;
        }
        let handle1 = self.handles[index1];
        let handle2 = self.handles[index2];

        self.handles.swap(index1, index2);
        self.instances.swap(index1, index2);

        self.handle_to_index.insert(index1, handle2);
        self.handle_to_index.insert(index2, handle1);
    }

    pub fn in_visible(&self, handle: usize) -> Result<bool, InvalidHandle> {
        if let Some(index) = self.handle_to_index.get(&handle) {
            Ok(index < &self.first_invisible)
        } else {
            Err(InvalidHandle)
        }
    }

    pub fn make_visible(&mut self, handle: usize) -> Result<(), InvalidHandle> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            if index < self.first_invisible {
                return Ok(());
            }

            self.swap_by_index(index, self.first_invisible);
            self.first_invisible += 1;
            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }

    pub fn make_invisible(&mut self, handle: usize) -> Result<(), InvalidHandle> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            if index >= self.first_invisible {
                return Ok(());
            }

            self.swap_by_index(index, self.first_invisible - 1);
            self.first_invisible -= 1;
            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }

    pub fn insert(&mut self, element: I) -> usize {
        let handle = self.next_handle;
        self.next_handle += 1;

        let index = self.instances.len();
        self.instances.push(element);
        self.handles.push(handle);
        self.handle_to_index.insert(handle, index);

        handle
    }

    pub fn insert_visibly(&mut self, element: I) -> usize {
        let new_handle = self.insert(element);
        self.make_visible(new_handle).ok();

        new_handle
    }

    pub fn remove(&mut self, handle: usize) -> Result<I, InvalidHandle> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            if index < self.first_invisible {
                self.swap_by_index(index, self.first_invisible - 1);
                self.first_invisible -= 1;
            }
            self.swap_by_index(self.first_invisible, self.instances.len() - 1);
            self.handles.pop();
            self.handle_to_index.remove(&handle);

            Ok(self.instances.pop().unwrap())
        } else {
            Err(InvalidHandle)
        }
    }

    pub fn update_vertex_buffer(
        &mut self,
        logical_device: &ash::Device,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> anyhow::Result<()> {
        if let Some(buffer) = &mut self.vertex_buffer {
            buffer.fill(logical_device, &self.vertex_data, memory_properties)?;
            anyhow::Ok(())
        } else {
            let bytes = self.vertex_data.len() * std::mem::size_of::<V>();
            let mut buffer = Buffer::init(
                bytes,
                ash::vk::BufferUsageFlags::VERTEX_BUFFER,
                memory_properties,
                logical_device,
            )?;
            buffer.fill(logical_device, &self.vertex_data, memory_properties)?;
            self.vertex_buffer = Some(buffer);

            Ok(())
        }
    }

    pub fn update_index_buffer(
        &mut self,
        logical_device: &ash::Device,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> anyhow::Result<()> {
        if let Some(buffer) = &mut self.index_buffer {
            buffer.fill(logical_device, &self.index_data, memory_properties)?;
            Ok(())
        } else {
            let bytes = self.index_data.len() * std::mem::size_of::<u32>();
            let mut buffer = Buffer::init(
                bytes,
                vk::BufferUsageFlags::INDEX_BUFFER,
                memory_properties,
                logical_device,
            )?;
            buffer.fill(logical_device, &self.index_data, memory_properties)?;
            self.index_buffer = Some(buffer);

            Ok(())
        }
    }

    pub fn update_instance_buffer(
        &mut self,
        logical_device: &ash::Device,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> anyhow::Result<()> {
        if let Some(buffer) = &mut self.instance_buffer {
            buffer.fill(
                logical_device,
                &self.instances[0..self.first_invisible],
                memory_properties,
            )?;
            Ok(())
        } else {
            let bytes = self.first_invisible * std::mem::size_of::<I>();
            let mut buffer = Buffer::init(
                bytes,
                ash::vk::BufferUsageFlags::VERTEX_BUFFER,
                memory_properties,
                logical_device,
            )?;
            buffer.fill(
                logical_device,
                &self.instances[0..self.first_invisible],
                memory_properties,
            )?;
            self.instance_buffer = Some(buffer);
            Ok(())
        }
    }

    pub fn draw(&self, logical_device: &ash::Device, command_buffer: vk::CommandBuffer) {
        if let Some(vertex_buffer) = &self.vertex_buffer {
            if let Some(instance_buffer) = &self.instance_buffer {
                if self.first_invisible > 0 {
                    unsafe {
                        logical_device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &[vertex_buffer.buffer],
                            &[0],
                        );
                        logical_device.cmd_bind_index_buffer(
                            command_buffer,
                            self.index_buffer.as_ref().unwrap().buffer,
                            0,
                            vk::IndexType::UINT32,
                        );
                        logical_device.cmd_bind_vertex_buffers(
                            command_buffer,
                            1,
                            &[instance_buffer.buffer],
                            &[0],
                        );
                        logical_device.cmd_draw_indexed(
                            command_buffer,
                            self.index_data.len() as u32,
                            self.first_invisible as u32,
                            0,
                            0,
                            0,
                        );
                    }
                }
            }
        }
    }
}

impl Model<[f32; 3], InstanceData> {
    pub fn cube() -> Self {
        let lbf = [-1.0, 1.0, 0.0]; //lbf: left-bottom-front
        let lbb = [-1.0, 1.0, 1.0];
        let ltf = [-1.0, -1.0, 0.0];
        let ltb = [-1.0, -1.0, 1.0];
        let rbf = [1.0, 1.0, 0.0];
        let rbb = [1.0, 1.0, 1.0];
        let rtf = [1.0, -1.0, 0.0];
        let rtb = [1.0, -1.0, 1.0];

        Model {
            vertex_data: vec![lbf, lbb, ltf, ltb, rbf, rbb, rtf, rtb],
            index_data: vec![
                0, 1, 5, 0, 5, 4, //bottom
                2, 7, 3, 2, 6, 7, //top
                0, 6, 2, 0, 4, 6, //front
                1, 3, 7, 1, 7, 5, //back
                0, 2, 1, 1, 2, 3, //left
                4, 5, 6, 5, 7, 6, //right
            ],
            handle_to_index: std::collections::HashMap::new(),
            handles: Vec::new(),
            instances: Vec::new(),
            first_invisible: 0,
            next_handle: 0,
            vertex_buffer: None,
            index_buffer: None,
            instance_buffer: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],
    pub colour: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct InvalidHandle;
impl std::fmt::Display for InvalidHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid handle")
    }
}
impl std::error::Error for InvalidHandle {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
