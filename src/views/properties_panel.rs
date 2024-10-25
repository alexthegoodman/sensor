use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;
use wgpu::util::DeviceExt;

use common_vector::basic::{
    color_to_wgpu, rgb_to_wgpu, string_to_f32, wgpu_to_human, Point, WindowSize,
};
use common_vector::editor::{self, Editor, InputValue, Viewport};
use common_vector::polygon::{Polygon, PolygonConfig};
use floem::peniko::{Brush, Color};
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::reactive::{SignalGet, SignalUpdate};
use floem::text::Weight;
use floem::views::Decorators;
use floem::views::{container, dyn_container, empty, label};
use floem::views::{h_stack, v_stack};
use floem::GpuHelper;
use floem::IntoView;

use crate::editor_state::{self, EditorState};

use super::buttons::small_button;
use super::inputs::styled_input;

pub fn properties_view(
    editor_state: Arc<Mutex<EditorState>>,
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
    let editor_cloned8 = Arc::clone(&editor);
    let editor_cloned9 = Arc::clone(&editor);
    let editor_cloned10 = Arc::clone(&editor);
    let editor_cloned11 = Arc::clone(&editor);

    let editor_state2 = Arc::clone(&editor_state);
    let editor_state3 = Arc::clone(&editor_state);
    let editor_state4 = Arc::clone(&editor_state);
    let editor_state5 = Arc::clone(&editor_state);
    let editor_state6 = Arc::clone(&editor_state);
    let editor_state7 = Arc::clone(&editor_state);
    let editor_state8 = Arc::clone(&editor_state);
    let editor_state9 = Arc::clone(&editor_state);
    let editor_state10 = Arc::clone(&editor_state);
    let editor_state11 = Arc::clone(&editor_state);
    let editor_state12 = Arc::clone(&editor_state);
    let editor_state13 = Arc::clone(&editor_state);
    let editor_state14 = Arc::clone(&editor_state);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    // let mut value_map: HashMap<String, RwSignal<String>> = HashMap::new();

    // println!("Create new value map...");

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
                        let mut editor_state = editor_state2.lock().unwrap();
                        editor_state.selected_polygon_id = Uuid::nil();
                        editor_state.polygon_selected = false;
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
                    move |mut editor_state, value| {
                        editor_state.update_width(&value);
                    }
                }),
                editor_state6,
                "width".to_string(),
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
                    move |mut editor_state, value| {
                        editor_state.update_height(&value);
                    }
                }),
                editor_state7,
                "height".to_string(),
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
                &wgpu_to_human(selected_polygon_data.read().borrow().fill[0]).to_string(),
                "0-255",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state.update_red(&value);
                    }
                }),
                editor_state8,
                "red".to_string(),
            )
            .style(move |s| s.width(thirds).margin_right(5.0)),
            styled_input(
                "Green:".to_string(),
                &wgpu_to_human(selected_polygon_data.read().borrow().fill[1]).to_string(),
                "0-255",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state.update_green(&value);
                    }
                }),
                editor_state9,
                "green".to_string(),
            )
            .style(move |s| s.width(thirds).margin_right(5.0)),
            styled_input(
                "Blue:".to_string(),
                &wgpu_to_human(selected_polygon_data.read().borrow().fill[2]).to_string(),
                "0-255",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state.update_blue(&value);
                    }
                }),
                editor_state10,
                "blue".to_string(),
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
                move |mut editor_state, value| {
                    editor_state.update_border_radius(&value);
                }
            }),
            editor_state11,
            "border_radius".to_string(),
        ),
        label(|| "Stroke").style(|s| s.margin_bottom(5.0)),
        h_stack((
            styled_input(
                "Thickness:".to_string(),
                &selected_polygon_data
                    .read()
                    .borrow()
                    .stroke
                    .thickness
                    .to_string(),
                "Enter thickness",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state.update_stroke_thickness(&value);
                    }
                }),
                editor_state12,
                "stroke_thickness".to_string(),
            )
            .style(move |s| s.width(quarters).margin_right(5.0)),
            styled_input(
                "Red:".to_string(),
                &wgpu_to_human(selected_polygon_data.read().borrow().stroke.fill[0]).to_string(),
                "Enter red",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state.update_stroke_red(&value);
                    }
                }),
                editor_state13,
                "stroke_red".to_string(),
            )
            .style(move |s| s.width(quarters).margin_right(5.0)),
            styled_input(
                "Green:".to_string(),
                &wgpu_to_human(selected_polygon_data.read().borrow().stroke.fill[1]).to_string(),
                "Enter green",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state.update_stroke_green(&value);
                    }
                }),
                editor_state14,
                "stroke_green".to_string(),
            )
            .style(move |s| s.width(quarters).margin_right(5.0)),
            styled_input(
                "Blue:".to_string(),
                &wgpu_to_human(selected_polygon_data.read().borrow().stroke.fill[2]).to_string(),
                "Enter blue",
                Box::new({
                    move |mut editor_state, value| {
                        editor_state.update_stroke_blue(&value);
                    }
                }),
                editor_state,
                "stroke_blue".to_string(),
            )
            .style(move |s| s.width(quarters)),
        ))
        .style(move |s| s.width(aside_width)),
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
