use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use {
    gray_matter::{engine::YAML, Matter},
    katex::{render_with_opts, Opts},
    markdown::{to_html_with_options, Options as MarkdownOptions},
    regex::Regex,
    std::path::{Path, PathBuf},
    syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet},
};

/// Represents a note with its metadata and content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

/// Represents an asset (file) associated with a note
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    pub path: String,
    pub name: String,
    pub mime_type: String,
}

/// Contains only the metadata of a note without its content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub references: Vec<String>,
}

#[cfg(feature = "ssr")]
pub struct NoteStore {
    root_path: PathBuf,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

#[cfg(feature = "ssr")]
impl NoteStore {
    /// Creates a new NoteStore with the specified root path
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let root_path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&root_path)?;
        Ok(Self {
            root_path,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        })
    }

    /// Lists all notes in the store
    pub fn list_notes(&self) -> std::io::Result<Vec<Note>> {
        let mut notes = Vec::new();
        for entry in std::fs::read_dir(&self.root_path)? {
            let entry = entry?;
            if entry.path().extension().is_some_and(|ext| ext == "md") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Some(note) = self.parse_note(&content, Some(&entry.path())) {
                        notes.push(note);
                    }
                }
            }
        }
        Ok(notes)
    }

    /// Retrieves a specific note by ID
    pub fn get_note(&self, id: &str) -> std::io::Result<Option<Note>> {
        let path = self.root_path.join(format!("{}.md", id));
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(self.parse_note(&content, Some(&path)))
        } else {
            Ok(None)
        }
    }

    /// Parses a note's content and converts it into a structured Note object
    fn parse_note(&self, content: &str, note_path: Option<&Path>) -> Option<Note> {
        let matter = Matter::<YAML>::new();
        let parsed = matter.parse_with_struct::<Note>(content)?;
        let theme = &self.theme_set.themes["base16-ocean.dark"];

        let options = MarkdownOptions::gfm();
        let html_output = match to_html_with_options(&parsed.content, &options) {
            Ok(html) => html,
            Err(_) => return None,
        };

        let code_block_regex =
            match Regex::new(r#"<pre><code class="language-([^"]+)">(.*?)</code></pre>"#) {
                Ok(re) => re,
                Err(_) => return None,
            };

        let highlighted = code_block_regex.replace_all(&html_output, |caps: &regex::Captures| {
            let language = &caps[1];
            let code_content = match html_escape::decode_html_entities(&caps[2]).to_string() {
                content => content,
            };

            match self.syntax_set.find_syntax_by_token(language) {
                Some(syntax) => {
                    match highlighted_html_for_string(
                        &code_content,
                        &self.syntax_set,
                        syntax,
                        theme,
                    ) {
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

        let katex_opts = match Opts::builder()
            .display_mode(false)
            .output_type(katex::OutputType::Html)
            .build()
        {
            Ok(opts) => opts,
            Err(_) => return None,
        };

        let math_processed = process_inline_math(&highlighted, &katex_opts);

        let display_opts = match Opts::builder()
            .display_mode(true)
            .output_type(katex::OutputType::Html)
            .build()
        {
            Ok(opts) => opts,
            Err(_) => return None,
        };

        let final_content = process_display_math(&math_processed, &display_opts);
        let assets = note_path.map(scan_assets).unwrap_or_default();

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

#[cfg(feature = "ssr")]
fn create_asset(entry: &std::fs::DirEntry) -> Option<Asset> {
    let path = entry.path();
    let name = path.file_name()?.to_string_lossy().into_owned();
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
