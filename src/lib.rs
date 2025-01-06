//! MiniRef - A digital zettelkasten/note-taking application
//!
//! This application provides a web interface for managing and viewing interconnected notes,
//! implementing a zettelkasten-style system with features like:
//! - Markdown rendering
//! - Syntax highlighting
//! - LaTeX math support
//! - Note linking and references

/// Main application UI components and routing
pub mod app;

/// Note type definitions and storage functionality
pub mod note;

/// Server-side API endpoints and data fetching
mod app_server;

/// Entry point for client-side hydration
///
/// This function is called automatically when the WASM module is loaded in the browser,
/// and is responsible for:
/// 1. Setting up the hydration of server-rendered content
/// 2. Initializing panic handling for better error reporting
/// 3. Mounting the application to the DOM
///
/// The #[cfg(feature = "hydrate")] attribute ensures this code only runs during
/// client-side hydration, not during server-side rendering.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;

    // Set up better panic messages in the browser console
    console_error_panic_hook::set_once();

    // Hydrate the application - this will attach event listeners and
    // set up reactivity for the server-rendered HTML
    leptos::mount::hydrate_body(App);
}
