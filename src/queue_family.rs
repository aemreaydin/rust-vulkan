#[derive(Default, Debug, Clone, Copy)]
pub struct VQueueFamilyIndices {
    pub compute: u32,
    pub graphics: u32,
    pub present: u32,
}
