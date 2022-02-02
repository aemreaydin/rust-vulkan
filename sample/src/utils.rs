pub fn pad_uniform_buffer_size(size: usize, min_uniform_alignment: u64) -> u64 {
    let mut aligned_size = size as u64;
    if min_uniform_alignment > 0 {
        aligned_size = (aligned_size + min_uniform_alignment - 1) & !(min_uniform_alignment - 1);
    }
    aligned_size
}
