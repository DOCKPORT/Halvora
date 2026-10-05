/// Number of blocks between each halving.
const HALVING_INTERVAL: u32 = 210_000;

/// Total halvings (1–32).
const HALVING_COUNT: u32 = 32;

/// Minutes per block assumption.
const MINUTES_PER_BLOCK: u64 = 10;

/// Minutes per day.
const MINUTES_PER_DAY: u64 = 60 * 24;

/// Minutes per year. A year is 365 days.
const MINUTES_PER_YEAR: u64 = MINUTES_PER_DAY * 365;

/// Minutes per month. A month is 365/12 days, so twelve months equal one year.
/// This value is exact: 525,600 / 12 = 43,800.
const MINUTES_PER_MONTH: u64 = MINUTES_PER_YEAR / 12;

/// Shared helper: find blocks remaining until the next halving.
/// Returns `None` if all halvings are past.
fn blocks_remaining_until_next_halving(current_tip_height: u32) -> Option<u64> {
    // The next halving is the first multiple of the interval strictly above
    // the current height. Computed directly instead of scanning 1..=32.
    // Use u64 for the multiply to rule out any overflow near u32::MAX.
    let height = u64::from(current_tip_height);
    let next_height = ((height / u64::from(HALVING_INTERVAL)) + 1) * u64::from(HALVING_INTERVAL);
    // When the next height is past the final halving, all halvings are done.
    if next_height > u64::from(HALVING_COUNT) * u64::from(HALVING_INTERVAL) {
        return None;
    }
    Some(next_height - (current_tip_height as u64))
}

/// Format a number with thousands commas.
fn with_thousands_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    result
}

/// Format a duration in minutes as "~Yy Mm Dd".
///
/// The duration stays in minutes until the final step, so no precision is lost.
/// A year is 365 days and a month is 365/12 days, so twelve months equal one
/// year exactly.
fn format_eta(minutes: u64) -> String {
    let years = minutes / MINUTES_PER_YEAR;
    let after_years = minutes % MINUTES_PER_YEAR;
    let months = after_years / MINUTES_PER_MONTH;
    let after_months = after_years % MINUTES_PER_MONTH;
    let days = after_months / MINUTES_PER_DAY;
    format!("~{years}y {months}m {days}d")
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
    format_eta(blocks_remaining * MINUTES_PER_BLOCK)
}

/// Compute an ETA string for a specific halving number based on the current
/// tip height.
///
/// Returns `"~0y 0m 0d"` when that halving has already been reached.
pub fn halving_eta(current_tip_height: u32, halving_number: u32) -> String {
    let height = halving_number.saturating_mul(HALVING_INTERVAL);
    let blocks_remaining = height.saturating_sub(current_tip_height);
    format_eta(u64::from(blocks_remaining) * MINUTES_PER_BLOCK)
}
