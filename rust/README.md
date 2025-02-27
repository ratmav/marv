# Marv - Markdown Viewer

A simple CLI tool that provides live preview functionality for Markdown files, with support for Mermaid diagrams.

## FeaturesWOO

- Live preview of Markdown files
- Auto-refresh preview when the source file changes
- Support for Mermaid diagrams in Markdown
- Dynamic port assignment in the 4000-4999 range
- Explicit start/stop commands
- Support for multiple documents simultaneously

## Project Structure

```
marv/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs          # Entry point and CLI handling
    ├── server.rs        # Server management (start/stop/info)
    ├── renderer.rs      # Markdown to HTML rendering
    ├── watcher.rs       # File watching functionality
    └── utils/
        ├── mod.rs       # Utils module definition
        ├── process.rs   # Process management utilities
        └── file.rs      # File operations utilities
```

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/marv.git
cd marv

# Build the project
cargo build --release

# Optionally, install the binary to your path
cargo install --path .
```

## Development & Testing

```bash
# Build a debug version (faster compile, includes debug info)
cargo build

# Run without building a binary (useful during development)
cargo run -- --start path/to/your/file.md

# Run tests
cargo test

# Check for errors without building
cargo check

# Run with logging enabled (outputs detailed debug info)
RUST_LOG=debug cargo run -- --start path/to/your/file.md
```

## Usage

```bash
# Start live preview for a markdown file
marv --start path/to/your/file.md

# Stop the preview server for a specific file
marv --stop path/to/your/file.md
```

## How It Works

1. Start marv with the `--start` command and your Markdown file path
2. A live preview server starts on an available port in the 4000-4999 range
3. Your default browser opens automatically to show the preview
4. Edit your Markdown file in any editor - the preview updates automatically
5. When you're done, use the `--stop` command to shut down the server

The tool saves server information in a `.marv` directory in your home folder, allowing it to manage multiple preview servers for different documents.

## Mermaid Diagram Support

You can include Mermaid diagrams in your Markdown files using code blocks with the `mermaid` language specified:

```mermaid
graph TD;
    A-->B;
    A-->C;
    B-->D;
    C-->D;
```

## Code Organization

- **main.rs**: Command-line interface and application entry point
- **server.rs**: HTTP server functionality and dynamic port allocation
- **renderer.rs**: Markdown parsing and HTML rendering
- **watcher.rs**: File system watching functionality
- **utils/process.rs**: Process management (checking, starting, stopping)
- **utils/file.rs**: File operations and server info storage

## License

MIT
