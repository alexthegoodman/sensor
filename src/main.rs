use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};

use common_vector::basic::{rgb_to_wgpu, Point, WindowSize};
use common_vector::dot::draw_dot;
use common_vector::editor::{self, Editor, Viewport};
use common_vector::guideline::create_guide_line_buffers;
use common_vector::polygon::{Polygon, PolygonConfig};
use common_vector::vertex::Vertex;
use floem::event::EventListener;
use floem::kurbo::Size;
use floem::peniko::Color;
use floem::reactive::create_signal;
use floem::views::label;
use floem::window::WindowConfig;
use floem_renderer::gpu_resources::{self, GpuResources};
use floem_winit::event::{ElementState, MouseButton};
// use winit::{event_loop, window};
use wgpu::util::DeviceExt;

use floem::context::PaintState;
// use floem::floem_reactive::SignalGet;
use floem::reactive::{SignalGet, SignalUpdate};
use floem::views::text;
use floem::views::Decorators;
use floem::views::{h_stack, svg, v_stack};
use floem::WindowHandle;
use floem::{
    views::{button, dropdown},
    IntoView,
};
use floem::{Application, CustomRenderCallback};

// Define an enum for our dropdown options
#[derive(Clone, PartialEq, Debug)]
enum DropdownOption {
    Option1,
    Option2,
    Option3,
}

impl std::fmt::Display for DropdownOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DropdownOption::Option1 => write!(f, "Option 1"),
            DropdownOption::Option2 => write!(f, "Option 2"),
            DropdownOption::Option3 => write!(f, "Option 3"),
        }
    }
}

struct Handler {
    button_handler: RefCell<Option<Box<dyn Fn(MutexGuard<'_, Editor>) + Send + 'static>>>,
}

impl Handler {
    pub fn new() -> Self {
        Handler {
            button_handler: RefCell::new(None),
        }
    }
    pub fn set_button_handler(
        &mut self,
        gpu_resources: Arc<GpuResources>,
        window_size: WindowSize,
        polygon_config: PolygonConfig,
    ) {
        let handler = Box::new(move |mut editor: MutexGuard<'_, Editor>| {
            println!("Button clicked, attempting to add polygon...");
            editor.polygons.push(Polygon::new(
                &window_size,
                &gpu_resources.device,
                polygon_config.points.clone(),
                polygon_config.dimensions,
                polygon_config.position,
                polygon_config.border_radius,
            ));
            println!("Polygon added successfully.");
        });
        self.button_handler.replace(Some(handler));
    }

    pub fn handle_button_click(&mut self, editor: MutexGuard<'_, Editor>) {
        // Step 1: Check if the handler exists and clone it if it does
        let handle = self.button_handler.borrow();
        let handler_option = handle.as_ref();

        // Step 2: If we have a handler, call it
        if let Some(handler) = handler_option {
            handler(editor);
        } else {
            println!("Button handler not set.");
        }
    }
}

fn app_view(
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_cloned: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    mut handler: std::sync::Arc<Mutex<Handler>>,
    mut square_handler: std::sync::Arc<Mutex<Handler>>,
) -> impl IntoView {
    let (counter, mut set_counter) = create_signal(0);
    let (selected_option, set_selected_option) = create_signal(DropdownOption::Option1);

    println!("selected_option {:?}", selected_option.get());

    (
        label(move || format!("Value: {counter}")).style(|s| s.margin_bottom(10)),
        (
            styled_button("Increment", "plus", move || set_counter += 1),
            styled_button("Decrement", "minus", move || set_counter -= 1),
            styled_button("Add Polygon", "plus", move || {
                let mut editor = editor.lock().unwrap();
                let mut handler = handler.lock().unwrap();
                println!("Handle click...");

                // if let Some(handle_click) = &editor.handle_button_click(editor) {
                //     println!("Handling click...");
                //     handle_click(editor);
                // }
                handler.handle_button_click(editor);
            }),
            styled_button("Add Square", "plus", move || {
                let mut editor_cloned = editor_cloned.lock().unwrap();
                let mut square_handler = square_handler.lock().unwrap();
                println!("Handle square...");

                // if let Some(handle_click) = &editor.handle_button_click(editor) {
                //     println!("Handling click...");
                //     handle_click(editor);
                // }
                square_handler.handle_button_click(editor_cloned);
            }),
        )
            .style(|s| s.flex_col().gap(10).margin_top(10)),
        dropdown::dropdown(
            // Active item (currently selected option)
            move || {
                let see = selected_option.get();
                println!("see {:?}", see);
                see
            },
            // Main view (what's always visible)
            |option: DropdownOption| Box::new(label(move || format!("Selected: {}", option))),
            // Iterator of all options
            vec![
                DropdownOption::Option1,
                DropdownOption::Option2,
                DropdownOption::Option3,
            ],
            // List item view (how each option in the dropdown is displayed)
            // move |option: DropdownOption| {
            //     let option_clone = option.clone();
            //     Box::new(button(option.to_string()).action(move || {
            //         println!("DropdownOption {:?}", option_clone.clone());
            //         set_selected_option.set(option_clone.clone());
            //     }))
            // },
            move |m| text(m.to_string()).into_any(),
        )
        .on_accept(move |new| set_selected_option.set(new)),
    )
        .style(|s| s.flex_col().items_center())
}

