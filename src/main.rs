use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};

use common_vector::basic::{rgb_to_wgpu, Point, WindowSize};
use common_vector::dot::draw_dot;
use common_vector::editor::{Editor, Viewport};
use common_vector::guideline::create_guide_line_buffers;
use common_vector::polygon::Polygon;
use common_vector::vertex::Vertex;
use floem::kurbo::Size;
use floem::peniko::Color;
use floem::views::label;
use floem::window::WindowConfig;
// use winit::{event_loop, window};
use wgpu::util::DeviceExt;

use floem::context::PaintState;
use floem::views::Decorators;
use floem::{Application, CustomRenderCallback};
use floem::{IntoView, WindowHandle};
use wgpu::CommandEncoder;

fn app_view() -> impl IntoView {
    // let (counter, mut set_counter) = create_signal(0);
    // let (selected_option, set_selected_option) = create_signal(DropdownOption::Option1);

    // println!("selected_option {:?}", selected_option.get());

    (label(move || format!("Hello there!"))
        .style(|s| s.margin_bottom(10).color(Color::rgb(0.5, 0.5, 0.5))),)
        .style(|s| s.flex_col().items_center())
}

type RenderCallback<'a> = dyn for<'b> Fn(wgpu::CommandEncoder, wgpu::SurfaceTexture, wgpu::TextureView, &WindowHandle)
    + 'a;

fn create_render_callback<'a>() -> Box<RenderCallback<'a>> {
    Box::new(
        move |mut encoder: wgpu::CommandEncoder,
              frame: wgpu::SurfaceTexture,
              view: wgpu::TextureView,
              window_handle: &WindowHandle| {
            let mut handle = window_handle.borrow();
            // let mut handle = window_handle.borrow_mut();
            // let mut handle = window_handle.borrow_mut();
            // let handle = window_handle;

            // if let Some(handle) = window_handle.upgrade() {
            // let mut handle = window_handle
            //     .try_borrow_mut()
            //     .expect("Couldn' get window_handle");

            if let Some(gpu_resources) = &handle.gpu_resources {
                // Use gpu_resources here
                // println!("Using GPU resources in render callback");

                // TODO: draw buffers here

                println!("Redraw");
                // editor.draw(&mut renderer, &surface, &device);

                // // TODO: overwriting other frame?
                // let frame = gpu_resources
                //     .surface
                //     .get_current_texture()
                //     .expect("Failed to acquire next swap chain texture");
                // let view = frame
                //     .texture
                //     .create_view(&wgpu::TextureViewDescriptor::default());

                // // Update the render pass to use the new vertex and index buffers
                // let mut encoder = gpu_resources
                //     .device
                //     .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                // load: wgpu::LoadOp::Clear(wgpu::Color {
                                //     // grey background
                                //     r: 0.15,
                                //     g: 0.15,
                                //     b: 0.15,
                                //     // white background
                                //     // r: 1.0,
                                //     // g: 1.0,
                                //     // b: 1.0,
                                //     a: 1.0,
                                // }),
                                load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        // depth_stencil_attachment: None,
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &handle
                                .depth_view
                                .as_ref()
                                .expect("Couldn't fetch depth view"), // This is the depth texture view
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0), // Clear to max depth
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None, // Set this if using stencil
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    // println!("Render frame...");

                    // Render partial screen content
                    // render_pass.set_viewport(100.0, 100.0, 200.0, 200.0, 0.0, 1.0);
                    render_pass.set_scissor_rect(100, 100, 200, 200);

                    render_pass.set_pipeline(
                        &handle
                            .render_pipeline
                            .as_ref()
                            .expect("Couldn't fetch render pipeline"),
                    );

                    let editor = handle
                        .user_editor
                        .as_ref()
                        .expect("Couldn't get user editor")
                        .lock()
                        .unwrap();

                    for (poly_index, polygon) in editor.polygons.iter().enumerate() {
                        // println!("Indices length {:?}", polygon.indices.len());
                        render_pass.set_vertex_buffer(0, polygon.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            polygon.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..polygon.indices.len() as u32, 0, 0..1);
                    }

                    // let window_size = WindowSize {
                    //     width: handle
                    //         .window
                    //         .as_ref()
                    //         .expect("Couldn't get window")
                    //         .inner_size()
                    //         .width,
                    //     height: handle
                    //         .window
                    //         .as_ref()
                    //         .expect("Couldn't get window")
                    //         .inner_size()
                    //         .height,
                    // };
                    let window_size = &handle
                        .window_size
                        .as_ref()
                        .expect("Couldn't get window size");

                    println!("Render size {:?}", window_size);

                    if let Some(edge_point) = editor.hover_point {
                        let (vertices, indices, vertex_buffer, index_buffer) = draw_dot(
                            &gpu_resources.device,
                            &window_size,
                            edge_point.point,
                            rgb_to_wgpu(47, 131, 222, 1.0),
                        ); // Green dot

                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                    }

                    // Draw guide lines
                    for guide_line in &editor.guide_lines {
                        println!(
                            "Rendering guideline {:?} {:?} {:?} {:?}",
                            guide_line.start.x,
                            guide_line.start.y,
                            guide_line.end.x,
                            guide_line.end.y
                        );
                        let (vertices, indices, vertex_buffer, index_buffer) =
                            create_guide_line_buffers(
                                &gpu_resources.device,
                                &window_size,
                                guide_line.start,
                                guide_line.end,
                                rgb_to_wgpu(47, 131, 222, 1.0), // Blue color for guide lines
                            );

                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                    }
                }

                // gpu_resources.queue.submit(Some(encoder.finish()));
                // frame.present();

                // implement here?
                let command_buffer = encoder.finish();
                gpu_resources.queue.submit(Some(command_buffer));
                gpu_resources.device.poll(wgpu::Maintain::Wait);
                frame.present();

                // supposed to fall in line with OS refresh rate (?)
                // std::thread::sleep(std::time::Duration::from_millis(2000));
                // window_handle
                //     .get_mut()
                //     .window
                //     .expect("Couldn't get the window")
                //     .request_redraw();
            } else {
                println!("GPU resources not available yet");
            }
            // }
        },
    )
}

