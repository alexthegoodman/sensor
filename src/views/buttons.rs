use std::borrow::{Borrow, BorrowMut};
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};

use floem::event::{Event, EventListener, EventPropagation};
use floem::peniko::Color;
use floem::style::{Background, CursorStyle, Transition};
use floem::taffy::AlignItems;
use floem::views::Decorators;
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    VirtualDirection, VirtualItemSize,
};
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
