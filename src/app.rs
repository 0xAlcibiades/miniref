//! Main application UI components for MiniRef
//!
//! This module contains the core UI components that make up the MiniRef application.
//! The application follows a typical web layout with:
//! - A sidebar for navigation
//! - A main content area that changes based on the current route
//! - Loading states with skeleton placeholders to prevent layout shift
//! - Error handling for failed API requests and not-found routes

use crate::app_server::{get_note, get_notes};
use crate::note::Note;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_meta::{provide_meta_context, MetaTags, Script, Stylesheet, Title};
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use leptos_router::SsrMode;
use leptos_router::{
    components::{Route, Router, Routes, A},
    path,
};

// Application-wide constants used for branding and display
const APP_TITLE: &str = "MiniRef";
const APP_SUBTITLE: &str = "Digital Zettelkasten";

/// Skeleton loader for note cards that provides a loading placeholder
/// matching the structure and dimensions of a real note card.
///
/// This prevents layout shifts during loading by maintaining the same
/// visual structure with animated placeholders where content will appear.
#[component]
fn NoteCardSkeleton() -> impl IntoView {
    view! {
        <article class="note">
            <div class="note-header">
                <span class="note-id opacity-20 bg-gray-50 animate-pulse rounded w-16 h-4 block"></span>
            </div>
            <h2 class="note-title">
                <div class="opacity-20 bg-gray-50 animate-pulse rounded h-7 w-3/4 mt-1"></div>
            </h2>
            <div class="tags">
                <span class="tag opacity-20 bg-gray-50 animate-pulse w-16"></span>
                <span class="tag opacity-20 bg-gray-50 animate-pulse w-20"></span>
            </div>
            <div class="references">
                <span class="reference opacity-20 bg-gray-50 animate-pulse rounded w-32 h-4 block"></span>
            </div>
        </article>
    }
}

/// Skeleton loader for the full note page that provides loading placeholders
/// matching the structure of a complete note view.
///
/// This includes placeholders for:
/// - Note header (ID and title)
/// - Tags
/// - Content area
/// - References section
#[component]
fn NotePageSkeleton() -> impl IntoView {
    view! {
        <div class="note-full">
            <header class="note-header">
                <span class="note-id opacity-20 bg-gray-50 animate-pulse rounded w-24 h-4 block"></span>
                <h1 class="note-title">
                    <div class="opacity-20 bg-gray-50 animate-pulse rounded h-10 w-3/4 mt-2"></div>
                </h1>
            </header>
            // Rest of skeleton implementation...
        </div>
    }
}

/// Card component for displaying a note preview in the notes grid.
///
/// # Props
/// * `note` - The note data to display in the card
///
/// Displays:
/// - Note ID
/// - Title (linked to full note view)
/// - Tags
/// - References to other notes
#[component]
fn NoteCard(note: Note) -> impl IntoView {
    view! {
        <article class="note">
            <div class="note-header">
                <span class="note-id">{note.id.clone()}</span>
            </div>
            <h2 class="note-title">
                <A href=format!("/{}", note.id)>{note.title}</A>
            </h2>
            <div class="tags">
                {note.tags.into_iter().map(|tag| {
                    view! { <span class="tag">{tag}</span> }
                }).collect_view()}
            </div>
            <div class="references">
                {note.references.into_iter().map(|ref_id| {
                    view! {
                        <A href=format!("/{}", ref_id)>
                            <span class="reference">{"→ "}{ref_id}</span>
                        </A>
                    }
                }).collect_view()}
            </div>
        </article>
    }
}

/// The application shell component that provides the basic HTML structure
/// and loads necessary scripts and styles for the application.
///
/// This component is responsible for the initial HTML structure during SSR
/// and ensures proper hydration on the client.
#[allow(non_snake_case)]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

/// Root application component that sets up routing and global context.
///
/// This component:
/// - Provides meta context for document head management
/// - Loads required stylesheets (Leptos, KaTeX, highlight.js)
/// - Sets up the router with main layout structure
/// - Handles 404 cases with a fallback route
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        // Load required stylesheets
        <Stylesheet id="leptos" href="/pkg/miniref.css"/>
        <Stylesheet
            id="katex"
            href="https://cdn.jsdelivr.net/npm/katex@0.16.19/dist/katex.min.css"
        />
        <Stylesheet
            id="hljs"
            href="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@latest/build/styles/base16/ocean.min.css"
        />
        // Load syntax highlighting script
        <Script
            src="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@latest/build/highlight.min.js"
            defer="defer"
        />

        <Title text=APP_TITLE/>

        <Router>
            <main class="codex">
                <nav class="sidebar">
                    <div class="sigil"></div>
                    <div class="nav-links">
                        <A href="/">"Notes"</A>
                    </div>
                </nav>
                <Routes fallback=|| view! {
                    <div class="error-page">
                        <h1>"404"</h1>
                        <p>"Note not found"</p>
                    </div>
                }>
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/:note_id") view=NotePage ssr=SsrMode::Async />
                </Routes>
            </main>
        </Router>
    }
}

