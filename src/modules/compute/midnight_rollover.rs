use futures_util::SinkExt;
use iced::Subscription;
use iced::futures::channel::mpsc;
use iced::stream;
use std::time::Duration;

/// Detects when a new UTC day starts and yields the new day's midnight timestamp
/// 5 seconds after the rollover.
///
/// Checks every 30 seconds whether the current UTC midnight has advanced
/// past the last recorded day. On detection, waits 5 seconds before emitting
/// the new timestamp so Bitstamp has time to publish the new candle.
pub fn detect() -> Subscription<i64> {
    Subscription::run(|| {
        stream::channel(100, |mut output: mpsc::Sender<i64>| async move {
            let mut last_day = current_utc_midnight();

            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;

                let today = current_utc_midnight();

                if today > last_day {
                    // New day detected — wait 5 seconds for Bitstamp to have data.
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    let _ = output.send(today).await;
                    last_day = today;
                }
            }
        })
    })
}

fn current_utc_midnight() -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    now - (now % 86_400)
}
