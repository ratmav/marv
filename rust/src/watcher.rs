use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use futures::channel::mpsc::channel;
use futures::{SinkExt, StreamExt};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::watch;
use tracing::{debug, info, warn};

use crate::utils::file;

// Watch file for changes
pub async fn watch_file(path: PathBuf, watch_tx: Arc<Mutex<watch::Sender<String>>>) -> Result<()> {
    info!("Starting file watcher for {:?}", path);

    let (mut tx, mut rx) = channel(100);
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    watcher.watch(
        path.parent().unwrap_or(Path::new(".")),
        RecursiveMode::NonRecursive,
    )?;
    debug!(
        "Watching directory: {:?}",
        path.parent().unwrap_or(Path::new("."))
    );

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                if let Event {
                    kind: notify::EventKind::Modify(_),
                    ..
                } = event
                {
                    debug!("File modification detected: {:?}", path);
                    if let Ok(content) = file::read_file(&path) {
                        let tx = watch_tx.lock().unwrap();
                        let _ = tx.send(content);
                        debug!("Updated content sent to server");
                    }
                }
            }
            Err(e) => warn!("Watch error: {:?}", e),
        }
    }

    Ok(())
}
