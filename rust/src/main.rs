mod renderer;
mod server;
mod utils;
mod watcher;

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about = "Markdown viewer with Mermaid support", long_about = None)]
struct Args {
    /// Start the preview server for a markdown file
    #[arg(long, group = "action", required = false, conflicts_with_all = ["stop", "bg_server", "kill_all"])]
    start: bool,

    /// Stop the preview server for a markdown file
    #[arg(long, group = "action", required = false, conflicts_with_all = ["start", "bg_server", "kill_all"])]
    stop: bool,

    /// Kill all running preview servers
    #[arg(long, group = "action", required = false, conflicts_with_all = ["start", "stop", "bg_server"])]
    kill_all: bool,

    /// Internal use only: run the server in background mode
    #[arg(long, group = "action", required = false, conflicts_with_all = ["start", "stop", "kill_all"], hide = true)]
    bg_server: bool,

    /// Input markdown file path
    #[arg(required = false)]
    input: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Handle kill all mode
    if args.kill_all {
        info!("Killing all running marv servers");
        return server::kill_all_servers();
    }

    // Make sure input is provided for all other actions
    let input = match args.input {
        Some(input) => input,
        None => {
            return Err(anyhow!(
                "Input file path is required when not using --kill-all"
            ))
        }
    };

    // Handle background server mode
    if args.bg_server {
        // This is the mode where we actually run the server in the background
        let input_path = PathBuf::from(&input);
        if !input_path.exists() {
            return Err(anyhow!("Input file does not exist: {}", input));
        }

        // Get absolute path
        let input_path = std::fs::canonicalize(&input_path)
            .context(format!("Failed to get absolute path for {}", input))?;

        info!("Running server in background mode for {:?}", input_path);

        // Create directory for server info
        let server_dir = utils::file::get_server_info_dir()?;

        // Run the actual server
        return server::run_server(&input_path, &server_dir).await;
    }

    // Get absolute path for consistency
    let input_path = std::fs::canonicalize(PathBuf::from(&input))
        .context(format!("Failed to get absolute path for {}", input))?;

    // Create directory for server info if it doesn't exist
    let server_dir = utils::file::get_server_info_dir()?;
    info!("Server info directory: {:?}", server_dir);

    // Ensure input file is markdown if starting a server
    if !args.stop {
        let extension = input_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if extension != "md" && extension != "markdown" && extension != "mmd" {
            return Err(anyhow!("Input file must be a markdown or mermaid file with .md, .markdown, or .mmd extension"));
        }

        if !input_path.exists() {
            return Err(anyhow!("Input file does not exist: {:?}", input_path));
        }
    }

    if args.stop {
        info!("Stopping preview server for {:?}", input_path);
        server::stop_preview_server(&input_path, &server_dir)
    } else {
        // Default to start mode
        info!("Starting preview server for {:?}", input_path);
        server::start_preview_server(&input_path, &server_dir).await
    }
}
