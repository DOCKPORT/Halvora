use iced::widget::{container, mouse_area, row, text};
use iced::{Element, Subscription, window, Font, Length};
use iced::window::Position;
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use crate::modules::compute::metrics::Metrics;
use crate::modules::compute::year_over_year::Candle;
use crate::modules::ui::line_chart::LineChartState;
use crate::modules::ui::scaling::Scaling;
use crate::modules::ui::mainwindow::dashboard_layout::dashboard;
use crate::modules::ui::mainwindow::sidebar::halving_sidebar;
use crate::modules::ui::mainwindow::sidebar::blockchain_sidebar;

/// Embed the GeistMono font as fallback — the system-installed SemiBold
/// variant will be used via the Font weight setting.
const GEIST_MONO_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/Font/geist-font/GeistMono/ttf/GeistMono-Regular.ttf"
));

pub fn run() -> iced::Result {
    let scaling = Scaling::global();
    let screen_size = scaling.screen_size;

    let window_settings = window::Settings {
        #[cfg(target_os = "linux")]
        size: screen_size,
        #[cfg(target_os = "linux")]
        position: Position::Centered,
        #[cfg(not(target_os = "linux"))]
        maximized: true,
        ..Default::default()
    };

    iced::application(Halvora::new, update, view)
        .font(GEIST_MONO_BYTES)
        .default_font(Font::with_name("Geist Mono"))
        .window(window_settings)
        .subscription(subscription)
        .run()
}

struct Halvora {
    selected_halving: Option<u32>,
    yoy_selected: bool,
    current_tip_height: u32,
    current_subsidy_sat: i64,
    next_halving_eta: String,
    blocks_to_next_halving: String,
    coins_issued: String,
    percentage_issued: String,
    remaining_issuance: String,
    live_price: Option<f64>,
    subsidy_value: String,
    sats_per_usd: String,
    all_time_high: String,
    metrics: Metrics,
    /// ETA to the selected halving, shown on future halving pages that have
    /// no data yet. `None` when YOY is active or no halving is selected.
    selected_halving_eta: Option<String>,
    /// Block subsidy of the selected halving in BTC, shown on future halving
    /// pages. `None` when YOY is active or no halving is selected.
    selected_halving_subsidy: Option<String>,
    /// Subsidy to show in the metric bar for the active period, in BTC.
    subsidy_label: String,
    /// Cached Year-Over-Year candles used when the YOY page is active.
    /// `line_chart_state.candles` holds the currently displayed page's data.
    yoy_candles: Vec<Candle>,
    line_chart_state: LineChartState,
    volume_sync_start: Instant,
    show_calmar_dialog: bool,
}

impl Halvora {
    /// Decide the "current" price used by the metric bar's P/L.
    ///
    /// A completed period ends in the past, so its last candle close is the
    /// true end and the websocket price must not feed the P/L. A live period
    /// (YOY, or the current halving) ends today, so the websocket price is
    /// the running close.
    fn metric_current_price(state: &Halvora) -> Option<f64> {
        let last_ts = state.line_chart_state.candles.last().map(|c| c.timestamp)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let today_midnight = now - (now % 86_400);
        if last_ts == today_midnight {
            // Live period: use the websocket price as today's running close.
            state.live_price
        } else {
            // Completed period: use its final candle close.
            state.line_chart_state.candles.last().map(|c| c.close)
        }
    }

    fn new() -> Self {
        let current_tip_height = Self::load_tip_height();
        let current_subsidy_sat = Self::load_current_subsidy();
        let next_halving_eta = crate::modules::compute::halving_eta::next_halving_eta(current_tip_height);
        let blocks_to_next_halving = crate::modules::compute::halving_eta::blocks_to_next_halving(current_tip_height);
        let coins_issued = crate::modules::compute::coins_issued::coins_issued(current_tip_height);
        let percentage_issued = crate::modules::compute::coins_issued::percentage_issued(current_tip_height);
        let remaining_issuance = crate::modules::compute::coins_issued::remaining_issuance(current_tip_height);
        let all_time_high = crate::modules::compute::price_stats::all_time_high(None);

        let candles = crate::modules::compute::year_over_year::trailing_365_candles();

        let metrics = crate::modules::compute::metrics::compute(&candles, None);

        Self {
            selected_halving: None,
            yoy_selected: true,
            current_tip_height,
            current_subsidy_sat,
            next_halving_eta,
            blocks_to_next_halving,
            coins_issued,
            percentage_issued,
            remaining_issuance,
            live_price: None,
            subsidy_value: crate::modules::compute::price_stats::subsidy_value(None, current_subsidy_sat),
            sats_per_usd: crate::modules::compute::price_stats::sats_per_usd(None),
            all_time_high,
            metrics,
            selected_halving_eta: None,
            selected_halving_subsidy: None,
            subsidy_label: crate::modules::compute::halving_period::subsidy_btc_from_sat(
                current_subsidy_sat,
            ),
            yoy_candles: candles.clone(),
            line_chart_state: LineChartState::new(candles),
            volume_sync_start: Instant::now(),
            show_calmar_dialog: false,
        }
    }

