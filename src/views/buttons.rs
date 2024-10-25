use std::borrow::{Borrow, BorrowMut};
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};

use common_vector::basic::rgb_to_wgpu;
use floem::event::{Event, EventListener, EventPropagation};
use floem::kurbo::Point;
use floem::peniko::{Brush, Color, ColorStop, ColorStops, Extend, Gradient, GradientKind};
use floem::style::{Background, CursorStyle, Transition};
use floem::taffy::AlignItems;
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    VirtualDirection, VirtualItemSize,
};
use floem::views::{h_stack, Decorators};
use floem::views::{svg, v_stack};
use floem::{views::button, IntoView};

use floem::unit::{DurationUnitExt, UnitExt};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::time::Duration;

static ICON_CACHE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn create_icon(name: &str) -> String {
    // Try to retrieve from cache first
    if let Some(icon) = ICON_CACHE.lock().unwrap().get(name) {
        return icon.clone();
    }

    // If not in cache, load and cache it
    let icon = match name {
        "plus" => include_str!("../assets/plus-thin.svg"),
        "minus" => include_str!("../assets/minus-thin.svg"),
        "windmill" => include_str!("../assets/windmill-thin.svg"),
        "gear" => include_str!("../assets/gear-six-thin.svg"),
        "brush" => include_str!("../assets/paint-brush-thin.svg"),
        "shapes" => include_str!("../assets/shapes-thin.svg"),
        "arrow-left" => include_str!("../assets/arrow-left-thin.svg"),
        "polygon" => include_str!("../assets/polygon-thin.svg"),
        "octagon" => include_str!("../assets/octagon-thin.svg"),
        "square" => include_str!("../assets/square-thin.svg"),
        "triangle" => include_str!("../assets/triangle-thin.svg"),
        _ => "",
    };

    // Store in cache
    ICON_CACHE
        .lock()
        .unwrap()
        .insert(name.to_string(), icon.to_string());

    icon.to_string()
}

pub fn small_button(
    text: &'static str,
    icon_name: &'static str,
    action: impl FnMut(&Event) + 'static,
    active: bool,
) -> impl IntoView {
    button(
        v_stack((
            svg(create_icon(icon_name)).style(|s| s.width(24).height(24).color(Color::BLACK)),
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
            .border(0)
            // .border_color(Color::BLACK)
            .border_radius(15)
            .transition(Background, Transition::ease_in_out(200.millis()))
            .focus_visible(|s| s.border(2.).border_color(Color::BLUE))
            .hover(|s| s.background(Color::LIGHT_GRAY).cursor(CursorStyle::Pointer))
            .z_index(20)
    })
}

pub fn success_button(
    text: &'static str,
    icon_name: &'static str,
    action: Option<impl FnMut() + 'static>,
    active: bool,
) -> impl IntoView {
    // Radial gradient with different start and end circles
    let green = rgb_to_wgpu(153, 199, 162, 1.0);
    let yellow = rgb_to_wgpu(200, 204, 124, 1.0);
    // let green = (153, 199, 162, 1.0);
    // let yellow = (200, 204, 124, 1.0);

    // Linear gradient from left to right
    let gradient = Gradient {
        kind: GradientKind::Linear {
            start: Point::new(50.0, 0.0), // Start further left
            end: Point::new(200.0, 50.0), // End further right to allow more space
        },
        extend: Extend::Pad,
        stops: ColorStops::from_vec(vec![
            ColorStop {
                offset: 0.5,
                color: Color::rgb(green[0] as f64, green[1] as f64, green[2] as f64),
            },
            ColorStop {
                offset: 1.0,
                color: Color::rgb(yellow[0] as f64, yellow[1] as f64, yellow[2] as f64),
            },
        ]),
    };

    button(
        v_stack((
            svg(create_icon(icon_name)).style(|s| s.width(24).height(24).color(Color::BLACK)),
            label(move || text).style(|s| s.margin_top(4.0)),
        ))
        .style(|s| s.justify_center().align_items(AlignItems::Center)),
    )
    .action(action)
    .style(move |s| {
        s.height(100)
            .width(100.0)
            .justify_center()
            .align_items(AlignItems::Center)
            .background(
                Gradient::new_linear(
                    floem::kurbo::Point::new(0.0, 0.0),
                    floem::kurbo::Point::new(100.0, 100.0),
                )
                .with_stops([
                    (0.0, Color::rgba(0.2, 0.4, 0.6, 1.0)), // start color
                    (1.0, Color::rgba(0.4, 0.6, 0.8, 1.0)), // end color
                ]),
            )
            .border(0)
            // .border_color(Color::BLACK)
            .border_radius(15)
            .transition(Background, Transition::ease_in_out(200.millis()))
            .focus_visible(|s| s.border(2.).border_color(Color::BLUE))
            .hover(|s| s.background(Color::LIGHT_GRAY).cursor(CursorStyle::Pointer))
            .z_index(20)
    })
}

pub fn nav_button(
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

pub fn option_button(
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

pub fn layer_button(layer_name: String, icon_name: &'static str) -> impl IntoView {
    // let (visible, set_visible) = create_signal(true);
    // let (locked, set_locked) = create_signal(false);

    h_stack((
        svg(create_icon(icon_name))
            .style(|s| s.width(24).height(24).color(Color::BLACK))
            .style(|s| s.margin_right(4.0)),
        label(move || layer_name.to_string()),
        // toggle(visible, set_visible)
        //     .style(Style::BASE.margin_left(8))
        //     .on_change(|new_state| {
        //         println!("Layer visibility changed to: {}", new_state);
        //     }),
        // svg(include_str!("assets/lock_icon.svg"))
        //     .style(Style::BASE.width(16).height(16).margin_left(8))
        //     .on_click(move |_| {
        //         set_locked.update(|locked| !*locked);
        //         println!("Layer locked state changed to: {}", locked.get());
        //     }),
    ))
    .style(|s| {
        s.align_items(AlignItems::Center)
            .padding_vert(8)
            .background(Color::rgb(255.0, 239.0, 194.0))
            .border_bottom(1)
            .border_color(Color::rgb(200.0, 200.0, 200.0))
            .border_radius(4)
            .hover(|s| s.background(Color::rgb(222.0, 206.0, 160.0)))
            .active(|s| s.background(Color::rgb(237.0, 218.0, 164.0)))
    })
    .on_click(|_| {
        println!("Layer selected");
        EventPropagation::Stop
    })
}
