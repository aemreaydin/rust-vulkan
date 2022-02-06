use crate::{
    camera::{Camera, CameraData},
    frame_data::FrameData,
    macros::U8Slice,
    mesh::{Mesh, MeshPushConstants},
    model::Model,
};
use ash::vk::{PipelineBindPoint, PipelineLayout, ShaderStageFlags};
use glam::{Mat4, Vec3, Vec4};
use std::{collections::HashMap, mem::size_of};
use vulkan_renderer::{buffer::VBuffer, cmd::*, device::VDevice, utils::pad_uniform_buffer_size};

#[derive(Default, Debug, Clone, Copy)]
pub struct SceneData {
    pub fog_color: Vec4,
    pub fog_distance: Vec4,
    pub ambient_color: Vec4,
    pub sunlight_direction: Vec4,
    pub sunlight_color: Vec4,
}

#[derive(Default, Clone)]
pub struct Scene {
    pub camera: Camera,
    pub meshes: HashMap<String, Mesh>,
    pub models: Vec<Model>,

    pub scene_data: SceneData,
    pub scene_buffer: VBuffer,
}

impl Scene {
    pub fn new(
        camera: Camera,
        scene_data: SceneData,
        scene_buffer: VBuffer,
        meshes: HashMap<String, Mesh>,
    ) -> Self {
        Self {
            camera,
            meshes,
            scene_data,
            scene_buffer,
            ..Default::default()
        }
    }

    pub fn add_models(&mut self, mut models: Vec<Model>) {
        self.models.append(&mut models);
    }

    pub fn get_mesh(&self, model: &Model) -> Option<&Mesh> {
        self.meshes.get(&model.mesh_uuid)
    }

    pub fn draw(&self, device: &VDevice, pipeline_layout: PipelineLayout, frame_data: &FrameData) {
        for model in &self.models {
            let mesh = if let Some(mesh) = self.get_mesh(model) {
                mesh
            } else {
                eprintln!("Failed to find the mesh for the model {}.", model.mesh_uuid);
                continue;
            };

            cmd_bind_vertex_buffer(
                device,
                frame_data.command_buffer,
                &[mesh.vertex_buffer.buffer()],
                &[0],
            );
            cmd_bind_index_buffer(
                device,
                frame_data.command_buffer,
                mesh.index_buffer.buffer(),
                0,
            );

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

            self.scene_buffer
                .map_memory(device, &[self.scene_data])
                .expect("Failed to map memory.");

            let dynamic_offsets =
                &[
                    pad_uniform_buffer_size(device, size_of::<SceneData>() * frame_data.frame_index)
                        as u32,
                ];
            cmd_bind_descriptor_sets(
                device,
                frame_data.command_buffer,
                PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                &[frame_data.desc_set],
                dynamic_offsets,
            );

            let mvp = Mat4::from_translation(model.transform.position)
                * Mat4::from_rotation_y(model.transform.rotation.y);
            let constants = MeshPushConstants { mvp };

            cmd_push_constants(
                device,
                frame_data.command_buffer,
                pipeline_layout,
                ShaderStageFlags::VERTEX,
                constants.as_u8_slice(),
            );

            cmd_draw_indexed(
                device,
                frame_data.command_buffer,
                mesh.indices.len() as u32,
                1,
            );
        }
    }
}
