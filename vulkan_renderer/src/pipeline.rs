use crate::{device::VDevice, impl_get, RendererResult};
use ash::vk::{
    CompareOp, CullModeFlags, DescriptorSetLayout, FrontFace, GraphicsPipelineCreateInfo, LogicOp,
    Pipeline, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
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
    depth_stencil_create_info: PipelineDepthStencilStateCreateInfo,
    viewport: PipelineViewportStateCreateInfo,
}

impl VGraphicsPipelineBuilder {
    pub fn start() -> Self {
        Self {
            input_assembly: Self::input_assembly_create_info(PrimitiveTopology::TRIANGLE_LIST),
            vertex_input: Self::vertex_input_create_info(&[], &[]),
            rasterization: Self::rasterization_create_info(CullModeFlags::BACK, PolygonMode::FILL),
            color_blend_state: Self::color_blend_state_create_info(&[]),
            multisample: Self::multisample_create_info(),
            pipeline_layout_create_info: Self::pipeline_layout_create_info(&[], &[]),
            depth_stencil_create_info: Self::depth_stencil_create_info(),
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
            p_depth_stencil_state: &self.depth_stencil_create_info,
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
        descriptor_set_layouts: &[DescriptorSetLayout],
        push_constants: &[PushConstantRange],
    ) -> Self {
        self.pipeline_layout_create_info =
            Self::pipeline_layout_create_info(descriptor_set_layouts, push_constants);
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
            front_face: FrontFace::COUNTER_CLOCKWISE,
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

    fn depth_stencil_create_info() -> PipelineDepthStencilStateCreateInfo {
        PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: CompareOp::LESS_OR_EQUAL,
            min_depth_bounds: 0.0,
            max_depth_bounds: 1.0,
            ..Default::default()
        }
    }
}
