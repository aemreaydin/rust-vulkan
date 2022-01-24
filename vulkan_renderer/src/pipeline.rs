use crate::{device::VDevice, impl_get, RendererResult};
use ash::vk::{
    CullModeFlags, DescriptorSetLayout, FrontFace, GraphicsPipelineCreateInfo, LogicOp, Pipeline,
    PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineDepthStencilStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout,
    PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo,
    PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo,
    PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode,
    PrimitiveTopology, PushConstantRange, Rect2D, RenderPass, SampleCountFlags, ShaderModule,
    ShaderStageFlags, VertexInputAttributeDescription, VertexInputBindingDescription, Viewport,
};
use std::ffi::CStr;

#[derive(Debug, Clone, Copy, Default)]
pub struct VGraphicsPipeline {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
}

impl_get!(VGraphicsPipeline, pipeline, Pipeline);
impl_get!(VGraphicsPipeline, pipeline_layout, PipelineLayout);

#[derive(Default)]
pub struct VGraphicsPipelineBuilder {
    shader_stages: Vec<PipelineShaderStageCreateInfo>,
    input_assembly: PipelineInputAssemblyStateCreateInfo,
    vertex_input: PipelineVertexInputStateCreateInfo,
    rasterization: PipelineRasterizationStateCreateInfo,
    color_blend_state: PipelineColorBlendStateCreateInfo,
    multisample: PipelineMultisampleStateCreateInfo,
    pipeline_layout_create_info: PipelineLayoutCreateInfo,
    viewport: PipelineViewportStateCreateInfo,
}

impl VGraphicsPipelineBuilder {
    pub fn start() -> Self {
        Self {
            input_assembly: Self::input_assembly_create_info(PrimitiveTopology::TRIANGLE_LIST),
            vertex_input: Self::vertex_input_create_info(&[], &[]),
            rasterization: Self::rasterization_create_info(CullModeFlags::NONE, PolygonMode::FILL),
            color_blend_state: Self::color_blend_state_create_info(&[]),
            multisample: Self::multisample_create_info(),
            pipeline_layout_create_info: Self::pipeline_layout_create_info(&[], &[]),
            ..Default::default()
        }
    }

    pub fn build(&self, device: &VDevice) -> RendererResult<VGraphicsPipeline> {
        let pipeline_layout = unsafe {
            device
                .get()
                .create_pipeline_layout(&self.pipeline_layout_create_info, None)?
        };
        let create_infos = &[Self::graphics_pipeline_create_info(
            self,
            pipeline_layout,
            device.render_pass(),
        )];
        let pipelines_result = unsafe {
            device
                .get()
                .create_graphics_pipelines(PipelineCache::null(), create_infos, None)
        };
        match pipelines_result {
            Ok(pipelines) => Ok(VGraphicsPipeline {
                pipeline: pipelines[0],
                pipeline_layout,
            }),
            Err((_, err)) => Err(Box::new(err)),
        }
    }

    fn graphics_pipeline_create_info(
        &self,
        layout: PipelineLayout,
        render_pass: RenderPass,
    ) -> GraphicsPipelineCreateInfo {
        GraphicsPipelineCreateInfo {
            stage_count: self.shader_stages.len() as u32,
            p_stages: self.shader_stages.as_ptr(),
            p_vertex_input_state: &self.vertex_input,
            p_input_assembly_state: &self.input_assembly,
            p_viewport_state: &self.viewport,
            p_rasterization_state: &self.rasterization,
            p_multisample_state: &self.multisample,
            // p_depth_stencil_state: &self.depth_stencil_create_info(),
            p_color_blend_state: &self.color_blend_state,
            layout,
            render_pass,
            subpass: 0,
            ..Default::default()
        }
    }

    /// Must be called
    pub fn shader_stages(mut self, shader_infos: &[(ShaderStageFlags, ShaderModule)]) -> Self {
        self.shader_stages = shader_infos
            .iter()
            .map(|&(stage, module)| Self::shader_stage_create_info(stage, module))
            .collect();
        self
    }

