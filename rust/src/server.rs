use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use tokio::sync::{mpsc, watch};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{debug, info};
use warp::ws::{Message, WebSocket};
use warp::Filter;

use crate::renderer;
use crate::utils;
use crate::watcher;

// Start preview server
pub async fn start_preview_server(input_path: &Path, server_dir: &Path) -> Result<()> {
    let server_info_path = utils::file::get_server_info_path(input_path, server_dir);
    info!("Server info path: {:?}", server_info_path);

    // Check if server is already running
    if server_info_path.exists() {
        if let Ok((port, pid)) = utils::file::read_server_info(&server_info_path) {
            if utils::process::is_process_running(pid) {
                println!(
                    "A preview server is already running for this file on port {}.",
                    port
                );
                println!("Preview available at http://localhost:{}", port);
                println!(
                    "Use 'marv --stop {:?}' to stop the server first if you want to restart it.",
                    input_path
                );
                return Ok(());
            } else {
                // Process is not running but file exists, remove stale file
                std::fs::remove_file(&server_info_path)
                    .context("Failed to remove stale server info file")?;

                // Also remove file path reference if it exists
                let filepath_info = server_info_path.with_extension("filepath");
                if filepath_info.exists() {
                    std::fs::remove_file(&filepath_info)
                        .context("Failed to remove stale filepath info file")?;
                }
            }
        }
    }

    // Find an available port
    let port = find_available_port().await?;

    println!(
        "Starting preview server for {:?} on port {}",
        input_path, port
    );
    println!("Preview will be available at http://localhost:{}", port);

    // Create the server info directory if it doesn't exist
    if let Some(parent) = server_info_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {:?}", parent))?;
            info!("Created server info directory: {:?}", parent);
        }
    }

    // Launch the server in a background process
    let server_executable = std::env::current_exe()?;

    // Start the server in a background process with nohup or similar
    #[cfg(target_family = "unix")]
    {
        // Use nohup to keep the process running after terminal closes
        let input_path_str = input_path.to_str().unwrap_or_default();

        // Start a detached process that runs our server
        use std::process::Stdio;
        let process = Command::new("bash")
            .args([
                "-c",
                &format!(
                    "nohup {} --bg-server \"{}\" > /dev/null 2>&1 & echo $!",
                    server_executable.display(),
                    input_path_str
                ),
            ])
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to start background server process")?;

        // Get the PID of the nohup process
        let output = process
            .wait_with_output()
            .context("Failed to get output from background process")?;

        let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let pid = pid_str
            .parse::<u32>()
            .context(format!("Failed to parse PID from output: {}", pid_str))?;

        // Record server information with the background process PID
        info!("Started background server process with PID: {}", pid);
        utils::file::write_server_info(&server_info_path, port, pid)?;
    }

    #[cfg(target_os = "windows")]
    {
        // Windows version using start command
        let input_path_str = input_path.to_str().unwrap_or_default();

        // Get the CMD window hidden by using /B flag
        let process = Command::new("cmd")
            .args([
                "/C",
                "start",
                "/B",
                server_executable.to_str().unwrap_or_default(),
                "--bg-server",
                input_path_str,
            ])
            .spawn()
            .context("Failed to start background server process")?;

        // Record server information with the current process PID
        // We'll need to manually find and kill this process on Windows
        let pid = process.id();
        info!("Started background server process with PID: {}", pid);
        utils::file::write_server_info(&server_info_path, port, pid)?;
    }

    // Also save the file path reference
    utils::file::save_file_path_in_server_info(&server_info_path, input_path)?;

    // Open browser after a short delay to ensure server is ready
    let port_clone = port;
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        utils::process::open_browser(&format!("http://localhost:{}", port_clone));
    });

    println!("Marv server started in the background on port {}", port);
    println!("Preview available at http://localhost:{}", port);
    println!(
        "Use 'marv --stop {:?}' to stop the server when done",
        input_path
    );

    Ok(())
}

