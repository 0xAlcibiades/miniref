# MiniRef

A Zettelkasten note-taking system built with Leptos, featuring server-side rendering and WebAssembly support.

## Features

**Core System**

- Server-side rendering with Axum web framework
- WebAssembly-powered frontend
- File-based note storage system
- REST API for note management

**API Endpoints**

- `GET /api/notes` - Retrieve list of all notes
- `GET /api/notes/:id` - Fetch specific note by ID

## Development Setup

**Required Tools**

```bash
# Install Rust nightly
rustup toolchain install nightly --allow-downgrade

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install cargo-generate
cargo install cargo-generate

# Install SASS compiler
npm install -g sass

# Install end-to-end testing dependencies
cd end2end && npm install
```

## Project Structure

```
miniref/
├── src/
│   ├── main.rs      # Server setup & API routes
│   ├── app/         # Leptos components
│   ├── note/        # Note management
│   └── lib.rs       # Core functionality
├── notes/           # Note storage directory
└── end2end/         # Test suite
```

## Development Workflow

**Running Development Server**

```bash
cargo leptos watch
```

**Building for Release**

```bash
cargo leptos build --release
```

## Testing

**Running Tests**

```bash
# Development tests
cargo leptos end-to-end

# Production tests
cargo leptos end-to-end --release
```

## Production Deployment

**Required Files**

- Server binary: `target/server/release/miniref`
- Site assets: `target/site/*`

**Directory Structure**

```
miniref/
└── site/
```

**Environment Configuration**

```bash
export LEPTOS_OUTPUT_NAME="miniref"
export LEPTOS_SITE_ROOT="site"
export LEPTOS_SITE_PKG_DIR="pkg"
export LEPTOS_SITE_ADDR="127.0.0.1:3000"
export LEPTOS_RELOAD_PORT="3001"
```

## Implementation Details

**Server Features**

- Axum-based web server with integrated routing
- Note storage management system
- JSON API responses
- Error handling with proper HTTP status codes

**Frontend Features**

- Leptos components for UI
- Hydration support for server-side rendered content
- WebAssembly optimization
- Fallback handlers for unmatched routes

## Note Management

The system uses a file-based note storage system located in the `./notes` directory. Notes are:

- Loaded on server startup
- Accessible via REST API
- Managed through the NoteStore interface
- Retrieved individually or as a complete list

## Contributing

1. Fork the repository
2. Install all development dependencies
3. Create a feature branch
4. Add tests for new functionality
5. Submit a pull request
