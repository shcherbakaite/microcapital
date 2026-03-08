mod app;
mod bounds;
mod capital_xml;
mod model;
mod outline;
mod schematic_view;

pub use app::MicroCapitalApp;
pub use model::{Design, DiagramContent, DiagramInfo};

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

/// Web entry point: called from JavaScript to start the app.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");
    let canvas = document
        .get_element_by_id("microcapital_canvas")
        .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
        .expect("canvas element not found");
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async move {
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(MicroCapitalApp::from_storage(cc.storage)))),
            )
            .await
            .expect("failed to start eframe");
    });
}
