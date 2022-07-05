use crate::engine::assets::space3;
use crate::engine::assets::space3::S3Mesh;
use crate::engine::base::{create_buffer, Queue};
use ash::{vk, Device};
use glam::Mat4;
use log::{error, info};
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct MeshAsset {
    data: Arc<RefCell<MeshAssetData>>,
}

impl MeshAsset {
    #[inline]
    pub fn index(&self) -> vk::Buffer {
        self.data.borrow().index.bind()
    }

    #[inline]
    pub fn vertex(&self) -> vk::Buffer {
        self.data.borrow().vertex.bind()
    }

    #[inline]
    pub fn vertices(&self) -> u32 {
        self.data.borrow().index.count()
    }

    #[inline]
    pub fn update(&mut self, data: MeshAssetData) {
        let mut this = self.data.borrow_mut();
        *this = data;
    }

    pub fn from_data(data: Arc<RefCell<MeshAssetData>>) -> Self {
        Self { data }
    }
}

#[derive(Clone)]
pub struct MeshAssetData {
    index: IndexBuffer,
    vertex: VertexBuffer,
}

impl MeshAssetData {
    pub fn fallback(queue: &Queue) -> Result<Self, MeshAssetError> {
        let json = JsonMesh {
            vertices: vec![
                Vertex {
                    pos: [-1.0, 1.0, 0.0, 1.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                    uv: [0.0, 0.0],
                },
                Vertex {
                    pos: [1.0, 1.0, 0.0, 1.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                    uv: [1.0, 0.0],
                },
                Vertex {
                    pos: [0.0, -1.0, 0.0, 1.0],
                    color: [1.0, 0.0, 0.0, 1.0],
                    uv: [0.5, 1.0],
                },
            ],
            indices: vec![0, 1, 2],
        };
        Self::from_json(queue, json)
    }

    pub fn from_json_file<P: AsRef<Path>>(queue: &Queue, path: P) -> Result<Self, MeshAssetError> {
        let file = File::open(path).map_err(MeshAssetError::Io)?;
        let json = serde_json::from_reader(file).map_err(MeshAssetError::Serde)?;
        Self::from_json(queue, json)
    }

    pub fn from_json(queue: &Queue, mesh: JsonMesh) -> Result<Self, MeshAssetError> {
        let index = IndexBuffer::create(&queue.device, &queue.device_memory, mesh.indices);
        let vertex = VertexBuffer::create(&queue.device, &queue.device_memory, mesh.vertices);
        Ok(Self { index, vertex })
    }

    pub fn from_space3<P: AsRef<Path>>(queue: &Queue, path: P) -> Result<Self, MeshAssetError> {
        let mut scene = space3::read_scene_from_file(path).map_err(MeshAssetError::Space3)?;
        if scene.meshes.len() != 1 {
            return Err(MeshAssetError::Space3Content);
        }
        // todo: optimize struct, remove translation and collect
        let mesh = std::mem::replace(&mut scene.meshes[0], S3Mesh::default());
        let json = JsonMesh {
            vertices: mesh
                .vertices
                .into_iter()
                .map(|vertex| Vertex {
                    pos: [
                        vertex.position[0],
                        vertex.position[1],
                        vertex.position[2],
                        1.0,
                    ],
                    color: [1.0; 4],
                    uv: vertex.uv,
                })
                .collect(),
            indices: mesh.triangles,
        };
        Self::from_json(queue, json)
    }
}

#[derive(serde::Deserialize)]
pub struct JsonMesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

#[derive(Debug)]
pub enum MeshAssetError {
    Io(io::Error),
    Serde(serde_json::Error),
    Space3(space3::S3SceneError),
    Space3Content,
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

#[repr(C)]
pub struct Transform {
    pub matrix: Mat4,
}

#[derive(Clone, Copy)]
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

#[derive(Default, Clone, Debug, Copy, bytemuck::Pod, bytemuck::Zeroable, serde::Deserialize)]
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
