//! Build script for Halvora.
//!
//! Halvora embeds the pre-compiled Bitstamp database into the binary so users
//! start with full price history on the first run. The database is embedded
//! from `OUT_DIR` (see `bitstamp_data.rs`), and this script keeps that copy in
//! sync with the repository file.
//!
//! A plain `include_bytes!` on the repository file is unreliable: cargo and
//! rustc track included files by mtime, so replacing the database with a copy
//! that keeps an older timestamp (for example a `mv`, `cp -p`, or a download)
//! can silently reuse a cached compilation and embed stale bytes. Declaring
//! the database as an input of this script and copying it into `OUT_DIR` gives
//! every database update a deterministic path into the binary.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));

    let db_src = manifest_dir.join("bitstamp_data").join("btcusd.db");
    let db_dst = out_dir.join("btcusd.db");

    // Re-run this script whenever the database changes. Cargo treats a changed
    // script input as a changed crate, so the binary is rebuilt with the new
    // database bytes.
    println!("cargo:rerun-if-changed={}", db_src.display());

    fs::copy(&db_src, &db_dst).unwrap_or_else(|e| {
        panic!(
            "failed to copy {} to {}: {e}",
            db_src.display(),
            db_dst.display()
        )
    });
}
