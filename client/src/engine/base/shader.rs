use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ptr;

use ash::vk::Handle;
use ash::{vk, Device};
use lazy_static::lazy_static;

use crate::assets::TextureAsset;

lazy_static! {
    static ref METRIC_DESCRIBES_TOTAL: prometheus::IntCounterVec =
        prometheus::register_int_counter_vec!(
            "shader_describes_total",
            "shader_describes_total",
            &["cache"]
        )
        .unwrap();
}

pub struct ShaderDataSet<const B: usize> {
    device: Device,
    pool: vk::DescriptorPool,
    pub layout: vk::DescriptorSetLayout,
    descriptors: HashMap<u64, Vec<vk::DescriptorSet>>,
    bindings: [vk::DescriptorType; B],
}

pub enum ShaderData {
    Texture(vk::DescriptorImageInfo),
    Uniform(vk::DescriptorBufferInfo),
}

impl From<&TextureAsset> for ShaderData {
    fn from(texture: &TextureAsset) -> Self {
        ShaderData::Texture(vk::DescriptorImageInfo {
            sampler: texture.sampler,
            image_view: texture.view,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        })
    }
}

impl From<&mut TextureAsset> for ShaderData {
    fn from(texture: &mut TextureAsset) -> Self {
        ShaderData::Texture(vk::DescriptorImageInfo {
            sampler: texture.sampler,
            image_view: texture.view,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        })
    }
}

impl ShaderData {
    pub fn id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        match self {
            ShaderData::Texture(info) => {
                let id = [info.sampler.as_raw(), info.image_view.as_raw()];
                id.hash(&mut hasher);
                hasher.finish()
            }
            ShaderData::Uniform(info) => {
                let id = info.buffer.as_raw();
                id.hash(&mut hasher);
                hasher.finish()
            }
        }
    }
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

    fn identify_writes(&self, writes: &Vec<[ShaderData; B]>) -> u64 {
        let mut hasher = DefaultHasher::new();
        let mut ids = vec![];
        for write in writes {
            for data in write {
                ids.push(data.id());
            }
        }
        ids.hash(&mut hasher);
        hasher.finish()
    }

    pub fn describe(&mut self, writes: Vec<[ShaderData; B]>) -> Vec<vk::DescriptorSet> {
        let id = self.identify_writes(&writes);
        if let Some(descriptors) = self.descriptors.get(&id) {
            METRIC_DESCRIBES_TOTAL.with_label_values(&["hit"]).inc();
            return descriptors.clone();
        }
        METRIC_DESCRIBES_TOTAL.with_label_values(&["miss"]).inc();
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
                        ShaderData::Texture(info) => info,
                        _ => ptr::null(),
                    },
                    p_buffer_info: match data {
                        ShaderData::Uniform(info) => info,
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
