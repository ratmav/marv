use std::process::Command;

use anyhow::{Context, Result};
use tracing::{debug, info};

// Check if a process is running
pub fn is_process_running(pid: u32) -> bool {
    debug!("Checking if process {} is running", pid);

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output();

        match output {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let result = output_str.contains(&format!("{}", pid));
                debug!(
                    "Process {} is {}",
                    pid,
                    if result { "running" } else { "not running" }
                );
                result
            }
            Err(e) => {
                debug!("Error checking process status: {}", e);
                false
            }
        }
    }

    #[cfg(target_family = "unix")]
    {
        let output = Command::new("ps")
            .args(["-p", &format!("{}", pid)])
            .output();

        match output {
            Ok(output) => {
                let result = output.status.success();
                debug!(
                    "Process {} is {}",
                    pid,
                    if result { "running" } else { "not running" }
                );
                result
            }
            Err(e) => {
                debug!("Error checking process status: {}", e);
                false
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_family = "unix")))]
    {
        debug!("Process check not supported on this platform");
        false // Default for unsupported platforms
    }
}

// Kill a process
pub fn kill_process(pid: u32) -> Result<()> {
    info!("Killing process {}", pid);

    #[cfg(target_os = "windows")]
    {
        Command::new("taskkill")
            .args(["/F", "/PID", &format!("{}", pid)])
            .output()
            .context("Failed to kill process")?;
    }

    #[cfg(target_family = "unix")]
    {
        Command::new("kill")
            .args([&format!("{}", pid)])
            .output()
            .context("Failed to kill process")?;
    }

    #[cfg(not(any(target_os = "windows", target_family = "unix")))]
    {
        return Err(anyhow!(
            "Process termination not supported on this platform"
        ));
    }

    Ok(())
}

// Open the default browser
pub fn open_browser(url: &str) {
    info!("Opening browser to {}", url);

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", url]).spawn().ok();
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn().ok();
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn().ok();
    }
}
