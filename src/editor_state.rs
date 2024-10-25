use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use common_vector::basic::wgpu_to_human;
use common_vector::editor::InputValue;
use common_vector::{basic::string_to_f32, editor::Editor};
use floem::keyboard::ModifiersState;
use floem::reactive::{RwSignal, SignalUpdate};
use undo::Edit;
use undo::Record;
use uuid::Uuid;

// Define all possible edit operations
#[derive(Debug)]
pub enum PolygonProperty {
    Width(f32),
    Height(f32),
    // Position(f32, f32),
    Red(f32),
    Green(f32),
    Blue(f32),
    BorderRadius(f32),
    StrokeThickness(f32),
    StrokeRed(f32),
    StrokeGreen(f32),
    StrokeBlue(f32),
}

#[derive(Debug)]
pub struct PolygonEdit {
    pub polygon_id: Uuid,
    pub field_name: String,
    pub old_value: PolygonProperty,
    pub new_value: PolygonProperty,
    pub signal: RwSignal<String>,
    // editor: Arc<Mutex<Editor>>,
}

impl Edit for PolygonEdit {
    type Target = RecordState;
    type Output = ();

    fn edit(&mut self, record_state: &mut RecordState) {
        let mut editor = record_state.editor.lock().unwrap();
        match &self.new_value {
            PolygonProperty::Width(w) => {
                editor.update_polygon(self.polygon_id, "width", InputValue::Number(*w));

                let mut width = w.to_string();
                self.signal.set(width);
            }
            PolygonProperty::Height(h) => {
                editor.update_polygon(self.polygon_id, "height", InputValue::Number(*h));

                let mut height = h.to_string();
                self.signal.set(height);
            }
            PolygonProperty::Red(h) => {
                editor.update_polygon(self.polygon_id, "red", InputValue::Number(*h));

                let mut red = h.to_string();
                self.signal.set(red);
            }
            PolygonProperty::Green(h) => {
                editor.update_polygon(self.polygon_id, "green", InputValue::Number(*h));

                let mut green = h.to_string();
                self.signal.set(green);
            }
            PolygonProperty::Blue(h) => {
                editor.update_polygon(self.polygon_id, "blue", InputValue::Number(*h));

                let mut blue = h.to_string();
                self.signal.set(blue);
            }
            PolygonProperty::BorderRadius(h) => {
                editor.update_polygon(self.polygon_id, "border_radius", InputValue::Number(*h));

                let mut border_radius = h.to_string();
                self.signal.set(border_radius);
            }
            PolygonProperty::StrokeThickness(h) => {
                editor.update_polygon(self.polygon_id, "stroke_thickness", InputValue::Number(*h));

                let mut stroke_thickness = h.to_string();
                self.signal.set(stroke_thickness);
            }
            PolygonProperty::StrokeRed(h) => {
                editor.update_polygon(self.polygon_id, "stroke_red", InputValue::Number(*h));

                let mut stroke_red = h.to_string();
                self.signal.set(stroke_red);
            }
            PolygonProperty::StrokeGreen(h) => {
                editor.update_polygon(self.polygon_id, "stroke_green", InputValue::Number(*h));

                let mut stroke_green = h.to_string();
                self.signal.set(stroke_green);
            }
            PolygonProperty::StrokeBlue(h) => {
                editor.update_polygon(self.polygon_id, "stroke_blue", InputValue::Number(*h));

                let mut stroke_blue = h.to_string();
                self.signal.set(stroke_blue);
            }
        }
    }

    fn undo(&mut self, record_state: &mut RecordState) {
        let mut editor = record_state.editor.lock().unwrap();

        match &self.old_value {
            PolygonProperty::Width(w) => {
                editor.update_polygon(self.polygon_id, "width", InputValue::Number(*w));

                let mut width = w.to_string();
                self.signal.set(width);
            }
            PolygonProperty::Height(h) => {
                editor.update_polygon(self.polygon_id, "height", InputValue::Number(*h));

                let mut height = h.to_string();
                self.signal.set(height);
            }
            PolygonProperty::Red(h) => {
                // let mut stroke_green = h.to_string();
                let red_human = wgpu_to_human(*h);

                editor.update_polygon(self.polygon_id, "red", InputValue::Number(red_human));

                self.signal.set(red_human.to_string());
            }
            PolygonProperty::Green(h) => {
                // let mut stroke_green = h.to_string();
                let green_human = wgpu_to_human(*h);

                editor.update_polygon(self.polygon_id, "green", InputValue::Number(green_human));

                self.signal.set(green_human.to_string());
            }
            PolygonProperty::Blue(h) => {
                // let mut stroke_green = h.to_string();
                let blue_human = wgpu_to_human(*h);

                editor.update_polygon(self.polygon_id, "blue", InputValue::Number(blue_human));

                self.signal.set(blue_human.to_string());
            }
            PolygonProperty::BorderRadius(h) => {
                editor.update_polygon(self.polygon_id, "border_radius", InputValue::Number(*h));

                let mut border_radius = h.to_string();
                self.signal.set(border_radius);
            }
            PolygonProperty::StrokeThickness(h) => {
                editor.update_polygon(self.polygon_id, "stroke_thickness", InputValue::Number(*h));

                let mut stroke_thickness = h.to_string();
                self.signal.set(stroke_thickness);
            }
            PolygonProperty::StrokeRed(h) => {
                // let mut stroke_red = h.to_string();
                let red_human = wgpu_to_human(*h);

                editor.update_polygon(self.polygon_id, "stroke_red", InputValue::Number(red_human));

                self.signal.set(red_human.to_string());
            }
            PolygonProperty::StrokeGreen(h) => {
                // let mut stroke_green = h.to_string();
                let green_human = wgpu_to_human(*h);

                editor.update_polygon(
                    self.polygon_id,
                    "stroke_green",
                    InputValue::Number(green_human),
                );

                self.signal.set(green_human.to_string());
            }
            PolygonProperty::StrokeBlue(h) => {
                // let mut stroke_blue = h.to_string();
                let blue_human = wgpu_to_human(*h);

                editor.update_polygon(
                    self.polygon_id,
                    "stroke_blue",
                    InputValue::Number(blue_human),
                );

                self.signal.set(blue_human.to_string());
            }
        }
    }
}

