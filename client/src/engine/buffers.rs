use crate::engine::base::create_buffer;
use ash::{vk, Device};
use glam::Mat4;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct CameraUniform {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

#[derive(Clone, Copy)]
pub struct LightUniform {
    pub color: [[f32; 4]; 512],
    pub position: [[f32; 4]; 512],
}

pub struct UniformBuffer<T> {
    device: Device,
    pub buffers: Vec<vk::Buffer>,
    memory: Vec<vk::DeviceMemory>,
    memory_size: Vec<vk::DeviceSize>,
    _data: PhantomData<T>,
}

impl<T> UniformBuffer<T>
where
    T: Copy,
{
    pub fn create(
        device: Device,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        swapchain_image_count: usize,
    ) -> Self {
        let buffer_size = std::mem::size_of::<T>();

        let mut buffers = vec![];
        let mut memory = vec![];
        let mut memory_size = vec![];

        for _ in 0..swapchain_image_count {
            let (buffer, device_memory, size) = create_buffer(
                &device,
                buffer_size as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                device_memory_properties,
            );
            buffers.push(buffer);
            memory.push(device_memory);
            memory_size.push(size);
        }

        Self {
            buffers,
            memory,
            device,
            memory_size,
            _data: Default::default(),
        }
    }

    pub fn info(&self, present_index: usize) -> vk::DescriptorBufferInfo {
        vk::DescriptorBufferInfo {
            buffer: self.buffers[present_index],
            offset: 0,
            range: std::mem::size_of::<T>() as u64,
        }
    }

    pub fn update(&self, current: usize, value: T) {
        let ptr = unsafe {
            self.device
                .map_memory(
                    self.memory[current],
                    0,
                    self.memory_size[current],
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap()
        };
        let mut alignment = unsafe {
            ash::util::Align::new(
                ptr,
                std::mem::align_of::<T>() as u64,
                self.memory_size[current],
            )
        };
        alignment.copy_from_slice(&[value]);
        unsafe {
            self.device.unmap_memory(self.memory[current]);
        }

        // let values = [value];
        // let buffer_size = (std::mem::size_of::<T>() * values.len()) as u64;
        // let device_memory = self.memory[current];
        //
        // unsafe {
        //     let ptr = self
        //         .device
        //         .map_memory(device_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
        //         .unwrap() as *mut T;
        //     ptr.copy_from_nonoverlapping(values.as_ptr(), values.len());
        //     self.device.unmap_memory(device_memory);
        // }
    }
}

#[derive(Clone, Copy)]
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
        let size = (4 * indices.len()) as u64;

        let (buffer, device_memory, memory_size) = create_buffer(
            device,
            size,
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

#[derive(Clone, Copy)]
pub struct VertexBuffer {
    buffer: vk::Buffer,
    device_memory: vk::DeviceMemory,
    device_size: vk::DeviceSize,
    pub vertices: usize,
}

impl VertexBuffer {
    #[inline]
    pub fn bind(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn create<T: Copy>(
        device: &Device,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        vertices: Vec<T>,
    ) -> Self {
        let (buffer, device_memory, device_size) = create_buffer(
            device,
            (vertices.len() * std::mem::size_of::<T>()) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device_memory_properties,
        );

        // WRITE
        let ptr = unsafe {
            device
                .map_memory(device_memory, 0, device_size, vk::MemoryMapFlags::empty())
                .unwrap()
        };
        let mut alignment =
            unsafe { ash::util::Align::new(ptr, std::mem::align_of::<T>() as u64, device_size) };
        alignment.copy_from_slice(&vertices);
        unsafe {
            device.unmap_memory(device_memory);
        }

        Self {
            buffer,
            device_memory,
            device_size,
            vertices: vertices.len(),
        }
    }

    pub fn update<T: Copy>(&self, vertices: Vec<T>, device: &Device) {
        // WRITE
        let ptr = unsafe {
            device
                .map_memory(
                    self.device_memory,
                    0,
                    self.device_size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap()
        };
        let mut alignment = unsafe {
            ash::util::Align::new(ptr, std::mem::align_of::<T>() as u64, self.device_size)
        };
        alignment.copy_from_slice(&vertices);
        unsafe {
            device.unmap_memory(self.device_memory);
        }
    }
}
