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

use super::assets_panel::assets_view;
use super::buttons::nav_button;
use super::settings_panel::settings_view;
use super::tools_panel::tools_view;

pub fn tab_interface(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    // editor_cloned: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    viewport: Arc<Mutex<Viewport>>,
    // mut handler: std::sync::Arc<Mutex<Handler>>,
    // mut square_handler: std::sync::Arc<Mutex<Handler>>,
    polygon_selected: RwSignal<bool>,
) -> impl View {
    // let editor_cloned = Arc::clone(&editor);

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

    container(
        container((
            list,
            dyn_container(
                move || !polygon_selected.get(),
                move |show_content| {
                    let editor = editor.clone();
                    // let editor_cloned = editor_cloned.clone();
                    let viewport = viewport.clone();
                    let gpu_helper = gpu_helper.clone();
                    // let handler = handler.clone();
                    // let square_handler = square_handler.clone();
                    if show_content {
                        tab(
                            move || active_tab.get(),
                            move || tabs.get(),
                            |it| *it,
                            move |it| match it {
                                "Tools" => tools_view(
                                    gpu_helper.clone(),
                                    editor.clone(),
                                    // editor_cloned.clone(),
                                    viewport.clone(),
                                    // handler.clone(),
                                    // square_handler.clone(),
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
