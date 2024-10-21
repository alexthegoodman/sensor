use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};

use bytemuck::Contiguous;
use common_vector::basic::{rgb_to_wgpu, Point, WindowSize};
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
        // window_size: WindowSize,
        viewport: Arc<Mutex<Viewport>>,
        polygon_config: PolygonConfig,
    ) {
        let handler = Box::new(move |mut editor: MutexGuard<'_, Editor>| {
            println!("Button clicked, attempting to add polygon...");
            let viewport = viewport.lock().unwrap();
            let window_size = WindowSize {
                width: viewport.width as u32,
                height: viewport.height as u32,
            };
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

fn string_to_f32(s: &str) -> Result<f32, std::num::ParseFloatError> {
    let trimmed = s.trim();

    if trimmed.is_empty() {
        return Ok(0.0);
    }

    // Check if there's at least one digit in the string
    if !trimmed.chars().any(|c| c.is_ascii_digit()) {
        return Ok(0.0);
    }

    // At this point, we know there's at least one digit, so let's try to parse
    match trimmed.parse::<f32>() {
        Ok(num) => Ok(num),
        Err(e) => {
            // If parsing failed, check if it's because of a misplaced dash
            if trimmed.contains('-') && trimmed != "-" {
                // Remove all dashes and try parsing again
                let without_dashes = trimmed.replace('-', "");
                without_dashes.parse::<f32>().map(|num| -num.abs())
            } else {
                Err(e)
            }
        }
    }
}

fn assets_view() -> impl IntoView {
    (label(move || format!("Assets")).style(|s| s.margin_bottom(10)),)
}

fn tools_view(
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_cloned: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    mut handler: std::sync::Arc<Mutex<Handler>>,
    mut square_handler: std::sync::Arc<Mutex<Handler>>,
) -> impl IntoView {
    (
        // label(move || format!("Tools")).style(|s| s.margin_bottom(10)),
        container((
            option_button(
                "Add Polygon",
                "plus",
                Some(move || {
                    let mut editor = editor.lock().unwrap();
                    let mut handler = handler.lock().unwrap();
                    println!("Handle click...");

                    // if let Some(handle_click) = &editor.handle_button_click(editor) {
                    //     println!("Handling click...");
                    //     handle_click(editor);
                    // }
                    handler.handle_button_click(editor);
                }),
                false,
            )
            .style(|s| s.margin_right(5.0)),
            option_button(
                "Add Square",
                "plus",
                Some(move || {
                    let mut editor_cloned = editor_cloned.lock().unwrap();
                    let mut square_handler = square_handler.lock().unwrap();
                    println!("Handle square...");

                    // if let Some(handle_click) = &editor.handle_button_click(editor) {
                    //     println!("Handling click...");
                    //     handle_click(editor);
                    // }
                    square_handler.handle_button_click(editor_cloned);
                }),
                false,
            ),
        ))
        .style(|s| s.padding_vert(15.0).z_index(1))
    )
}

fn settings_view() -> impl IntoView {
    (label(move || format!("Settings")).style(|s| s.margin_bottom(10)),)
}

use floem::unit::{DurationUnitExt, UnitExt};
use std::time::Duration;

fn tab_interface(
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_cloned: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    mut handler: std::sync::Arc<Mutex<Handler>>,
    mut square_handler: std::sync::Arc<Mutex<Handler>>,
    polygon_selected: RwSignal<bool>,
) -> impl View {
    let tabs: im::Vector<&str> = vec!["Tools", "Assets", "Settings"].into_iter().collect();
    let (tabs, _set_tabs) = create_signal(tabs);
    let (active_tab, set_active_tab) = create_signal(0);

    let list = scroll({
        virtual_stack(
            VirtualDirection::Vertical,
            VirtualItemSize::Fixed(Box::new(|| 90.0)),
            move || tabs.get(),
            move |item| *item,
            move |item| {
                let index = tabs
                    .get_untracked()
                    .iter()
                    .position(|it| *it == item)
                    .unwrap();
                let active = index == active_tab.get();
                let icon_name = match item {
                    "Tools" => "brush",
                    "Assets" => "shapes",
                    "Settings" => "gear",
                    _ => "plus",
                };
                stack((
                    // label(move || item).style(|s| s.font_size(18.0)),
                    // svg(create_icon("plus")).style(|s| s.width(24).height(24)),
                    nav_button(
                        item,
                        icon_name,
                        Some(move || {
                            println!("Click...");
                            set_active_tab.update(|v: &mut usize| {
                                *v = tabs
                                    .get_untracked()
                                    .iter()
                                    .position(|it| *it == item)
                                    .unwrap();
                            });
                            // EventPropagation::Continue
                        }),
                        active,
                    ),
                ))
                // .on_click()
                .on_event(EventListener::KeyDown, move |e| {
                    if let Event::KeyDown(key_event) = e {
                        let active = active_tab.get();
                        if key_event.modifiers.is_empty() {
                            match key_event.key.logical_key {
                                Key::Named(NamedKey::ArrowUp) => {
                                    if active > 0 {
                                        set_active_tab.update(|v| *v -= 1)
                                    }
                                    EventPropagation::Stop
                                }
                                Key::Named(NamedKey::ArrowDown) => {
                                    if active < tabs.get().len() - 1 {
                                        set_active_tab.update(|v| *v += 1)
                                    }
                                    EventPropagation::Stop
                                }
                                _ => EventPropagation::Continue,
                            }
                        } else {
                            EventPropagation::Continue
                        }
                    } else {
                        EventPropagation::Continue
                    }
                })
                .keyboard_navigatable()
                .style(move |s| {
                    s.margin_bottom(15.0)
                        .border_radius(15)
                        .apply_if(active, |s| s.border(1.0).border_color(Color::GRAY))
                })
            },
        )
        .style(|s| {
            s.flex_col()
                .height_full()
                .width(110.0)
                .padding_vert(15.0)
                .padding_horiz(20.0)
        })
    })
    .scroll_style(|s| s.shrink_to_fit());

    // let tab_content = tab(
    //     move || active_tab.get(),
    //     move || tabs.get(),
    //     |it| *it,
    //     move |it| match it {
    //         "Tools" => tools_view(
    //             editor.clone(),
    //             editor_cloned.clone(),
    //             handler.clone(),
    //             square_handler.clone(),
    //         )
    //         .into_any(),
    //         "Assets" => assets_view().into_any(),
    //         "Settings" => settings_view().into_any(),
    //         _ => label(|| "Not implemented".to_owned()).into_any(),
    //     },
    // )
    // .style(|s| s.flex_col().items_start());

    container(
        container((
            list,
            dyn_container(
                move || !polygon_selected.get(),
                move |show_content| {
                    let editor = editor.clone();
                    let editor_cloned = editor_cloned.clone();
                    let handler = handler.clone();
                    let square_handler = square_handler.clone();
                    if show_content {
                        tab(
                            move || active_tab.get(),
                            move || tabs.get(),
                            |it| *it,
                            move |it| match it {
                                "Tools" => tools_view(
                                    editor.clone(),
                                    editor_cloned.clone(),
                                    handler.clone(),
                                    square_handler.clone(),
                                )
                                .into_any(),
                                "Assets" => assets_view().into_any(),
                                "Settings" => settings_view().into_any(),
                                _ => label(|| "Not implemented".to_owned()).into_any(),
                            },
                        )
                        .style(|s| s.flex_col().items_start())
                        .into_any()
                    } else {
                        empty().into_any()
                    }
                },
            ),
        ))
        .style(|s| s.flex_col().width_full().height_full()),
    )
    .style(|s| s.width_full().height_full())
}

fn styled_input(
    label_text: String,
    initial_value: &str,
    placeholder: &str,
    on_event_stop: Box<dyn Fn(String) + 'static>,
) -> impl IntoView {
    let value = create_rw_signal(initial_value.to_string());

    h_stack((
        label(move || label_text.clone()).style(|s| s.min_width(100)),
        text_input(value)
            .on_event_stop(EventListener::KeyUp, move |event: &Event| {
                if let Event::KeyUp(key) = event {
                    match key.key.logical_key {
                        Key::Named(NamedKey::ArrowUp) => {
                            // Handle up arrow key
                            println!("Up arrow pressed");
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            // Handle down arrow key
                            println!("Down arrow pressed");
                        }
                        _ => {
                            println!("value {:?}", value.get());
                            let value = value.get();
                            on_event_stop(value);
                        }
                    }
                }
            })
            .placeholder(placeholder)
            .style(|s| {
                s.border(1)
                    .border_color(Color::GRAY)
                    .border_radius(4)
                    .padding_horiz(5)
                    .padding_vert(3)
            }),
    ))
    .style(|s| s.items_center().margin_bottom(10))
}

fn properties_view(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
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

                        // Second iteration: update the selected polygon
                        if let Some(selected_polygon) = editor.polygons.get_mut(index) {
                            selected_polygon.update_data_from_dimensions(
                                &window_size,
                                &device,
                                (width, selected_polygon.dimensions.1),
                            );
                        }
                    } else {
                        println!("No polygon found with the selected ID: {}", selected_id);
                    }
                }
            }),
        ),
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

                        // Second iteration: update the selected polygon
                        if let Some(selected_polygon) = editor.polygons.get_mut(index) {
                            selected_polygon.update_data_from_dimensions(
                                &window_size,
                                &device,
                                (selected_polygon.dimensions.0, height),
                            );
                        }
                    } else {
                        println!("No polygon found with the selected ID: {}", selected_id);
                    }
                }
            }),
        ),
        styled_input(
            "Red:".to_string(),
            "255",
            "0-255",
            Box::new({
                move |value| {
                    let selected_id = selected_polygon_id.get();
                    let editor = editor_cloned2.lock().unwrap();

                    if let Some(selected_polygon) =
                        editor.polygons.iter().find(|p| p.id == selected_id)
                    {
                        // Now you have the selected polygon
                        println!("Found selected polygon with ID: {}", selected_id);
                        // You can now work with the selected_polygon
                    } else {
                        println!("No polygon found with the selected ID: {}", selected_id);
                    }
                }
            }),
        ),
        styled_input(
            "Green:".to_string(),
            "0",
            "0-255",
            Box::new({
                move |value| {
                    let selected_id = selected_polygon_id.get();
                    let editor = editor_cloned3.lock().unwrap();

                    if let Some(selected_polygon) =
                        editor.polygons.iter().find(|p| p.id == selected_id)
                    {
                        // Now you have the selected polygon
                        println!("Found selected polygon with ID: {}", selected_id);
                        // You can now work with the selected_polygon
                    } else {
                        println!("No polygon found with the selected ID: {}", selected_id);
                    }
                }
            }),
        ),
        styled_input(
            "Blue:".to_string(),
            "0",
            "0-255",
            Box::new({
                move |value| {
                    let selected_id = selected_polygon_id.get();
                    let editor = editor_cloned4.lock().unwrap();

                    if let Some(selected_polygon) =
                        editor.polygons.iter().find(|p| p.id == selected_id)
                    {
                        // Now you have the selected polygon
                        println!("Found selected polygon with ID: {}", selected_id);
                        // You can now work with the selected_polygon
                    } else {
                        println!("No polygon found with the selected ID: {}", selected_id);
                    }
                }
            }),
        ),
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

                        // Second iteration: update the selected polygon
                        if let Some(selected_polygon) = editor.polygons.get_mut(index) {
                            selected_polygon.update_data_from_border_radius(
                                &window_size,
                                &device,
                                border_radius,
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

type PolygonClickHandler = dyn Fn() -> Option<Box<dyn FnMut(Uuid, PolygonConfig)>>;
use std::ops::Not;

fn app_view(
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor_cloned: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_cloned2: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_cloned3: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_cloned4: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    mut handler: std::sync::Arc<Mutex<Handler>>,
    mut square_handler: std::sync::Arc<Mutex<Handler>>,
) -> impl IntoView {
    // // let (counter, mut set_counter) = create_signal(0);
    // let (polygon_selected, mut set_polygon_selected) = create_signal(false);
    // let (selected_polygon_id, mut set_selected_polygon_id) = create_signal(Uuid::nil());
    let polygon_selected = create_rw_signal(false);
    let selected_polygon_id = create_rw_signal(Uuid::nil());
    let selected_polygon_data = create_rw_signal(PolygonConfig {
        points: Vec::new(),
        dimensions: (100.0, 100.0),
        position: Point { x: 0.0, y: 0.0 },
        border_radius: 0.0,
    });

    // Create a RefCell to hold the set_counter function
    // let set_counter_ref = Arc::new(Mutex::new(set_counter));
    let polygon_selected_ref = Arc::new(Mutex::new(polygon_selected));
    let selected_polygon_id_ref = Arc::new(Mutex::new(selected_polygon_id));
    let selected_polygon_data_ref = Arc::new(Mutex::new(selected_polygon_data));

    let editor_cloned2 = editor_cloned2.clone();

    // Create the handle_polygon_click function
    let handle_polygon_click: Arc<PolygonClickHandler> = Arc::new({
        // let set_counter_ref = Arc::clone(&set_counter_ref);
        let polygon_selected_ref = Arc::clone(&polygon_selected_ref);
        let selected_polygon_id_ref = Arc::clone(&selected_polygon_id_ref);
        let selected_polygon_data_ref = Arc::clone(&selected_polygon_data_ref);
        move || {
            let new_editor = editor_cloned2.clone();
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
            editor,
            editor_cloned,
            handler,
            square_handler,
            polygon_selected,
        ),
        dyn_container(
            move || polygon_selected.get(),
            move |polygon_selected_real| {
                if polygon_selected_real {
                    properties_view(
                        gpu_helper.clone(),
                        editor_cloned4.clone(),
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

use once_cell::sync::Lazy;
use std::collections::HashMap;

static ICON_CACHE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn create_icon(name: &str) -> String {
    // Try to retrieve from cache first
    if let Some(icon) = ICON_CACHE.lock().unwrap().get(name) {
        return icon.clone();
    }

    // If not in cache, load and cache it
    let icon = match name {
        "plus" => include_str!("assets/plus-thin.svg"),
        "minus" => include_str!("assets/minus-thin.svg"),
        "windmill" => include_str!("assets/windmill-thin.svg"),
        "gear" => include_str!("assets/gear-six-thin.svg"),
        "brush" => include_str!("assets/paint-brush-thin.svg"),
        "shapes" => include_str!("assets/shapes-thin.svg"),
        "arrow-left" => include_str!("assets/arrow-left-thin.svg"),
        _ => "",
    };

    // Store in cache
    ICON_CACHE
        .lock()
        .unwrap()
        .insert(name.to_string(), icon.to_string());

    icon.to_string()
}

fn small_button(
    text: &'static str,
    icon_name: &'static str,
    action: impl FnMut(&Event) + 'static,
    active: bool,
) -> impl IntoView {
    button(
        v_stack((
            svg(create_icon(icon_name)).style(|s| s.width(16).height(16)),
            // label(move || text).style(|s| s.margin_top(4.0)),
        ))
        .style(|s| s.justify_center().align_items(AlignItems::Center)),
    )
    .on_click_stop(action)
    .style(move |s| {
        s.width(28)
            .height(28)
            .justify_center()
            .align_items(AlignItems::Center)
            .background(Color::WHITE)
            .border_radius(15)
            .transition(Background, Transition::ease_in_out(200.millis()))
            .focus_visible(|s| s.border(2.).border_color(Color::BLUE))
            .hover(|s| s.background(Color::LIGHT_GRAY).cursor(CursorStyle::Pointer))
            .z_index(20)
    })
}

fn nav_button(
    text: &'static str,
    icon_name: &'static str,
    action: Option<impl FnMut() + 'static>,
    active: bool,
) -> impl IntoView {
    button(
        v_stack((
            svg(create_icon(icon_name)).style(|s| s.width(30).height(30)),
            label(move || text).style(|s| s.margin_top(4.0)),
        ))
        .style(|s| s.justify_center().align_items(AlignItems::Center)),
    )
    .action(action)
    .style(move |s| {
        s.width(70)
            .height(70)
            .justify_center()
            .align_items(AlignItems::Center)
            .border(0)
            .border_radius(15)
            .box_shadow_blur(15)
            .box_shadow_spread(4)
            .box_shadow_color(Color::rgba(0.0, 0.0, 0.0, 0.36))
            .transition(Background, Transition::ease_in_out(200.millis()))
            .focus_visible(|s| s.border(2.).border_color(Color::BLUE))
            .hover(|s| s.background(Color::LIGHT_GRAY).cursor(CursorStyle::Pointer))
    })
}

fn option_button(
    text: &'static str,
    icon_name: &'static str,
    action: Option<impl FnMut() + 'static>,
    active: bool,
) -> impl IntoView {
    button(
        v_stack((
            svg(create_icon(icon_name)).style(|s| s.width(24).height(24)),
            label(move || text).style(|s| s.margin_top(4.0).font_size(9.0)),
        ))
        .style(|s| s.justify_center().align_items(AlignItems::Center)),
    )
    .action(action)
    .style(move |s| {
        s.width(60)
            .height(60)
            .justify_center()
            .align_items(AlignItems::Center)
            .border(1.0)
            .border_color(Color::GRAY)
            .border_radius(15)
            .transition(Background, Transition::ease_in_out(200.millis()))
            .focus_visible(|s| s.border(2.).border_color(Color::BLUE))
            .hover(|s| s.background(Color::LIGHT_GRAY).cursor(CursorStyle::Pointer))
    })
}

type RenderCallback<'a> = dyn for<'b> Fn(
        wgpu::CommandEncoder,
        wgpu::SurfaceTexture,
        wgpu::TextureView,
        wgpu::TextureView,
        &WindowHandle,
    ) + 'a;

fn create_render_callback<'a>() -> Box<RenderCallback<'a>> {
    Box::new(
        move |mut encoder: wgpu::CommandEncoder,
              frame: wgpu::SurfaceTexture,
              view: wgpu::TextureView,
              resolve_view: wgpu::TextureView,
              window_handle: &WindowHandle| {
            let mut handle = window_handle.borrow();

            if let Some(gpu_resources) = &handle.gpu_resources {
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: Some(&resolve_view),
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
                                // load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        // depth_stencil_attachment: None,
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &handle
                                .gpu_helper
                                .as_ref()
                                .expect("Couldn't get gpu helper")
                                .lock()
                                .unwrap()
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

                    let viewport = editor.viewport.lock().unwrap();
                    let window_size = WindowSize {
                        width: viewport.width as u32,
                        height: viewport.height as u32,
                    };

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

                let command_buffer = encoder.finish();
                gpu_resources.queue.submit(Some(command_buffer));
                gpu_resources.device.poll(wgpu::Maintain::Poll);
                frame.present();
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
    // window_size: WindowSize,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn Fn(f64, f64)>> {
    Some(Box::new(move |positionX: f64, positionY: f64| {
        let mut editor = editor.lock().unwrap();
        let viewport = viewport.lock().unwrap();
        let window_size = WindowSize {
            width: viewport.width as u32,
            height: viewport.height as u32,
        };
        // println!("window size {:?}", window_size);
        // println!("positions {:?} {:?}", positionX, positionY);
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
    // window_size: WindowSize,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn Fn(MouseButton, ElementState)>> {
    Some(Box::new(move |button, state| {
        let mut editor = editor.lock().unwrap();
        let viewport = viewport.lock().unwrap();
        let window_size = WindowSize {
            width: viewport.width as u32,
            height: viewport.height as u32,
        };
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

fn handle_window_resize(
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    // window_size: WindowSize, // need newest window size
    gpu_helper: std::sync::Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(PhysicalSize<u32>, LogicalSize<f64>)>> {
    Some(Box::new(move |size, logical_size| {
        let mut editor = editor.lock().unwrap();

        let window_size = WindowSize {
            width: size.width,
            height: size.height,
        };

        let mut viewport = viewport.lock().unwrap();

        viewport.width = size.width as f32;
        viewport.height = size.height as f32;

        editor.update_date_from_window_resize(&window_size, &gpu_resources.device);

        gpu_helper
            .lock()
            .unwrap()
            .recreate_depth_view(&gpu_resources, &window_size);
    }))
}

fn main() {
    let app = Application::new();

    // Get the primary monitor's size
    let monitor = app.primary_monitor().expect("Couldn't get primary monitor");
    let monitor_size = monitor.size();

    // Calculate a reasonable window size (e.g., 80% of the screen size)
    let window_width = (monitor_size.width.into_integer() as f32 * 0.8) as u32;
    let window_height = (monitor_size.height.into_integer() as f32 * 0.8) as u32;

    let window_size = WindowSize {
        width: window_width,
        height: window_height,
    };

    let mut gpu_helper = Arc::new(Mutex::new(GpuHelper::new()));

    let gpu_cloned = Arc::clone(&gpu_helper);
    let gpu_clonsed2 = Arc::clone(&gpu_helper);
    let gpu_cloned3 = Arc::clone(&gpu_helper);

    let viewport = Arc::new(Mutex::new(Viewport::new(
        window_size.width as f32,
        window_size.height as f32,
    ))); // Or whatever your window size is
    let mut editor = Arc::new(Mutex::new(Editor::new(viewport.clone())));
    let mut handler = Arc::new(Mutex::new(Handler::new()));
    let mut square_handler = Arc::new(Mutex::new(Handler::new()));

    // let viewport = Arc::new(Mutex::new(viewport));

    let cloned_handler = Arc::clone(&handler);
    let cloned_square_handler = Arc::clone(&square_handler);
    let cloned_square_handler6 = Arc::clone(&square_handler);

    let cloned = Arc::clone(&editor);
    let cloned2 = Arc::clone(&editor);
    let cloned3 = Arc::clone(&editor);
    let cloned4 = Arc::clone(&editor);
    let cloned5 = Arc::clone(&editor);
    let cloned6 = Arc::clone(&editor);
    let cloned7 = Arc::clone(&editor);
    let cloned8 = Arc::clone(&editor);
    let cloned9 = Arc::clone(&editor);
    let cloned10 = Arc::clone(&editor);

    let (mut app, window_id) = app.window(
        move |_| {
            app_view(
                Arc::clone(&editor),
                Arc::clone(&gpu_helper),
                cloned6,
                cloned8,
                cloned9,
                cloned10,
                handler,
                cloned_square_handler6,
            )
        },
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

        // Create and set the render callback
        let render_callback = create_render_callback();

        // window_handle.set_render_callback(render_callback);
        window_handle.set_encode_callback(render_callback);
        window_handle.window_size = Some(window_size);

        println!("Ready...");

        window_handle.user_editor = Some(cloned);

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

                gpu_cloned
                    .lock()
                    .unwrap()
                    .recreate_depth_view(&gpu_resources, &window_size);

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
                                count: 4, // effect performance
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                        });

                window_handle.render_pipeline = Some(render_pipeline);
                // window_handle.depth_view = gpu_helper.depth_view;

                println!("Initialized...");

                window_handle.handle_cursor_moved =
                    handle_cursor_moved(cloned2.clone(), gpu_resources.clone(), viewport.clone());
                window_handle.handle_mouse_input =
                    handle_mouse_input(cloned3.clone(), gpu_resources.clone(), viewport.clone());
                window_handle.handle_window_resized = handle_window_resize(
                    cloned7,
                    gpu_resources.clone(),
                    gpu_cloned3,
                    viewport.clone(),
                );

                let editor_clone = cloned4.clone();

                // test items
                let mut editor = cloned5.lock().unwrap();

                // editor.handle_button_click =
                //     handle_button_click(editor_clone, gpu_resources.clone(), window_size);
                let mut cloned_handler = cloned_handler.lock().unwrap();
                cloned_handler.set_button_handler(
                    Arc::clone(&gpu_resources),
                    viewport.clone(),
                    PolygonConfig {
                        points: vec![
                            Point { x: 0.0, y: 0.0 },
                            Point { x: 1.0, y: 0.0 },
                            Point { x: 0.5, y: 1.0 },
                        ],
                        dimensions: (100.0, 100.0),
                        position: Point { x: 600.0, y: 100.0 },
                        border_radius: 5.0,
                    },
                );

                let mut cloned_square_handler = cloned_square_handler.lock().unwrap();
                cloned_square_handler.set_button_handler(
                    Arc::clone(&gpu_resources),
                    viewport.clone(),
                    PolygonConfig {
                        points: vec![
                            Point { x: 0.0, y: 0.0 },
                            Point { x: 1.0, y: 0.0 },
                            Point { x: 1.0, y: 1.0 },
                            Point { x: 0.0, y: 1.0 },
                        ],
                        dimensions: (100.0, 100.0),
                        position: Point { x: 600.0, y: 100.0 },
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
                    Point { x: 600.0, y: 100.0 },
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
                    Point { x: 900.0, y: 200.0 },
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
                    Point {
                        x: 1100.0,
                        y: 300.0,
                    },
                    8.0, // border radius
                ));

                // editor.polygons[0].update_data_from_dimensions(&window_size, &device, (200.0, 50.0));

                gpu_clonsed2.lock().unwrap().gpu_resources = Some(Arc::clone(&gpu_resources));
                editor.gpu_resources = Some(Arc::clone(&gpu_resources));
                window_handle.gpu_resources = Some(gpu_resources);
                window_handle.gpu_helper = Some(gpu_clonsed2);
            }
            PaintState::Initialized { .. } => {
                println!("Renderer is already initialized");
            }
        }
    }

    app.run();
}
