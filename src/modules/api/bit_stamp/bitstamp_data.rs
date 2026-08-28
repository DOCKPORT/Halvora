//! Pre-compiled BTC/USD database seeding.
//!
//! Embeds the historical daily-candle database directly into the binary at
//! compile time. On the first run (when the user's database is missing or
//! older than the embedded snapshot), the embedded copy is written to disk so
//! the app starts with full history instead of performing a long backfill from
//! the Bitstamp API. Once present, normal gap-fetching resumes.

use crate::modules::app_data_dir::btcusd_db_path;
use rusqlite::Connection;
use std::fs;
use std::io::Write;
use std::path::Path;

/// The embedded historical BTC/USD database, baked into the binary at compile
/// time. This file is a snapshot of daily candles up to its generation date.
///
/// The build script copies `bitstamp_data/btcusd.db` into `OUT_DIR` and this
/// crate embeds that copy. Embedding through `OUT_DIR` (instead of reading the
/// repository file directly) makes cargo treat the database as a real build
/// input: replacing the database always triggers a rebuild, so a fresh
/// database can never be silently skipped by a cached compilation.
const EMBEDDED_BTCUSD_DB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/btcusd.db"));

/// Write the embedded database to the user's data directory when the existing
/// file is missing, empty, or older than the embedded snapshot.
///
/// Runs once at startup after [`crate::modules::app_data_dir::ensure`]. A
/// non-empty existing file that is at least as fresh as the embedded snapshot
/// is left untouched so a working database is never overwritten. A missing
/// file, an empty file (for example from an interrupted run), or a database
/// older than the embedded snapshot is replaced with the seed to avoid gaps
/// and a full network backfill.
pub fn seed_if_missing() {
    let db_path = btcusd_db_path();

    // A non-empty existing file is treated as a working database, but only
    // when it is at least as fresh as the embedded snapshot. An older file
    // (for example one seeded by a previous build) is replaced so the newest
    // pre-compiled history wins.
    if file_has_data(&db_path) && !embedded_is_newer_than(&db_path) {
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

/// Returns `true` when the embedded snapshot has a newer latest candle than
/// the database at `path`.
///
/// Returns `false` when either database cannot be read, so an existing file is
/// never replaced based on a read failure.
fn embedded_is_newer_than(db_path: &Path) -> bool {
    let local_newest = match newest_candle_ts(db_path) {
        Some(ts) => ts,
        None => return false,
    };
    match embedded_newest_ts() {
        Some(ts) => ts > local_newest,
        None => false,
    }
}

/// Read the newest `daily_candles` timestamp from the database file at `path`.
fn newest_candle_ts(db_path: &Path) -> Option<i64> {
    let conn = Connection::open(db_path).ok()?;
    conn.query_row("SELECT MAX(timestamp) FROM daily_candles", [], |row| {
        row.get(0)
    })
    .ok()
    .flatten()
}

/// Read the newest `daily_candles` timestamp from the embedded snapshot.
///
/// The embedded database exists only in memory, so it is written to a
/// temporary file and opened as a regular SQLite database.
fn embedded_newest_ts() -> Option<i64> {
    let mut tmp_path = std::env::temp_dir();
    tmp_path.push(format!("halvora-embedded-{}.db", std::process::id()));

    fs::write(&tmp_path, EMBEDDED_BTCUSD_DB).ok()?;
    let ts = newest_candle_ts(&tmp_path);
    let _ = fs::remove_file(&tmp_path);
    ts
}
