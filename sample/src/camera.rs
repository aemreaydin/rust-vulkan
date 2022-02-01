use glam::{Mat4, Vec3};

#[derive(Default, Debug, Clone, Copy)]
pub struct CameraData {
    pub view: Mat4,
    pub projection: Mat4,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vec3,
    pub camera_data: CameraData,
}
