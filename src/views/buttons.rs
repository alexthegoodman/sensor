use std::borrow::{Borrow, BorrowMut};
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};
use std::usize;

use common_vector::basic::rgb_to_wgpu;
use floem::common::create_icon;
use floem::event::{Event, EventListener, EventPropagation};
use floem::kurbo::Point;
use floem::peniko::{Brush, Color, ColorStop, ColorStops, Extend, Gradient, GradientKind};
use floem::reactive::RwSignal;
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
use uuid::Uuid;

use floem::reactive::SignalGet;
use floem::reactive::SignalUpdate;

use super::tools_panel::Layer;

pub fn sortable_item(
    editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    sortable_items: RwSignal<Vec<Layer>>,
    dragger_id: RwSignal<Uuid>,
    item_id: Uuid,
    layer_name: String,
    icon_name: &'static str,
) -> impl IntoView {
    h_stack((
        svg(create_icon(icon_name))
            .style(|s| s.width(24).height(24).color(Color::BLACK))
            .style(|s| s.margin_right(7.0))
            .on_event_stop(
                floem::event::EventListener::PointerDown,
                |_| { /* Disable dragging for this view */ },
            ),
        label(move || layer_name.to_string())
            .style(|s| s.selectable(false).cursor(CursorStyle::RowResize)),
    ))
    .style(|s| s.selectable(false).cursor(CursorStyle::RowResize))
    .draggable()
    .on_event(floem::event::EventListener::DragStart, move |_| {
        dragger_id.set(item_id);
        floem::event::EventPropagation::Continue
    })
    .on_event(floem::event::EventListener::DragOver, move |_| {
        let mut editor = editor.lock().unwrap();
        let dragger_id = dragger_id.get_untracked();
        if dragger_id != item_id {
            let dragger_pos = sortable_items
                .get()
                .iter()
                .position(|layer| layer.instance_id == dragger_id)
                .or_else(|| Some(usize::MAX))
                .expect("Couldn't get dragger_pos");
            let hover_pos = sortable_items
                .get()
                .iter()
                .position(|layer| layer.instance_id == item_id)
                .or_else(|| Some(usize::MAX))
                .expect("Couldn't get hover_pos");

            sortable_items.update(|items| {
                if (dragger_pos <= items.len() && hover_pos <= items.len()) {
                    let item = items.get(dragger_pos).cloned();
                    items.remove(dragger_pos);
                    editor.layer_list.remove(dragger_pos);

                    if let Some(selected_item) = item {
                        items.insert(hover_pos, selected_item.clone());
                        editor
                            .layer_list
                            .insert(hover_pos, selected_item.instance_id);
                    }
                }
            });
        }
        floem::event::EventPropagation::Continue
    })
    .dragging_style(|s| {
        s.box_shadow_blur(3)
            .box_shadow_color(Color::rgba(100.0, 100.0, 100.0, 0.5))
            .box_shadow_spread(2)
    })
    .style(|s| {
        s.width(220.0)
            .border_radius(15.0)
            .align_items(AlignItems::Center)
            .padding_vert(8)
            .background(Color::rgb(255.0, 239.0, 194.0))
            .border_bottom(1)
            .border_color(Color::rgb(200.0, 200.0, 200.0))
            .hover(|s| s.background(Color::rgb(222.0, 206.0, 160.0)))
            .active(|s| s.background(Color::rgb(237.0, 218.0, 164.0)))
    })
    // .on_click(|_| {
    //     println!("Layer selected");
    //     EventPropagation::Stop
    // })
}