fn main() {
    let window_size = WindowSize {
        width: 800,
        height: 500,
    };

    let app = Application::new();
    let (mut app, window_id) = app.window(
        move |_| app_view(),
        // Some(WindowConfig {
        //     size: Some(Size::new(
        //         window_size.width as f64,
        //         window_size.height as f64,
        //     )),
        //     ..Default::default()
        // }),
        Some(
            WindowConfig::default()
                .size(Size::new(
                    window_size.width as f64,
                    window_size.height as f64,
                ))
                .title("CommonOS Sensor"),
        ),
    );

    let window_id = window_id.expect("Couldn't get window id");

    {
        let app_handle = app.handle.as_mut().expect("Couldn't get handle");
        let window_handle = app_handle
            .window_handles
            .get_mut(&window_id)
            .expect("Couldn't get window handle");
        // let window_handle = window_handle.borrow_mut();

        // let window_handle_rc = Rc::new(RefCell::new(window_handle));
        // let window_handle_weak = Rc::downgrade(&window_handle_rc);
        // let window_handle: Arc<Mutex<WindowHandle>> = Arc::new(Mutex::new(window_handle));

        // let window_handle_rc = Rc::new(RefCell::new(window_handle));

        // Create and set the render callback
        let render_callback = create_render_callback();

        // window_handle.set_render_callback(render_callback);
        window_handle.set_encode_callback(render_callback);
        window_handle.window_size = Some(window_size);

        println!("Ready...");

        let viewport = Viewport::new(window_size.width as f32, window_size.height as f32); // Or whatever your window size is
        let mut editor = Arc::new(Mutex::new(Editor::new(viewport)));

        window_handle.user_editor = Some(Arc::clone(&editor));

        // *window_handle.user_editor.borrow_mut() = Some(Arc::clone(&editor));

        // Receive and store GPU resources
        match &mut window_handle.paint_state {
            PaintState::PendingGpuResources { rx, .. } => {
                let gpu_resources = Arc::new(rx.recv().unwrap().unwrap());

                println!("Initializing pipeline...");

                let sampler = gpu_resources
                    .device
                    .create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                    });

                let depth_texture = gpu_resources
                    .device
                    .create_texture(&wgpu::TextureDescriptor {
                        size: wgpu::Extent3d {
                            width: window_size.width.clone(),
                            height: window_size.height.clone(),
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth24Plus,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        label: Some("Depth Texture"),
                        view_formats: &[],
                    });

                let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

                let depth_stencil_state = wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                };

                // Define the layouts
                let pipeline_layout =
                    gpu_resources
                        .device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("Pipeline Layout"),
                            // bind_group_layouts: &[&bind_group_layout],
                            bind_group_layouts: &[], // No bind group layouts
                            push_constant_ranges: &[],
                        });

                // Load the shaders
                let shader_module_vert_primary =
                    gpu_resources
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("Primary Vert Shader"),
                            source: wgpu::ShaderSource::Wgsl(
                                include_str!("shaders/vert_primary.wgsl").into(),
                            ),
                        });

                let shader_module_frag_primary =
                    gpu_resources
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("Primary Frag Shader"),
                            source: wgpu::ShaderSource::Wgsl(
                                include_str!("shaders/frag_primary.wgsl").into(),
                            ),
                        });

                // let swapchain_capabilities = gpu_resources
                //     .surface
                //     .get_capabilities(&gpu_resources.adapter);
                // let swapchain_format = swapchain_capabilities.formats[0]; // Choosing the first available format
                let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb; // hardcode for now

                // Configure the render pipeline
                let render_pipeline =
                    gpu_resources
                        .device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some("Common Vector Primary Render Pipeline"),
                            layout: Some(&pipeline_layout),
                            multiview: None,
                            cache: None,
                            vertex: wgpu::VertexState {
                                module: &shader_module_vert_primary,
                                entry_point: "vs_main", // name of the entry point in your vertex shader
                                buffers: &[Vertex::desc()], // Make sure your Vertex::desc() matches your vertex structure
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &shader_module_frag_primary,
                                entry_point: "fs_main", // name of the entry point in your fragment shader
                                targets: &[Some(wgpu::ColorTargetState {
                                    format: swapchain_format,
                                    // blend: Some(wgpu::BlendState::REPLACE),
                                    blend: Some(wgpu::BlendState {
                                        color: wgpu::BlendComponent {
                                            src_factor: wgpu::BlendFactor::SrcAlpha,
                                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                            operation: wgpu::BlendOperation::Add,
                                        },
                                        alpha: wgpu::BlendComponent {
                                            src_factor: wgpu::BlendFactor::One,
                                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                            operation: wgpu::BlendOperation::Add,
                                        },
                                    }),
                                    write_mask: wgpu::ColorWrites::ALL,
                                })],
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            }),
                            // primitive: wgpu::PrimitiveState::default(),
                            // depth_stencil: None,
                            // multisample: wgpu::MultisampleState::default(),
                            primitive: wgpu::PrimitiveState {
                                conservative: false,
                                topology: wgpu::PrimitiveTopology::TriangleList, // how vertices are assembled into geometric primitives
                                // strip_index_format: Some(wgpu::IndexFormat::Uint32),
                                strip_index_format: None,
                                front_face: wgpu::FrontFace::Ccw, // Counter-clockwise is considered the front face
                                // none cull_mode
                                cull_mode: None,
                                polygon_mode: wgpu::PolygonMode::Fill,
                                // Other properties such as conservative rasterization can be set here
                                unclipped_depth: false,
                            },
                            depth_stencil: Some(depth_stencil_state), // Optional, only if you are using depth testing
                            multisample: wgpu::MultisampleState {
                                count: 1,
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                        });

                window_handle.render_pipeline = Some(render_pipeline);
                window_handle.depth_view = Some(depth_view);

                println!("Initialized...");

                let mut mouse_position = (0.0, 0.0);

                // test items
                let mut editor = editor.lock().unwrap();

                // Create a triangle
                editor.polygons.push(Polygon::new(
                    &window_size,
                    &gpu_resources.device,
                    vec![
                        Point { x: 0.0, y: 0.0 },
                        Point { x: 1.0, y: 0.0 },
                        Point { x: 0.5, y: 1.0 },
                    ],
                    (100.0, 100.0),
                    Point { x: 100.0, y: 100.0 },
                    5.0, // border radius
                ));

                // Create a rectangle
                editor.polygons.push(Polygon::new(
                    &window_size,
                    &gpu_resources.device,
                    vec![
                        Point { x: 0.0, y: 0.0 },
                        Point { x: 1.0, y: 0.0 },
                        Point { x: 1.0, y: 1.0 },
                        Point { x: 0.0, y: 1.0 },
                    ],
                    (150.0, 100.0),
                    Point { x: 300.0, y: 200.0 },
                    10.0, // border radius
                ));

                // Create a pentagon
                editor.polygons.push(Polygon::new(
                    &window_size,
                    &gpu_resources.device,
                    vec![
                        Point { x: 0.5, y: 0.0 },
                        Point { x: 1.0, y: 0.4 },
                        Point { x: 0.8, y: 1.0 },
                        Point { x: 0.2, y: 1.0 },
                        Point { x: 0.0, y: 0.4 },
                    ],
                    (120.0, 120.0),
                    Point { x: 500.0, y: 300.0 },
                    8.0, // border radius
                ));

                // editor.polygons[0].update_data_from_dimensions(&window_size, &device, (200.0, 50.0));

                window_handle.gpu_resources = Some(gpu_resources);
            }
            PaintState::Initialized { .. } => {
                println!("Renderer is already initialized");
            }
        }
    }

    app.run();
}
