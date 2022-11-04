use crate::engine::assets::space3;
use crate::engine::assets::space3::S3Mesh;
use crate::engine::base::{create_buffer, Queue};
use ash::vk::Handle;
use ash::{vk, Device};
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct MeshAsset {
    pub data: Arc<RefCell<MeshAssetData>>,
}

impl MeshAsset {
    #[inline]
    pub fn id(&self) -> u64 {
        self.index().as_raw()
    }

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
    pub fn bounds(&self) -> MeshBounds {
        self.data.borrow().bounds
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

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct MeshBounds {
    pub x: [f32; 2],
    pub y: [f32; 2],
    pub z: [f32; 2],
}

impl MeshBounds {
    pub fn length(&self) -> [f32; 3] {
        [
            self.x[0].abs() + self.x[1].abs(),
            self.y[0].abs() + self.y[1].abs(),
            self.z[0].abs() + self.z[1].abs(),
        ]
    }

    pub fn radius(&self) -> f32 {
        self.length()[0]
    }

    pub fn offset(&self) -> [f32; 3] {
        [
            (self.x[0] + self.x[1]) / 2.0,
            (self.y[0] + self.y[1]) / 2.0 + 0.5,
            (self.z[0] + self.z[1]) / 2.0,
        ]
    }
}

#[derive(Clone)]
pub struct MeshAssetData {
    index: IndexBuffer,
    vertex: VertexBuffer,
    bounds: MeshBounds,
}

impl MeshAssetData {
    pub fn cube(queue: &Queue) -> Result<Self, MeshAssetError> {
        let json = JsonMesh {
            vertices: vec![
                Vertex {
                    position: [0.5, 0.5, 0.5],
                    uv: [0.625, 0.5],
                    normal: [1.0; 3],
                    bones: [-1; 4],
                    weights: [1.0; 4],
                },
                Vertex {
                    position: [0.5, 0.5, -0.5],
                    uv: [0.375, 0.5],
                    normal: [1.0; 3],
                    bones: [-1; 4],
                    weights: [1.0; 4],
                },
                Vertex {
                    position: [0.5, -0.5, 0.5],
                    uv: [0.625, 0.75],
                    normal: [1.0; 3],
                    bones: [-1; 4],
                    weights: [1.0; 4],
                },
                Vertex {
                    position: [0.5, -0.5, -0.5],
                    uv: [0.375, 0.75],
                    normal: [1.0; 3],
                    bones: [-1; 4],
                    weights: [1.0; 4],
                },
                Vertex {
                    position: [-0.5, 0.5, 0.5],
                    uv: [0.625, 0.25],
                    normal: [1.0; 3],
                    bones: [-1; 4],
                    weights: [1.0; 4],
                },
                Vertex {
                    position: [-0.5, 0.5, -0.5],
                    uv: [0.375, 0.25],
                    normal: [1.0; 3],
                    bones: [-1; 4],
                    weights: [1.0; 4],
                },
                Vertex {
                    position: [-0.5, -0.5, 0.5],
                    uv: [0.625, 0.0],
                    normal: [1.0; 3],
                    bones: [-1; 4],
                    weights: [1.0; 4],
                },
                Vertex {
                    position: [-0.5, -0.5, -0.5],
                    uv: [0.125, 0.75],
                    normal: [1.0; 3],
                    bones: [-1; 4],
                    weights: [1.0; 4],
                },
            ],

            indices: vec![
                0, 4, 6, 0, 6, 2, 3, 2, 6, 3, 6, 7, 7, 6, 4, 7, 4, 5, 5, 1, 3, 5, 3, 7, 1, 0, 2, 1,
                2, 3, 5, 4, 0, 5, 0, 1,
            ],
        };
        Self::from_json(queue, json)
    }

    pub fn fallback(queue: &Queue) -> Result<Self, MeshAssetError> {
        let json = JsonMesh {
            vertices: vec![
                Vertex {
                    position: [-1.0, 1.0, 0.0],
                    normal: [1.0; 3],
                    uv: [0.0, 0.0],
                    bones: [-1; 4],
                    weights: [1.0; 4],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                    normal: [1.0; 3],
                    uv: [1.0, 0.0],
                    bones: [-1; 4],
                    weights: [1.0; 4],
                },
                Vertex {
                    position: [0.0, -1.0, 0.0],
                    normal: [1.0; 3],
                    uv: [0.5, 1.0],
                    bones: [-1; 4],
                    weights: [1.0; 4],
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
        let mut bounds = MeshBounds::default();
        for vertex in &mesh.vertices {
            let vertex = vertex.position;
            // x
            if vertex[0] < bounds.x[0] {
                bounds.x[0] = vertex[0];
            }
            if vertex[0] > bounds.x[1] {
                bounds.x[1] = vertex[0];
            }
            // y
            if vertex[1] < bounds.y[0] {
                bounds.y[0] = vertex[1];
            }
            if vertex[1] > bounds.y[1] {
                bounds.y[1] = vertex[1];
            }
            // z
            if vertex[2] < bounds.z[0] {
                bounds.z[0] = vertex[2];
            }
            if vertex[2] > bounds.z[1] {
                bounds.z[1] = vertex[2];
            }
        }
        let vertex = VertexBuffer::create(&queue.device, &queue.device_memory, mesh.vertices);
        Ok(Self {
            index,
            vertex,
            bounds,
        })
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
                .map(|vertex| {
                    let vertex = Vertex {
                        position: vertex.position,
                        normal: vertex.normal,
                        uv: vertex.uv,
                        bones: vertex.bones.map(|value| value as i32),
                        weights: vertex.weights,
                    };
                    vertex
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

#[derive(Clone, Copy)]
pub struct VertexBuffer {
    buffer: vk::Buffer,
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
        let (buffer, device_memory, memory_size) = create_buffer(
            device,
            (vertices.len() * std::mem::size_of::<T>()) as u64,
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
        let mut alignment =
            unsafe { ash::util::Align::new(ptr, std::mem::align_of::<T>() as u64, memory_size) };
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
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub bones: [i32; 4],
    pub weights: [f32; 4],
}

impl Vertex {
    pub const BINDINGS: [vk::VertexInputBindingDescription; 1] =
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];

    pub const ATTRIBUTES: [vk::VertexInputAttributeDescription; 5] = [
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 12,
        },
        vk::VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 24,
        },
        vk::VertexInputAttributeDescription {
            location: 3,
            binding: 0,
            format: vk::Format::R32G32B32A32_SINT,
            offset: 32,
        },
        vk::VertexInputAttributeDescription {
            location: 4,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 48,
        },
    ];
}
