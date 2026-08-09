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

/// Open the blocks DB and read a single named column from the `current_tip` row.
fn query_tip_column<T: rusqlite::types::FromSql>(column: &str) -> Option<T> {
    let db_path = blocks_db_path();
    let conn = Connection::open(&db_path).ok()?;
    let sql = format!("SELECT {column} FROM current_tip LIMIT 1");
    conn.query_row(&sql, [], |row| row.get(0)).ok()
}

/// Query the most recent tip height from the database.
pub fn load_tip_height() -> u32 {
    query_tip_column("height").unwrap_or(0)
}

/// Query the current subsidy (sats) from the database.
pub fn load_current_subsidy() -> i64 {
    query_tip_column("subsidy").unwrap_or(0)
}

/// Query the current mining difficulty from the database.
pub fn load_mining_difficulty() -> f64 {
    query_tip_column("difficulty").unwrap_or(0.0)
}
