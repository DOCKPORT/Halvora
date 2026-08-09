//! Pre-compiled BTC/USD database seeding.
//!
//! Embeds the historical daily-candle database directly into the binary at
//! compile time. On the first run (when the user's database is missing), the
//! embedded copy is written to disk so the app starts with full history
//! instead of performing a long backfill from the Bitstamp API. Once present,
//! normal gap-fetching resumes.

use crate::modules::app_data_dir::btcusd_db_path;
use std::fs;
use std::io::Write;
use std::path::Path;

/// The embedded historical BTC/USD database, baked into the binary at compile
/// time. This file is a snapshot of daily candles up to its generation date.
const EMBEDDED_BTCUSD_DB: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/bitstamp_data/btcusd.db"
));

/// Write the embedded database to the user's data directory if it is missing
/// or empty.
///
/// Runs once at startup after [`crate::modules::app_data_dir::ensure`]. A
/// non-empty existing file is left untouched so a working database is never
/// overwritten. An empty file (for example from an interrupted run) holds no
/// data, so it is replaced with the seed to avoid a full network backfill.
pub fn seed_if_missing() {
    let db_path = btcusd_db_path();

    // A non-empty existing file is treated as a working database. An empty
    // file holds no data, so seed it as if it were missing.
    if file_has_data(&db_path) {
        return;
    }

    // Ensure the parent directory exists, then write the embedded bytes.
    let parent = db_path.parent();
    if let Some(dir) = parent {
        if let Err(e) = fs::create_dir_all(dir) {
            eprintln!("[bitstamp] could not create data dir {dir:?}: {e}");
            return;
        }
    }

    match fs::File::create(&db_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(EMBEDDED_BTCUSD_DB) {
                eprintln!("[bitstamp] failed to write seeded database: {e}");
                let _ = fs::remove_file(&db_path);
                return;
            }
            eprintln!(
                "[bitstamp] seeded {} bytes of historical data to {}",
                EMBEDDED_BTCUSD_DB.len(),
                db_path.display()
            );
        }
        Err(e) => {
            eprintln!("[bitstamp] failed to create database file {db_path:?}: {e}");
        }
    }
}

/// Returns `true` when the file exists and has a non-zero size.
///
/// A missing file or an empty file both count as "no data", so the caller
/// seeds the embedded database for either case.
fn file_has_data(path: &Path) -> bool {
    fs::metadata(path).map_or(false, |meta| meta.len() > 0)
}
