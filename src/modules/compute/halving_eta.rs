/// Number of blocks between each halving.
const HALVING_INTERVAL: u32 = 210_000;

/// Total halvings (1–32).
const HALVING_COUNT: u32 = 32;

/// Minutes per block assumption.
const MINUTES_PER_BLOCK: u64 = 10;

/// Shared helper: find blocks remaining until the next halving.
/// Returns `None` if all halvings are past.
fn blocks_remaining_until_next_halving(current_tip_height: u32) -> Option<u64> {
    let next_height = (1..=HALVING_COUNT)
        .map(|n| n * HALVING_INTERVAL)
        .find(|&h| h > current_tip_height)?;
    Some((next_height - current_tip_height) as u64)
}

/// Format a number with thousands commas.
fn with_thousands_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

/// Compute the number of blocks remaining until the next halving.
///
/// Returns a string like `"97,780"` or `"0"` if all 32 halvings are past.
pub fn blocks_to_next_halving(current_tip_height: u32) -> String {
    match blocks_remaining_until_next_halving(current_tip_height) {
        Some(blocks) => with_thousands_commas(blocks),
        None => "0".to_string(),
    }
}

/// Compute an ETA string for the next halving based on the current tip height.
///
/// Returns a string like `"~1y 7m 14d"` or `"Halvings Completed"` if all 32 halvings are past.
pub fn next_halving_eta(current_tip_height: u32) -> String {
    let Some(blocks_remaining) = blocks_remaining_until_next_halving(current_tip_height) else {
        return "Halvings Completed".to_string();
    };
    let minutes_remaining = blocks_remaining * MINUTES_PER_BLOCK;

    let total_minutes = minutes_remaining;
    let total_days = total_minutes / (60 * 24);

    let years = total_days / 365;
    let months = (total_days % 365) / 30;
    let days = (total_days % 365) % 30;

    format!("~{}y {}m {}d", years, months, days)
}