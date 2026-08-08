/// Volume-Weighted Average Price (VWAP) using only close price and volume.
///
/// # Formula
///
/// ```text
/// VWAP = Σ(close_i × volume_i) / Σ(volume_i)
/// ```
///
/// Zero-volume candles add nothing to the numerator but still count in the
/// denominator.
///
/// Returns the cumulative VWAP anchored at index 0 for every candle, or
/// `None` when the running volume is zero.
///
/// # Example
///
/// ```
/// use halvora::modules::compute::vwap::anchored_vwap;
///
/// let prices = [(100.0, 1000.0), (110.0, 2000.0), (105.0, 1500.0)];
/// let vwaps = anchored_vwap(&prices);
///
/// assert!((vwaps[0].unwrap() - 100.0).abs() < 1e-10);
/// assert!((vwaps[1].unwrap() - 106.66666666666667).abs() < 1e-10);
/// assert!((vwaps[2].unwrap() - 106.11111111111111).abs() < 1e-10);
/// ```
pub fn anchored_vwap(prices: &[(f64, f64)]) -> Vec<Option<f64>> {
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
    fn anchored_vwap_basic() {
        let prices = [(100.0, 1000.0), (110.0, 2000.0), (105.0, 1500.0)];
        let vwaps = anchored_vwap(&prices);

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
    fn anchored_vwap_empty() {
        let result = anchored_vwap(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn anchored_vwap_single() {
        let prices = [(42.0, 100.0)];
        let vwaps = anchored_vwap(&prices);
        assert_eq!(vwaps.len(), 1);
        assert!((vwaps[0].unwrap() - 42.0).abs() < 1e-10);
    }

    #[test]
    fn anchored_vwap_zero_volume_starts() {
        // First entry has zero volume — index 0 should be None,
        // index 1 should have VWAP from the second candle only.
        let prices = [(100.0, 0.0), (110.0, 2000.0)];
        let vwaps = anchored_vwap(&prices);
        assert!(vwaps[0].is_none());
        assert!((vwaps[1].unwrap() - 110.0).abs() < 1e-10);
    }
}