use crate::enums::EOperationType;
use ash::{vk::Queue, Device};

#[derive(Clone, Copy, Default, Debug)]
pub struct VQueueFamilyIndices {
    pub compute: u32,
    pub graphics: u32,
    pub present: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct VQueues {
    pub compute: Queue,
    pub graphics: Queue,
    pub present: Queue,
}

impl VQueues {
    pub fn new(device: &Device, queue_family_indices: VQueueFamilyIndices) -> Self {
        let compute = unsafe { device.get_device_queue(queue_family_indices.compute, 0) };
        let graphics = unsafe { device.get_device_queue(queue_family_indices.graphics, 0) };
        let present = unsafe { device.get_device_queue(queue_family_indices.present, 0) };
        Self {
            compute,
            graphics,
            present,
        }
    }

    pub fn get(&self, operation_type: EOperationType) -> Queue {
        match operation_type {
            EOperationType::Compute => self.compute,
            EOperationType::Graphics => self.graphics,
            EOperationType::Present => self.present,
        }
    }
}
