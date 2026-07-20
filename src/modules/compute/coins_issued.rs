/// Number of blocks between each halving.
const HALVING_INTERVAL: u64 = 210_000;

/// Starting subsidy in satoshis (50 BTC).
const INITIAL_SUBSIDY_SAT: u64 = 5_000_000_000;

/// Maximum number of halving eras (32 halvings → 33 eras including genesis).
const MAX_ERA: u64 = 33;

/// Maximum BTC supply (hard cap).
const MAX_SUPPLY_BTC: u64 = 21_000_000;

/// Shared helper: compute the total satoshis minted at the given block height.
fn total_sats_minted_at(height: u64) -> u64 {
    let mut total_sats: u64 = 0;

    for era in 0..MAX_ERA {
        let era_start = era * HALVING_INTERVAL;
        let era_end   = era_start + HALVING_INTERVAL - 1;

        if height < era_start {
            break;
        }

        let blocks_in_era = if height >= era_end {
            HALVING_INTERVAL
        } else {
            height - era_start + 1
        };

        let subsidy_sat = INITIAL_SUBSIDY_SAT >> era;
        if subsidy_sat == 0 {
            break;
        }

        total_sats = total_sats.saturating_add(blocks_in_era * subsidy_sat);
    }

    total_sats
}

/// Format a BTC whole number (u64) with thousands commas and 2 decimal places.
fn format_btc(whole_btc: u64, cents: u64) -> String {
    let whole = whole_btc.to_string();
    let mut result = String::with_capacity(whole.len() + whole.len() / 3 + 3);
    for (i, c) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.push('.');
    result.push_str(&format!("{:02}", cents));
    result
}

/// Compute the total BTC issued (minted) up to the current tip height.
///
/// Returns a string with thousands separators and 2 decimals, e.g. `"19,542,150.00"`.
pub fn coins_issued(current_tip_height: u32) -> String {
    let height = current_tip_height as u64;
    if height == 0 {
        return "0.00".to_string();
    }

    let total_sats = total_sats_minted_at(height);

    let whole_btc = total_sats / 100_000_000;
    let frac_sats = total_sats % 100_000_000;
    let cents = (frac_sats * 100) / 100_000_000;

    format_btc(whole_btc, cents)
}

/// Compute the percentage of the 21M hard cap that has been issued.
///
/// Returns a string like `"95.52%"`.
pub fn percentage_issued(current_tip_height: u32) -> String {
    let height = current_tip_height as u64;
    if height == 0 {
        return "0.00%".to_string();
    }

    let total_sats = total_sats_minted_at(height);
    let whole_btc = total_sats / 100_000_000;
    let frac_sats = total_sats % 100_000_000;
    let cents = (frac_sats * 100) / 100_000_000;

    let mined_btc = whole_btc as f64 + (cents as f64 / 100.0);
    let pct = (mined_btc / MAX_SUPPLY_BTC as f64) * 100.0;

    format!("{:.2}%", pct)
}

/// Compute the remaining BTC to be issued until the 21M hard cap.
///
/// Returns a string with thousands separators and 2 decimals, e.g. `"940,846.88"`.
pub fn remaining_issuance(current_tip_height: u32) -> String {
    let height = current_tip_height as u64;
    if height == 0 {
        return "21,000,000.00".to_string();
    }

    let total_sats = total_sats_minted_at(height);
    let whole_btc = total_sats / 100_000_000;
    let frac_sats = total_sats % 100_000_000;
    let cents = (frac_sats * 100) / 100_000_000;

    let mined_btc = whole_btc as f64 + (cents as f64 / 100.0);
    let remaining_btc = MAX_SUPPLY_BTC as f64 - mined_btc;

    let remaining_whole = remaining_btc.floor() as u64;
    let remaining_cents = ((remaining_btc - remaining_btc.floor()) * 100.0).round() as u64;

    format_btc(remaining_whole, remaining_cents)
}