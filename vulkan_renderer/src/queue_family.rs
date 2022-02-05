use crate::enums::EOperationType;
use ash::{vk::Queue, Device};

#[derive(Debug, Clone, Copy)]
pub struct VQueueFamilyIndices {
    pub compute: u32,
    pub graphics: u32,
    pub present: u32,
}

impl Default for VQueueFamilyIndices {
    fn default() -> Self {
        Self {
            compute: u32::MAX,
            graphics: u32::MAX,
            present: u32::MAX,
        }
    }
}

impl VQueueFamilyIndices {
    pub fn get(&self, operation_type: EOperationType) -> u32 {
        match operation_type {
            EOperationType::Compute => self.compute,
            EOperationType::Graphics => self.graphics,
            EOperationType::Present => self.present,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct VQueues {
    pub compute: Queue,
    pub graphics: Queue,
    pub present: Queue,
}

impl VQueues {
    pub fn new(device: &Device, queue_family_indices: VQueueFamilyIndices) -> Self {
        let mut queues = Self::default();
        if queue_family_indices.compute != u32::MAX {
            queues.compute = unsafe { device.get_device_queue(queue_family_indices.compute, 0) };
        }
        queues.graphics = unsafe { device.get_device_queue(queue_family_indices.graphics, 0) };
        if queue_family_indices.graphics == queue_family_indices.present {
            queues.present = queues.graphics;
        } else {
            queues.present = unsafe { device.get_device_queue(queue_family_indices.present, 0) };
        }
        queues
    }

    pub fn get(&self, operation_type: EOperationType) -> Queue {
        match operation_type {
            EOperationType::Compute => self.compute,
            EOperationType::Graphics => self.graphics,
            EOperationType::Present => self.present,
        }
    }
}
