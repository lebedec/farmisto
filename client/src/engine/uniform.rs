use crate::engine::base::create_buffer;
use ash::{vk, Device};
use glam::Mat4;

#[derive(Clone, Copy)]
pub struct CameraUniform {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

pub struct UniformBuffer {
    device: Device,
    pub buffers: Vec<vk::Buffer>,
    memory: Vec<vk::DeviceMemory>,
    memory_size: Vec<vk::DeviceSize>,
}

impl UniformBuffer {
    pub fn create<T>(
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
        }
    }

    pub fn update<T: Copy>(&self, current: usize, value: T) {
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
