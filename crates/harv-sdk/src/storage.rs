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
        restrict_directory_permissions(parent).await?;
    }

    let path = path.to_path_buf();
    let written_path = path.clone();
    tokio::task::spawn_blocking(move || {
        AtomicFile::new(path, AllowOverwrite)
            .write(|file| file.write_all(&contents))
            .map_err(|error| HarvError::Other(format!("Failed to write file atomically: {error}")))
    })
    .await
    .map_err(|error| HarvError::Other(format!("Atomic write task failed: {error}")))??;

    restrict_file_permissions(written_path.as_path()).await
}

#[cfg(unix)]
async fn restrict_directory_permissions(path: &Path) -> Result<(), HarvError> {
    use std::os::unix::fs::PermissionsExt;

    tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).await?;
    Ok(())
}

#[cfg(not(unix))]
async fn restrict_directory_permissions(_path: &Path) -> Result<(), HarvError> {
    Ok(())
}

#[cfg(unix)]
async fn restrict_file_permissions(path: &Path) -> Result<(), HarvError> {
    use std::os::unix::fs::PermissionsExt;

    tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await?;
    Ok(())
}

#[cfg(not(unix))]
async fn restrict_file_permissions(_path: &Path) -> Result<(), HarvError> {
    Ok(())
}
