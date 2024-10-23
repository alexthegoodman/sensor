use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fmt::Display;
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
use floem::peniko::{Brush, Color};
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::style::{Background, CursorStyle, Transition};
use floem::taffy::{
    AlignItems, GridTrackRepetition, LengthPercentage, MaxTrackSizingFunction, MinMax,
    MinTrackSizingFunction, TrackSizingFunction,
};
use floem::text::Weight;
use floem::views::editor::view;
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    RadioButton, StackExt, VirtualDirection, VirtualItemSize,
};
use floem::window::WindowConfig;
use floem_renderer::gpu_resources::{self, GpuResources};
use floem_winit::dpi::{LogicalSize, PhysicalSize};
use floem_winit::event::{ElementState, MouseButton};
use strum_macros::EnumIter;
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

use super::aside;
use super::buttons::small_button;
use super::inputs::styled_input;

pub fn properties_view(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    polygon_selected: RwSignal<bool>,
    selected_polygon_id: RwSignal<Uuid>,
    selected_polygon_data: RwSignal<PolygonConfig>,
) -> impl IntoView {
    // let polygon_data = selected_polygon_data.read();

    let editor_cloned = Arc::clone(&editor);
    let editor_cloned2 = Arc::clone(&editor);
    let editor_cloned3 = Arc::clone(&editor);
    let editor_cloned4 = Arc::clone(&editor);
    let editor_cloned5 = Arc::clone(&editor);
    let editor_cloned6 = Arc::clone(&editor);
    let editor_cloned7 = Arc::clone(&editor);

    let cloned_helper = Arc::clone(&gpu_helper);
    let cloned_helper2 = Arc::clone(&gpu_helper);
    let cloned_helper3 = Arc::clone(&gpu_helper);
    let cloned_helper4 = Arc::clone(&gpu_helper);
    let cloned_helper5 = Arc::clone(&gpu_helper);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    v_stack((
        h_stack((
            small_button(
                "",
                "arrow-left",
                {
                    move |_| {
                        println!("Click back!");
                        // this action runs on_click_stop so should stop propagation
                        polygon_selected.update(|v| {
                            *v = false;
                        });
                        selected_polygon_id.update(|v| {
                            *v = Uuid::nil();
                        });
                    }
                },
                false,
            )
            .style(|s| s.margin_right(7.0)),
            label(|| "Properties").style(|s| s.font_size(24.0).font_weight(Weight::THIN)),
        ))
        .style(|s| s.margin_bottom(12.0)),
        h_stack((
            styled_input(
                "Width:".to_string(),
                &selected_polygon_data
                    .read()
                    .borrow()
                    .dimensions
                    .0
                    .to_string(),
                "Enter width",
                Box::new({
                    move |value| {
                        let selected_id = selected_polygon_id.get();
                        let width = string_to_f32(&value).expect("Couldn't convert string");

                        let mut editor = editor_cloned7.lock().unwrap();
                        let mut gpu_helper = cloned_helper.lock().unwrap();

                        // First iteration: find the index of the selected polygon
                        let polygon_index =
                            editor.polygons.iter().position(|p| p.id == selected_id);

                        if let Some(index) = polygon_index {
                            println!("Found selected polygon with ID: {}", selected_id);

                            // Get the necessary data from editor
                            let viewport_width = editor.viewport.lock().unwrap().width;
                            let viewport_height = editor.viewport.lock().unwrap().height;
                            let device = &gpu_helper
                                .gpu_resources
                                .as_ref()
                                .expect("Couldn't get gpu resources")
                                .device;

                            let window_size = WindowSize {
                                width: viewport_width as u32,
                                height: viewport_height as u32,
                            };

                            let camera = editor.camera.expect("Couldn't get camera");

                            // Second iteration: update the selected polygon
                            if let Some(selected_polygon) = editor.polygons.get_mut(index) {
                                selected_polygon.update_data_from_dimensions(
                                    &window_size,
                                    &device,
                                    (width, selected_polygon.dimensions.1),
                                    &camera,
                                );
                            }
                        } else {
                            println!("No polygon found with the selected ID: {}", selected_id);
                        }
                    }
                }),
            )
            .style(move |s| s.width(halfs).margin_right(5.0)),
            styled_input(
                "Height:".to_string(),
                &selected_polygon_data
                    .read()
                    .borrow()
                    .dimensions
                    .1
                    .to_string(),
                "Enter height",
                Box::new({
                    move |value| {
                        let selected_id = selected_polygon_id.get();
                        let height = string_to_f32(&value).expect("Couldn't convert string");

                        let mut editor = editor_cloned6.lock().unwrap();
                        let mut gpu_helper = gpu_helper.lock().unwrap();

                        // First iteration: find the index of the selected polygon
                        let polygon_index =
                            editor.polygons.iter().position(|p| p.id == selected_id);

                        if let Some(index) = polygon_index {
                            println!("Found selected polygon with ID: {}", selected_id);

                            // Get the necessary data from editor
                            let viewport_width = editor.viewport.lock().unwrap().width;
                            let viewport_height = editor.viewport.lock().unwrap().height;
                            let device = &gpu_helper
                                .gpu_resources
                                .as_ref()
                                .expect("Couldn't get gpu resources")
                                .device;

                            let window_size = WindowSize {
                                width: viewport_width as u32,
                                height: viewport_height as u32,
                            };

                            let camera = editor.camera.expect("Couldn't get camera");

                            // Second iteration: update the selected polygon
                            if let Some(selected_polygon) = editor.polygons.get_mut(index) {
                                selected_polygon.update_data_from_dimensions(
                                    &window_size,
                                    &device,
                                    (selected_polygon.dimensions.0, height),
                                    &camera,
                                );
                            }
                        } else {
                            println!("No polygon found with the selected ID: {}", selected_id);
                        }
                    }
                }),
            )
            .style(move |s| s.width(halfs)),
        ))
        .style(move |s| {
            s.width(aside_width)
            // .display(Display::Grid)
            // .grid_template_columns(vec![TrackSizingFunction::Repeat(
            //     floem::taffy::GridTrackRepetition::Count(2),
            //     vec![MinMax::from(MinMax {
            //         min: MinTrackSizingFunction::Fixed(LengthPercentage::Percent(100.0 / 2.0)),
            //         max: MaxTrackSizingFunction::Fixed(LengthPercentage::Percent(100.0 / 2.0)),
            //     })],
            // )])
        }),
        h_stack((
            styled_input(
                "Red:".to_string(),
                &wgpu_to_hex(selected_polygon_data.read().borrow().fill[0]).to_string(),
                "0-255",
                Box::new({
                    move |value| {
                        let selected_id = selected_polygon_id.get();
                        let mut editor = editor_cloned2.lock().unwrap();
                        let mut gpu_helper = cloned_helper3.lock().unwrap();

                        let polygon_index =
                            editor.polygons.iter().position(|p| p.id == selected_id);

                        if let Some(index) = polygon_index {
                            // Now you have the selected polygon
                            println!("Found selected polygon with ID: {}", selected_id);
                            // You can now work with the selected_polygon

                            // Get the necessary data from editor
                            let viewport_width = editor.viewport.lock().unwrap().width;
                            let viewport_height = editor.viewport.lock().unwrap().height;
                            let device = &gpu_helper
                                .gpu_resources
                                .as_ref()
                                .expect("Couldn't get gpu resources")
                                .device;

                            let window_size = WindowSize {
                                width: viewport_width as u32,
                                height: viewport_height as u32,
                            };

                            let camera = editor.camera.expect("Couldn't get camera");

                            // Second iteration: update the selected polygon
                            if let Some(selected_polygon) = editor.polygons.get_mut(index) {
                                selected_polygon.update_data_from_fill(
                                    &window_size,
                                    &device,
                                    [
                                        color_to_wgpu(
                                            string_to_f32(&value).expect("Couldn't convert string"),
                                        ),
                                        selected_polygon.fill[1],
                                        selected_polygon.fill[2],
                                        selected_polygon.fill[3],
                                    ],
                                    &camera,
                                );
                            }
                        } else {
                            println!("No polygon found with the selected ID: {}", selected_id);
                        }
                    }
                }),
            )
            .style(move |s| s.width(thirds).margin_right(5.0)),
            styled_input(
                "Green:".to_string(),
                &wgpu_to_hex(selected_polygon_data.read().borrow().fill[1]).to_string(),
                "0-255",
                Box::new({
                    move |value| {
                        let selected_id = selected_polygon_id.get();
                        let mut editor = editor_cloned3.lock().unwrap();

                        let mut gpu_helper = cloned_helper4.lock().unwrap();

                        let polygon_index =
                            editor.polygons.iter().position(|p| p.id == selected_id);

                        if let Some(index) = polygon_index {
                            // Now you have the selected polygon
                            println!("Found selected polygon with ID: {}", selected_id);
                            // You can now work with the selected_polygon

                            // Get the necessary data from editor
                            let viewport_width = editor.viewport.lock().unwrap().width;
                            let viewport_height = editor.viewport.lock().unwrap().height;
                            let device = &gpu_helper
                                .gpu_resources
                                .as_ref()
                                .expect("Couldn't get gpu resources")
                                .device;

                            let window_size = WindowSize {
                                width: viewport_width as u32,
                                height: viewport_height as u32,
                            };

                            let camera = editor.camera.expect("Couldn't get camera");

                            // Second iteration: update the selected polygon
                            if let Some(selected_polygon) = editor.polygons.get_mut(index) {
                                selected_polygon.update_data_from_fill(
                                    &window_size,
                                    &device,
                                    [
                                        selected_polygon.fill[0],
                                        color_to_wgpu(
                                            string_to_f32(&value).expect("Couldn't convert string"),
                                        ),
                                        selected_polygon.fill[2],
                                        selected_polygon.fill[3],
                                    ],
                                    &camera,
                                );
                            }
                        } else {
                            println!("No polygon found with the selected ID: {}", selected_id);
                        }
                    }
                }),
            )
            .style(move |s| s.width(thirds).margin_right(5.0)),
            styled_input(
                "Blue:".to_string(),
                &wgpu_to_hex(selected_polygon_data.read().borrow().fill[2]).to_string(),
                "0-255",
                Box::new({
                    move |value| {
                        let selected_id = selected_polygon_id.get();
                        let mut editor = editor_cloned4.lock().unwrap();

                        let mut gpu_helper = cloned_helper5.lock().unwrap();

                        let polygon_index =
                            editor.polygons.iter().position(|p| p.id == selected_id);

                        if let Some(index) = polygon_index {
                            // Now you have the selected polygon
                            println!("Found selected polygon with ID: {}", selected_id);
                            // You can now work with the selected_polygon

                            // Get the necessary data from editor
                            let viewport_width = editor.viewport.lock().unwrap().width;
                            let viewport_height = editor.viewport.lock().unwrap().height;
                            let device = &gpu_helper
                                .gpu_resources
                                .as_ref()
                                .expect("Couldn't get gpu resources")
                                .device;

                            let window_size = WindowSize {
                                width: viewport_width as u32,
                                height: viewport_height as u32,
                            };

                            let camera = editor.camera.expect("Couldn't get camera");

                            // Second iteration: update the selected polygon
                            if let Some(selected_polygon) = editor.polygons.get_mut(index) {
                                selected_polygon.update_data_from_fill(
                                    &window_size,
                                    &device,
                                    [
                                        selected_polygon.fill[0],
                                        selected_polygon.fill[1],
                                        color_to_wgpu(
                                            string_to_f32(&value).expect("Couldn't convert string"),
                                        ),
                                        selected_polygon.fill[3],
                                    ],
                                    &camera,
                                );
                            }
                        } else {
                            println!("No polygon found with the selected ID: {}", selected_id);
                        }
                    }
                }),
            )
            .style(move |s| s.width(thirds)),
        ))
        .style(move |s| {
            s.width(aside_width)
            // .display(Display::Grid)
            // .grid_template_columns(vec![TrackSizingFunction::Repeat(
            //     // floem::taffy::GridTrackRepetition::Count(3),
            //     GridTrackRepetition::AutoFill,
            //     vec![MinMax::from(MinMax {
            //         min: MinTrackSizingFunction::Fixed(LengthPercentage::Length(100.0)),
            //         max: MaxTrackSizingFunction::Fraction(1.0),
            //     })],
            // )])
        }),
        styled_input(
            "Border Radius:".to_string(),
            &selected_polygon_data
                .read()
                .borrow()
                .border_radius
                .to_string(),
            "Enter radius",
            Box::new({
                move |value| {
                    let selected_id = selected_polygon_id.get();
                    let border_radius = string_to_f32(&value).expect("Couldn't convert string");

                    let mut editor = editor_cloned5.lock().unwrap();
                    let mut gpu_helper = cloned_helper2.lock().unwrap();

                    // First iteration: find the index of the selected polygon
                    let polygon_index = editor.polygons.iter().position(|p| p.id == selected_id);

                    if let Some(index) = polygon_index {
                        println!("Found selected polygon with ID: {}", selected_id);

                        // Get the necessary data from editor
                        let viewport_width = editor.viewport.lock().unwrap().width;
                        let viewport_height = editor.viewport.lock().unwrap().height;
                        let device = &gpu_helper
                            .gpu_resources
                            .as_ref()
                            .expect("Couldn't get gpu resources")
                            .device;

                        let window_size = WindowSize {
                            width: viewport_width as u32,
                            height: viewport_height as u32,
                        };

                        let camera = editor.camera.expect("Couldn't get camera");

                        // Second iteration: update the selected polygon
                        if let Some(selected_polygon) = editor.polygons.get_mut(index) {
                            selected_polygon.update_data_from_border_radius(
                                &window_size,
                                &device,
                                border_radius,
                                &camera,
                            );
                        }
                    } else {
                        println!("No polygon found with the selected ID: {}", selected_id);
                    }
                }
            }),
        ),
    ))
    .style(|s| {
        s.width(300)
            .padding(20)
            .background(Color::rgba(240.0, 240.0, 240.0, 255.0))
            .border_radius(15)
            .box_shadow_blur(15)
            .box_shadow_spread(4)
            .box_shadow_color(Color::rgba(0.0, 0.0, 0.0, 0.36))
    })
    .style(|s| {
        s
            // .absolute()
            .height(800.0)
            .margin_left(0.0)
            .margin_top(20)
            .z_index(10)
    })
}