pub struct EditorState {
    pub editor: Arc<Mutex<Editor>>,
    pub record: Arc<Mutex<Record<PolygonEdit>>>,
    pub record_state: RecordState,
    pub polygon_selected: bool,
    pub selected_polygon_id: Uuid,
    pub value_signals: Arc<Mutex<HashMap<String, RwSignal<String>>>>,
    pub current_modifiers: ModifiersState,
}

pub struct RecordState {
    pub editor: Arc<Mutex<Editor>>,
    pub record: Arc<Mutex<Record<PolygonEdit>>>,
}

impl EditorState {
    pub fn new(editor: Arc<Mutex<Editor>>) -> Self {
        let record = Arc::new(Mutex::new(Record::new()));
        Self {
            editor: Arc::clone(&editor),
            record: Arc::clone(&record),
            record_state: RecordState {
                editor: Arc::clone(&editor),
                record: Arc::clone(&record),
            },
            polygon_selected: false,
            selected_polygon_id: Uuid::nil(),
            value_signals: Arc::new(Mutex::new(HashMap::new())),
            current_modifiers: ModifiersState::empty(),
        }
    }

    // Helper method to register a new signal
    pub fn register_signal(&mut self, name: String, signal: RwSignal<String>) {
        let mut signals = self.value_signals.lock().unwrap();
        signals.insert(name + &self.selected_polygon_id.to_string(), signal);
    }

