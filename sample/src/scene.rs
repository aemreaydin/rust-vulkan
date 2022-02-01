use crate::{camera::Camera, mesh::Mesh, model::Model};
use std::collections::HashMap;

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

    pub fn add_mesh(&mut self, mesh: (String, Mesh)) {
        let (name, mesh) = mesh;
        if self.meshes.contains_key(&name) {
            println!("Mesh already loaded.");
            return;
        }
        self.meshes.insert(name, mesh);
    }
}
