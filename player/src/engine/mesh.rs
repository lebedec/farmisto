use crate::engine::base::index_memory_type;
use ash::{vk, Device};

pub struct IndexBuffer {
    buffer: vk::Buffer,
    count: u32,
}

impl IndexBuffer {
    pub fn bind(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn create(
        device: &Device,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        indices: Vec<u32>,
    ) -> Self {
        let info = vk::BufferCreateInfo::builder()
            .size(std::mem::size_of_val(&indices) as u64)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.create_buffer(&info, None).unwrap() };
        let memory = unsafe { device.get_buffer_memory_requirements(buffer) };

        let memory_type_index = index_memory_type(
            &memory,
            device_memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("Unable to find suitable memory type for the index buffer.");

        let allocation = vk::MemoryAllocateInfo {
            allocation_size: memory.size,
            memory_type_index,
            ..Default::default()
        };

        let device_memory = unsafe { device.allocate_memory(&allocation, None).unwrap() };

        // WRITE

        let ptr = unsafe {
            device
                .map_memory(device_memory, 0, memory.size, vk::MemoryMapFlags::empty())
                .unwrap()
        };
        let mut alignment =
            unsafe { ash::util::Align::new(ptr, std::mem::align_of::<u32>() as u64, memory.size) };
        alignment.copy_from_slice(&indices);

        unsafe {
            device.unmap_memory(device_memory);
            device.bind_buffer_memory(buffer, device_memory, 0).unwrap();
        }

        Self {
            buffer,
            count: indices.len() as u32,
        }
    }
}

pub struct VertexBuffer {
    buffer: vk::Buffer,
}

impl VertexBuffer {
    #[inline]
    pub fn bind(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn create(
        device: &Device,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        vertices: Vec<Vertex>,
    ) -> Self {
        let info = vk::BufferCreateInfo {
            size: (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = unsafe { device.create_buffer(&info, None).unwrap() };
        let memory = unsafe { device.get_buffer_memory_requirements(buffer) };

        let memory_type_index = index_memory_type(
            &memory,
            device_memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("Unable to find suitable memory type for the vertex buffer.");

        let allocation = vk::MemoryAllocateInfo {
            allocation_size: memory.size,
            memory_type_index,
            ..Default::default()
        };

        let device_memory = unsafe { device.allocate_memory(&allocation, None).unwrap() };

        // WRITE

        let ptr = unsafe {
            device
                .map_memory(device_memory, 0, memory.size, vk::MemoryMapFlags::empty())
                .unwrap()
        };

        let mut alignment = unsafe {
            ash::util::Align::new(ptr, std::mem::align_of::<Vertex>() as u64, memory.size)
        };
        alignment.copy_from_slice(&vertices);

        unsafe {
            device.unmap_memory(device_memory);
            device.bind_buffer_memory(buffer, device_memory, 0).unwrap();
        }

        Self { buffer }
    }
}

#[derive(Default, Clone, Debug, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

impl Vertex {
    #[inline(always)]
    pub fn describe() -> vk::PipelineVertexInputStateCreateInfo {
        let binding = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let attributes = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 16,
            },
        ];
        vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&attributes)
            .vertex_binding_descriptions(&binding)
            .build()
    }
}