    pub fn update_width(&mut self, new_width_str: &str) -> Result<(), String> {
        let new_width =
            string_to_f32(new_width_str).map_err(|_| "Couldn't convert string to f32")?;

        let old_width = {
            let editor = self.record_state.editor.lock().unwrap();
            editor.get_polygon_width(self.selected_polygon_id)
        };

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::Width(old_width),
            new_value: PolygonProperty::Width(new_width),
            field_name: "width".to_string(),
            signal: self
                .value_signals
                .lock()
                .unwrap()
                .get(&format!("width{}", self.selected_polygon_id))
                .cloned()
                .expect("Couldn't get width value signal"),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_height(&mut self, new_height_str: &str) -> Result<(), String> {
        let new_height =
            string_to_f32(new_height_str).map_err(|_| "Couldn't convert string to f32")?;

        let old_height = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_height(self.selected_polygon_id)
        };

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::Height(old_height),
            new_value: PolygonProperty::Height(new_height),
            field_name: "height".to_string(),
            signal: self
                .value_signals
                .lock()
                .unwrap()
                .get(&format!("height{}", self.selected_polygon_id))
                .cloned()
                .expect("Couldn't get width value signal"),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_red(&mut self, new_red_str: &str) -> Result<(), String> {
        let new_red = string_to_f32(new_red_str).map_err(|_| "Couldn't convert string to f32")?;

        let old_red = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_red(self.selected_polygon_id)
        };

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::Red(old_red),
            new_value: PolygonProperty::Red(new_red),
            field_name: "red".to_string(),
            signal: self
                .value_signals
                .lock()
                .unwrap()
                .get(&format!("red{}", self.selected_polygon_id))
                .cloned()
                .expect("Couldn't get width value signal"),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_green(&mut self, new_green_str: &str) -> Result<(), String> {
        let new_green =
            string_to_f32(new_green_str).map_err(|_| "Couldn't convert string to f32")?;

        let old_green = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_green(self.selected_polygon_id)
        };

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::Green(old_green),
            new_value: PolygonProperty::Green(new_green),
            field_name: "green".to_string(),
            signal: self
                .value_signals
                .lock()
                .unwrap()
                .get(&format!("green{}", self.selected_polygon_id))
                .cloned()
                .expect("Couldn't get green value signal"),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_blue(&mut self, new_blue_str: &str) -> Result<(), String> {
        let new_blue = string_to_f32(new_blue_str).map_err(|_| "Couldn't convert string to f32")?;

        let old_blue = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_blue(self.selected_polygon_id)
        };

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::Blue(old_blue),
            new_value: PolygonProperty::Blue(new_blue),
            field_name: "blue".to_string(),
            signal: self
                .value_signals
                .lock()
                .unwrap()
                .get(&format!("blue{}", self.selected_polygon_id))
                .cloned()
                .expect("Couldn't get blue value signal"),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_border_radius(&mut self, new_border_radius_str: &str) -> Result<(), String> {
        let new_border_radius = string_to_f32(new_border_radius_str)
            .map_err(|_| "Couldn't convert string to height")?;

        let old_border_radius = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_border_radius(self.selected_polygon_id)
        };

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::BorderRadius(old_border_radius),
            new_value: PolygonProperty::BorderRadius(new_border_radius),
            field_name: "border_radius".to_string(),
            signal: self
                .value_signals
                .lock()
                .unwrap()
                .get(&format!("border_radius{}", self.selected_polygon_id))
                .cloned()
                .expect("Couldn't get border_radius value signal"),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_stroke_thickness(
        &mut self,
        new_stroke_thickness_str: &str,
    ) -> Result<(), String> {
        let new_stroke_thickness = string_to_f32(new_stroke_thickness_str)
            .map_err(|_| "Couldn't convert string to height")?;

        let old_stroke_thickness = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_stroke_thickness(self.selected_polygon_id)
        };

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::StrokeThickness(old_stroke_thickness),
            new_value: PolygonProperty::StrokeThickness(new_stroke_thickness),
            field_name: "stroke_thickness".to_string(),
            signal: self
                .value_signals
                .lock()
                .unwrap()
                .get(&format!("stroke_thickness{}", self.selected_polygon_id))
                .cloned()
                .expect("Couldn't get stroke_thickness value signal"),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_stroke_red(&mut self, new_stroke_red_str: &str) -> Result<(), String> {
        let new_stroke_red =
            string_to_f32(new_stroke_red_str).map_err(|_| "Couldn't convert string to height")?;

        let old_stroke_red = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_stroke_red(self.selected_polygon_id)
        };

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::StrokeRed(old_stroke_red),
            new_value: PolygonProperty::StrokeRed(new_stroke_red),
            field_name: "stroke_red".to_string(),
            signal: self
                .value_signals
                .lock()
                .unwrap()
                .get(&format!("stroke_red{}", self.selected_polygon_id))
                .cloned()
                .expect("Couldn't get stroke_red value signal"),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_stroke_green(&mut self, new_stroke_green_str: &str) -> Result<(), String> {
        let new_stroke_green =
            string_to_f32(new_stroke_green_str).map_err(|_| "Couldn't convert string to height")?;

        let old_stroke_green = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_stroke_green(self.selected_polygon_id)
        };

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::StrokeGreen(old_stroke_green),
            new_value: PolygonProperty::StrokeGreen(new_stroke_green),
            field_name: "stroke_green".to_string(),
            signal: self
                .value_signals
                .lock()
                .unwrap()
                .get(&format!("stroke_green{}", self.selected_polygon_id))
                .cloned()
                .expect("Couldn't get stroke_green value signal"),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn update_stroke_blue(&mut self, new_stroke_blue_str: &str) -> Result<(), String> {
        let new_stroke_blue =
            string_to_f32(new_stroke_blue_str).map_err(|_| "Couldn't convert string to height")?;

        let old_stroke_blue = {
            let editor = self.editor.lock().unwrap();
            editor.get_polygon_stroke_blue(self.selected_polygon_id)
        };

        let edit = PolygonEdit {
            polygon_id: self.selected_polygon_id,
            old_value: PolygonProperty::StrokeBlue(old_stroke_blue),
            new_value: PolygonProperty::StrokeBlue(new_stroke_blue),
            field_name: "stroke_blue".to_string(),
            signal: self
                .value_signals
                .lock()
                .unwrap()
                .get(&format!("stroke_blue{}", self.selected_polygon_id))
                .cloned()
                .expect("Couldn't get stroke_blue value signal"),
        };

        let mut record = self.record.lock().unwrap();
        record.edit(&mut self.record_state, edit);

        Ok(())
    }

    pub fn undo(&mut self) {
        let mut record = self.record.lock().unwrap();

        if record.undo(&mut self.record_state).is_some() {
            println!("Undo successful");
            // println!("record cannB... {:?}", self.record.head());
        }
    }

    pub fn redo(&mut self) {
        let mut record = self.record.lock().unwrap();

        if record.redo(&mut self.record_state).is_some() {
            println!("Redo successful");
        }
    }
}
