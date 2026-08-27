use std::fs;
use std::path::PathBuf;

/// Subdirectory constants for app data organisation.
pub const MEMPOOL: &str = "Mempool";
pub const EXCHANGE: &str = "Exchange";
pub const POSITION: &str = "Position";

/// Path to the BTC/USD daily candle database (`daily_candles` table).
pub fn btcusd_db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Halvora").join(EXCHANGE).join("btcusd.db")
}

/// Path to the user position database (`user_position` table).
pub fn position_db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Halvora").join(POSITION).join("user_position.db")
}

/// Ensure the app data directory tree at `~/.local/share/Halvora/` exists.
/// Creates `Halvora/`, `Halvora/Mempool/`, `Halvora/Exchange/`, and
/// `Halvora/Position/`. Returns the base path if successful, or logs a
/// warning otherwise.
pub fn ensure() -> Option<PathBuf> {
    let base = dirs::data_dir()?;
    let app_dir = base.join("Halvora");
    let subdirs = [MEMPOOL, EXCHANGE, POSITION];

    // Create the base dir and each subdirectory in one pass.
    let mut dirs = vec![app_dir.clone()];
    dirs.extend(subdirs.iter().map(|name| app_dir.join(name)));

    for dir in &dirs {
        if let Err(e) = fs::create_dir_all(dir) {
            eprintln!("[halvora] warning: could not create directory {dir:?}: {e}");
            return None;
        }
    }

    eprintln!("[halvora] data directory: {}", app_dir.display());
    Some(app_dir)
}
