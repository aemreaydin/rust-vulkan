use ash::{vk::Queue, Device};

#[derive(Clone, Copy, Default, Debug)]
pub struct VQueueFamilyIndices {
    pub compute: u32,
    pub graphics: u32,
    pub present: u32,
}

pub enum VQueueType {
    Compute,
    Graphics,
    Present,
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

    pub fn get_queue(&self, queue_type: VQueueType) -> Queue {
        match queue_type {
            VQueueType::Compute => self.compute,
            VQueueType::Graphics => self.graphics,
            VQueueType::Present => self.present,
        }
    }
}
