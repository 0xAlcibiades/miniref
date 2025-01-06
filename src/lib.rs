pub mod app;
pub mod note;

mod app_server;

// This will hydrate the app when the wasm module is loaded in the browser
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