    /// Query the most recent tip height from the database.
    fn load_tip_height() -> u32 {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let db_path = base.join("Halvora").join("Mempool").join("blocks.db");
        if let Ok(conn) = Connection::open(&db_path) {
            if let Ok(height) = conn.query_row(
                "SELECT height FROM current_tip LIMIT 1",
                [],
                |row| row.get(0),
            ) {
                return height;
            }
        }
        0
    }

    /// Query the current subsidy (sats) from the database.
    fn load_current_subsidy() -> i64 {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let db_path = base.join("Halvora").join("Mempool").join("blocks.db");
        if let Ok(conn) = Connection::open(&db_path) {
            if let Ok(subsidy) = conn.query_row(
                "SELECT subsidy FROM current_tip LIMIT 1",
                [],
                |row| row.get(0),
            ) {
                return subsidy;
            }
        }
        0
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    HalvingSelected(u32),
    YoYSelected,
    Tick,
    LivePrice(f64),
    NewDay(i64),
    CalmarClicked,
    CloseCalmarDialog,
    SelectAVWAP,
    SelectRange,
}

fn subscription(_state: &Halvora) -> Subscription<Message> {
    Subscription::batch(vec![
        iced::time::every(std::time::Duration::from_secs(600)).map(|_| Message::Tick),
        crate::modules::api::bit_stamp::ws::live_price().map(Message::LivePrice),
        crate::modules::compute::midnight_rollover::detect().map(Message::NewDay),
    ])
}

fn update(state: &mut Halvora, message: Message) {
    match message {
        Message::HalvingSelected(n) => {
            state.selected_halving = Some(n);
            state.yoy_selected = false;
            // Record the ETA and subsidy for a future halving page.
            state.selected_halving_eta =
                Some(crate::modules::compute::halving_eta::halving_eta(
                    state.current_tip_height,
                    n,
                ));
            state.selected_halving_subsidy =
                Some(crate::modules::compute::halving_period::halving_subsidy_btc(n));
            // The metric bar shows this halving's subsidy.
            state.subsidy_label =
                crate::modules::compute::halving_period::halving_subsidy_btc(n);
            // Load this halving period's candles. Future halvings return an
            // empty set, so `metrics::compute` naturally produces dashes.
            // Use set_candles so the drawing tool and drawings are preserved.
            let candles = crate::modules::compute::halving_period::halving_period_candles(n);
            state.line_chart_state.set_candles(candles);
            state.metrics = crate::modules::compute::metrics::compute(
                &state.line_chart_state.candles,
                Halvora::metric_current_price(state),
            );
            // Close the YOY-only Calmar breakdown if it is open.
            state.show_calmar_dialog = false;
            state.line_chart_state.dialog_open.set(false);
        }
        Message::YoYSelected => {
            state.yoy_selected = true;
            state.selected_halving = None;
            state.selected_halving_eta = None;
            state.selected_halving_subsidy = None;
            // Metric bar shows the current tip subsidy on YOY.
            state.subsidy_label =
                crate::modules::compute::halving_period::subsidy_btc_from_sat(
                    state.current_subsidy_sat,
                );
            // Restore the cached YOY candles and recompute metrics for them.
            // Use set_candles so drawings are preserved across page switches.
            state.line_chart_state.set_candles(state.yoy_candles.clone());
            state.metrics = crate::modules::compute::metrics::compute(
                &state.line_chart_state.candles,
                state.live_price,
            );
        }
        Message::Tick => {
            crate::modules::api::mempool::rest::halve_blocks::fetch_and_store();
            state.current_tip_height = Halvora::load_tip_height();
            state.current_subsidy_sat = Halvora::load_current_subsidy();
            // Keep the metric bar subsidy current for the active page.
            if let Some(n) = state.selected_halving {
                state.subsidy_label =
                    crate::modules::compute::halving_period::halving_subsidy_btc(n);
            } else {
                state.subsidy_label =
                    crate::modules::compute::halving_period::subsidy_btc_from_sat(
                        state.current_subsidy_sat,
                    );
            }
            state.next_halving_eta = crate::modules::compute::halving_eta::next_halving_eta(state.current_tip_height);
            state.blocks_to_next_halving = crate::modules::compute::halving_eta::blocks_to_next_halving(state.current_tip_height);
            // Keep the selected halving's ETA and subsidy current. The tip
            // advances and the live price may change.
            if let Some(n) = state.selected_halving {
                state.selected_halving_eta =
                    Some(crate::modules::compute::halving_eta::halving_eta(
                        state.current_tip_height,
                        n,
                    ));
                state.selected_halving_subsidy =
                    Some(crate::modules::compute::halving_period::halving_subsidy_btc(n));
            }
            state.coins_issued = crate::modules::compute::coins_issued::coins_issued(state.current_tip_height);
            state.percentage_issued = crate::modules::compute::coins_issued::percentage_issued(state.current_tip_height);
            state.remaining_issuance = crate::modules::compute::coins_issued::remaining_issuance(state.current_tip_height);

            // Hourly volume sync for today's partial candle (with 1h cooldown at startup).
            if state.volume_sync_start.elapsed() >= Duration::from_secs(3600) {
                crate::modules::api::bit_stamp::candle_sync::update_today_volume();
                let candles = crate::modules::compute::year_over_year::trailing_365_candles();
                state.yoy_candles = candles;
                state.volume_sync_start = Instant::now();
                // Only refresh the active page when YOY is selected; halving
                // pages keep their empty candle set and dash metrics.
                if state.yoy_selected {
                    state.line_chart_state.set_candles(state.yoy_candles.clone());
                    state.metrics = crate::modules::compute::metrics::compute(
                        &state.line_chart_state.candles,
                        Halvora::metric_current_price(state),
                    );
                }
            }
        }
        Message::LivePrice(price) => {
            state.live_price = Some(price);
            state.subsidy_value = crate::modules::compute::price_stats::subsidy_value(Some(price), state.current_subsidy_sat);
            state.sats_per_usd = crate::modules::compute::price_stats::sats_per_usd(Some(price));
            state.all_time_high = crate::modules::compute::price_stats::all_time_high(Some(price));

            // Update today's candle close/high/low in the cached YOY data.
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let today_midnight = now - (now % 86_400);
            if let Some(last) = state.yoy_candles.last_mut() {
                if last.timestamp == today_midnight {
                    if price > last.high { last.high = price; }
                    if price < last.low  { last.low = price; }
                    last.close = price;
                }
            }

            // Update the active page's chart state. This covers both YOY and
            // the live halving period (which ends at today). Completed halving
            // periods have no today candle, so the update is a no-op for them.
            if state.yoy_selected {
                state.line_chart_state.set_candles(state.yoy_candles.clone());
            }
            if let Some(last) = state.line_chart_state.candles.last_mut() {
                if last.timestamp == today_midnight {
                    if price > last.high { last.high = price; }
                    if price < last.low  { last.low = price; }
                    last.close = price;
                }
            }
            state.metrics = crate::modules::compute::metrics::compute(
                &state.line_chart_state.candles,
                Halvora::metric_current_price(state),
            );
            // Refresh the selected halving's subsidy value.
            if let Some(n) = state.selected_halving {
                state.selected_halving_subsidy =
                    Some(crate::modules::compute::halving_period::halving_subsidy_btc(n));
            }
        }
        Message::CalmarClicked => {
            state.show_calmar_dialog = true;
            state.line_chart_state.dialog_open.set(true);
        }
        Message::CloseCalmarDialog => {
            state.show_calmar_dialog = false;
            state.line_chart_state.dialog_open.set(false);
        }
        Message::SelectAVWAP => {
            state.line_chart_state.drawing_mode
                .set(crate::modules::ui::line_chart::state::DrawingMode::AVWAP);
        }
        Message::SelectRange => {
            state.line_chart_state.drawing_mode
                .set(crate::modules::ui::line_chart::state::DrawingMode::Range);
        }
        Message::NewDay(_ts) => {
            // Midnight rollover — fetch the new day's candle and refresh the cache.
            crate::modules::api::bit_stamp::candle_sync::fetch_and_store();
            let candles = crate::modules::compute::year_over_year::trailing_365_candles();
            state.yoy_candles = candles;
            state.volume_sync_start = Instant::now();
            // Only refresh the active page when YOY is selected; halving
            // pages keep their empty candle set and dash metrics.
            if state.yoy_selected {
                state.line_chart_state.set_candles(state.yoy_candles.clone());
                state.metrics = crate::modules::compute::metrics::compute(
                    &state.line_chart_state.candles,
                    state.live_price,
                );
            }
        }
    }
}

fn view(state: &Halvora) -> Element<'_, Message> {
    let main_content: Element<'_, Message> = row![
        halving_sidebar::view(state.selected_halving, state.yoy_selected),
        dashboard::view(
            state.selected_halving,
            state.yoy_selected,
            state.selected_halving_eta.as_deref(),
            state.selected_halving_subsidy.as_deref(),
            &state.subsidy_label,
            &state.line_chart_state,
            &state.metrics,
        ),
        blockchain_sidebar::view(state.current_tip_height, state.current_subsidy_sat, &state.next_halving_eta, &state.blocks_to_next_halving, &state.coins_issued, &state.percentage_issued, &state.remaining_issuance, state.live_price, &state.subsidy_value, &state.sats_per_usd, &state.all_time_high),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    if state.show_calmar_dialog {
        // Semi-transparent overlay
        let overlay = container(
            container(
                iced::widget::column![
                    text("Calmar Ratio Details")
                        .size(20)
                        .color(iced::Color::WHITE)
                        .font(iced::Font::with_name("Geist Mono")),
                    text("─")
                        .size(12)
                        .color(iced::Color::from_rgb(0.4, 0.4, 0.4))
                        .font(iced::Font::with_name("Geist Mono")),
                    text("Formula: Annualized Return ÷ Max Drawdown")
                        .size(13)
                        .color(iced::Color::from_rgb(0.8, 0.8, 0.8))
                        .font(iced::Font::with_name("Geist Mono")),
                    text("• Daily P/L%: (Close − Open) / Open")
                        .size(12)
                        .color(iced::Color::from_rgb(0.7, 0.7, 0.7))
                        .font(iced::Font::with_name("Geist Mono")),
                    text("• Weighted Avg: Σ(P/L% × Vol) / Σ(Vol)")
                        .size(12)
                        .color(iced::Color::from_rgb(0.7, 0.7, 0.7))
                        .font(iced::Font::with_name("Geist Mono")),
                    text("• Annualized: Weighted Avg × 365")
                        .size(12)
                        .color(iced::Color::from_rgb(0.7, 0.7, 0.7))
                        .font(iced::Font::with_name("Geist Mono")),
                    text("• Ratio: Annualized / Max DD")
                        .size(12)
                        .color(iced::Color::from_rgb(0.7, 0.7, 0.7))
                        .font(iced::Font::with_name("Geist Mono")),
                    text("─")
                        .size(12)
                        .color(iced::Color::from_rgb(0.4, 0.4, 0.4))
                        .font(iced::Font::with_name("Geist Mono")),
                    text(format!("Weighted Avg P/L:  {}", &state.metrics.calmar_breakdown.weighted_avg_pl))
                        .size(14)
                        .color(iced::Color::from_rgb(0.7, 0.7, 0.7))
                        .font(iced::Font::with_name("Geist Mono")),
                    text(format!("Annualized Return:  {}", &state.metrics.calmar_breakdown.annualized_return))
                        .size(14)
                        .color(iced::Color::from_rgb(0.7, 0.7, 0.7))
                        .font(iced::Font::with_name("Geist Mono")),
                    text(format!("Max Drawdown:  {}", &state.metrics.calmar_breakdown.max_drawdown))
                        .size(14)
                        .color(iced::Color::from_rgb(0.7, 0.7, 0.7))
                        .font(iced::Font::with_name("Geist Mono")),
                    text(format!("Calmar Ratio:  {}", &state.metrics.calmar_breakdown.ratio))
                        .size(14)
                        .color(iced::Color::from_rgb(0.7, 0.7, 0.7))
                        .font(iced::Font::with_name("Geist Mono")),
                    iced::widget::button(
                        text("Close")
                            .size(14)
                            .color(iced::Color::WHITE)
                    )
                    .on_press(Message::CloseCalmarDialog)
                    .padding(iced::Padding::new(8.0).horizontal(16.0))
                    .style(|_theme, _status| iced::widget::button::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(0.3, 0.3, 0.3))),
                        border: iced::border::rounded(6),
                        shadow: Default::default(),
                        text_color: Default::default(),
                        snap: false,
                    }),
                ]
                .spacing(12)
                .align_x(iced::Alignment::Center)
                .padding(32),
            )
            .width(Length::Fixed(400.0))
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.15, 0.15, 0.15))),
                border: iced::border::rounded(12)
                    .color(iced::Color::from_rgb(0.3, 0.3, 0.3))
                    .width(1.5),
                ..Default::default()
            }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
            ..Default::default()
        })
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center);

        iced::widget::stack(vec![main_content, mouse_area(overlay).into()]).into()
    } else {
        main_content
    }
}
