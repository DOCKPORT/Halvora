//! Database access for the current blockchain tip values shown in the UI.
//!
//! Reads the `current_tip` row (height, subsidy, difficulty) from the mempool
//! blocks database. The DB path is built once by a shared helper.

use crate::modules::app_data_dir::{MEMPOOL, position_db_path};
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

/// Satoshis per Bitcoin. The balance is stored as integer satoshis so the
/// value round-trips losslessly, avoiding binary float artifacts.
const SATS_PER_BTC: f64 = 100_000_000.0;

/// Cents per USD. The DCA price is stored as integer cents for the same
/// lossless-round-trip reason.
const CENTS_PER_USD: f64 = 100.0;

/// Load the stored user position as `(btc_balance, dca_price)`.
///
/// Returns `None` when no position has been saved yet. The single-row table
/// is created on first use so a fresh database works immediately.
pub fn load_position() -> Option<(f64, f64)> {
    let conn = Connection::open(position_db_path()).ok()?;
    let _ = conn.execute_batch(create_position_table_sql());
    conn.query_row(
        "SELECT btc_balance_sat, dca_price_cents FROM user_position WHERE id = 1",
        [],
        |row| {
            let sat: i64 = row.get(0)?;
            let cents: i64 = row.get(1)?;
            Ok(from_integer_units(sat, cents))
        },
    )
    .ok()
}

/// Persist the user position in the single `id = 1` row.
///
/// Creates the table and the parent directory on first use, inserts the row
/// on the first save, and updates it afterwards. The values are converted to
/// integer satoshis and cents before writing.
pub fn save_position(btc_balance: f64, dca_price: f64) {
    let db_path = position_db_path();
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let Ok(conn) = Connection::open(&db_path) else {
        eprintln!("[halvora] failed to open position database {db_path:?}");
        return;
    };
    if let Err(e) = conn.execute_batch(create_position_table_sql()) {
        eprintln!("[halvora] failed to create user_position table: {e}");
        return;
    }
    let (sat, cents) = to_integer_units(btc_balance, dca_price);
    if let Err(e) = conn.execute(
        "INSERT INTO user_position (id, btc_balance_sat, dca_price_cents) VALUES (1, ?1, ?2)
         ON CONFLICT(id) DO UPDATE SET
            btc_balance_sat = excluded.btc_balance_sat,
            dca_price_cents = excluded.dca_price_cents",
        rusqlite::params![sat, cents],
    ) {
        eprintln!("[halvora] failed to save user position: {e}");
    }
}

/// The `user_position` schema. One row holds the whole position, keyed by
/// `id = 1`, so loading and saving stay trivial.
fn create_position_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS user_position (
        id              INTEGER PRIMARY KEY CHECK (id = 1),
        btc_balance_sat INTEGER NOT NULL DEFAULT 0,
        dca_price_cents INTEGER NOT NULL DEFAULT 0
    );"
}

/// Convert a BTC balance and USD DCA price to integer units (satoshis, cents).
fn to_integer_units(btc_balance: f64, dca_price: f64) -> (i64, i64) {
    let sat = (btc_balance * SATS_PER_BTC).round();
    let cents = (dca_price * CENTS_PER_USD).round();
    (sat as i64, cents as i64)
}

/// Convert integer units (satoshis, cents) back to the `f64` values the UI uses.
fn from_integer_units(sat: i64, cents: i64) -> (f64, f64) {
    (sat as f64 / SATS_PER_BTC, cents as f64 / CENTS_PER_USD)
}
