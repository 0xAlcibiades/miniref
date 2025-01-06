//! Server-side API functions for note operations
//!
//! This module contains server functions that handle communication between
//! the client-side UI and the backend API. These functions are automatically
//! transformed by Leptos into client-side functions that make API requests.

use crate::note::Note;
use leptos::prelude::ServerFnError;
use leptos::server;

/// Fetches all available notes from the API.
///
/// This function is marked with the #[server] attribute, which means Leptos will:
/// 1. Run this implementation on the server during SSR
/// 2. Generate a client-side version that makes the API request
///
/// # Returns
/// - `Ok(Vec<Note>)` - A list of all notes if successful
/// - `Err(ServerFnError)` - If any step of the request fails:
///   - Network errors during the request
///   - Non-200 status codes from the API
///   - JSON deserialization errors
#[server(GetNotes)]
pub async fn get_notes() -> Result<Vec<Note>, ServerFnError<String>> {
    // Create a reusable HTTP client
    let client = reqwest::Client::new();

    // Make the request to the notes API endpoint
    let response = client
        .get("http://127.0.0.1:3000/api/notes")
        .send()
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?
        // Ensure we got a successful status code
        .error_for_status()
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?;

    // Parse the JSON response into our Note type
    response
        .json()
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}

/// Fetches a specific note by ID from the API.
///
/// This function is marked with the #[server] attribute, which means Leptos will:
/// 1. Run this implementation on the server during SSR
/// 2. Generate a client-side version that makes the API request
///
/// # Arguments
/// * `id` - The unique identifier of the note to fetch
///
/// # Returns
/// - `Ok(Note)` - The requested note if found
/// - `Err(ServerFnError)` - If any step of the request fails:
///   - Network errors during the request
///   - Non-200 status codes from the API (including 404)
///   - JSON deserialization errors
#[server(GetNote)]
pub async fn get_note(id: String) -> Result<Note, ServerFnError<String>> {
    let client = reqwest::Client::new();

    // Make the request to the specific note's API endpoint
    let response = client
        .get(format!("http://127.0.0.1:3000/api/notes/{}", id))
        .send()
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?;

    // Check the status code before trying to parse the response
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(ServerFnError::ServerError("Note not found".to_string()));
    }

    // Handle other error status codes
    if !response.status().is_success() {
        return Err(ServerFnError::ServerError(format!(
            "API error: {}",
            response.status()
        )));
    }

    // Parse the JSON response into our Note type
    response
        .json()
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}
