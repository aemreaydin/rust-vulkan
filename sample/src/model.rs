use crate::transform::Transform;

#[derive(Default, Debug, Clone)]
pub struct Model {
    pub mesh_uuid: String,
    pub transform: Transform,
}
