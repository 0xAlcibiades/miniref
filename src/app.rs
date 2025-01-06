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

// Application-wide constants
const APP_TITLE: &str = "MiniRef";
const APP_SUBTITLE: &str = "Digital Zettelkasten";

/// Skeleton loader for note cards that mimics the structure of a real note card
/// to prevent layout shift during loading
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

/// Skeleton loader for a full note page
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
            <div class="tags">
                <span class="tag opacity-20 bg-gray-50 animate-pulse w-20"></span>
                <span class="tag opacity-20 bg-gray-50 animate-pulse w-24"></span>
                <span class="tag opacity-20 bg-gray-50 animate-pulse w-16"></span>
            </div>
            <div class="note-content">
                <div class="space-y-6 mt-6">
                    <div class="h-4 opacity-20 bg-gray-50 animate-pulse rounded w-full"></div>
                    <div class="h-4 opacity-20 bg-gray-50 animate-pulse rounded w-5/6"></div>
                    <div class="h-4 opacity-20 bg-gray-50 animate-pulse rounded w-4/6"></div>
                    <div class="h-4 opacity-20 bg-gray-50 animate-pulse rounded w-full"></div>
                    <div class="h-4 opacity-20 bg-gray-50 animate-pulse rounded w-3/4"></div>
                </div>
            </div>
            <div class="references">
                <h3>"References"</h3>
                <div class="flex gap-2">
                    <span class="reference opacity-20 bg-gray-50 animate-pulse rounded w-32 h-4 block"></span>
                    <span class="reference opacity-20 bg-gray-50 animate-pulse rounded w-32 h-4 block"></span>
                </div>
            </div>
        </div>
    }
}

/// Reusable note card component for displaying a note preview
/// in the grid of notes on the home page
#[component]
fn NoteCard(
    /// The note data to display
    note: Note,
) -> impl IntoView {
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

/// The outer shell for the Leptos App that provides the HTML structure
/// and includes necessary scripts and stylesheets
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

/// Main application component that sets up routing and global context
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

/// Home page component that displays a grid of all notes
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

/// Parameters for the note page route
#[derive(Debug, Clone, Params, PartialEq)]
struct NoteParams {
    note_id: String,
}

/// Individual note page component that displays a single note's full content
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

    // Create a node reference for our content div
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

    view! {
        <div class="folio">
            <Suspense
                fallback=move || view! { <NotePageSkeleton/> }
            >
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
                                <header class="note-header">
                                    <span class="note-id">{note.id}</span>
                                    <h1 class="note-title">{note.title}</h1>
                                </header>
                                <div class="tags">
                                    {note.tags.into_iter().map(|tag| {
                                        view! { <span class="tag">{tag}</span> }
                                    }).collect_view()}
                                </div>
                                <div class="note-content" node_ref=content_ref inner_html=note.content/>
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
