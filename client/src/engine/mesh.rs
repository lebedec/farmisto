use crate::engine::base::index_memory_type;
use ash::{vk, Device};
use glam::Mat4;
use log::info;

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
        let (buffer, device_memory, memory_size) = create_buffer(
            device,
            std::mem::size_of_val(&indices) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device_memory_properties,
        );

        // WRITE
        let ptr = unsafe {
            device
                .map_memory(device_memory, 0, memory_size, vk::MemoryMapFlags::empty())
                .unwrap()
        };
        let mut alignment =
            unsafe { ash::util::Align::new(ptr, std::mem::align_of::<u32>() as u64, memory_size) };
        alignment.copy_from_slice(&indices);
        unsafe {
            device.unmap_memory(device_memory);
        }

        Self {
            buffer,
            count: indices.len() as u32,
        }
    }
}

#[repr(C)]
pub struct Transform {
    pub matrix: Mat4,
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
        let (buffer, device_memory, memory_size) = create_buffer(
            device,
            (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device_memory_properties,
        );

        // WRITE
        let ptr = unsafe {
            device
                .map_memory(device_memory, 0, memory_size, vk::MemoryMapFlags::empty())
                .unwrap()
        };
        let mut alignment = unsafe {
            ash::util::Align::new(ptr, std::mem::align_of::<Vertex>() as u64, memory_size)
        };
        alignment.copy_from_slice(&vertices);
        unsafe {
            device.unmap_memory(device_memory);
        }

        Self { buffer }
    }
}

#[derive(Default, Clone, Debug, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl Vertex {
    #[inline(always)]
    pub fn describe() -> vk::PipelineVertexInputStateCreateInfo {
        let bindings = [vk::VertexInputBindingDescription {
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
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: 32,
            },
        ];
        vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&attributes)
            .vertex_binding_descriptions(&bindings)
            .build()
    }
}

pub fn create_buffer(
    device: &Device,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    memory_flags: vk::MemoryPropertyFlags,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
) -> (vk::Buffer, vk::DeviceMemory, vk::DeviceSize) {
    let info = vk::BufferCreateInfo {
        size,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let buffer = unsafe { device.create_buffer(&info, None).unwrap() };
    let memory = unsafe { device.get_buffer_memory_requirements(buffer) };

    let memory_type_index = index_memory_type(&memory, memory_properties, memory_flags).unwrap();

    let allocation = vk::MemoryAllocateInfo {
        allocation_size: memory.size,
        memory_type_index,
        ..Default::default()
    };

    let device_memory = unsafe { device.allocate_memory(&allocation, None).unwrap() };

    unsafe {
        device.bind_buffer_memory(buffer, device_memory, 0).unwrap();
    }

    (buffer, device_memory, memory.size)
}
