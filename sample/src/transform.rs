use glam::{Quat, Vec3};

#[derive(Default, Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3,
    pub quaternion: Quat,
}
