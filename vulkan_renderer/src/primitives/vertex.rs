use crate::impl_get;
use ash::vk::{
    Format, VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate,
};
use memoffset::offset_of;
use nalgebra_glm::Vec4;
use std::mem::size_of;

#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    position: Vec4,
    color: Vec4,
    normal: Vec4,
}

pub struct VVertexInputDescription {
    pub attributes: Vec<VertexInputAttributeDescription>,
    pub bindings: Vec<VertexInputBindingDescription>,
}

impl Vertex {
    pub fn new(position: Vec4, color: Vec4, normal: Vec4) -> Self {
        Self {
            position,
            color,
            normal,
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
            format: Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(Vertex, position) as u32,
        };

        let color_attribute_desc = VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(Vertex, color) as u32,
        };

        let normal_attribute_desc = VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(Vertex, normal) as u32,
        };

        let bindings = vec![binding_desc];
        let attributes = vec![
            position_attribute_desc,
            color_attribute_desc,
            normal_attribute_desc,
        ];
        VVertexInputDescription {
            attributes,
            bindings,
        }
    }
}

impl_get!(Vertex, position, Vec4);
impl_get!(Vertex, color, Vec4);
impl_get!(Vertex, normal, Vec4);