    pub fn input_assembly(mut self, topology: PrimitiveTopology) -> Self {
        self.input_assembly = Self::input_assembly_create_info(topology);
        self
    }

    pub fn vertex_input(
        mut self,
        vertex_binding_descriptions: &[VertexInputBindingDescription],
        vertex_attribute_descriptions: &[VertexInputAttributeDescription],
    ) -> Self {
        self.vertex_input = Self::vertex_input_create_info(
            vertex_binding_descriptions,
            vertex_attribute_descriptions,
        );
        self
    }

    pub fn rasterization(mut self, cull_mode: CullModeFlags, polygon_mode: PolygonMode) -> Self {
        self.rasterization = Self::rasterization_create_info(cull_mode, polygon_mode);
        self
    }

    pub fn color_blend_state(mut self, attachments: &[PipelineColorBlendAttachmentState]) -> Self {
        self.color_blend_state = Self::color_blend_state_create_info(attachments);
        self
    }

    // Add multisampling
    pub fn multisample(mut self) -> Self {
        self.multisample = Self::multisample_create_info();
        self
    }

    pub fn pipeline_layout(
        mut self,
        descriptor_sets: &[DescriptorSetLayout],
        push_constants: &[PushConstantRange],
    ) -> Self {
        self.pipeline_layout_create_info =
            Self::pipeline_layout_create_info(descriptor_sets, push_constants);
        self
    }

    pub fn viewport(mut self, viewports: &[Viewport], scissors: &[Rect2D]) -> Self {
        self.viewport = Self::viewport_create_info(viewports, scissors);
        self
    }

    fn shader_stage_create_info(
        stage: ShaderStageFlags,
        module: ShaderModule,
    ) -> PipelineShaderStageCreateInfo {
        PipelineShaderStageCreateInfo {
            stage,
            module,
            p_name: CStr::from_bytes_with_nul(b"main\0")
                .expect("Module name not null-terminated.")
                .as_ptr(),
            ..Default::default()
        }
    }

    fn input_assembly_create_info(
        topology: PrimitiveTopology,
    ) -> PipelineInputAssemblyStateCreateInfo {
        PipelineInputAssemblyStateCreateInfo {
            topology,
            ..Default::default()
        }
    }

    fn vertex_input_create_info(
        vertex_binding_descriptions: &[VertexInputBindingDescription],
        vertex_attribute_descriptions: &[VertexInputAttributeDescription],
    ) -> PipelineVertexInputStateCreateInfo {
        PipelineVertexInputStateCreateInfo {
            vertex_attribute_description_count: vertex_attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: vertex_attribute_descriptions.as_ptr(),
            vertex_binding_description_count: vertex_binding_descriptions.len() as u32,
            p_vertex_binding_descriptions: vertex_binding_descriptions.as_ptr(),
            ..Default::default()
        }
    }

    fn rasterization_create_info(
        cull_mode: CullModeFlags,
        polygon_mode: PolygonMode,
    ) -> PipelineRasterizationStateCreateInfo {
        PipelineRasterizationStateCreateInfo {
            line_width: 1.0,
            cull_mode,
            polygon_mode,
            front_face: FrontFace::CLOCKWISE,
            ..Default::default()
        }
    }

    fn multisample_create_info() -> PipelineMultisampleStateCreateInfo {
        PipelineMultisampleStateCreateInfo {
            rasterization_samples: SampleCountFlags::TYPE_1,
            min_sample_shading: 1.0,
            ..Default::default()
        }
    }

    fn pipeline_layout_create_info(
        descriptor_sets: &[DescriptorSetLayout],
        push_constants: &[PushConstantRange],
    ) -> PipelineLayoutCreateInfo {
        PipelineLayoutCreateInfo {
            set_layout_count: descriptor_sets.len() as u32,
            p_set_layouts: descriptor_sets.as_ptr(),
            push_constant_range_count: push_constants.len() as u32,
            p_push_constant_ranges: push_constants.as_ptr(),
            ..Default::default()
        }
    }

