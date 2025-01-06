//! Server-side entry point for the MiniRef application.
//! Configures and starts the web server with routing for both the API and SSR components.

// Import server-side dependencies when the "ssr" feature is enabled
#[cfg(feature = "ssr")]
use axum::extract::{Path, State};
#[cfg(feature = "ssr")]
use axum::Json;
#[cfg(feature = "ssr")]
use http::StatusCode;
#[cfg(feature = "ssr")]
use std::sync::Arc;

// Import our Note-related types for the server
#[cfg(feature = "ssr")]
use miniref::note::{Note, NoteStore};

/// Server entry point - sets up and runs the web server with both API and SSR routes
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{routing::get, Router};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use miniref::app::*;

    // Load application configuration
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Initialize the note store which provides access to our notes directory
    let note_store = Arc::new(NoteStore::new("./notes").expect("Failed to init store"));

    // Generate routes from our Leptos App component
    let routes = generate_route_list(App);

    // Create a router for our REST API endpoints
    let api_router = Router::new()
        .route("/notes", get(list_notes_handler)) // GET /api/notes - List all notes
        .route("/notes/:id", get(get_note_handler)) // GET /api/notes/:id - Get a specific note
        .with_state(note_store);

    // Create the main application router that handles both API and SSR routes
    let app = Router::new()
        // Nest our API routes under /api
        .nest("/api", api_router)
        // Add routes for server-side rendered pages
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        // Add a fallback handler for unmatched routes
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // Start the server
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// API handler for listing all notes
///
/// Returns a JSON array of all notes in the store
#[cfg(feature = "ssr")]
async fn list_notes_handler(State(store): State<Arc<NoteStore>>) -> Json<Vec<Note>> {
    Json(store.list_notes().expect("Failed to load notes"))
}

/// API handler for getting a specific note by ID
///
/// Returns:
/// - 200 OK with note JSON if found
/// - 404 Not Found if note doesn't exist
#[cfg(feature = "ssr")]
async fn get_note_handler(
    State(store): State<Arc<NoteStore>>,
    Path(note_id): Path<String>,
) -> Result<Json<Note>, StatusCode> {
    match store.get_note(&note_id).expect("Failed to load note") {
        Some(note) => Ok(Json(note)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Client-side entry point (disabled when using SSR)
///
/// This is left empty as we use hydration from lib.rs instead.
/// Could be implemented for client-only testing with tools like Trunk.
#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