fn create_icon(name: &str) -> String {
    match name {
        "plus" => r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path fill="none" d="M0 0h24v24H0z"/><path d="M11 11V5h2v6h6v2h-6v6h-2v-6H5v-2z"/></svg>"#.to_string(),
        "minus" => r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path fill="none" d="M0 0h24v24H0z"/><path d="M5 11h14v2H5z"/></svg>"#.to_string(),
        _ => "".to_string(),
    }
}

fn styled_button(
    text: &'static str,
    icon_name: &'static str,
    action: impl FnMut() + 'static,
) -> impl IntoView {
    // button(text)
    button(v_stack((
        svg(create_icon(icon_name)).style(|s| s.width(24).height(24)),
        label(move || text),
    )))
    .action(action)
    .style(|s| {
        s.width(70)
            .height(70)
            .border_radius(15)
            .box_shadow_blur(15)
            .box_shadow_spread(4)
            .box_shadow_color(Color::rgba(0.0, 0.0, 0.0, 0.16))
            // .transition("all 0.2s")
            .hover(|s| s.box_shadow_color(Color::rgba(0.0, 0.0, 0.0, 0.32)))
    })
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

                // println!("Redraw");
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
                                // load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                load: wgpu::LoadOp::Load,
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
                    // render_pass.set_scissor_rect(100, 100, 200, 200);

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

                    // println!("Render size {:?}", window_size);

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
                        // println!(
                        //     "Rendering guideline {:?} {:?} {:?} {:?}",
                        //     guide_line.start.x,
                        //     guide_line.start.y,
                        //     guide_line.end.x,
                        //     guide_line.end.y
                        // );
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

fn handle_cursor_moved(
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    window_size: WindowSize,
) -> Option<Box<dyn Fn(f64, f64)>> {
    Some(Box::new(move |positionX: f64, positionY: f64| {
        let mut editor = editor.lock().unwrap();
        editor.handle_mouse_move(
            &window_size,
            &gpu_resources.device,
            positionX as f32,
            positionY as f32,
        );
    }))
}

fn handle_mouse_input(
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    window_size: WindowSize,
) -> Option<Box<dyn Fn(MouseButton, ElementState)>> {
    Some(Box::new(move |button, state| {
        let mut editor = editor.lock().unwrap();
        if button == MouseButton::Left {
            match state {
                ElementState::Pressed => editor.handle_mouse_down(
                    // mouse_position.0,
                    // mouse_position.1,
                    &window_size,
                    &gpu_resources.device,
                ),
                ElementState::Released => editor.handle_mouse_up(),
            }
        }
    }))
}

// fn handle_button_click(
//     editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
//     gpu_resources: std::sync::Arc<GpuResources>,
//     window_size: WindowSize,
// ) -> Option<Box<dyn Fn()>> {
//     Some(Box::new(move || {
//         println!("Button clicked, attempting to add polygon...");
//         if let Ok(mut editor) = editor.lock() {
//             println!("Editor locked successfully. Adding polygon...");
//             editor.polygons.push(Polygon::new(
//                 &window_size,
//                 &gpu_resources.device,
//                 vec![
//                     Point { x: 0.0, y: 0.0 },
//                     Point { x: 1.0, y: 0.0 },
//                     Point { x: 0.5, y: 1.0 },
//                 ],
//                 (100.0, 100.0),
//                 Point { x: 50.0, y: 50.0 },
//                 5.0,
//             ));
//             println!("Polygon added successfully.");
//         } else {
//             println!("Failed to lock the editor.");
//         }
//     }))
// }

// fn handle_button_click(
//     editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
//     gpu_resources: &Arc<GpuResources>,
//     window_size: &WindowSize,
// ) {
//     let editor = editor.clone();

//     std::thread::spawn(move || {
//         println!("Button clicked, attempting to add polygon...");
//         match editor.lock() {
//             Ok(mut guard) => {
//                 println!("Editor locked successfully. Adding polygon...");
//                 guard.polygons.push(Polygon::new(
//                     window_size,
//                     &gpu_resources.device,
//                     vec![
//                         Point { x: 0.0, y: 0.0 },
//                         Point { x: 1.0, y: 0.0 },
//                         Point { x: 0.5, y: 1.0 },
//                     ],
//                     (100.0, 100.0),
//                     Point { x: 50.0, y: 50.0 },
//                     5.0,
//                 ));
//                 println!("Polygon added successfully.");
//             }
//             Err(e) => println!("Failed to lock the editor: {:?}", e),
//         }
//     });
// }

fn main() {
    let window_size = WindowSize {
        width: 800,
        height: 500,
    };

    let viewport = Viewport::new(window_size.width as f32, window_size.height as f32); // Or whatever your window size is
    let mut editor = Arc::new(Mutex::new(Editor::new(viewport)));
    let mut handler = Arc::new(Mutex::new(Handler::new()));
    let mut square_handler = Arc::new(Mutex::new(Handler::new()));

    let cloned_handler = Arc::clone(&handler);
    let cloned_square_handler = Arc::clone(&square_handler);
    let cloned_square_handler6 = Arc::clone(&square_handler);

    let cloned = Arc::clone(&editor);
    let cloned2 = Arc::clone(&editor);
    let cloned3 = Arc::clone(&editor);
    let cloned4 = Arc::clone(&editor);
    let cloned5 = Arc::clone(&editor);
    let cloned6 = Arc::clone(&editor);

    let app = Application::new();
    let (mut app, window_id) = app.window(
        move |_| {
            app_view(
                Arc::clone(&editor),
                cloned6,
                handler,
                cloned_square_handler6,
            )
        },
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

        // let mut editor = editor.lock().unwrap();

        window_handle.user_editor = Some(cloned);

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

                // let device = std::sync::Arc::new(gpu_resources.device);

                window_handle.handle_cursor_moved =
                    handle_cursor_moved(cloned2.clone(), gpu_resources.clone(), window_size);
                window_handle.handle_mouse_input =
                    handle_mouse_input(cloned3.clone(), gpu_resources.clone(), window_size);

                // window_handle
                //     .id
                //     .add_event_listener(EventListener::Click, move |_| {
                //         // Your action here
                //         println!("Window clicked!");
                //         // If you need to update state or trigger other actions, do it here
                //         // For example:
                //         // state.update(|s| s.clicked = true);
                //     });

                let editor_clone = cloned4.clone();

                // test items
                let mut editor = cloned5.lock().unwrap();

                // editor.handle_button_click =
                //     handle_button_click(editor_clone, gpu_resources.clone(), window_size);
                let mut cloned_handler = cloned_handler.lock().unwrap();
                cloned_handler.set_button_handler(
                    Arc::clone(&gpu_resources),
                    window_size,
                    PolygonConfig {
                        points: vec![
                            Point { x: 0.0, y: 0.0 },
                            Point { x: 1.0, y: 0.0 },
                            Point { x: 0.5, y: 1.0 },
                        ],
                        dimensions: (100.0, 100.0),
                        position: Point { x: 100.0, y: 100.0 },
                        border_radius: 5.0,
                    },
                );

                let mut cloned_square_handler = cloned_square_handler.lock().unwrap();
                cloned_square_handler.set_button_handler(
                    Arc::clone(&gpu_resources),
                    window_size,
                    PolygonConfig {
                        points: vec![
                            Point { x: 0.0, y: 0.0 },
                            Point { x: 1.0, y: 0.0 },
                            Point { x: 1.0, y: 1.0 },
                            Point { x: 0.0, y: 1.0 },
                        ],
                        dimensions: (100.0, 100.0),
                        position: Point { x: 100.0, y: 100.0 },
                        border_radius: 5.0,
                    },
                );

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
