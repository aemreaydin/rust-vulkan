use ash::vk::{
    Format, VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate,
};
use glam::{Vec2, Vec3};
use memoffset::offset_of;
use std::mem::size_of;

#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

pub struct VVertexInputDescription {
    pub attributes: Vec<VertexInputAttributeDescription>,
    pub bindings: Vec<VertexInputBindingDescription>,
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self {
            position,
            normal,
            uv,
        }
    }

    pub fn vertex_description() -> VVertexInputDescription {
        let binding_desc = VertexInputBindingDescription {
            binding: 0,
            input_rate: VertexInputRate::VERTEX,
            stride: size_of::<Vertex>() as u32,
        };

        let position_attribute_desc = VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: Format::R32G32B32_SFLOAT,
            offset: offset_of!(Vertex, position) as u32,
        };

        let normal_attribute_desc = VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: Format::R32G32B32_SFLOAT,
            offset: offset_of!(Vertex, normal) as u32,
        };

        let uv_attribute_desc = VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: Format::R32G32_SFLOAT,
            offset: offset_of!(Vertex, uv) as u32,
        };

        let bindings = vec![binding_desc];
        let attributes = vec![
            position_attribute_desc,
            normal_attribute_desc,
            uv_attribute_desc,
        ];
        VVertexInputDescription {
            attributes,
            bindings,
        }
    }
}