// Run the server (used by start_preview_server)
pub async fn run_server(input_path: &Path, server_dir: &Path) -> Result<()> {
    // Get server info
    let server_info_path = utils::file::get_server_info_path(input_path, server_dir);
    let (port, _) = utils::file::read_server_info(&server_info_path)?;

    // Set up file watching
    let (watch_tx, watch_rx) = watch::channel(String::new());
    let watch_tx = Arc::new(Mutex::new(watch_tx));

    // Initial read of the file
    let content = utils::file::read_file(input_path)?;
    {
        let watch_tx_clone = watch_tx.lock().unwrap();
        watch_tx_clone.send(content)?;
    } // lock is dropped at the end of this scope

    // Start file watcher
    let input_path_clone = input_path.to_path_buf();
    let watch_tx_for_watcher = watch_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = watcher::watch_file(input_path_clone, watch_tx_for_watcher).await {
            eprintln!("Error watching file: {}", e);
        }
    });

    // Open browser after a short delay to ensure server is ready
    let port_clone = port;
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        utils::process::open_browser(&format!("http://localhost:{}", port_clone));
    });

    // Start web server for preview
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

    // Create the warp filter that will pass the content receiver to our handlers
    let content_filter = warp::any().map(move || watch_rx.clone());

    // WebSocket route
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(content_filter.clone())
        .map(|ws: warp::ws::Ws, content| {
            ws.on_upgrade(move |websocket| handle_ws_connection(websocket, content))
        });

    // Use full file path for the title
    let filepath = input_path.to_string_lossy().to_string();

    // Main HTML route with auto-refresh script
    let html_route =
        warp::path::end()
            .and(content_filter)
            .map(move |content: watch::Receiver<String>| {
                let current_content = content.borrow().clone();
                let mut html_content = renderer::markdown_to_html(&current_content, &filepath);

                // Insert live reload JavaScript before the closing body tag
                let script = r#"
<script>
    // Connect to WebSocket server
    const socket = new WebSocket(`ws://${window.location.host}/ws`);
    
    // Handle messages from the server
    socket.onmessage = function(event) {
        if (event.data === 'refresh') {
            console.log('Refreshing page due to file change');
            window.location.reload();
        }
    };
    
    // Handle connection open
    socket.onopen = function() {
        console.log('WebSocket connected for live reload');
    };
    
    // Handle connection close
    socket.onclose = function() {
        console.log('WebSocket connection closed');
        // Try to reconnect after a delay
        setTimeout(() => {
            window.location.reload();
        }, 5000);
    };
    
    // Handle connection errors
    socket.onerror = function(error) {
        console.error('WebSocket error:', error);
    };
</script>
</body>
"#;

                // Replace the closing body tag with our script + closing body tag
                html_content = html_content.replace("</body>", script);

                warp::reply::html(html_content)
            });

    // Combine routes
    let routes = html_route.or(ws_route);

    // Print user information
    println!("Marv server running for {:?}", input_path);
    println!("Preview available at http://localhost:{}", port);
    println!(
        "Press Ctrl+C to stop, or use 'marv --stop {:?}' from another terminal",
        input_path
    );

    // Run the web server (this blocks until the server stops)
    info!("Starting web server on port {}", port);
    warp::serve(routes).run(addr).await;

    Ok(())
}

// Stop preview server
pub fn stop_preview_server(input_path: &Path, server_dir: &Path) -> Result<()> {
    let server_info_path = utils::file::get_server_info_path(input_path, server_dir);

    // Check if server info file exists
    if !server_info_path.exists() {
        return Err(anyhow!("No server found for {:?}", input_path));
    }

    // Read server information
    let (port, pid) = utils::file::read_server_info(&server_info_path)?;

    // Try to terminate the process
    if utils::process::is_process_running(pid) {
        utils::process::kill_process(pid)?;
        println!(
            "Stopped preview server for {:?} (port: {}, pid: {})",
            input_path, port, pid
        );
    } else {
        println!(
            "Server process was not running (port: {}, pid: {})",
            port, pid
        );
    }

    // Remove server info file
    std::fs::remove_file(&server_info_path).context("Failed to remove server info file")?;

    // Also remove file path reference if it exists
    let filepath_info = server_info_path.with_extension("filepath");
    if filepath_info.exists() {
        std::fs::remove_file(&filepath_info).context("Failed to remove filepath info file")?;
    }

    Ok(())
}

