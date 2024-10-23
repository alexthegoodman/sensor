use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::sync::{Arc, Mutex, MutexGuard};

use common_vector::basic::WindowSize;
use common_vector::camera::Camera;
use common_vector::editor::{Editor, Viewport};
use common_vector::polygon::{Polygon, PolygonConfig};
use floem_renderer::gpu_resources::GpuResources;

struct Handler {
    button_handler: RefCell<Option<Box<dyn Fn(MutexGuard<'_, Editor>) + Send + 'static>>>,
}

// impl Handler {
//     pub fn new() -> Self {
//         Handler {
//             button_handler: RefCell::new(None),
//         }
//     }
//     pub fn set_button_handler(
//         &mut self,
//         gpu_resources: Arc<GpuResources>,
//         // window_size: WindowSize,
//         viewport: Arc<Mutex<Viewport>>,
//         polygon_config: PolygonConfig,
//         camera: &Camera,
//     ) {
//         let handler = Box::new(move |mut editor: MutexGuard<'_, Editor>| {
//             println!("Button clicked, attempting to add polygon...");
//             let viewport = viewport.lock().unwrap();
//             let window_size = WindowSize {
//                 width: viewport.width as u32,
//                 height: viewport.height as u32,
//             };
//             editor.polygons.push(Polygon::new(
//                 &window_size,
//                 &gpu_resources.device,
//                 camera,
//                 polygon_config.points.clone(),
//                 polygon_config.dimensions,
//                 polygon_config.position,
//                 polygon_config.border_radius,
//                 polygon_config.fill,
//             ));
//             println!("Polygon added successfully.");
//         });
//         self.button_handler.replace(Some(handler));
//     }

//     pub fn handle_button_click(&mut self, editor: MutexGuard<'_, Editor>) {
//         // Step 1: Check if the handler exists and clone it if it does
//         let handle = self.button_handler.borrow();
//         let handler_option = handle.as_ref();

//         // Step 2: If we have a handler, call it
//         if let Some(handler) = handler_option {
//             handler(editor);
//         } else {
//             println!("Button handler not set.");
//         }
//     }
// }
