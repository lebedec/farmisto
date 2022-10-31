use log::{debug, info};
use std::fs::File;
use std::io::{Cursor, Error, Read};
use std::path::Path;
use std::string::FromUtf8Error;
use std::time::Instant;
use std::{io, mem};

use flate2::read::ZlibDecoder;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct S3Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub bones: [i8; 4],
    pub weights: [f32; 4],
}

#[derive(Default)]
pub struct S3Mesh {
    pub name: String,
    pub vertices: Vec<S3Vertex>,
    pub triangles: Vec<u32>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct S3Channel {
    pub node: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

pub struct S3Bone {
    pub name: String,
    pub parent: i32,
    pub matrix: [[f32; 4]; 4],
}

#[derive(Default)]
pub struct S3Armature {
    pub bones: Vec<S3Bone>,
}

pub struct S3Keyframe {
    pub channels: Vec<S3Channel>,
}

#[derive(Default)]
pub struct S3Animation {
    pub name: String,
    pub armature: S3Armature,
    pub keyframes: Vec<S3Keyframe>,
}

pub struct S3Scene {
    pub meshes: Vec<S3Mesh>,
    pub animation: S3Animation,
}

#[derive(Debug)]
pub enum S3SceneError {
    InvalidFile,
    Io(io::Error),
    Utf8(FromUtf8Error),
}

impl From<std::io::Error> for S3SceneError {
    fn from(error: Error) -> Self {
        Self::Io(error)
    }
}

impl From<FromUtf8Error> for S3SceneError {
    fn from(error: FromUtf8Error) -> Self {
        Self::Utf8(error)
    }
}

pub fn read_scene_from_file<P>(path: P) -> Result<S3Scene, S3SceneError>
where
    P: AsRef<Path>,
{
    let mut file = match File::open(path.as_ref().clone()) {
        Ok(file) => file,
        Err(error) => panic!("Unable to read scene from file {:?}", error),
    };
    let time = Instant::now();
    let scene = read_scene(&mut file)?;
    let parsing_time = time.elapsed().as_secs_f32();
    debug!(
        "Scene {:?} read time: {} seconds",
        path.as_ref(),
        parsing_time
    );
    Ok(scene)
}

pub fn read_scene<R>(stream: &mut R) -> Result<S3Scene, S3SceneError>
where
    R: Read,
{
    let mut magic = [0; 6];
    stream.read_exact(&mut magic)?;
    if &magic != b"Scene3" {
        return Err(S3SceneError::InvalidFile);
    }

    let meshes_length = read_u8(stream)? as usize;
    let mut meshes = Vec::with_capacity(meshes_length);
    for _ in 0..meshes_length {
        let name = read_name(stream)?;
        let archive = &mut decompress(stream)?;
        let vertices: Vec<S3Vertex> = read_vec(archive)?;
        let triangles = read_vec(stream)?;
        let mesh = S3Mesh {
            name,
            vertices,
            triangles,
        };

        meshes.push(mesh);
    }

    let name = read_name(stream)?;

    let bones_length = read_u8(stream)? as usize;
    let mut bones = Vec::with_capacity(bones_length);
    for _ in 0..bones_length {
        let name = read_name(stream)?;
        let parent = read_i32(stream)?;
        let mut matrix = [0; 64];
        stream.read_exact(&mut matrix)?;
        bones.push(S3Bone {
            name,
            parent,
            matrix: bytemuck::cast(matrix),
        });
    }
    let armature = S3Armature { bones };

    let stream = &mut decompress(stream)?;
    let keyframes_length = read_i32(stream)? as usize;
    let mut keyframes = Vec::with_capacity(keyframes_length);
    for _ in 0..keyframes_length {
        let keyframe = S3Keyframe {
            channels: read_vec(stream)?,
        };
        keyframes.push(keyframe);
    }

    Ok(S3Scene {
        meshes,
        animation: S3Animation {
            name,
            armature,
            keyframes,
        },
    })
}

pub fn read_f32<R>(stream: &mut R) -> io::Result<f32>
where
    R: Read,
{
    let mut buf = [0; 4];
    stream.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

pub fn read_i32<R>(stream: &mut R) -> io::Result<i32>
where
    R: Read,
{
    let mut buf = [0; 4];
    stream.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

pub fn read_i8<R>(stream: &mut R) -> io::Result<i8>
where
    R: Read,
{
    let mut buf = [0; 1];
    stream.read_exact(&mut buf)?;
    Ok(i8::from_be_bytes(buf))
}

pub fn read_u8<R>(stream: &mut R) -> io::Result<u8>
where
    R: Read,
{
    let mut buf = [0; 1];
    stream.read_exact(&mut buf)?;
    Ok(u8::from_be_bytes(buf))
}

pub fn read_name<R>(stream: &mut R) -> Result<String, S3SceneError>
where
    R: Read,
{
    let length = read_u8(stream)? as usize;
    let mut buf = vec![0; length];
    stream.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

pub fn read_vec<R, T>(stream: &mut R) -> Result<Vec<T>, S3SceneError>
where
    R: Read,
    T: Copy,
{
    let count = read_i32(stream)? as usize;
    let size = mem::size_of::<T>();
    // info!("count: {}, size: {}", count, size);

    let mut data = vec![0; count * size];
    stream.read_exact(&mut data)?;
    let items = unsafe {
        let ptr = data.as_mut_ptr();
        Vec::from_raw_parts(ptr as *mut T, count, count)
    };
    mem::forget(data);

    Ok(items)
}

pub fn decompress<R>(stream: &mut R) -> io::Result<Cursor<Vec<u8>>>
where
    R: Read,
{
    let archive_length = read_i32(stream)? as usize;
    let mut archive = vec![0; archive_length];
    stream.read_exact(&mut archive)?;
    let mut data = Vec::new();
    ZlibDecoder::new(Cursor::new(archive)).read_to_end(&mut data)?;
    let cursor = Cursor::new(data);
    Ok(cursor)
}