// Find an available port in the 4000-4999 range
async fn find_available_port() -> Result<u16> {
    for port in 4000..5000 {
        debug!("Trying port {}", port);
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        if tokio::net::TcpListener::bind(addr).await.is_ok() {
            info!("Found available port: {}", port);
            return Ok(port);
        }
    }
    Err(anyhow!("No available ports in the range 4000-4999"))
}

// Kill all running marv servers
pub fn kill_all_servers() -> Result<()> {
    info!("Attempting to kill all marv servers...");

    // Get server info directory
    let server_dir = utils::file::get_server_info_dir()?;

    // Ensure directory exists
    if !server_dir.exists() {
        println!("No marv servers are running.");
        return Ok(());
    }

    // Count terminated servers
    let mut terminated_count = 0;

    // Read the directory and kill all server processes
    if let Ok(entries) = fs::read_dir(server_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Only process .server files
            if let Some(ext) = path.extension() {
                if ext != "server" {
                    continue;
                }

                // Try to read the server info
                if let Ok((port, pid)) = utils::file::read_server_info(&path) {
                    // Try to terminate the process
                    if utils::process::is_process_running(pid) {
                        if let Err(e) = utils::process::kill_process(pid) {
                            eprintln!("Error terminating process (pid: {}): {}", pid, e);
                        } else {
                            println!("Terminated server on port {} (pid: {})", port, pid);
                            terminated_count += 1;
                        }
                    }

                    // Remove server info file regardless of whether process was running
                    if let Err(e) = fs::remove_file(&path) {
                        eprintln!("Error removing server info file {:?}: {}", path, e);
                    }

                    // Also remove file path reference if it exists
                    let filepath_info = path.with_extension("filepath");
                    if filepath_info.exists() {
                        if let Err(e) = fs::remove_file(&filepath_info) {
                            eprintln!(
                                "Error removing filepath info file {:?}: {}",
                                filepath_info, e
                            );
                        }
                    }
                }
            }
        }
    }

    if terminated_count > 0 {
        println!(
            "Successfully terminated {} marv server(s)",
            terminated_count
        );
    } else {
        println!("No active marv servers were found");
    }

    Ok(())
}

// Handle websocket connections
async fn handle_ws_connection(ws: WebSocket, mut file_updates: watch::Receiver<String>) {
    // Split the websocket into sender and receiver
    let (ws_tx, mut ws_rx) = ws.split();

    // Create a channel for messages to send to the websocket
    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);

    // Forward messages from the channel to the websocket
    tokio::task::spawn(rx.forward(ws_tx));

    // Clone the sender for our file watcher
    let tx_clone = tx.clone();

    // Spawn a task to watch for file changes
    tokio::task::spawn(async move {
        // Set up a changed watcher
        let mut last_seen = file_updates.borrow_and_update().clone();

        while file_updates.changed().await.is_ok() {
            let current = file_updates.borrow().clone();
            if current != last_seen {
                debug!("File content changed, sending refresh notification");
                if tx_clone.send(Ok(Message::text("refresh"))).is_err() {
                    // Client disconnected
                    break;
                }
                last_seen = current;
            }
        }
    });

    // Handle any incoming messages (not really needed for our use case)
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(_) => (),
            Err(e) => {
                debug!("WebSocket error: {}", e);
                break;
            }
        }
    }

    debug!("WebSocket connection closed");
}
