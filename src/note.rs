//! Note management module for MiniRef
//!
//! This module provides the core functionality for storing, processing, and retrieving notes.
//! It includes:
//! - File-based note storage with YAML frontmatter
//! - Markdown processing with syntax highlighting
//! - LaTeX math rendering (both inline and display)
//! - Asset management for note attachments
//! - In-memory caching with file modification tracking

use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use {
    gray_matter::{engine::YAML, Matter}, // For YAML frontmatter parsing
    katex::{render_with_opts, Opts},     // For LaTeX math rendering
    markdown::{to_html_with_options, Options as MarkdownOptions}, // For Markdown processing
    parking_lot::RwLock,
    regex::Regex, // For pattern matching
    std::collections::HashMap,
    std::path::{Path, PathBuf}, // For filesystem operations
    std::time::SystemTime,
    syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet}, // For syntax highlighting
};

/// Represents a complete note with all its metadata and content.
///
/// This struct is used both for storing notes and transmitting them between
/// the server and client. It includes all note data including content and assets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    /// Unique identifier for the note
    pub id: String,
    /// Display title of the note
    pub title: String,
    /// Main content of the note (rendered HTML)
    #[serde(default)]
    pub content: String,
    /// List of tags associated with the note
    #[serde(default)]
    pub tags: Vec<String>,
    /// List of IDs of other notes this note references
    #[serde(default)]
    pub references: Vec<String>,
    /// List of files/attachments associated with this note
    #[serde(default)]
    pub assets: Vec<Asset>,
}

/// Represents a file or attachment associated with a note.
///
/// Assets are stored in a directory alongside their parent note
/// and can include images, PDFs, or other file types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    /// Filesystem path to the asset
    pub path: String,
    /// Display name of the asset
    pub name: String,
    /// MIME type of the asset (e.g., "image/png")
    pub mime_type: String,
}

/// Lightweight version of Note containing only metadata.
///
/// Used for operations where the full note content isn't needed,
/// such as listing notes or displaying previews.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub references: Vec<String>,
}

/// Cached version of a processed note along with its metadata
#[cfg(feature = "ssr")]
struct CachedNote {
    /// The processed note
    note: Note,
    /// Last modified time of the source file when this cache entry was created
    last_modified: SystemTime,
}

/// Manages the storage, processing, and caching of notes.
///
/// The NoteStore handles all file operations and content processing,
/// including:
/// - File reading and writing
/// - YAML frontmatter parsing
/// - Markdown rendering
/// - Syntax highlighting
/// - LaTeX math rendering
/// - Asset management
/// - Caching of processed notes
#[cfg(feature = "ssr")]
pub struct NoteStore {
    /// Root directory where notes are stored
    root_path: PathBuf,
    /// Collection of syntax definitions for code highlighting
    syntax_set: SyntaxSet,
    /// Collection of color themes for syntax highlighting
    theme_set: ThemeSet,
    /// Cache of processed notes, protected by a read-write lock
    note_cache: RwLock<HashMap<String, CachedNote>>,
}

#[cfg(feature = "ssr")]
impl NoteStore {
    /// Creates a new NoteStore at the specified path.
    ///
    /// # Arguments
    /// * `path` - Directory path where notes will be stored
    ///
    /// # Returns
    /// * `Result<Self, std::io::Error>` - New NoteStore instance or IO error
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let root_path = path.as_ref().to_path_buf();
        // Create the notes directory if it doesn't exist
        std::fs::create_dir_all(&root_path)?;

