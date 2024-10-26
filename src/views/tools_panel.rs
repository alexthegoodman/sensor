use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};

use bytemuck::Contiguous;
use common_vector::basic::{
    color_to_wgpu, rgb_to_wgpu, string_to_f32, wgpu_to_human, Point, WindowSize,
};
use common_vector::dot::draw_dot;
use common_vector::editor::{self, ControlMode, Editor, ToolCategory, Viewport};
use common_vector::guideline::create_guide_line_buffers;
use common_vector::polygon::{Polygon, PolygonConfig, Stroke};
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::style::{Background, CursorStyle, Transition};
use floem::taffy::{AlignItems, Display as TaffyDisplay, FlexDirection, FlexWrap};
use floem::views::{
    container, dyn_container, dyn_stack, empty, label, list, scroll, stack, tab, text_input,
    virtual_stack, RadioButton, StackExt, VirtualDirection, VirtualItemSize,
};

use floem_renderer::gpu_resources;
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

use crate::LayersUpdateHandler;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::buttons::{layer_button, option_button, small_button, sortable_item, success_button};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LayerKind {
    Polygon,
    // Path,
    // Image,
    // Text,
    // Group,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Layer {
    pub instance_id: Uuid,
    pub instance_name: String,
    pub instance_kind: LayerKind,
}

impl Layer {
    pub fn from_polygon_config(config: &PolygonConfig) -> Self {
        Layer {
            instance_id: config.id,
            instance_name: config.name.clone(),
            instance_kind: LayerKind::Polygon,
        }
    }
}

