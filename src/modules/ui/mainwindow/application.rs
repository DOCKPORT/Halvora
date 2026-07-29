use iced::widget::row;
use iced::{Element, Subscription, window, Font, Length};
use iced::window::Position;
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use crate::modules::compute::metrics::Metrics;
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
    line_chart_state: LineChartState,
    volume_sync_start: Instant,
}

impl Halvora {
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
            line_chart_state: LineChartState::new(candles),
            volume_sync_start: Instant::now(),
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
        }
        Message::YoYSelected => {
            state.yoy_selected = true;
            state.selected_halving = None;
        }
        Message::Tick => {
            crate::modules::api::mempool::rest::halve_blocks::fetch_and_store();
            state.current_tip_height = Halvora::load_tip_height();
            state.current_subsidy_sat = Halvora::load_current_subsidy();
            state.next_halving_eta = crate::modules::compute::halving_eta::next_halving_eta(state.current_tip_height);
            state.blocks_to_next_halving = crate::modules::compute::halving_eta::blocks_to_next_halving(state.current_tip_height);
            state.coins_issued = crate::modules::compute::coins_issued::coins_issued(state.current_tip_height);
            state.percentage_issued = crate::modules::compute::coins_issued::percentage_issued(state.current_tip_height);
            state.remaining_issuance = crate::modules::compute::coins_issued::remaining_issuance(state.current_tip_height);

            // Hourly volume sync for today's partial candle (with 1h cooldown at startup).
            if state.volume_sync_start.elapsed() >= Duration::from_secs(3600) {
                crate::modules::api::bit_stamp::candle_sync::update_today_volume();
                let candles = crate::modules::compute::year_over_year::trailing_365_candles();
                state.line_chart_state = LineChartState::new(candles);
                state.volume_sync_start = Instant::now();
                state.metrics = crate::modules::compute::metrics::compute(
                    &state.line_chart_state.candles,
                    state.live_price,
                );
            }
        }
        Message::LivePrice(price) => {
            state.live_price = Some(price);
            state.subsidy_value = crate::modules::compute::price_stats::subsidy_value(Some(price), state.current_subsidy_sat);
            state.sats_per_usd = crate::modules::compute::price_stats::sats_per_usd(Some(price));
            state.all_time_high = crate::modules::compute::price_stats::all_time_high(Some(price));

            // Update today's candle close/high/low in the in-memory chart state.
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let today_midnight = now - (now % 86_400);
            if let Some(last) = state.line_chart_state.candles.last_mut() {
                if last.timestamp == today_midnight {
                    if price > last.high { last.high = price; }
                    if price < last.low  { last.low = price; }
                    last.close = price;
                }
            }
            state.metrics = crate::modules::compute::metrics::compute(
                &state.line_chart_state.candles,
                Some(price),
            );
        }
        Message::NewDay(_ts) => {
            // Midnight rollover — fetch the new day's candle and refresh the chart.
            crate::modules::api::bit_stamp::candle_sync::fetch_and_store();
            let candles = crate::modules::compute::year_over_year::trailing_365_candles();
            state.line_chart_state = LineChartState::new(candles);
            state.volume_sync_start = Instant::now();
            state.metrics = crate::modules::compute::metrics::compute(
                &state.line_chart_state.candles,
                state.live_price,
            );
        }
    }
}

fn view(state: &Halvora) -> Element<'_, Message> {
    row![
        halving_sidebar::view(state.selected_halving, state.yoy_selected),
        dashboard::view(
            state.selected_halving,
            state.yoy_selected,
            &state.line_chart_state,
            &state.metrics,
        ),
        blockchain_sidebar::view(state.current_tip_height, state.current_subsidy_sat, &state.next_halving_eta, &state.blocks_to_next_halving, &state.coins_issued, &state.percentage_issued, &state.remaining_issuance, state.live_price, &state.subsidy_value, &state.sats_per_usd, &state.all_time_high),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