    fn viewport_create_info(
        viewports: &[Viewport],
        scissors: &[Rect2D],
    ) -> PipelineViewportStateCreateInfo {
        PipelineViewportStateCreateInfo {
            viewport_count: viewports.len() as u32,
            p_viewports: viewports.as_ptr(),
            scissor_count: scissors.len() as u32,
            p_scissors: scissors.as_ptr(),
            ..Default::default()
        }
    }

    pub fn color_blend_state_create_info(
        attachments: &[PipelineColorBlendAttachmentState],
    ) -> PipelineColorBlendStateCreateInfo {
        PipelineColorBlendStateCreateInfo {
            logic_op: LogicOp::COPY,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    fn depth_stencil_create_info(&self) -> PipelineDepthStencilStateCreateInfo {
        PipelineDepthStencilStateCreateInfo {
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::{
        device::VDevice,
        instance::VInstance,
        physical_device::VPhysicalDevice,
        primitives::{mesh::MeshPushConstants, vertex::Vertex},
        shader_utils::VShaderUtils,
        surface::VSurface,
        RendererResult,
    };
    use ash::vk::{
        ColorComponentFlags, Handle, PipelineColorBlendAttachmentState, PushConstantRange, Rect2D,
        ShaderStageFlags, Viewport,
    };
    use winit::platform::windows::EventLoopExtWindows;

    use super::VGraphicsPipelineBuilder;

    #[test]
    fn creates_pipeline() -> RendererResult<()> {
        let instance = VInstance::new("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface = VSurface::new(&instance, &EventLoopExtWindows::new_any_thread())?;
            let physical_device = VPhysicalDevice::new(&instance, &surface)?;
            let device = VDevice::new(&instance, &physical_device)?;

            let vertex_code = VShaderUtils::load_shader("../sample/shaders/base.vert.spv")
                .expect("Failed to load vertex shader code.");
            let vertex_shader_module = VShaderUtils::create_shader_module(&device, &vertex_code)
                .expect("Failed to create vertex shader module.");
            let fragment_code = VShaderUtils::load_shader("../sample/shaders/base.frag.spv")
                .expect("Failed to load fragment shader code.");
            let fragment_shader_module =
                VShaderUtils::create_shader_module(&device, &fragment_code)
                    .expect("Failed to create fragment shader module.");
            let builder = VGraphicsPipelineBuilder::start();
            let shader_infos = &[
                (ShaderStageFlags::VERTEX, vertex_shader_module),
                (ShaderStageFlags::FRAGMENT, fragment_shader_module),
            ];
            let viewports = &[Viewport {
                x: 0.0,
                y: 0.0,
                max_depth: 1.0,
                min_depth: 0.0,
                height: surface.extent_2d().height as f32,
                width: surface.extent_2d().width as f32,
            }];
            let scissors = &[Rect2D {
                extent: surface.extent_2d(),
                ..Default::default()
            }];
            let color_blend_attachments = &[PipelineColorBlendAttachmentState {
                color_write_mask: ColorComponentFlags::RGBA,
                ..Default::default()
            }];
            let vertex_input_desc = Vertex::vertex_description();
            let push_constants = &[PushConstantRange {
                stage_flags: ShaderStageFlags::VERTEX,
                size: size_of::<MeshPushConstants>() as u32,
                offset: 0,
            }];
            let builder = builder
                .shader_stages(shader_infos)
                .vertex_input(&vertex_input_desc.bindings, &vertex_input_desc.attributes)
                .viewport(viewports, scissors)
                .color_blend_state(color_blend_attachments)
                .pipeline_layout(&[], push_constants);
            let pipeline = builder
                .build(&device)
                .expect("Failed to create graphics pipeline.");
            assert_ne!(pipeline.pipeline.as_raw(), 0);
            assert_ne!(pipeline.pipeline_layout.as_raw(), 0);
        }
        Ok(())
    }
}
