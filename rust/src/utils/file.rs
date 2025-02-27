use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use tracing::{debug, info};

// Server information storage
const SERVER_INFO_DIR: &str = ".marv";

// Get or create the directory for server information
pub fn get_server_info_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;

    let server_dir = home_dir.join(SERVER_INFO_DIR);

    if !server_dir.exists() {
        fs::create_dir_all(&server_dir).context("Failed to create server info directory")?;
        info!("Created server info directory at {:?}", server_dir);
    }

    Ok(server_dir)
}

// Get the server info file path for a specific markdown file
pub fn get_server_info_path(input_path: &Path, server_dir: &Path) -> PathBuf {
    // Create a unique filename based on the input path
    let file_hash = format!(
        "{:x}",
        md5::compute(input_path.to_string_lossy().as_bytes())
    );
    let path = server_dir.join(format!("{}.server", file_hash));
    debug!("Server info path for {:?}: {:?}", input_path, path);
    path
}

// Write server information to file
pub fn write_server_info(server_info_path: &Path, port: u16, pid: u32) -> Result<()> {
    let info = format!("{}:{}", port, pid);

    // Ensure parent directory exists
    if let Some(parent) = server_info_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {:?}", parent))?;
        }
    }

    fs::write(server_info_path, info).context(format!(
        "Failed to write server information to {:?}",
        server_info_path
    ))?;
    info!(
        "Wrote server info to {:?}: port={}, pid={}",
        server_info_path, port, pid
    );
    Ok(())
}

// Read server information from file
pub fn read_server_info(server_info_path: &Path) -> Result<(u16, u32)> {
    let content =
        fs::read_to_string(server_info_path).context("Failed to read server information")?;

    let parts: Vec<&str> = content.trim().split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid server info format"));
    }

    let port = parts[0]
        .parse::<u16>()
        .context("Failed to parse port number")?;
    let pid = parts[1]
        .parse::<u32>()
        .context("Failed to parse process ID")?;

    debug!(
        "Read server info from {:?}: port={}, pid={}",
        server_info_path, port, pid
    );
    Ok((port, pid))
}

// Save file path reference in the server info file
pub fn save_file_path_in_server_info(server_info_path: &Path, file_path: &Path) -> Result<()> {
    let file_path_str = file_path.to_string_lossy().to_string();

    // Ensure parent directory exists
    if let Some(parent) = server_info_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {:?}", parent))?;
        }
    }

    let filepath_info_path = server_info_path.with_extension("filepath");
    fs::write(&filepath_info_path, &file_path_str).context(format!(
        "Failed to write file path information to {:?}",
        filepath_info_path
    ))?;

    info!(
        "Wrote file path info to {:?}: {}",
        filepath_info_path, file_path_str
    );
    Ok(())
}
// Read file content
pub fn read_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    debug!("Read {} bytes from {:?}", content.len(), path);
    Ok(content)
}
