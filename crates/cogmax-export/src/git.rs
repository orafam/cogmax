use std::{fs, path::Path, process::Command};

use super::{verify_manifest, ExportError, SnapshotManifest};

/// Copies a verified snapshot into a Git repository and creates a local commit.
/// It deliberately never pushes and refuses to overwrite unrelated work.
pub fn sync_snapshot(
    repository: impl AsRef<Path>,
    snapshot: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    message: &str,
) -> Result<(), ExportError> {
    let repository = repository.as_ref();
    let snapshot = snapshot.as_ref();
    let destination = repository.join(destination);
    let markdown = fs::read_to_string(snapshot.join("memories.md"))?;
    let manifest: SnapshotManifest =
        serde_json::from_slice(&fs::read(snapshot.join("manifest.json"))?)
            .map_err(|error| ExportError::Git(format!("invalid manifest: {error}")))?;
    verify_manifest(&markdown, &manifest)?;
    ensure_git(repository, &["status", "--porcelain"])?;
    let status = git_output(repository, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        return Err(ExportError::Git(
            "repository has uncommitted changes".into(),
        ));
    }
    let existing_manifest = destination.join("manifest.json");
    if existing_manifest.is_file() {
        let existing: SnapshotManifest = serde_json::from_slice(&fs::read(&existing_manifest)?)
            .map_err(|error| ExportError::Git(format!("invalid destination manifest: {error}")))?;
        if existing == manifest {
            return Ok(());
        }
        return Err(ExportError::SnapshotConflict);
    }
    fs::create_dir_all(&destination)?;
    for name in [
        "memories.md",
        "memories.json",
        "memories.parquet",
        "manifest.json",
    ] {
        fs::copy(snapshot.join(name), destination.join(name))?;
    }
    let relative = destination
        .strip_prefix(repository)
        .map_err(|error| ExportError::Git(error.to_string()))?;
    ensure_git(repository, &["add", "--", relative.to_str().unwrap_or("")])?;
    ensure_git(repository, &["commit", "-m", message])?;
    Ok(())
}

fn ensure_git(repository: &Path, args: &[&str]) -> Result<(), ExportError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()
        .map_err(|error| ExportError::Git(error.to_string()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(ExportError::Git(
            String::from_utf8_lossy(&output.stderr).trim().into(),
        ))
    }
}

fn git_output(repository: &Path, args: &[&str]) -> Result<String, ExportError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()
        .map_err(|error| ExportError::Git(error.to_string()))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into())
    } else {
        Err(ExportError::Git(
            String::from_utf8_lossy(&output.stderr).trim().into(),
        ))
    }
}
