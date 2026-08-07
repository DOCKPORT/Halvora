use futures_util::SinkExt;
use futures_util::StreamExt;
use iced::Subscription;
use iced::futures::channel::mpsc;
use iced::stream;
use serde::Deserialize;
use std::time::Duration;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

/// Bitstamp WebSocket endpoint.
const WS_URL: &str = "wss://ws.bitstamp.net";

/// Channel to subscribe to live BTC/USD trades.
const SUBSCRIBE_MSG: &str =
    r#"{"event": "bts:subscribe", "data": {"channel": "live_trades_btcusd"}}"#;

/// Delay before reconnecting after a disconnect.
const RECONNECT_DELAY_SECS: u64 = 5;

/// How often to send a heartbeat ping to keep the link alive.
const PING_INTERVAL_SECS: u64 = 45;

/// How long to wait for a reply after a heartbeat ping before reconnecting.
const LIVENESS_TIMEOUT_SECS: u64 = 5;

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
///
/// A heartbeat keeps the connection healthy:
/// - A ping is sent every `PING_INTERVAL_SECS` to keep NAT/firewall
///   mappings alive and to probe the link on quiet stretches.
/// - If no reply arrives within `LIVENESS_TIMEOUT_SECS` after a ping,
///   the link is treated as dead (for example, after a WiFi drop) and
///   the connection is re-established instead of blocking forever.
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

                // Heartbeat schedule. Skip the immediate first tick of the
                // interval so the first ping is not sent right after linking.
                let mut ping_interval =
                    tokio::time::interval(Duration::from_secs(PING_INTERVAL_SECS));
                ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                ping_interval.tick().await;

                // Armed only while a reply to a heartbeat ping is pending.
                let mut liveness_probe: Option<std::pin::Pin<Box<tokio::time::Sleep>>> = None;

                // Read incoming messages.
                loop {
                    tokio::select! {
                        biased;

                        msg = read.next() => {
                            // Any message counts as proof of life.
                            liveness_probe = None;

                            let msg = match msg {
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

                        _ = ping_interval.tick() => {
                            // Keep the link alive and start the liveness window.
                            let _ = write.send(Message::Ping(Vec::new().into())).await;
                            liveness_probe = Some(Box::pin(tokio::time::sleep(
                                Duration::from_secs(LIVENESS_TIMEOUT_SECS),
                            )));
                        }

                        _ = async {
                            if let Some(probe) = &mut liveness_probe {
                                probe.as_mut().await;
                            } else {
                                std::future::pending::<()>().await;
                            }
                        } => {
                            // No reply within the probe window. The link is dead.
                            eprintln!("[bitstamp ws] no reply during liveness probe, reconnecting");
                            break;
                        }
                    }
                }

                // Delay before reconnecting.
                tokio::time::sleep(Duration::from_secs(RECONNECT_DELAY_SECS)).await;
            }
        })
    })
}