        Ok(Self {
            root_path,
            // Load default syntax highlighting definitions
            syntax_set: SyntaxSet::load_defaults_newlines(),
            // Load default color themes
            theme_set: ThemeSet::load_defaults(),
            // Initialize empty cache
            note_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Gets the last modified time for a file
    ///
    /// # Arguments
    /// * `path` - Path to the file to check
    ///
    /// # Returns
    /// * `Result<SystemTime, std::io::Error>` - Last modified time or error
    fn get_file_modified_time(path: &Path) -> std::io::Result<SystemTime> {
        path.metadata()?.modified()
    }

    /// Checks if a cached note is still valid by comparing timestamps
    ///
    /// # Arguments
    /// * `id` - ID of the note to check
    /// * `cached` - The cached note entry to validate
    ///
    /// # Returns
    /// * `Result<bool, std::io::Error>` - Whether the cache is still valid
    fn is_cache_valid(&self, id: &str, cached: &CachedNote) -> std::io::Result<bool> {
        let path = self.root_path.join(format!("{}.md", id));
        let current_modified = Self::get_file_modified_time(&path)?;
        Ok(cached.last_modified >= current_modified)
    }

    /// Lists all notes in the store, using cache when possible.
    ///
    /// Scans the root directory for markdown files and returns a list of all valid notes.
    /// Uses cached versions of notes when available and still valid.
    ///
    /// # Returns
    /// * `Result<Vec<Note>, std::io::Error>` - List of notes or IO error
    pub fn list_notes(&self) -> std::io::Result<Vec<Note>> {
        let mut notes = Vec::new();
        let mut cache = self.note_cache.write();

        // Iterate through all files in the notes directory
        for entry in std::fs::read_dir(&self.root_path)? {
            let entry = entry?;
            // Only process markdown files
            if entry.path().extension().is_some_and(|ext| ext == "md") {
                let file_name = entry.file_name();
                let id = file_name
                    .to_str()
                    .and_then(|s| s.strip_suffix(".md"))
                    .unwrap_or_default()
                    .to_string();

                // Check if we have a valid cached version
                if let Some(cached) = cache.get(&id) {
                    if self.is_cache_valid(&id, cached)? {
                        notes.push(cached.note.clone());
                        continue;
                    }
                }

                // No valid cache, need to process the note
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Some(note) = self.parse_note(&content, Some(&entry.path())) {
                        // Update cache with the new processed note
                        if let Ok(modified) = Self::get_file_modified_time(&entry.path()) {
                            cache.insert(
                                id,
                                CachedNote {
                                    note: note.clone(),
                                    last_modified: modified,
                                },
                            );
                        }
                        notes.push(note);
                    }
                }
            }
        }
        Ok(notes)
    }

    /// Retrieves a specific note by ID, using cache when possible.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the note to retrieve
    ///
    /// # Returns
    /// * `Result<Option<Note>, std::io::Error>` - The note if found, None if not found, or IO error
    pub fn get_note(&self, id: &str) -> std::io::Result<Option<Note>> {
        let path = self.root_path.join(format!("{}.md", id));
        if !path.exists() {
            return Ok(None);
        }

        // Check cache first
        {
            let cache = self.note_cache.read();
            if let Some(cached) = cache.get(id) {
                if self.is_cache_valid(id, cached)? {
                    return Ok(Some(cached.note.clone()));
                }
            }
        }

        // No valid cache, need to process the note
        let content = std::fs::read_to_string(&path)?;
        if let Some(note) = self.parse_note(&content, Some(&path)) {
            // Update cache with the new processed note
            if let Ok(modified) = Self::get_file_modified_time(&path) {
                let mut cache = self.note_cache.write();
                cache.insert(
                    id.to_string(),
                    CachedNote {
                        note: note.clone(),
                        last_modified: modified,
                    },
                );
            }
            Ok(Some(note))
        } else {
            Ok(None)
        }
    }

    /// Clears the entire note cache
    ///
    /// This forces all subsequent note requests to reprocess the source files.
    pub fn clear_cache(&self) {
        let mut cache = self.note_cache.write();
        cache.clear();
    }

    /// Removes a specific note from the cache
    ///
    /// # Arguments
    /// * `id` - ID of the note to remove from cache
    pub fn invalidate_cache(&self, id: &str) {
        let mut cache = self.note_cache.write();
        cache.remove(id);
    }

    /// Parses and processes a note's raw content into a structured Note object.
    ///
    /// This function handles:
    /// 1. YAML frontmatter extraction
    /// 2. Markdown to HTML conversion
    /// 3. Syntax highlighting for code blocks
    /// 4. LaTeX math rendering
    /// 5. Asset scanning
    ///
    /// # Arguments
    /// * `content` - Raw note content including frontmatter
    /// * `note_path` - Optional filesystem path to the note (for asset scanning)
    ///
    /// # Returns
    /// * `Option<Note>` - Parsed and processed note, or None if parsing fails
    fn parse_note(&self, content: &str, note_path: Option<&Path>) -> Option<Note> {
        // Parse YAML frontmatter and content
        let matter = Matter::<YAML>::new();
        let parsed = matter.parse_with_struct::<Note>(content)?;
        let theme = &self.theme_set.themes["base16-ocean.dark"];

        // Convert Markdown to HTML with GitHub-flavored Markdown options
        let options = MarkdownOptions::gfm();
        let html_output = match to_html_with_options(&parsed.content, &options) {
            Ok(html) => html,
            Err(_) => return None,
        };

        // Regular expression for finding code blocks
        let code_block_regex =
            match Regex::new(r#"<pre><code class="language-([^"]+)">(.*?)</code></pre>"#) {
                Ok(re) => re,
                Err(_) => return None,
            };

        // Process code blocks with syntax highlighting
        let highlighted = code_block_regex.replace_all(&html_output, |caps: &regex::Captures| {
            let language = &caps[1];
            let content = html_escape::decode_html_entities(&caps[2]).to_string();

            match self.syntax_set.find_syntax_by_token(language) {
                Some(syntax) => {
                    match highlighted_html_for_string(&content, &self.syntax_set, syntax, theme) {
                        Ok(highlighted_html) => format!(
                            r#"<pre><code class="language-{}">{}</code></pre>"#,
                            language, highlighted_html
                        ),
                        Err(_) => caps[0].to_string(),
                    }
                }
                None => caps[0].to_string(),
            }
        });

        // Configure KaTeX options for inline math
        let katex_opts = match Opts::builder()
            .display_mode(false)
            .output_type(katex::OutputType::Html)
            .build()
        {
            Ok(opts) => opts,
            Err(_) => return None,
        };

        // Process inline LaTeX math expressions
        let math_processed = process_inline_math(&highlighted, &katex_opts);

        // Configure KaTeX options for display math
        let display_opts = match Opts::builder()
            .display_mode(true)
            .output_type(katex::OutputType::Html)
            .build()
        {
            Ok(opts) => opts,
            Err(_) => return None,
        };

        // Process display (block) LaTeX math expressions
        let final_content = process_display_math(&math_processed, &display_opts);

        // Scan for associated assets if we have a note path
        let assets = note_path.map(scan_assets).unwrap_or_default();

        // Construct the final note object
        Some(Note {
            id: parsed.data.id,
            title: parsed.data.title,
            content: final_content,
            tags: parsed.data.tags,
            references: parsed.data.references,
            assets,
        })
    }
}

#[cfg(feature = "ssr")]
/// Processes inline LaTeX math expressions (surrounded by single $).
///
/// # Arguments
/// * `content` - HTML content containing math expressions
/// * `opts` - KaTeX rendering options
///
/// # Returns
/// * `String` - Processed content with rendered math
fn process_inline_math(content: &str, opts: &Opts) -> String {
    match Regex::new(r"\$([^\$]+?)\$") {
        Ok(re) => re
            .replace_all(content, |caps: &regex::Captures| {
                match render_with_opts(&caps[1], opts) {
                    Ok(rendered) => rendered,
                    Err(_) => caps[0].to_string(),
                }
            })
            .to_string(),
        Err(_) => content.to_string(),
    }
}

#[cfg(feature = "ssr")]
/// Processes display/block LaTeX math expressions (surrounded by double $$).
///
/// # Arguments
/// * `content` - HTML content containing math expressions
/// * `opts` - KaTeX rendering options
///
/// # Returns
/// * `String` - Processed content with rendered math
fn process_display_math(content: &str, opts: &Opts) -> String {
    match Regex::new(r"\$\$([^\$]+?)\$\$") {
        Ok(re) => re
            .replace_all(content, |caps: &regex::Captures| {
                match render_with_opts(&caps[1], opts) {
                    Ok(rendered) => format!("<div class=\"math-display\">{}</div>", rendered),
                    Err(_) => caps[0].to_string(),
                }
            })
            .to_string(),
        Err(_) => content.to_string(),
    }
}

/// Scans for assets associated with a note.
///
/// Assets are stored in a directory with the same name as the note
/// but with a ".assets" extension instead of ".md". For example:
/// - note.md
/// - note.assets/
///   - image.png
///   - document.pdf
///
/// # Arguments
/// * `note_path` - Path to the note's markdown file
///
/// # Returns
/// * `Vec<Asset>` - List of assets found in the note's asset directory
#[cfg(feature = "ssr")]
fn scan_assets(note_path: &Path) -> Vec<Asset> {
    let assets_dir = note_path.with_extension("assets");
    if !assets_dir.exists() {
        return Vec::new();
    }

    let mut assets = Vec::new();
    if let Ok(entries) = std::fs::read_dir(assets_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(asset) = create_asset(&entry) {
                        assets.push(asset);
                    }
                }
            }
        }
    }
    assets
}

/// Creates an Asset struct from a directory entry.
///
/// This function extracts metadata about an asset file including:
/// - File path
/// - Display name
/// - MIME type (guessed from file extension)
///
/// # Arguments
/// * `entry` - Directory entry representing an asset file
///
/// # Returns
/// * `Option<Asset>` - Asset metadata if successfully created, None if any required
///                     information (filename, etc.) cannot be determined
#[cfg(feature = "ssr")]
fn create_asset(entry: &std::fs::DirEntry) -> Option<Asset> {
    let path = entry.path();
    let name = path.file_name()?.to_string_lossy().into_owned();

    // Guess the MIME type based on file extension
    let mime_type = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    Some(Asset {
        path: path.to_string_lossy().into_owned(),
        name,
        mime_type,
    })
}
