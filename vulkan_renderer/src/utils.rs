use crate::device::VDevice;

pub fn pad_uniform_buffer_size(device: &VDevice, size: usize) -> u64 {
    let min_uniform_alignment = device
        .physical_device_properties()
        .limits
        .min_uniform_buffer_offset_alignment;
    let mut aligned_size = size as u64;
    if min_uniform_alignment > 0 {
        aligned_size = (aligned_size + min_uniform_alignment - 1) & !(min_uniform_alignment - 1);
    }
    aligned_size
}
