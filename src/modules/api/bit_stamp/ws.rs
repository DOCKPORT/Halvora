use futures_util::SinkExt;
use futures_util::StreamExt;
use iced::futures::channel::mpsc;
use iced::stream;
use iced::Subscription;
use serde::Deserialize;
use std::time::Duration;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

/// Bitstamp WebSocket endpoint.
const WS_URL: &str = "wss://ws.bitstamp.net";

/// Channel to subscribe to live BTC/USD trades.
const SUBSCRIBE_MSG: &str = r#"{"event": "bts:subscribe", "data": {"channel": "live_trades_btcusd"}}"#;

/// Delay before reconnecting after a disconnect.
const RECONNECT_DELAY_SECS: u64 = 5;

// ── JSON response shapes ────────────────────────────────────────────────

/// Incoming Bitstamp WebSocket message envelope.
/// Trade messages don't include an `event` field; subscription confirmation does.
/// Keep `event` optional so both message shapes parse correctly.
#[derive(Deserialize)]
struct WsMessage {
    #[serde(default)]
    event: String,
    data: Option<TradeData>,
}

#[derive(Deserialize)]
struct TradeData {
    price: Option<f64>,
}

/// Return an iced `Subscription` that streams live BTC/USD trade prices.
///
/// The subscription connects to Bitstamp's WebSocket, subscribes to
/// `live_trades_btcusd`, and yields `f64` prices for every new trade.
/// Automatically reconnects on disconnect with a 5-second delay.
pub fn live_price() -> Subscription<f64> {
    Subscription::run(|| {
        stream::channel(100, |mut output: mpsc::Sender<f64>| async move {
            loop {
                eprintln!("[bitstamp ws] connecting to {}", WS_URL);

                let ws = match connect_async(WS_URL).await {
                    Ok((ws, _)) => {
                        eprintln!("[bitstamp ws] connected");
                        ws
                    }
                    Err(e) => {
                        eprintln!(
                            "[bitstamp ws] connection failed ({}), retrying in {}s",
                            e, RECONNECT_DELAY_SECS
                        );
                        tokio::time::sleep(Duration::from_secs(RECONNECT_DELAY_SECS)).await;
                        continue;
                    }
                };

                let (mut write, mut read) = ws.split();

                // Send subscription message.
                if let Err(e) = write
                    .send(Message::Text(SUBSCRIBE_MSG.to_string().into()))
                    .await
                {
                    eprintln!("[bitstamp ws] subscribe failed ({}), reconnecting", e);
                    tokio::time::sleep(Duration::from_secs(RECONNECT_DELAY_SECS)).await;
                    continue;
                }

                // Read incoming messages.
                loop {
                    let msg = match read.next().await {
                        Some(Ok(msg)) => msg,
                        Some(Err(e)) => {
                            eprintln!("[bitstamp ws] read error ({}), reconnecting", e);
                            break;
                        }
                        None => {
                            eprintln!("[bitstamp ws] connection closed, reconnecting");
                            break;
                        }
                    };

                    let text = match &msg {
                        Message::Text(t) => t.to_string(),
                        Message::Ping(p) => {
                            let _ = write.send(Message::Pong(p.clone())).await;
                            continue;
                        }
                        Message::Pong(_) | Message::Binary(_) | Message::Frame(_) | Message::Close(_) => {
                            continue;
                        }
                    };

                    // Parse the message. Trade messages don't have an `event` field,
                    // subscription confirmation does — both parse fine now.
                    let Ok(parsed) = serde_json::from_str::<WsMessage>(&text) else {
                        continue;
                    };

                    // Skip subscription confirmation messages (have no `data.price`).
                    let Some(data) = parsed.data else {
                        continue;
                    };

                    let Some(price) = data.price else {
                        continue;
                    };

                    // Yield the price into the iced message loop.
                    let _ = output.send(price).await;
                }

                // Delay before reconnecting.
                tokio::time::sleep(Duration::from_secs(RECONNECT_DELAY_SECS)).await;
            }
        })
    })
}