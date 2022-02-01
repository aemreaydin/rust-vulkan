use crate::{
    camera::{Camera, CameraData},
    frame_data::FrameData,
    macros::U8Slice,
    mesh::{Mesh, MeshPushConstants},
    model::Model,
};
use ash::vk::{PipelineBindPoint, PipelineLayout, ShaderStageFlags};
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use vulkan_renderer::device::VDevice;

#[derive(Default, Clone)]
pub struct Scene {
    pub camera: Camera,
    pub meshes: HashMap<String, Mesh>,
    pub models: Vec<Model>,
}

impl Scene {
    pub fn new(camera: Camera, meshes: HashMap<String, Mesh>) -> Self {
        Self {
            camera,
            meshes,
            ..Default::default()
        }
    }

    pub fn add_models(&mut self, mut models: Vec<Model>) {
        self.models.append(&mut models);
    }

    pub fn get_mesh(&self, model: &Model) -> Option<&Mesh> {
        self.meshes.get(&model.mesh_uuid)
    }

    pub fn update_models(&mut self, delta_time: f64) {
        for model in &mut self.models {
            model.transform.rotation.y += delta_time as f32;
        }
    }

    pub fn draw(&self, device: &VDevice, pipeline_layout: PipelineLayout, frame_data: &FrameData) {
        for model in &self.models {
            let mesh = if let Some(mesh) = self.get_mesh(model) {
                mesh
            } else {
                eprintln!("Failed to find the mesh for the model {}.", model.mesh_uuid);
                continue;
            };

            device.bind_vertex_buffer(
                frame_data.command_buffer,
                &[mesh.vertex_buffer.buffer()],
                &[0],
            );
            device.bind_index_buffer(frame_data.command_buffer, mesh.index_buffer.buffer(), 0);

            // Camera and Model
            let view = Mat4::look_at_rh(
                self.camera.position,
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            );
            // let view = Mat4::from_translation(camera);
            let mut projection =
                Mat4::perspective_rh(70.0f32.to_radians(), 1920.0 / 1080.0, 0.1, 100.0);
            projection.col_mut(1)[1] *= -1.0;
            let camera_data = CameraData { view, projection };

            frame_data
                .camera_buffer
                .map_memory(device, &[camera_data])
                .expect("Failed to map memory.");

            device.descriptor_sets(
                frame_data.command_buffer,
                PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                &[frame_data.desc_set],
            );

            let mvp = Mat4::from_translation(model.transform.position)
                * Mat4::from_rotation_y(model.transform.rotation.y);
            let constants = MeshPushConstants { mvp };

            device.push_constants(
                frame_data.command_buffer,
                pipeline_layout,
                ShaderStageFlags::VERTEX,
                constants.as_u8_slice(),
            );

            device.draw_indexed(frame_data.command_buffer, mesh.indices.len() as u32, 1);
        }
    }
}
