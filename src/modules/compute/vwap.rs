/// Volume-Weighted Average Price (VWAP) using only close price and volume.
///
/// # Formula
///
/// ```text
/// VWAP = Σ(close_i × volume_i) / Σ(volume_i)
/// ```
///
/// All functions operate on a slice of `(close, volume)` tuples.  
/// Volumes must be non-negative; entries with zero volume contribute nothing
/// to the numerator but are included in the denominator sum, which is
/// consistent with standard VWAP treatment.

/// Compute the **cumulative** VWAP over the entire price series.
///
/// Returns `None` when the slice is empty or total volume is zero.
///
/// # Examples
///
/// ```
/// use halvora::modules::compute::vwap::cumulative_vwap;
///
/// // Two periods: (100, 1000), (110, 2000)
/// // VWAP = (100*1000 + 110*2000) / (1000 + 2000) = 320000 / 3000 ≈ 106.6667
/// let prices = [(100.0, 1000.0), (110.0, 2000.0)];
/// let result = cumulative_vwap(&prices);
/// assert!((result.unwrap() - 106.66666666666667).abs() < 1e-10);
/// ```
pub fn cumulative_vwap(prices: &[(f64, f64)]) -> Option<f64> {
    let (sum_pv, sum_v) = prices.iter().fold((0.0_f64, 0.0_f64), |(spv, sv), &(c, v)| {
        (spv + c * v, sv + v)
    });

    if sum_v == 0.0 {
        return None;
    }

    Some(sum_pv / sum_v)
}

/// Compute a **rolling / sliding-window** VWAP.
///
/// The result vector has the same length as `prices`.  The first
/// `window_size - 1` entries are `None` because there aren't enough
/// data points to fill the window.
///
/// Uses an O(n) prefix-sum accumulator — constant time per element,
/// no re-scanning.
///
/// # Panics
///
/// Panics if `window_size == 0`.
///
/// # Examples
///
/// ```
/// use halvora::modules::compute::vwap::rolling_vwap;
///
/// let prices = [(100.0, 1000.0), (110.0, 2000.0), (105.0, 1500.0)];
/// let result = rolling_vwap(&prices, 2);
///
/// // Position 0: too few points → None
/// assert!(result[0].is_none());
///
/// // Position 1: window = [(100,1000), (110,2000)]
/// // VWAP = (100*1000 + 110*2000) / (1000+2000) = 106.6667
/// assert!((result[1].unwrap() - 106.66666666666667).abs() < 1e-10);
///
/// // Position 2: window = [(110,2000), (105,1500)]
/// // VWAP = (110*2000 + 105*1500) / (2000+1500) = 377500 / 3500 ≈ 107.8571
/// assert!((result[2].unwrap() - 107.85714285714286).abs() < 1e-10);
/// ```
pub fn rolling_vwap(prices: &[(f64, f64)], window_size: usize) -> Vec<Option<f64>> {
    assert!(window_size > 0, "window_size must be > 0");

    let n = prices.len();
    if n == 0 || window_size > n {
        return vec![None; n];
    }

    // Prefix sums of (close * volume) and volume.
    let mut cum_pv = Vec::with_capacity(n);
    let mut cum_v = Vec::with_capacity(n);
    let mut running_pv = 0.0_f64;
    let mut running_v = 0.0_f64;

    for &(c, v) in prices {
        running_pv += c * v;
        running_v += v;
        cum_pv.push(running_pv);
        cum_v.push(running_v);
    }

    let mut result = Vec::with_capacity(n);

    for i in 0..n {
        if i < window_size - 1 {
            result.push(None);
            continue;
        }

        let sum_pv = if i >= window_size {
            cum_pv[i] - cum_pv[i - window_size]
        } else {
            cum_pv[i]
        };

        let sum_v = if i >= window_size {
            cum_v[i] - cum_v[i - window_size]
        } else {
            cum_v[i]
        };

        if sum_v == 0.0 {
            result.push(None);
        } else {
            result.push(Some(sum_pv / sum_v));
        }
    }

    result
}

