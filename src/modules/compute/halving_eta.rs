/// Number of blocks between each halving.
const HALVING_INTERVAL: u32 = 210_000;

/// Total halvings (1–32).
const HALVING_COUNT: u32 = 32;

/// Minutes per block assumption.
const MINUTES_PER_BLOCK: u64 = 10;

/// Compute an ETA string for the next halving based on the current tip height.
///
/// Returns a string like `"~1y 7m 14d"` or `"Halvings Completed"` if all 32 halvings are past.
pub fn next_halving_eta(current_tip_height: u32) -> String {
    // Find the first halving whose height is greater than the tip.
    let next_height = (1..=HALVING_COUNT)
        .map(|n| n * HALVING_INTERVAL)
        .find(|&h| h > current_tip_height);

    let Some(next_height) = next_height else {
        return "Halvings Completed".to_string();
    };

    let blocks_remaining = (next_height - current_tip_height) as u64;
    let minutes_remaining = blocks_remaining * MINUTES_PER_BLOCK;

    let total_minutes = minutes_remaining;
    let total_days = total_minutes / (60 * 24);

    let years = total_days / 365;
    let months = (total_days % 365) / 30;
    let days = (total_days % 365) % 30;

    format!("~{}y {}m {}d", years, months, days)
}