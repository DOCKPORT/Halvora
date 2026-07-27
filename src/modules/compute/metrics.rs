use crate::modules::compute::year_over_year::Candle;

/// Display-ready values for the top metric cards.
#[derive(Debug, Clone)]
pub struct Metrics {
    pub p_l: String,
    pub high: String,
    pub low: String,
    pub draw_down: String,
    pub run_up: String,
    pub calmar: String,
}

/// Compute all top-bar metrics from a slice of daily candles.
///
/// Each field is a formatted display string.  When the input is empty or a
/// metric cannot be computed the field shows an em-dash `"—"`.
///
/// # Logic stubs
///
/// Currently all fields render `"—"`.  The computation logic will be filled
/// in next — the wiring to the UI is already in place.
pub fn compute(candles: &[Candle]) -> Metrics {
    let dash = "\u{2014}".to_string();

    Metrics {
        p_l: dash.clone(),
        high: dash.clone(),
        low: dash.clone(),
        draw_down: dash.clone(),
        run_up: dash.clone(),
        calmar: dash,
    }
}