pub fn tools_view(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl IntoView {
    // let ui_update_trigger = create_rw_signal(0);
    let window_height = create_rw_signal(0.0);
    let layers: RwSignal<Vec<Layer>> = create_rw_signal(Vec::new());

    let layers_ref = Arc::new(Mutex::new(layers));

    let editor_cloned = Arc::clone(&editor);
    let editor_cloned2 = Arc::clone(&editor);
    let editor_cloned3 = Arc::clone(&editor);
    let editor_cloned4 = Arc::clone(&editor);
    let gpu_cloned = Arc::clone(&gpu_helper);
    let viewport_cloned = Arc::clone(&viewport);

    let shape_tab_active = RwSignal::new(true);
    let brush_tab_active = RwSignal::new(false);
    let point_mode_active = RwSignal::new(true);
    let edge_mode_active = RwSignal::new(false);
    let tool_category = RwSignal::new(ToolCategory::Shape);
    let control_mode = RwSignal::new(ControlMode::Point);

    // let mode_picker = ControlMode::iter()
    //     .map(move |fm| RadioButton::new_labeled_rw(fm, control_mode, move || fm))
    //     .h_stack();

    create_effect({
        move |_| {
            let selected_mode = control_mode.get();
            let mut editor = editor_cloned4.lock().unwrap();
            println!("selected_mode {:?}", selected_mode);
            editor.control_mode = selected_mode;
        }
    });

    create_effect({
        let editor_cloned3 = Arc::clone(&editor_cloned2);
        move |_| {
            let mut editor = editor_cloned3.lock().unwrap();
            let viewport = editor.viewport.lock().unwrap();

            window_height.set(viewport.height);
        }
    });

    // Create the handle_polygon_click function
    let handle_layers_update: Arc<LayersUpdateHandler> = Arc::new({
        // let set_counter_ref = Arc::clone(&set_counter_ref);
        let layers_ref = Arc::clone(&layers_ref);
        move || {
            let new_editor = editor_cloned2.clone();
            // let set_counter_ref = set_counter_ref.clone();
            let layers_ref = layers_ref.clone();
            Some(Box::new(move |polygons_data: Vec<PolygonConfig>| {
                // cannot lock editor here!
                // {
                //     let mut editor = new_editor.lock().unwrap();
                //     // Update editor as needed
                // }

                if let Ok(mut layers) = layers_ref.lock() {
                    // layers.update(|c| {
                    //     *c = true;
                    // });
                    let new_layers: Vec<Layer> = polygons_data
                        .iter()
                        .map(Layer::from_polygon_config)
                        .collect();

                    if (new_layers.len() > 0) {
                        // println!("Layers change {:?}", new_layers.len());
                        // layers.set(new_layers);
                        // layers.update(|c| c.push(new_layers[0].clone()));
                        layers.update(|l| *l = new_layers);
                        // ui_update_trigger.update(|count| *count += 1);
                    }
                }
            }) as Box<dyn FnMut(Vec<PolygonConfig>)>)
        }
    });

    // Use create_effect to set the handler only once
    create_effect({
        let handle_layers_update = Arc::clone(&handle_layers_update);
        let editor_cloned3 = Arc::clone(&editor_cloned3);
        move |_| {
            let mut editor = editor_cloned3.lock().unwrap();
            editor.handle_layers_update = Some(Arc::clone(&handle_layers_update));
            editor.run_layers_update();
        }
    });

    let dragger_id = create_rw_signal(Uuid::nil());

    v_stack((
        // label(move || format!("Tools")).style(|s| s.margin_bottom(10)),
        v_stack((
            label(|| "Tools").style(|s| s.font_size(14.0).margin_bottom(15.0)),
            v_stack((
                // mode_picker,
                h_stack((
                    small_button(
                        "Shapes",
                        "triangle",
                        {
                            move |_| {
                                tool_category.set(ToolCategory::Shape);
                                control_mode.set(ControlMode::Point);

                                shape_tab_active.set(true);
                                brush_tab_active.set(false);
                                point_mode_active.set(true);
                                edge_mode_active.set(false);
                            }
                        },
                        shape_tab_active,
                    ),
                    small_button(
                        "Brushes",
                        "brush",
                        {
                            move |_| {
                                tool_category.set(ToolCategory::Brush);
                                control_mode.set(ControlMode::Brush);

                                shape_tab_active.set(false);
                                brush_tab_active.set(true);
                                point_mode_active.set(true);
                                edge_mode_active.set(false);
                            }
                        },
                        brush_tab_active,
                    ),
                ))
                .style(|s| s.margin_bottom(7.0)),
                // success_button("Export", "plus", None::<fn()>, false),
                dyn_container(
                    move || tool_category.get(),
                    move |tool_category_real| {
                        let editor = editor.clone();
                        let gpu_helper = gpu_helper.clone();
                        let viewport = viewport.clone();

                        let editor_cloned = editor_cloned.clone();
                        let gpu_cloned = gpu_cloned.clone();
                        let viewport_cloned = viewport_cloned.clone();

                        if tool_category_real == ToolCategory::Shape {
                            v_stack((
                                h_stack((
                                    small_button(
                                        "Points",
                                        "dot",
                                        {
                                            move |_| {
                                                control_mode.set(ControlMode::Point);

                                                point_mode_active.set(true);
                                                edge_mode_active.set(false);
                                            }
                                        },
                                        point_mode_active,
                                    ),
                                    small_button(
                                        "Edges",
                                        "dots-vertical",
                                        {
                                            move |_| {
                                                control_mode.set(ControlMode::Edge);

                                                point_mode_active.set(false);
                                                edge_mode_active.set(true);
                                            }
                                        },
                                        edge_mode_active,
                                    ),
                                ))
                                .style(|s| s.margin_bottom(7.0)),
                                container((
                                    option_button(
                                        "Add Polygon",
                                        "triangle",
                                        Some(move || {
                                            let mut editor = editor.lock().unwrap();
                                            // let mut handler = handler.lock().unwrap();
                                            println!("Handle click...");

                                            // handler.handle_button_click(editor);

                                            let polygon_config = PolygonConfig {
                                                id: Uuid::new_v4(),
                                                name: "Polygon".to_string(),
                                                points: vec![
                                                    Point { x: 0.0, y: 0.0 },
                                                    Point { x: 1.0, y: 0.0 },
                                                    Point { x: 0.5, y: 1.0 },
                                                ],
                                                dimensions: (100.0, 100.0),
                                                position: Point { x: 600.0, y: 100.0 },
                                                border_radius: 5.0,
                                                fill: [1.0, 1.0, 1.0, 1.0],
                                                stroke: Stroke {
                                                    fill: [1.0, 1.0, 1.0, 1.0],
                                                    thickness: 2.0,
                                                },
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
                                            let camera =
                                                editor.camera.expect("Couldn't get camera");
                                            editor.add_polygon(Polygon::new(
                                                &window_size,
                                                &device,
                                                &camera,
                                                polygon_config.points.clone(),
                                                polygon_config.dimensions,
                                                polygon_config.position,
                                                polygon_config.border_radius,
                                                polygon_config.fill,
                                                "Polygon".to_string(),
                                            ));
                                        }),
                                        false,
                                    )
                                    .style(|s| s.margin_right(5.0)),
                                    option_button(
                                        "Add Square",
                                        "square",
                                        Some(move || {
                                            let mut editor = editor_cloned.lock().unwrap();
                                            // let mut square_handler = square_handler.lock().unwrap();
                                            println!("Handle square...");

                                            // square_handler.handle_button_click(editor_cloned);

                                            let polygon_config = PolygonConfig {
                                                id: Uuid::new_v4(),
                                                name: "Square".to_string(),
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
                                                stroke: Stroke {
                                                    fill: [1.0, 1.0, 1.0, 1.0],
                                                    thickness: 2.0,
                                                },
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
                                            let camera =
                                                editor.camera.expect("Couldn't get camera");
                                            editor.add_polygon(Polygon::new(
                                                &window_size,
                                                &device,
                                                &camera,
                                                polygon_config.points.clone(),
                                                polygon_config.dimensions,
                                                polygon_config.position,
                                                polygon_config.border_radius,
                                                polygon_config.fill,
                                                "Polygon".to_string(),
                                            ));
                                        }),
                                        false,
                                    ),
                                ))
                                .style(|s| s.flex_wrap(FlexWrap::Wrap).margin_top(5.0)),
                            ))
                            .into_any()
                        } else if tool_category_real == ToolCategory::Brush {
                            container((
                                option_button(
                                    "Use Solid",
                                    "brush",
                                    Some(move || {
                                        // let mut editor = editor.lock().unwrap();
                                        // let mut handler = handler.lock().unwrap();
                                        println!("Handle Solid...");
                                    }),
                                    false,
                                )
                                .style(|s| s.margin_right(5.0)),
                                option_button(
                                    "Use Calligraphy",
                                    "brush",
                                    Some(move || {
                                        // let mut editor = editor.lock().unwrap();
                                        // let mut handler = handler.lock().unwrap();
                                        println!("Handle Calligraphy...");
                                    }),
                                    false,
                                )
                                .style(|s| s.margin_right(5.0)),
                                option_button(
                                    "Use Airbrush",
                                    "brush",
                                    Some(move || {
                                        // let mut editor = editor_cloned.lock().unwrap();
                                        // let mut square_handler = square_handler.lock().unwrap();
                                        println!("Handle Airbrush...");
                                    }),
                                    false,
                                ),
                            ))
                            .style(|s| s.flex_wrap(FlexWrap::Wrap).margin_top(5.0))
                            .into_any()
                        } else {
                            empty().into_any()
                        }
                    },
                ),
            )),
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
        .style(move |s| {
            s
                // .absolute()
                .height(window_height.get() / 2.0 - 120.0)
                .margin_left(0.0)
                .margin_top(20)
                .z_index(1)
        }),
        v_stack((
            label(|| "Scene").style(|s| s.font_size(14.0).margin_bottom(15.0)),
            scroll(
                // dyn_stack(
                //     move || layers.get(),
                //     |layer| layer.instance_id, // Assuming each layer has a unique id
                //     |layer| {
                //         let icon_name = match layer.instance_kind {
                //             LayerKind::Polygon => "triangle",
                //             // LayerKind::Path =>
                //             //         // LayerKind::Imag(data) =>
                //             //         // LayerKind::Text =>
                //             //         // LayerKind::Group =>
                //         };
                //         layer_button(layer.instance_name.clone(), &icon_name)
                //     },
                // )
                // .style(|s| {
                //     s.display(TaffyDisplay::Flex)
                //         .flex_direction(FlexDirection::Column)
                // }),
                dyn_stack(
                    move || layers.get(),
                    |layer: &Layer| layer.instance_id,
                    move |layer| {
                        let icon_name = match layer.instance_kind {
                            LayerKind::Polygon => "triangle",
                            // LayerKind::Path =>
                            //         // LayerKind::Imag(data) =>
                            //         // LayerKind::Text =>
                            //         // LayerKind::Group =>
                        };
                        sortable_item(
                            layers,
                            dragger_id,
                            layer.instance_id,
                            layer.instance_name.clone(),
                            icon_name,
                        )
                    },
                )
                .style(|s: floem::style::Style| s.flex_col().column_gap(5).padding(10))
                .into_view(),
            )
            .style(move |s| s.height(window_height.get() / 2.0 - 190.0)),
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
        .style(move |s| {
            s
                // .absolute()
                .height(window_height.get() / 2.0 - 120.0)
                .margin_left(0.0)
                .margin_top(20)
                .z_index(1)
        }),
    ))
}
