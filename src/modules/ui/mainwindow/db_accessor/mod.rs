//! Database access for the current blockchain tip values shown in the UI.
//!
//! Reads the `current_tip` row (height, subsidy, difficulty) from the mempool
//! blocks database. The DB path is built once by a shared helper.

use crate::modules::app_data_dir::MEMPOOL;
use rusqlite::Connection;
use std::path::PathBuf;

/// Path to the mempool blocks database (`current_tip` and `blocks` tables).
fn blocks_db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Halvora").join(MEMPOOL).join("blocks.db")
}

/// Query the most recent tip height from the database.
pub fn load_tip_height() -> u32 {
    let db_path = blocks_db_path();
    if let Ok(conn) = Connection::open(&db_path)
        && let Ok(height) = conn.query_row("SELECT height FROM current_tip LIMIT 1", [], |row| {
            row.get(0)
        })
    {
        return height;
    }
    0
}

/// Query the current subsidy (sats) from the database.
pub fn load_current_subsidy() -> i64 {
    let db_path = blocks_db_path();
    if let Ok(conn) = Connection::open(&db_path)
        && let Ok(subsidy) = conn.query_row("SELECT subsidy FROM current_tip LIMIT 1", [], |row| {
            row.get(0)
        })
    {
        return subsidy;
    }
    0
}

/// Query the current mining difficulty from the database.
pub fn load_mining_difficulty() -> f64 {
    let db_path = blocks_db_path();
    if let Ok(conn) = Connection::open(&db_path)
        && let Ok(difficulty) =
            conn.query_row("SELECT difficulty FROM current_tip LIMIT 1", [], |row| {
                row.get(0)
            })
    {
        return difficulty;
    }
    0.0
}
