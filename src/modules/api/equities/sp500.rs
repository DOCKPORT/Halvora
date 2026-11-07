//! Fetch the current S&P 500 value from the public Ez-Fred data feed.
//!
//! The feed is a single JSON document with a `series` map. The `SP500` entry
//! carries the most recent index value under `latest.value`. Only that value
//! is needed for the Brass Tacks comparison, so the rest is ignored.

use serde::Deserialize;
use std::time::Duration;

/// The public JSON feed that holds the latest equity index values.
const URL: &str = "https://raw.githubusercontent.com/DOCKPORT/Ez-Fred/main/Data/equities.json";

/// How long a fetch may run before it fails. Without a timeout a dead or slow
/// server can hang the fetch thread forever, so the task never resolves.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Top-level feed document.
#[derive(Deserialize)]
struct EquitiesFeed {
    series: SeriesMap,
}

/// The `series` object, narrowed to the single index we read.
#[derive(Deserialize)]
struct SeriesMap {
    #[serde(rename = "SP500")]
    sp500: Series,
}

/// One index series.
#[derive(Deserialize)]
struct Series {
    latest: Latest,
}

/// The most recent data point of a series.
#[derive(Deserialize)]
struct Latest {
    value: f64,
}

/// Fetch the current S&P 500 value.
///
/// Returns `None` on any network or parse error. This uses the blocking HTTP
/// client, so it must run on an OS thread and never inside iced's runtime.
pub fn fetch_latest() -> Option<f64> {
    let client = reqwest::blocking::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .ok()?;
    let text = client.get(URL).send().ok()?.text().ok()?;
    let feed: EquitiesFeed = serde_json::from_str(&text).ok()?;
    Some(feed.series.sp500.latest.value)
}
