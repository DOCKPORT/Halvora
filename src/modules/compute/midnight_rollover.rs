use futures_util::SinkExt;
use iced::Subscription;
use iced::futures::channel::mpsc;
use iced::stream;
use std::time::Duration;

/// Detects when a new UTC day starts and yields the new day's midnight timestamp
/// 5 seconds after the rollover.
///
/// Sleeps until the next UTC midnight (plus a 5-second buffer) so the emitter
/// wakes only on rollover instead of polling every 30 seconds. Sends the new
/// timestamp so Bitstamp has time to publish the new candle.
pub fn detect() -> Subscription<i64> {
    Subscription::run(|| {
        stream::channel(100, |mut output: mpsc::Sender<i64>| async move {
            let mut last_day = current_utc_midnight();

            loop {
                // Sleep until the next UTC midnight, then wait 5 seconds so
                // Bitstamp has time to publish the new candle.
                let now = now_secs();
                tokio::time::sleep(Duration::from_secs(seconds_until_next_midnight(now))).await;
                tokio::time::sleep(Duration::from_secs(5)).await;

                let today = current_utc_midnight();
                // Guard against a system clock that moves backwards.
                if today > last_day {
                    let _ = output.send(today).await;
                    last_day = today;
                }
            }
        })
    })
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

fn current_utc_midnight() -> i64 {
    let now = now_secs();
    now - (now % 86_400)
}

/// Seconds until the next UTC midnight boundary from `now`.
fn seconds_until_next_midnight(now: i64) -> u64 {
    let since_midnight = now.rem_euclid(86_400);
    (86_400 - since_midnight) as u64
}