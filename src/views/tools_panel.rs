use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};

use bytemuck::Contiguous;
use common_vector::basic::{
    color_to_wgpu, rgb_to_wgpu, string_to_f32, wgpu_to_hex, Point, WindowSize,
};
use common_vector::dot::draw_dot;
use common_vector::editor::{self, Editor, Viewport};
use common_vector::guideline::create_guide_line_buffers;
use common_vector::polygon::{Polygon, PolygonConfig};
use common_vector::vertex::Vertex;
use floem::event::{Event, EventListener, EventPropagation};
use floem::keyboard::{Key, KeyCode, NamedKey};
use floem::kurbo::Size;
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::style::{Background, CursorStyle, Transition};
use floem::taffy::AlignItems;
use floem::text::Weight;
use floem::views::editor::view;
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    VirtualDirection, VirtualItemSize,
};
use floem::window::WindowConfig;
use floem_renderer::gpu_resources::{self, GpuResources};
use floem_winit::dpi::{LogicalSize, PhysicalSize};
use floem_winit::event::{ElementState, MouseButton};
use uuid::Uuid;
// use views::buttons::{nav_button, option_button, small_button};
// use winit::{event_loop, window};
use wgpu::util::DeviceExt;

use floem::context::PaintState;
// use floem::floem_reactive::SignalGet;
use floem::reactive::{SignalGet, SignalUpdate};
use floem::views::text;
use floem::views::Decorators;
use floem::views::{h_stack, svg, v_stack};
use floem::{
    views::{button, dropdown},
    IntoView,
};
use floem::{Application, CustomRenderCallback};
use floem::{GpuHelper, View, WindowHandle};

use super::buttons::option_button;

pub fn tools_view(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    // editor_cloned: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    viewport: Arc<Mutex<Viewport>>,
    // mut handler: std::sync::Arc<Mutex<Handler>>,
    // mut square_handler: std::sync::Arc<Mutex<Handler>>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let gpu_cloned = Arc::clone(&gpu_helper);
    let viewport_cloned = Arc::clone(&viewport);

    (
        // label(move || format!("Tools")).style(|s| s.margin_bottom(10)),
        container((
            option_button(
                "Add Polygon",
                "plus",
                Some(move || {
                    let mut editor = editor.lock().unwrap();
                    // let mut handler = handler.lock().unwrap();
                    println!("Handle click...");

                    // handler.handle_button_click(editor);

                    let polygon_config = PolygonConfig {
                        points: vec![
                            Point { x: 0.0, y: 0.0 },
                            Point { x: 1.0, y: 0.0 },
                            Point { x: 0.5, y: 1.0 },
                        ],
                        dimensions: (100.0, 100.0),
                        position: Point { x: 600.0, y: 100.0 },
                        border_radius: 5.0,
                        fill: [1.0, 1.0, 1.0, 1.0],
                    };
                    let gpu_helper = gpu_helper.lock().unwrap();
                    let device = &gpu_helper
                        .gpu_resources
                        .as_ref()
                        .expect("Couldn't get gpu resources")
                        .device;
                    let viewport = viewport.lock().unwrap();
                    let window_size = WindowSize {
                        width: viewport.width as u32,
                        height: viewport.height as u32,
                    };
                    editor.polygons.push(Polygon::new(
                        &window_size,
                        &device,
                        polygon_config.points.clone(),
                        polygon_config.dimensions,
                        polygon_config.position,
                        polygon_config.border_radius,
                        polygon_config.fill,
                    ));
                }),
                false,
            )
            .style(|s| s.margin_right(5.0)),
            option_button(
                "Add Square",
                "plus",
                Some(move || {
                    let mut editor = editor_cloned.lock().unwrap();
                    // let mut square_handler = square_handler.lock().unwrap();
                    println!("Handle square...");

                    // square_handler.handle_button_click(editor_cloned);

                    let polygon_config = PolygonConfig {
                        points: vec![
                            Point { x: 0.0, y: 0.0 },
                            Point { x: 1.0, y: 0.0 },
                            Point { x: 1.0, y: 1.0 },
                            Point { x: 0.0, y: 1.0 },
                        ],
                        dimensions: (100.0, 100.0),
                        position: Point { x: 600.0, y: 100.0 },
                        border_radius: 5.0,
                        fill: [1.0, 1.0, 1.0, 1.0],
                    };
                    let gpu_helper = gpu_cloned.lock().unwrap();
                    let device = &gpu_helper
                        .gpu_resources
                        .as_ref()
                        .expect("Couldn't get gpu resources")
                        .device;
                    let viewport = viewport_cloned.lock().unwrap();
                    let window_size = WindowSize {
                        width: viewport.width as u32,
                        height: viewport.height as u32,
                    };
                    editor.polygons.push(Polygon::new(
                        &window_size,
                        &device,
                        polygon_config.points.clone(),
                        polygon_config.dimensions,
                        polygon_config.position,
                        polygon_config.border_radius,
                        polygon_config.fill,
                    ));
                }),
                false,
            ),
        ))
        .style(|s| s.padding_vert(15.0).z_index(1))
    )
}
