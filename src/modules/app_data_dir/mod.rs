use std::path::PathBuf;
use std::fs;

/// Subdirectory constants for app data organisation.
pub const MEMPOOL: &str = "Mempool";
pub const EXCHANGE: &str = "Exchange";

/// Ensure the app data directory tree at `~/.local/share/Halvora/` exists.
/// Creates `Halvora/`, `Halvora/Mempool/`, and `Halvora/Exchange/`.
/// Returns the base path if successful, or logs a warning otherwise.
pub fn ensure() -> Option<PathBuf> {
    let base = dirs::data_dir()?;
    let app_dir = base.join("Halvora");
    let subdirs = [MEMPOOL, EXCHANGE];

    // Create the base dir and each subdirectory in one pass.
    let mut dirs = vec![app_dir.clone()];
    dirs.extend(subdirs.iter().map(|name| app_dir.join(name)));

    for dir in &dirs {
        if let Err(e) = fs::create_dir_all(dir) {
            eprintln!(
                "[halvora] warning: could not create directory {:?}: {}",
                dir, e
            );
            return None;
        }
    }

    eprintln!("[halvora] data directory: {}", app_dir.display());
    Some(app_dir)
}
