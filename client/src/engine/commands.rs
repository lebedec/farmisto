use ash::vk::FenceCreateInfo;
use ash::{vk, Device};
use std::ptr;
use std::time::Duration;

pub struct Single {
    pub buffer: vk::CommandBuffer,
    command_pool: vk::CommandPool,
    fence: vk::Fence,
}

impl Single {
    pub fn begin(device: &Device, command_pool: vk::CommandPool) -> Self {
        let allocation = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: 1,
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let buffer = unsafe {
            device
                .allocate_command_buffers(&allocation)
                .expect("Failed to allocate Command Buffers!")
        }[0];

        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        };

        unsafe {
            device
                .begin_command_buffer(buffer, &command_buffer_begin_info)
                .unwrap();
        }

        let fence = unsafe {
            device
                .create_fence(&FenceCreateInfo::builder().build(), None)
                .unwrap()
        };

        Self {
            buffer,
            command_pool,
            fence,
        }
    }

    pub fn submit(self, device: &Device, queue: vk::Queue) {
        unsafe {
            device.end_command_buffer(self.buffer).unwrap();
        }

        let buffers = [self.buffer];

        let info = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: ptr::null(),
            command_buffer_count: 1,
            p_command_buffers: buffers.as_ptr(),
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        }];

        unsafe {
            let fence = device
                .create_fence(&FenceCreateInfo::builder().build(), None)
                .unwrap();
            device.queue_submit(queue, &info, fence).unwrap();
            device
                .wait_for_fences(&[fence], true, Duration::from_secs(10).as_nanos() as u64)
                .unwrap();
            device.free_command_buffers(self.command_pool, &buffers);

            // device
            //     .queue_submit(queue, &info, vk::Fence::null())
            //     .unwrap();
            // device.queue_wait_idle(queue).unwrap();
            // device.free_command_buffers(self.command_pool, &buffers);
        }
    }

    pub fn wait() {}
}
