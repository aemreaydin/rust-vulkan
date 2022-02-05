use crate::{device::VDevice, RendererResult};
use ash::vk::{
    Buffer, ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandBufferUsageFlags, CommandPool, DescriptorSet, DeviceSize, Extent2D,
    Framebuffer, IndexType, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, Rect2D,
    RenderPass, RenderPassBeginInfo, ShaderStageFlags, SubpassContents,
};

pub fn allocate_command_buffers(
    device: &VDevice,
    command_pool: CommandPool,
    command_buffer_count: u32,
) -> RendererResult<Vec<CommandBuffer>> {
    let command_buffer_allocate_info = CommandBufferAllocateInfo {
        command_buffer_count,
        level: CommandBufferLevel::PRIMARY,
        command_pool,
        ..Default::default()
    };

    unsafe {
        Ok(device
            .get()
            .allocate_command_buffers(&command_buffer_allocate_info)?)
    }
}

pub fn begin_command_buffer(device: &VDevice, command_buffer: CommandBuffer) -> RendererResult<()> {
    let begin_info = CommandBufferBeginInfo {
        flags: CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
    };
    unsafe {
        device
            .get()
            .begin_command_buffer(command_buffer, &begin_info)?
    }
    Ok(())
}

pub fn end_command_buffer(device: &VDevice, command_buffer: CommandBuffer) -> RendererResult<()> {
    unsafe { device.get().end_command_buffer(command_buffer)? };
    Ok(())
}

pub fn cmd_begin_render_pass(
    device: &VDevice,
    command_buffer: CommandBuffer,
    render_pass: RenderPass,
    framebuffer: Framebuffer,
    clear_values: &[ClearValue],
    extent: Extent2D,
) {
    let render_pass_begin_info = RenderPassBeginInfo {
        clear_value_count: clear_values.len() as u32,
        p_clear_values: clear_values.as_ptr(),
        render_pass,
        framebuffer,
        render_area: Rect2D {
            offset: Offset2D { x: 0, y: 0 },
            extent,
        },
        ..Default::default()
    };
    unsafe {
        device.get().cmd_begin_render_pass(
            command_buffer,
            &render_pass_begin_info,
            SubpassContents::INLINE,
        );
    }
}

pub fn cmd_bind_pipeline(
    device: &VDevice,
    command_buffer: CommandBuffer,
    bind_point: PipelineBindPoint,
    pipeline: Pipeline,
) {
    unsafe {
        device
            .get()
            .cmd_bind_pipeline(command_buffer, bind_point, pipeline);
    };
}

pub fn cmd_bind_vertex_buffer(
    device: &VDevice,
    command_buffer: CommandBuffer,
    buffers: &[Buffer],
    offsets: &[DeviceSize],
) {
    unsafe {
        device
            .get()
            .cmd_bind_vertex_buffers(command_buffer, 0, buffers, offsets);
    }
}

pub fn cmd_bind_index_buffer(
    device: &VDevice,
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
) {
    unsafe {
        device
            .get()
            .cmd_bind_index_buffer(command_buffer, buffer, offset, IndexType::UINT32);
    }
}

pub fn cmd_push_constants(
    device: &VDevice,
    command_buffer: CommandBuffer,
    layout: PipelineLayout,
    stage_flags: ShaderStageFlags,
    constants: &[u8],
) {
    unsafe {
        device
            .get()
            .cmd_push_constants(command_buffer, layout, stage_flags, 0, constants);
    }
}

pub fn cmd_bind_descriptor_sets(
    device: &VDevice,
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    layout: PipelineLayout,
    descriptor_sets: &[DescriptorSet],
    dynamic_offsets: &[u32],
) {
    unsafe {
        device.get().cmd_bind_descriptor_sets(
            command_buffer,
            pipeline_bind_point,
            layout,
            0,
            descriptor_sets,
            dynamic_offsets,
        );
    }
}

pub fn cmd_draw(
    device: &VDevice,
    command_buffer: CommandBuffer,
    vertex_count: u32,
    instance_count: u32,
) {
    unsafe {
        device
            .get()
            .cmd_draw(command_buffer, vertex_count, instance_count, 0, 0);
    }
}

pub fn cmd_draw_indexed(
    device: &VDevice,
    command_buffer: CommandBuffer,
    index_count: u32,
    instance_count: u32,
) {
    unsafe {
        device
            .get()
            .cmd_draw_indexed(command_buffer, index_count, instance_count, 0, 0, 0);
    }
}

pub fn cmd_end_render_pass(device: &VDevice, command_buffer: CommandBuffer) {
    unsafe { device.get().cmd_end_render_pass(command_buffer) }
}
