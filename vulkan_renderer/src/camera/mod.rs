use glam::Mat4;

#[derive(Default, Clone, Copy)]
pub struct VCameraData {
    pub view: Mat4,
    pub projection: Mat4,
}

pub struct VCamera {}
