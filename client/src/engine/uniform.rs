use crate::engine::base::create_buffer;
use ash::{vk, Device};
use glam::Mat4;
use std::ptr;

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
}

impl UniformBuffer {
    pub fn create_descriptor_pool(device: &Device, descriptor_count: u32) -> vk::DescriptorPool {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count,
            },
        ];

        let info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(descriptor_count)
            .pool_sizes(&pool_sizes);

        unsafe { device.create_descriptor_pool(&info, None).unwrap() }
    }

    pub fn create_descriptor_sets<T>(
        device: &Device,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniforms_buffers: &Vec<vk::Buffer>,
        swapchain_images_size: usize,
    ) -> Vec<vk::DescriptorSet> {
        let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];
        for _ in 0..swapchain_images_size {
            layouts.push(descriptor_set_layout);
        }

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool,
            descriptor_set_count: swapchain_images_size as u32,
            p_set_layouts: layouts.as_ptr(),
        };

        let descriptor_sets = unsafe {
            device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };

        for (i, &descritptor_set) in descriptor_sets.iter().enumerate() {
            let descriptor_buffer_info = [vk::DescriptorBufferInfo {
                buffer: uniforms_buffers[i],
                offset: 0,
                range: std::mem::size_of::<T>() as u64,
            }];

            let descriptor_write_sets = [vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: descritptor_set,
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_image_info: ptr::null(),
                p_buffer_info: descriptor_buffer_info.as_ptr(),
                p_texel_buffer_view: ptr::null(),
            }];

            unsafe {
                device.update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }

        descriptor_sets
    }

    pub fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
        let bindings = [vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: ptr::null(),
        }];

        let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

        unsafe { device.create_descriptor_set_layout(&info, None).unwrap() }
    }

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
        }
    }

    pub fn update<T>(&self, current: usize, value: T) {
        let values = [value];
        let buffer_size = (std::mem::size_of::<T>() * values.len()) as u64;
        let device_memory = self.memory[current];

        unsafe {
            let ptr = self
                .device
                .map_memory(device_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .unwrap() as *mut T;
            ptr.copy_from_nonoverlapping(values.as_ptr(), values.len());
            self.device.unmap_memory(device_memory);
        }
    }
}
