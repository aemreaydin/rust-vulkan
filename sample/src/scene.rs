use crate::{
    camera::{Camera, CameraData},
    frame_data::FrameData,
    macros::U8Slice,
    mesh::{Mesh, MeshPushConstants},
    model::Model,
    utils::pad_uniform_buffer_size,
};
use ash::vk::{PipelineBindPoint, PipelineLayout, ShaderStageFlags};
use glam::{Mat4, Vec3, Vec4};
use std::{collections::HashMap, mem::size_of};
use vulkan_renderer::{buffer::VBuffer, device::VDevice};

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

    pub fn update_scene(&mut self, frame_number: f32) {
        // for model in &mut self.models {
        //     model.transform.rotation.y += delta_time as f32;
        // }
        self.scene_data.ambient_color = Vec4::new(
            (frame_number / 1200.0).sin(),
            0.0,
            (frame_number / 1200.0).cos(),
            1.0,
        );
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

            self.scene_buffer
                .map_memory(device, &[self.scene_data])
                .expect("Failed to map memory.");

            let dynamic_offsets = &[pad_uniform_buffer_size(
                size_of::<SceneData>() * frame_data.frame_index,
                device
                    .physical_device_properties()
                    .limits
                    .min_uniform_buffer_offset_alignment,
            ) as u32];
            device.descriptor_sets(
                frame_data.command_buffer,
                PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                &[frame_data.desc_set],
                dynamic_offsets,
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