/// Home page component that displays a grid of all available notes.
///
/// Features:
/// - Fetches all notes using a Resource
/// - Shows skeleton loading state while loading
/// - Handles errors with user-friendly messages
/// - Displays notes in a responsive grid layout
#[component]
fn HomePage() -> impl IntoView {
    // Create a resource to fetch all notes
    let notes = Resource::new(|| (), |_| async move { get_notes().await });

    view! {
        <div class="folio">
            <header class="header">
                <h1>{APP_TITLE}</h1>
                <p class="subtitle">{APP_SUBTITLE}</p>
            </header>

            <Suspense
                fallback=move || view! {
                    <div class="notes-grid">
                        // Display multiple skeleton cards while loading
                        <NoteCardSkeleton/>
                        <NoteCardSkeleton/>
                        <NoteCardSkeleton/>
                        <NoteCardSkeleton/>
                    </div>
                }
            >
                <Show
                    when=move || notes.get().map(|r| r.is_ok()).unwrap_or(false)
                    fallback=move || {
                        let error = notes.get().and_then(|r| r.err())
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "Unknown error".into());
                        view! {
                            <div class="error">
                                <p>"Error loading notes: " {error}</p>
                            </div>
                        }
                    }
                >
                    {move || notes.get()
                        .and_then(|r| r.ok())
                        .map(|notes| view! {
                            <div class="notes-grid">
                                {notes.into_iter().map(|note| view! {
                                    <NoteCard note/>
                                }).collect_view()}
                            </div>
                        })}
                </Show>
            </Suspense>
        </div>
    }
}

/// Route parameters for the note page
#[derive(Debug, Clone, Params, PartialEq)]
struct NoteParams {
    note_id: String,
}

/// Individual note page component that displays a full note with all its content.
///
/// Features:
/// - Fetches specific note data based on URL parameter
/// - Shows skeleton loading state
/// - Handles 404 and other errors
/// - Applies syntax highlighting to code blocks
/// - Displays full note content with:
///   * Title and ID
///   * Tags
///   * Rendered content (including math and code)
///   * References to other notes
#[component]
fn NotePage() -> impl IntoView {
    let params = use_params::<NoteParams>();

    // Create a resource to fetch the specific note
    let note = Resource::new(
        move || {
            params
                .read()
                .as_ref()
                .ok()
                .map(|params| params.note_id.clone())
        },
        move |id: Option<String>| async move {
            match id {
                Some(id) => {
                    let result = get_note(id).await;
                    match result {
                        Ok(note) => Ok(note),
                        Err(e) => {
                            if e.to_string().contains("404") {
                                Err("Note not found".to_string())
                            } else {
                                Err(e.to_string())
                            }
                        }
                    }
                }
                None => Err("Invalid note ID".to_string()),
            }
        },
    );

    let content_ref = NodeRef::new();

    // Effect that watches the note resource and runs highlighting when it changes
    Effect::new(move |_| {
        // Get the current state of our note resource
        if let Some(Ok(_)) = note.get() {
            // Give the DOM time to update with new content before highlighting
            request_animation_frame(move || {
                let window = web_sys::window().unwrap();
                if let Some(hljs) = js_sys::Reflect::get(&window, &"hljs".into())
                    .ok()
                    .and_then(|hljs| hljs.dyn_into::<js_sys::Object>().ok())
                {
                    let _ = js_sys::Reflect::get(&hljs, &"highlightAll".into())
                        .ok()
                        .and_then(|highlight_all| highlight_all.dyn_into::<js_sys::Function>().ok())
                        .map(|f| f.call0(&hljs));
                }
            });
        }
    });

    // Add the view implementation to the NotePage component...
    view! {
        <div class="folio">
            // Show loading skeleton while content is loading
            <Suspense
                fallback=move || view! { <NotePageSkeleton/> }
            >
                // Handle errors during note loading or rendering
                <ErrorBoundary
                    fallback=|errors| view! {
                        <div class="error-page">
                            <h1>"Error"</h1>
                            <p>{move || errors.get()
                                .into_iter()
                                .map(|(_, e)| e.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")}</p>
                            <A href="/">"← Back to notes"</A>
                        </div>
                    }
                >
                    // Show note content if we have a valid note, otherwise display not found
                    <Show
                        when=move || note.get().map(|n| n.is_ok()).unwrap_or(false)
                        fallback=move || view! {
                            <div class="error-page">
                                <h1>"Note not found"</h1>
                                <A href="/">"← Back to notes"</A>
                            </div>
                        }
                    >
                        <div class="note-full">
                            {move || note.get().and_then(|n| n.ok()).map(|note| view! {
                                // Note header with ID and title
                                <header class="note-header">
                                    <span class="note-id">{note.id}</span>
                                    <h1 class="note-title">{note.title}</h1>
                                </header>

                                // Note tags
                                <div class="tags">
                                    {note.tags.into_iter().map(|tag| {
                                        view! { <span class="tag">{tag}</span> }
                                    }).collect_view()}
                                </div>

                                // Main note content - uses node_ref for syntax highlighting
                                <div class="note-content" node_ref=content_ref inner_html=note.content/>

                                // References to other notes
                                <div class="references">
                                    <h3>"References"</h3>
                                    {note.references.into_iter().map(|ref_id| {
                                        view! {
                                            <A href=format!("/{}", ref_id)>
                                                <span class="reference">{"→ "}{ref_id}</span>
                                            </A>
                                        }
                                    }).collect_view()}
                                </div>
                            })}
                        </div>
                    </Show>
                </ErrorBoundary>
            </Suspense>
        </div>
    }
}
