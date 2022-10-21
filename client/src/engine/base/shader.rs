use ash::{vk, Device};
use std::collections::HashMap;
use std::ptr;

pub struct ShaderDataSet<const B: usize> {
    device: Device,
    pool: vk::DescriptorPool,
    pub layout: vk::DescriptorSetLayout,
    descriptors: HashMap<u64, Vec<vk::DescriptorSet>>,
    bindings: [vk::DescriptorType; B],
}

pub enum ShaderData {
    Texture([vk::DescriptorImageInfo; 1]),
    Uniform([vk::DescriptorBufferInfo; 1]),
}

impl<const B: usize> ShaderDataSet<B> {
    pub fn create(
        device: Device,
        descriptor_count: u32,
        stage_flags: vk::ShaderStageFlags,
        bindings: [vk::DescriptorType; B],
    ) -> Self {
        let pool_sizes = bindings.map(|ty| vk::DescriptorPoolSize {
            ty,
            descriptor_count,
        });
        let info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(descriptor_count)
            .pool_sizes(&pool_sizes);
        let pool = unsafe { device.create_descriptor_pool(&info, None).unwrap() };
        let layout = create_descriptor_set_layout(&device, stage_flags, bindings);
        ShaderDataSet {
            device,
            pool,
            layout,
            descriptors: HashMap::default(),
            bindings,
        }
    }

    pub fn describe(&mut self, id: u64, writes: Vec<[ShaderData; B]>) -> Vec<vk::DescriptorSet> {
        if let Some(descriptors) = self.descriptors.get(&id) {
            return descriptors.clone();
        }
        let layouts = vec![self.layout; writes.len()];
        let allocation = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
            .set_layouts(&layouts);

        // TODO: recreate pool
        let descriptor_sets = unsafe {
            self.device
                .allocate_descriptor_sets(&allocation)
                .expect("Failed to allocate descriptor sets!")
        };

        for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
            let mut update = vec![];

            for binding in 0..B {
                let data = &writes[i][binding];
                let descriptor_type = self.bindings[binding];
                update.push(vk::WriteDescriptorSet {
                    s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next: ptr::null(),
                    dst_set: descriptor_set,
                    dst_binding: binding as u32,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type,
                    p_image_info: match data {
                        ShaderData::Texture(info) => info.as_ptr(),
                        _ => ptr::null(),
                    },
                    p_buffer_info: match data {
                        ShaderData::Uniform(info) => info.as_ptr(),
                        _ => ptr::null(),
                    },
                    p_texel_buffer_view: ptr::null(),
                })
            }

            unsafe {
                self.device.update_descriptor_sets(&update, &[]);
            }
        }

        self.descriptors.insert(id, descriptor_sets);
        self.descriptors.get(&id).unwrap().clone()
    }
}

fn create_descriptor_set_layout<const N: usize>(
    device: &Device,
    stage_flags: vk::ShaderStageFlags,
    bindings: [vk::DescriptorType; N],
) -> vk::DescriptorSetLayout {
    let bindings: Vec<vk::DescriptorSetLayoutBinding> = bindings
        .into_iter()
        .enumerate()
        .map(
            |(binding, descriptor_type)| vk::DescriptorSetLayoutBinding {
                binding: binding as u32,
                descriptor_type,
                descriptor_count: 1,
                stage_flags,
                p_immutable_samplers: ptr::null(),
            },
        )
        .collect();
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);
    unsafe { device.create_descriptor_set_layout(&info, None).unwrap() }
}
