use crate::transform::Transform;

#[derive(Default, Debug, Clone)]
pub struct Model {
    pub mesh_id: String,
    pub transform: Transform,
}
