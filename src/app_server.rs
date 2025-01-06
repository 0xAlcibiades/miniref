use crate::note::Note;
use leptos::prelude::ServerFnError;
use leptos::server;

#[server(GetNotes)]
pub async fn get_notes() -> Result<Vec<Note>, ServerFnError<String>> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:3000/api/notes")
        .send()
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?
        .error_for_status()
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?;

    response
        .json()
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}

#[server(GetNote)]
pub async fn get_note(id: String) -> Result<Note, ServerFnError<String>> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:3000/api/notes/{}", id))
        .send()
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?
        .error_for_status()
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?;

    response
        .json()
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}
