use ash::vk::{
    ClearColorValue, ClearDepthStencilValue, ClearValue, ColorComponentFlags, DescriptorType,
    Extent2D, Extent3D, Format, ImageAspectFlags, ImageUsageFlags, MemoryPropertyFlags,
    PipelineBindPoint, PipelineColorBlendAttachmentState, PipelineStageFlags, PushConstantRange,
    Rect2D, ShaderStageFlags, Viewport,
};
use camera::Camera;
use frame_data::FrameData;
use glam::Vec3;
use mesh::{Mesh, MeshPushConstants};
use model::Model;
use scene::{Scene, SceneData};
use std::{collections::HashMap, mem::size_of};
use transform::Transform;
use vertex::Vertex;
use vulkan_renderer::{
    buffer::VBuffer,
    cmd::*,
    descriptorset::{VDescriptorPool, VDescriptorSetLayout},
    device::VDevice,
    enums::EOperationType,
    framebuffer::VFramebuffers,
    image::VImage,
    instance::VInstance,
    pipeline::VGraphicsPipelineBuilder,
    shader_utils::VShaderUtils,
    swapchain::VSwapchain,
    utils::pad_uniform_buffer_size,
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod camera;
mod frame_data;
mod macros;
mod mesh;
mod model;
mod scene;
mod transform;
mod vertex;

const NUM_FRAMES: usize = 3;

fn main() {
    // Window and Event Loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Renderer")
        .with_inner_size(PhysicalSize::new(1920, 1080))
        .build(&event_loop)
        .expect("Failed to create window.");
    let extent = Extent2D {
        width: window.inner_size().width,
        height: window.inner_size().height,
    };

    // Instance, Device and Swapchain
    let instance = VInstance::new("Sample", 0).expect("Failed to create instance.");
    let device = VDevice::new(&instance, &window).expect("Failed to create device.");
    let swapchain =
        VSwapchain::new(&instance, &device, extent).expect("Failed to create swapchain.");

    // Shader Vars
    let vertex_code = VShaderUtils::load_shader("sample/shaders/base.vert.spv")
        .expect("Failed to load vertex shader code.");
    let vertex_shader_module = VShaderUtils::create_shader_module(&device, &vertex_code)
        .expect("Failed to create vertex shader module.");
    let fragment_code = VShaderUtils::load_shader("sample/shaders/base.frag.spv")
        .expect("Failed to load fragment shader code.");
    let fragment_shader_module = VShaderUtils::create_shader_module(&device, &fragment_code)
        .expect("Failed to create fragment shader module.");

    let depth_format = Format::D32_SFLOAT;
    let depth_extent = Extent3D {
        width: extent.width,
        height: extent.height,
        depth: 1,
    };
    let depth_image = VImage::new(
        &device,
        ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        depth_format,
        depth_extent,
        ImageAspectFlags::DEPTH,
    )
    .expect("Failed to create depth image.");

    // Framebuffer and command buffer
    let framebuffers = VFramebuffers::new(&device, &swapchain, depth_image.image_view(), extent)
        .expect("Failed to create framebuffers.");

    // Descriptor Set
    let bindings = &[
        VDescriptorSetLayout::layout_binding(
            0,
            1,
            DescriptorType::UNIFORM_BUFFER,
            ShaderStageFlags::VERTEX,
        ),
        VDescriptorSetLayout::layout_binding(
            1,
            1,
            DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
        ),
    ];
    let descriptor_pool = VDescriptorPool::new(&device).expect("Failed to create descriptor pool.");
    let descriptor_set_layout = VDescriptorSetLayout::new(&device, bindings)
        .expect("Failed to create descriptor set layout.");

    // Graphics Pipeline
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
        height: extent.height as f32,
        width: extent.width as f32,
    }];
    let scissors = &[Rect2D {
        extent,
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
    let descriptor_set_layouts = &[descriptor_set_layout.get()];
    let builder = builder
        .shader_stages(shader_infos)
        .vertex_input(&vertex_input_desc.bindings, &vertex_input_desc.attributes)
        .viewport(viewports, scissors)
        .color_blend_state(color_blend_attachments)
        .pipeline_layout(descriptor_set_layouts, push_constants);
    let pipeline = builder
        .build(&device, swapchain.get_renderpass())
        .expect("Failed to create graphics pipeline.");

    // Frame Data
    let scene_buffer_size =
        NUM_FRAMES as u64 * pad_uniform_buffer_size(&device, size_of::<SceneData>());
    let scene_buffer = VBuffer::new_uniform_buffer(
        &device,
        scene_buffer_size,
        MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
    )
    .expect("Failed to create scene buffer.");
    let frame_datas = (0..NUM_FRAMES)
        .map(|frame_ind| {
            FrameData::new(
                &device,
                device.get_queue_family_index(EOperationType::Graphics),
                descriptor_pool.get(),
                &[descriptor_set_layout.get()],
                scene_buffer,
                frame_ind,
            )
            .expect("Failed to create FrameData.")
        })
        .collect::<Vec<_>>();

    // SCENE DATA
    let camera = Camera {
        position: Vec3::new(0.0, 0.0, -5.0),
        ..Default::default()
    };
    let meshes = HashMap::from_iter([(
        "Helmet".to_owned(),
        Mesh::from_file(&device, "sample/assets/damaged_helmet/damaged_helmet.glb")
            .expect("Failed to load model."),
    )]);

    let mut scene = Scene::new(camera, SceneData::default(), scene_buffer, meshes);
    scene.add_models(vec![
        Model {
            mesh_uuid: "Helmet".to_owned(),
            transform: Transform {
                position: Vec3::new(-2.0, 0.0, 0.0),
                ..Default::default()
            },
        },
        Model {
            mesh_uuid: "Helmet".to_owned(),
            transform: Transform {
                position: Vec3::new(2.0, 0.0, 0.0),
                ..Default::default()
            },
        },
    ]);

    // let mut current_time = Instant::now();
    let mut frame_count = 0;
    event_loop.run(move |event, _, control_flow| {
        // let new_time = Instant::now();
        // let delta_time = (new_time - current_time).as_millis() as f64 / 1.0e3;
        // current_time = new_time;

        let frame_index = frame_count % NUM_FRAMES;
        let frame_data = &frame_datas[frame_index];

        let fences = &[frame_data.fence.get()];
        device
            .wait_for_fences(fences, 1_000_000_000)
            .expect("Failed to wait for fences.");
        device
            .reset_fences(fences)
            .expect("Failed to reset fences.");

        let (image_ind, _is_suboptimal) = swapchain
            .acquire_next_image(
                1_000_000_000,
                Some(frame_data.present_semaphore.get()),
                None,
            )
            .expect("Failed to acquire next image.");

        let flash = (frame_count as f32 / 1200.0).sin().abs();

        // Begin Rendering

        begin_command_buffer(&device, frame_data.command_buffer)
            .expect("Failed to begin command buffer.");

        let clear_values = &[
            ClearValue {
                color: ClearColorValue {
                    float32: [0.0, 0.0, flash, 1.0],
                },
            },
            ClearValue {
                depth_stencil: ClearDepthStencilValue {
                    depth: 1.0,
                    ..Default::default()
                },
            },
        ];
        cmd_begin_render_pass(
            &device,
            frame_data.command_buffer,
            swapchain.get_renderpass(),
            framebuffers[image_ind],
            clear_values,
            extent,
        );

        cmd_bind_pipeline(
            &device,
            frame_data.command_buffer,
            PipelineBindPoint::GRAPHICS,
            pipeline.pipeline(),
        );

        scene.update_scene(frame_count as f32);

        scene_buffer
            .map_padded_memory(
                &device,
                &[scene.scene_data],
                (frame_index as u64 * pad_uniform_buffer_size(&device, size_of::<SceneData>()))
                    as isize,
            )
            .expect("Failed to map padded memory.");

        scene.draw(&device, pipeline.pipeline_layout(), frame_data);

        cmd_end_render_pass(&device, frame_data.command_buffer);
        end_command_buffer(&device, frame_data.command_buffer)
            .expect("Failed to end command buffer.");

        let command_buffers = &[frame_data.command_buffer];
        let wait_semaphores = &[frame_data.present_semaphore.get()];
        let dst_semaphores = &[frame_data.render_semaphore.get()];
        let pipeline_stage_flags = &[PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = VDevice::create_queue_submit_info(
            command_buffers,
            wait_semaphores,
            dst_semaphores,
            pipeline_stage_flags,
        );

        device
            .queue_submit(
                device.get_queue(EOperationType::Graphics),
                &[submit_info],
                frame_data.fence.get(),
            )
            .expect("Failed to submit queue.");

        let image_indices = &[image_ind];
        let swapchains = &[swapchain.get_swapchain_khr()];
        let wait_semaphores = &[frame_data.render_semaphore.get()];
        let present_info =
            VSwapchain::create_present_info(image_indices, swapchains, wait_semaphores);
        swapchain
            .queue_present(device.get_queue(EOperationType::Graphics), present_info)
            .expect("Failed to present queue.");

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {}
            _ => (),
        }
        frame_count += 1;
    });
}
