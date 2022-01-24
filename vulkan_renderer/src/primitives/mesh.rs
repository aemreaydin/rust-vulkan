use super::vertex::Vertex;
use crate::{buffer::VBuffer, device::VDevice, impl_get, impl_get_ref};
use ash::vk::BufferUsageFlags;
use glam::Mat4;

pub type Index = u32;

#[derive(Default, Clone)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,

    vertex_buffer: VBuffer,
    index_buffer: VBuffer,
}

impl Mesh {
    pub fn new(device: &VDevice, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let vertex_buffer = VBuffer::new(device, &vertices, BufferUsageFlags::VERTEX_BUFFER)
            .expect("Failed to create vertex buffer.");

        let index_buffer = VBuffer::new(device, &indices, BufferUsageFlags::INDEX_BUFFER)
            .expect("Failed to create index buffer.");

        Self {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
        }
    }
}

impl_get_ref!(Mesh, vertices, &[Vertex]);
impl_get_ref!(Mesh, indices, &[Index]);
impl_get!(Mesh, vertex_buffer, VBuffer);
impl_get!(Mesh, index_buffer, VBuffer);

// TODO Temp
pub struct MeshPushConstants {
    pub mvp: Mat4,
}
