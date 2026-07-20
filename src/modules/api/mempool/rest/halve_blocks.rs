use rusqlite::Connection;
use serde::Deserialize;
use std::path::PathBuf;

/// Total number of halvings (each 210,000 blocks apart).
const HALVING_COUNT: u32 = 32;

/// Blocks between each halving.
const HALVING_INTERVAL: u32 = 210_000;

/// Starting block subsidy in satoshis (before any halving). 50 BTC = 5 000 000 000 sat.
const INITIAL_SUBSIDY_SAT: i64 = 5_000_000_000;

// ── JSON response shapes ────────────────────────────────────────────────

/// Shape of a block object from the mempool.space API.
#[derive(Deserialize)]
struct BlockJson {
    height: u32,
    timestamp: u64,
}


// ── Public entry point ──────────────────────────────────────────────────

/// Minimum seconds between API fetches (≈1 Bitcoin block interval).
const FETCH_COOLDOWN_SECS: u64 = 600;

/// Fetch halving blocks and the current tip from mempool.space,
/// then store them in the SQLite database at `~/.local/share/Halvora/Mempool/blocks.db`.
///
/// Call this once at application startup.
pub fn fetch_and_store() {
    let db_path = db_path();

    // Check cooldown before making any network request.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if let Ok(conn) = Connection::open(&db_path) {
        let stored: Option<(i64, i64)> = conn
            .query_row(
                "SELECT timestamp, height FROM current_tip LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();
        if let Some((ts, height)) = stored {
            let elapsed = now.saturating_sub(ts as u64);
            if elapsed < FETCH_COOLDOWN_SECS {
                let remaining = FETCH_COOLDOWN_SECS - elapsed;
                eprintln!(
                    "[mempool] cooldown – {}m {}s until next fetch (current tip: #{})",
                    remaining / 60,
                    remaining % 60,
                    height,
                );
                return;
            }
        }
    }

    let Some(tip) = fetch_latest_block() else {
        eprintln!("[mempool] could not fetch current tip – skipping sync");
        return;
    };

    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[mempool] failed to open database {:?}: {}", db_path, e);
            return;
        }
    };

    if let Err(e) = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS halve_blocks (
            halving_number INTEGER PRIMARY KEY,
            height          INTEGER NOT NULL,
            timestamp       INTEGER,
            subsidy         INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS current_tip (
            height    INTEGER NOT NULL,
            timestamp INTEGER NOT NULL,
            subsidy   INTEGER NOT NULL
        );",
    ) {
        eprintln!("[mempool] failed to create tables: {}", e);
        return;
    }

    // Ensure all 32 rows exist (with NULL timestamp for future halvings).
    for n in 1..=HALVING_COUNT {
        conn.execute(
            "INSERT OR IGNORE INTO halve_blocks (halving_number, height, subsidy)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![n, n * HALVING_INTERVAL, INITIAL_SUBSIDY_SAT / (1i64 << n)],
        ).ok();
    }

    // Derive the current subsidy from the most recent halving ≤ tip height.
    let subsidy: i64 = conn
        .query_row(
            "SELECT subsidy FROM halve_blocks WHERE height <= ?1 ORDER BY height DESC LIMIT 1",
            [tip.height],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Update the single current-tip row (already exists from first sync).
    if let Err(e) = conn.execute(
        "UPDATE current_tip SET height = ?1, timestamp = ?2, subsidy = ?3",
        rusqlite::params![tip.height, tip.timestamp as i64, subsidy],
    ) {
        eprintln!("[mempool] failed to update current_tip: {}", e);
        return;
    }

    // Fill timestamps for any halving that has been reached but still has NULL.
    for n in 1..=HALVING_COUNT {
        let height = n * HALVING_INTERVAL;
        if height > tip.height {
            break;
        }
        let has_ts: bool = conn
            .query_row(
                "SELECT timestamp IS NOT NULL FROM halve_blocks WHERE halving_number = ?1",
                [n as i64],
                |row| row.get(0),
            )
            .unwrap_or(false);
        if !has_ts {
            if let Some((_, ts)) = fetch_single_block(height) {
                upsert_halve_block(&conn, n as i64, height, Some(ts as i64));
            }
        }
    }

    eprintln!(
        "[mempool] sync complete – tip at height {}",
        tip.height,
    );
}

// ── Internal helpers ────────────────────────────────────────────────────

/// Return the path to the SQLite database.
fn db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Halvora").join("Mempool").join("blocks.db")
}

/// Fetch the latest block from the mempool.space API.
fn fetch_latest_block() -> Option<BlockJson> {
    let url = "https://mempool.space/api/blocks/tip";
    let text = reqwest::blocking::get(url).ok()?.text().ok()?;
    let blocks: Vec<BlockJson> = serde_json::from_str(&text).ok()?;
    blocks.into_iter().next()
}

/// Fetch a single block by height using the block-height → hash → block chain.
fn fetch_single_block(height: u32) -> Option<(u32, u64)> {
    // Step 1: get the block hash for this height.
    let hash_url = format!("https://mempool.space/api/block-height/{}", height);
    let hash = reqwest::blocking::get(&hash_url).ok()?.text().ok()?;
    let hash = hash.trim().to_string(); // remove trailing newline

    // Step 2: get the full block JSON.
    let block_url = format!("https://mempool.space/api/block/{}", hash);
    let text = reqwest::blocking::get(&block_url).ok()?.text().ok()?;
    let block: BlockJson = serde_json::from_str(&text).ok()?;

    Some((block.height, block.timestamp))
}

/// Insert or replace a row in the halve_blocks table.
/// `timestamp` can be `None` for blocks that have not been mined yet.
fn upsert_halve_block(conn: &Connection, halving_number: i64, height: u32, timestamp: Option<i64>) {
    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO halve_blocks (halving_number, height, timestamp, subsidy)
         VALUES (?1, ?2, ?3, (SELECT subsidy FROM halve_blocks WHERE halving_number = ?1))",
        rusqlite::params![halving_number, height, timestamp],
    ) {
        eprintln!(
            "[mempool] failed to upsert halving block {}: {}",
            height, e
        );
    }
}