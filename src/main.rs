#[cfg(feature = "ssr")]
use axum::extract::{Path, State};
#[cfg(feature = "ssr")]
use axum::Json;
#[cfg(feature = "ssr")]
use http::StatusCode;
#[cfg(feature = "ssr")]
use std::sync::Arc;

#[cfg(feature = "ssr")]
use miniref::note::{Note, NoteStore};

// This contains the server side functionality for the Leptos App
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{routing::get, Router};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use miniref::app::*;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Initialize NoteStore as application state
    let note_store = Arc::new(NoteStore::new("./notes").expect("Failed to init store"));

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    // Create an API router with note_store state
    let api_router = Router::new()
        .route("/notes", get(list_notes_handler))
        .route("/notes/:id", get(get_note_handler))
        .with_state(note_store);

    // Create the main app router with leptos_options state
    let app = Router::new()
        .nest("/api", api_router)
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// Add these handler functions in main.rs or a separate handlers.rs file
#[cfg(feature = "ssr")]
async fn list_notes_handler(State(store): State<Arc<NoteStore>>) -> Json<Vec<Note>> {
    Json(store.list_notes().expect("Failed to load notes"))
}

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

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