/// Compute **progressive (running) cumulative VWAP** at each index.
///
/// Each element `i` holds the VWAP of all periods from `0` to `i`
/// (inclusive).  The result has the same length as `prices`.
///
/// An entry is `None` when cumulative volume up to that index is zero.
///
/// Runs in O(n) time via a single-pass accumulator — no re-scanning.
///
/// # Examples
///
/// ```
/// use halvora::modules::compute::vwap::progressive_vwap;
///
/// let prices = [(100.0, 1000.0), (110.0, 2000.0), (105.0, 1500.0)];
/// let vwaps = progressive_vwap(&prices);
///
/// // Index 0: VWAP = 100.0
/// assert!((vwaps[0].unwrap() - 100.0).abs() < 1e-10);
///
/// // Index 1: (100*1000 + 110*2000) / (1000+2000) = 106.6667
/// assert!((vwaps[1].unwrap() - 106.66666666666667).abs() < 1e-10);
///
/// // Index 2: (100*1000 + 110*2000 + 105*1500) / (1000+2000+1500) ≈ 106.1111
/// assert!((vwaps[2].unwrap() - 106.11111111111111).abs() < 1e-10);
/// ```
pub fn progressive_vwap(prices: &[(f64, f64)]) -> Vec<Option<f64>> {
    let n = prices.len();
    if n == 0 {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(n);
    let mut sum_pv = 0.0_f64;
    let mut sum_v = 0.0_f64;

    for &(c, v) in prices {
        sum_pv += c * v;
        sum_v += v;

        if sum_v == 0.0 {
            result.push(None);
        } else {
            result.push(Some(sum_pv / sum_v));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cumulative_vwap_basic() {
        let prices = [(100.0, 1000.0), (110.0, 2000.0), (105.0, 1500.0)];
        // (100*1000 + 110*2000 + 105*1500) / (1000+2000+1500)
        // = (100000 + 220000 + 157500) / 4500
        // = 477500 / 4500 ≈ 106.11111...
        let vwap = cumulative_vwap(&prices).unwrap();
        let expected = 477_500.0 / 4_500.0;
        assert!((vwap - expected).abs() < 1e-10);
    }

    #[test]
    fn cumulative_vwap_empty() {
        assert!(cumulative_vwap(&[]).is_none());
    }

    #[test]
    fn cumulative_vwap_zero_volume() {
        let prices = [(100.0, 0.0), (110.0, 0.0)];
        assert!(cumulative_vwap(&prices).is_none());
    }

    #[test]
    fn cumulative_vwap_single_entry() {
        let prices = [(101.5, 500.0)];
        let vwap = cumulative_vwap(&prices).unwrap();
        assert!((vwap - 101.5).abs() < 1e-10);
    }

    #[test]
    fn cumulative_vwap_mixed_zero_volume() {
        // Zero-volume entries are ignored in the numerator but included
        // in the denominator.
        let prices = [(100.0, 1000.0), (110.0, 0.0), (105.0, 2000.0)];
        // (100*1000 + 105*2000) / (1000 + 0 + 2000) = 310000 / 3000 ≈ 103.3333
        let vwap = cumulative_vwap(&prices).unwrap();
        let expected = 310_000.0 / 3_000.0;
        assert!((vwap - expected).abs() < 1e-10);
    }

    // --- Rolling VWAP ---

    #[test]
    fn rolling_vwap_window_size_1() {
        let prices = [(100.0, 10.0), (110.0, 20.0), (105.0, 15.0)];
        // Window size 1 → each VWAP is just its own close.
        let vwaps = rolling_vwap(&prices, 1);
        assert!(vwaps.iter().all(|v| v.is_some()));
        assert!((vwaps[0].unwrap() - 100.0).abs() < 1e-10);
        assert!((vwaps[1].unwrap() - 110.0).abs() < 1e-10);
        assert!((vwaps[2].unwrap() - 105.0).abs() < 1e-10);
    }

    #[test]
    fn rolling_vwap_window_larger_than_input() {
        let prices = [(100.0, 1000.0), (110.0, 2000.0)];
        let vwaps = rolling_vwap(&prices, 10);
        assert_eq!(vwaps.len(), 2);
        assert!(vwaps[0].is_none());
        assert!(vwaps[1].is_none());
    }

    #[test]
    fn rolling_vwap_empty() {
        let vwaps = rolling_vwap(&[], 5);
        assert!(vwaps.is_empty());
    }

    #[test]
    fn rolling_vwap_zero_volume_window() {
        let prices = [(100.0, 0.0), (110.0, 0.0)];
        let vwaps = rolling_vwap(&prices, 2);
        assert_eq!(vwaps.len(), 2);
        assert!(vwaps[0].is_none());
        // sum_v == 0 → None
        assert!(vwaps[1].is_none());
    }

    #[test]
    fn rolling_vwap_documented_example() {
        let prices = [(100.0, 1000.0), (110.0, 2000.0), (105.0, 1500.0)];
        let result = rolling_vwap(&prices, 2);

        assert!(result[0].is_none());
        assert!((result[1].unwrap() - 106.66666666666667).abs() < 1e-10);
        assert!((result[2].unwrap() - 107.85714285714286).abs() < 1e-10);
    }

    #[test]
    #[should_panic(expected = "window_size must be > 0")]
    fn rolling_vwap_zero_window_size() {
        rolling_vwap(&[(1.0, 1.0)], 0);
    }

    // --- Progressive VWAP ---

    #[test]
    fn progressive_vwap_basic() {
        let prices = [(100.0, 1000.0), (110.0, 2000.0), (105.0, 1500.0)];
        let vwaps = progressive_vwap(&prices);

        assert_eq!(vwaps.len(), 3);

        // Index 0: only first candle
        assert!((vwaps[0].unwrap() - 100.0).abs() < 1e-10);

        // Index 1: (100*1000 + 110*2000) / (1000+2000) = 320000 / 3000 ≈ 106.6667
        assert!((vwaps[1].unwrap() - 106.66666666666667).abs() < 1e-10);

        // Index 2: all three
        let expected = 477_500.0 / 4_500.0;
        assert!((vwaps[2].unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn progressive_vwap_empty() {
        let result = progressive_vwap(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn progressive_vwap_single() {
        let prices = [(42.0, 100.0)];
        let vwaps = progressive_vwap(&prices);
        assert_eq!(vwaps.len(), 1);
        assert!((vwaps[0].unwrap() - 42.0).abs() < 1e-10);
    }

    #[test]
    fn progressive_vwap_zero_volume_starts() {
        // First entry has zero volume — index 0 should be None,
        // index 1 should have VWAP from the second candle only.
        let prices = [(100.0, 0.0), (110.0, 2000.0)];
        let vwaps = progressive_vwap(&prices);
        assert!(vwaps[0].is_none());
        assert!((vwaps[1].unwrap() - 110.0).abs() < 1e-10);
    }
}