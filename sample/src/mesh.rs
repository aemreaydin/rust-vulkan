use crate::{macros::impl_u8_slice, vertex::Vertex};
use ash::vk::BufferUsageFlags;
use glam::Mat4;
use gltf::image::Data;
use itertools::izip;
use vulkan_renderer::{buffer::VBuffer, device::VDevice, image::VImage};

#[derive(Default, Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub images: Vec<Data>,

    pub vertex_buffer: VBuffer,
    pub index_buffer: VBuffer,
    pub texture_images: Vec<VImage>,
}

impl Mesh {
    pub fn new(
        device: &VDevice,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        images: Vec<Data>,
    ) -> Self {
        let vertex_buffer =
            VBuffer::new_device_local_buffer(device, &vertices, BufferUsageFlags::VERTEX_BUFFER)
                .expect("Failed to create vertex buffer.");

        let index_buffer =
            VBuffer::new_device_local_buffer(device, &indices, BufferUsageFlags::INDEX_BUFFER)
                .expect("Failed to create index buffer.");

        // let texture_images = images
        //     .iter()
        //     .map(|image| {
        //         // let pixels = &image.pixels;
        //         let format = Self::convert_gltf_format_to_ash_format(image.format);
        //         let extent = Extent3D {
        //             width: image.width,
        //             height: image.height,
        //             depth: 1,
        //         };
        //         VImage::new(
        //             device,
        //             ImageUsageFlags::SAMPLED,
        //             format,
        //             extent,
        //             ImageAspectFlags::COLOR,
        //         )
        //         .expect("Failed to create image.")
        //     })
        //     .collect::<Vec<_>>();
        Self {
            vertices,
            indices,
            images,

            vertex_buffer,
            index_buffer,
            texture_images: vec![],
        }
    }

    pub fn from_file(device: &VDevice, file: &str) -> gltf::Result<Mesh> {
        let (gltf, buffers, images) = gltf::import(file)?;

        let mut vertices = Vec::with_capacity(buffers.len());
        let mut indices = Vec::with_capacity(buffers.len());

        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                if let (Some(pos_iter), Some(norm_iter), Some(tex_iter)) = (
                    reader.read_positions(),
                    reader.read_normals(),
                    reader.read_tex_coords(0),
                ) {
                    assert_eq!(pos_iter.len(), norm_iter.len());
                    for (position, normal, uv) in izip!(pos_iter, norm_iter, tex_iter.into_f32()) {
                        vertices.push(Vertex::new(position.into(), normal.into(), uv.into()));
                    }
                }
                if let Some(iter) = reader.read_indices() {
                    for index in iter.into_u32() {
                        indices.push(index)
                    }
                }
            }
        }

        Ok(Mesh::new(device, vertices, indices, images))
    }

    #[allow(dead_code)]
    fn convert_gltf_format_to_ash_format(format: gltf::image::Format) -> ash::vk::Format {
        match format {
            gltf::image::Format::B8G8R8 => ash::vk::Format::B8G8R8_SRGB,
            gltf::image::Format::B8G8R8A8 => ash::vk::Format::B8G8R8A8_SRGB,
            gltf::image::Format::R16 => ash::vk::Format::R16_SINT,
            gltf::image::Format::R16G16 => ash::vk::Format::R16G16_SINT,
            gltf::image::Format::R16G16B16 => ash::vk::Format::R16G16B16_SINT,
            gltf::image::Format::R16G16B16A16 => ash::vk::Format::R16G16B16A16_SINT,
            gltf::image::Format::R8 => ash::vk::Format::R8_SINT,
            gltf::image::Format::R8G8 => ash::vk::Format::R8G8_SINT,
            gltf::image::Format::R8G8B8 => ash::vk::Format::R8G8B8_SINT,
            gltf::image::Format::R8G8B8A8 => ash::vk::Format::R8G8B8A8_SINT,
        }
    }
}

pub struct MeshPushConstants {
    pub mvp: Mat4,
}

impl_u8_slice!(MeshPushConstants);
