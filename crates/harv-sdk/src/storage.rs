use std::io::Write;
use std::path::Path;

use atomicwrites::{AllowOverwrite, AtomicFile};
use harv_core::HarvError;

/// Writes a complete file or leaves the previous contents in place.
///
/// The blocking filesystem work is moved off the async runtime. Callers share
/// this helper so cache and configuration files have the same safety property.
pub(crate) async fn atomic_write(path: &Path, contents: Vec<u8>) -> Result<(), HarvError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        AtomicFile::new(path, AllowOverwrite)
            .write(|file| file.write_all(&contents))
            .map_err(|error| HarvError::Other(format!("Failed to write file atomically: {error}")))
    })
    .await
    .map_err(|error| HarvError::Other(format!("Atomic write task failed: {error}")))?
}
