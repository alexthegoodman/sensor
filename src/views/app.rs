use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};

use bytemuck::Contiguous;
use common_vector::basic::{
    color_to_wgpu, rgb_to_wgpu, string_to_f32, wgpu_to_human, Point, WindowSize,
};
use common_vector::dot::draw_dot;
use common_vector::editor::{self, Editor, Viewport};
use common_vector::guideline::create_guide_line_buffers;
use common_vector::polygon::{Polygon, PolygonConfig, Stroke};
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

use crate::editor_state::EditorState;
use crate::PolygonClickHandler;

use super::aside::tab_interface;
use super::properties_panel::properties_view;

pub fn app_view(
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    // editor_cloned: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    // editor_cloned2: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    // editor_cloned3: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    // editor_cloned4: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    // mut handler: std::sync::Arc<Mutex<Handler>>,
    // mut square_handler: std::sync::Arc<Mutex<Handler>>,
) -> impl IntoView {
    let editor_cloned = Arc::clone(&editor);
    let editor_cloned2 = Arc::clone(&editor);
    let editor_cloned3 = Arc::clone(&editor);
    let editor_cloned4 = Arc::clone(&editor);

    // // let (counter, mut set_counter) = create_signal(0);
    // let (polygon_selected, mut set_polygon_selected) = create_signal(false);
    // let (selected_polygon_id, mut set_selected_polygon_id) = create_signal(Uuid::nil());
    let polygon_selected = create_rw_signal(false);
    let selected_polygon_id = create_rw_signal(Uuid::nil());
    let selected_polygon_data = create_rw_signal(PolygonConfig {
        id: Uuid::nil(),
        name: String::new(),
        points: Vec::new(),
        dimensions: (100.0, 100.0),
        position: Point { x: 0.0, y: 0.0 },
        border_radius: 0.0,
        fill: [0.0, 0.0, 0.0, 1.0],
        stroke: Stroke {
            fill: [1.0, 1.0, 1.0, 1.0],
            thickness: 2.0,
        },
    });

    // Create a RefCell to hold the set_counter function
    // let set_counter_ref = Arc::new(Mutex::new(set_counter));
    let polygon_selected_ref = Arc::new(Mutex::new(polygon_selected));
    let selected_polygon_id_ref = Arc::new(Mutex::new(selected_polygon_id));
    let selected_polygon_data_ref = Arc::new(Mutex::new(selected_polygon_data));

    let editor_cloned2 = editor_cloned2.clone();

    // Create the handle_polygon_click function
    let handle_polygon_click: Arc<PolygonClickHandler> = Arc::new({
        let editor_state = editor_state.clone();
        // let set_counter_ref = Arc::clone(&set_counter_ref);
        let polygon_selected_ref = Arc::clone(&polygon_selected_ref);
        let selected_polygon_id_ref = Arc::clone(&selected_polygon_id_ref);
        let selected_polygon_data_ref = Arc::clone(&selected_polygon_data_ref);
        move || {
            let editor_state = editor_state.clone();
            // let set_counter_ref = set_counter_ref.clone();
            let polygon_selected_ref = polygon_selected_ref.clone();
            let selected_polygon_id_ref = selected_polygon_id_ref.clone();
            let selected_polygon_data_ref = selected_polygon_data_ref.clone();
            Some(
                Box::new(move |polygon_id: Uuid, polygon_data: PolygonConfig| {
                    // cannot lock editor here!
                    // {
                    //     let mut editor = new_editor.lock().unwrap();
                    //     // Update editor as needed
                    // }

                    if let Ok(mut polygon_selected) = polygon_selected_ref.lock() {
                        polygon_selected.update(|c| {
                            *c = true;
                        });
                    }
                    if let Ok(mut selected_polygon_id) = selected_polygon_id_ref.lock() {
                        selected_polygon_id.update(|c| {
                            *c = polygon_id;
                        });
                        let mut editor_state = editor_state.lock().unwrap();
                        editor_state.selected_polygon_id = polygon_id;
                        editor_state.polygon_selected = true;
                    }
                    if let Ok(mut selected_polygon_data) = selected_polygon_data_ref.lock() {
                        selected_polygon_data.update(|c| {
                            *c = polygon_data;
                        });
                    }
                }) as Box<dyn FnMut(Uuid, PolygonConfig)>,
            )
        }
    });

    // create_effect(move |_| {
    //     editor_cloned3.lock().unwrap().handle_polygon_click = Some(handle_polygon_click);
    // });

    // Use create_effect to set the handler only once
    create_effect({
        let handle_polygon_click = Arc::clone(&handle_polygon_click);
        let editor_cloned3 = Arc::clone(&editor_cloned3);
        move |_| {
            let mut editor = editor_cloned3.lock().unwrap();
            editor.handle_polygon_click = Some(Arc::clone(&handle_polygon_click));
        }
    });

    container((
        // label(move || format!("Value: {counter}")).style(|s| s.margin_bottom(10)),
        tab_interface(
            gpu_helper.clone(),
            editor,
            // editor_cloned,
            viewport.clone(),
            // handler,
            // square_handler,
            polygon_selected,
        ),
        dyn_container(
            move || polygon_selected.get(),
            move |polygon_selected_real| {
                if polygon_selected_real {
                    properties_view(
                        editor_state.clone(),
                        gpu_helper.clone(),
                        editor_cloned4.clone(),
                        viewport.clone(),
                        polygon_selected,
                        selected_polygon_id,
                        selected_polygon_data,
                    )
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
    ))
    // .style(|s| s.flex_col().items_center())
}
