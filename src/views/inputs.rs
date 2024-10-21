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

use floem::unit::{Auto, DurationUnitExt, Pct, UnitExt};
use std::time::Duration;

pub fn styled_input(
    label_text: String,
    initial_value: &str,
    placeholder: &str,
    on_event_stop: Box<dyn Fn(String) + 'static>,
) -> impl IntoView {
    let value = create_rw_signal(initial_value.to_string());

    v_stack((
        label(move || label_text.clone()).style(|s| s.font_size(10.0).margin_bottom(1.0)),
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
                s.width_full()
                    .border(1)
                    .border_color(Color::GRAY)
                    .border_radius(4)
                    .padding_horiz(5)
                    .padding_vert(3)
            }),
    ))
    .style(|s| s.margin_bottom(10))
}
