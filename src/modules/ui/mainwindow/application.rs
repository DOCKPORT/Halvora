use crate::modules::compute::metrics::{Metrics, PLSign};
use crate::modules::compute::year_over_year::Candle;
use crate::modules::ui::line_chart::LineChartState;
use crate::modules::ui::line_chart::state::PageDrawings;
use crate::modules::ui::mainwindow::about_dialog;
use crate::modules::ui::mainwindow::app_icon;
use crate::modules::ui::mainwindow::calmar_dialog;
use crate::modules::ui::mainwindow::dashboard_layout::dashboard;
use crate::modules::ui::mainwindow::db_accessor;
use crate::modules::ui::mainwindow::sidebar::blockchain_sidebar;
use crate::modules::ui::mainwindow::sidebar::halving_sidebar;
use crate::modules::ui::scaling::Scaling;
use crate::modules::ui::splash_screen::splash;
use crate::modules::ui::splash_screen::state::{MainFadeInState, SplashState};
use iced::widget::{container, mouse_area, row};
use iced::window::Position;
use iced::{Element, Font, Length, Subscription, window};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Embed the `GeistMono` font as fallback — the system-installed `SemiBold`
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
        #[cfg(target_os = "linux")]
        maximized: true,
        #[cfg(not(target_os = "linux"))]
        maximized: true,
        icon: app_icon::load_app_icon(),
        ..Default::default()
    };

    iced::application(Halvora::new, update, view)
        .font(GEIST_MONO_BYTES)
        .default_font(Font::with_name("Geist Mono"))
        .window(window_settings)
        .subscription(subscription)
        .run()
}

/// The application's top-level phase.
enum AppPhase {
    /// Shows the splash screen until its fixed duration elapses.
    Splash(SplashState),
    /// Fades the dashboard in after the splash, just before showing it fully.
    MainFadeIn(MainFadeInState),
    /// The main dashboard.
    Main,
}

/// A stable key identifying which page the chart is showing. `None` is the
/// Year-Over-Year page; `Some(n)` is halving number `n`. Used so each page
/// keeps its own set of drawings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PageKey {
    Yoy,
    Halving(u32),
}

struct Halvora {
    phase: AppPhase,
    selected_halving: Option<u32>,
    yoy_selected: bool,
    /// One drawings bundle per page, so a VWAP or range stays on the page
    /// where it was drawn and never leaks onto another page.
    page_drawings: HashMap<PageKey, PageDrawings>,
    /// The page key of the chart currently being shown, used to save that
    /// page's drawings before switching to another.
    active_page: PageKey,
    current_tip_height: u32,
    current_subsidy_sat: i64,
    mining_difficulty: f64,
    next_halving_eta: String,
    blocks_to_next_halving: String,
    coins_issued: String,
    percentage_issued: String,
    remaining_issuance: String,
    live_price: Option<f64>,
    /// Active websocket price flash (green up / red down), `None` when idle.
    ws_flash: Option<crate::modules::ui::ws_flash::WsFlash>,
    subsidy_value: String,
    sats_per_usd: String,
    all_time_high: String,
    /// Highest live price seen this session, so a new all-time high stays
    /// visible even after the price pulls back below the DB record.
    session_high: f64,
    metrics: Metrics,
    /// P/L sign for the Year-Over-Year period, used to color the sidebar button.
    yoy_pl_sign: PLSign,
    /// P/L sign for each halving period, indexed by halving number (`signs[n]`
    /// holds halving `n`, element 0 unused). Used to color the sidebar buttons.
    halving_pl_signs: Vec<PLSign>,
    /// The currently live halving number (the most recent one with data).
    /// Future halvings stay grey until this advances.
    current_halving: u32,
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
    show_about_dialog: bool,
    /// A window resize awaiting debounced application. While set, a short
    /// poll runs; when the resize events settle, the scale factor is applied.
    pending_resize: Option<(iced::Size, Instant)>,
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
            .map_or(0, |d| d.as_secs() as i64);
        let today_midnight = now - (now % 86_400);
        if last_ts == today_midnight {
            // Live period: use the websocket price as today's running close.
            state.live_price
        } else {
            // Completed period: use its final candle close.
            state.line_chart_state.candles.last().map(|c| c.close)
        }
    }

    /// Current price for an arbitrary candle set, mirroring `metric_current_price`.
    ///
    /// A completed period ends in the past, so its last candle close is the
    /// true end. A live period ends today, so the websocket price is the
    /// running close.
    fn period_current_price(candles: &[Candle], live_price: Option<f64>) -> Option<f64> {
        let last_ts = candles.last().map(|c| c.timestamp)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64);
        let today_midnight = now - (now % 86_400);
        if last_ts == today_midnight {
            live_price
        } else {
            candles.last().map(|c| c.close)
        }
    }

    /// Highest halving number with candle data, i.e. the currently live halving.
    ///
    /// Returns `0` when no halving has data yet.
    fn live_halving_number() -> u32 {
        let mut live = 0;
        for n in 1..=32 {
            if !crate::modules::compute::halving_period::halving_period_candles(n).is_empty() {
                live = n;
            }
        }
        live
    }

    /// Recompute the YOY and all 32 halving signs, and re-detect the live
    /// halving. Called on startup, Tick, and `NewDay` (infrequent paths).
    fn refresh_halving_signs(state: &mut Self) {
        state.yoy_pl_sign =
            crate::modules::compute::metrics::pl_sign(&state.yoy_candles, state.live_price);
        state.halving_pl_signs = (0..=32)
            .map(|n| {
                if n == 0 {
                    PLSign::NoChange
                } else {
                    let candles =
                        crate::modules::compute::halving_period::halving_period_candles(n);
                    let price = Self::period_current_price(&candles, state.live_price);
                    crate::modules::compute::metrics::pl_sign(&candles, price)
                }
            })
            .collect();
        state.current_halving = Self::live_halving_number();
    }

    /// Recompute only the YOY and live halving signs. Called on every
    /// websocket price tick to avoid 33 DB queries per update; completed
    /// halving signs are already stable.
    fn refresh_live_signs(state: &mut Self) {
        state.yoy_pl_sign =
            crate::modules::compute::metrics::pl_sign(&state.yoy_candles, state.live_price);
        if state.current_halving >= 1 && state.current_halving <= 32 {
            let n = state.current_halving as usize;
            let candles = crate::modules::compute::halving_period::halving_period_candles(n as u32);
            let price = Self::period_current_price(&candles, state.live_price);
            state.halving_pl_signs[n] = crate::modules::compute::metrics::pl_sign(&candles, price);
        }
    }

    /// Save the current page's drawings into the per-page collection, then
    /// load the target page's drawings into the active chart. `target` is the
    /// key of the page being switched to.
    fn switch_page(state: &mut Self, target: PageKey) {
        // If already on this page, do nothing.
        if state.active_page == target {
            return;
        }
        // Stash the old page's drawings.
        let snapshot = state.line_chart_state.snapshot_drawings();
        state.page_drawings.insert(state.active_page, snapshot);
        // Load the target page's drawings (empty if first visit).
        let drawings = state.page_drawings.entry(target).or_default().clone();
        state.line_chart_state.restore_drawings(drawings);
        state.active_page = target;
    }

    fn new() -> Self {
        let current_tip_height = db_accessor::load_tip_height();
        let current_subsidy_sat = db_accessor::load_current_subsidy();
        let mining_difficulty = db_accessor::load_mining_difficulty();
        let next_halving_eta =
            crate::modules::compute::halving_eta::next_halving_eta(current_tip_height);
        let blocks_to_next_halving =
            crate::modules::compute::halving_eta::blocks_to_next_halving(current_tip_height);
        let coins_issued = crate::modules::compute::coins_issued::coins_issued(current_tip_height);
        let percentage_issued =
            crate::modules::compute::coins_issued::percentage_issued(current_tip_height);
        let remaining_issuance =
            crate::modules::compute::coins_issued::remaining_issuance(current_tip_height);

        let candles = crate::modules::compute::year_over_year::trailing_365_candles();

        // Seed the live price with the latest known close from the database,
        // so the dashboard shows a value immediately. The websocket takes over
        // once the first trade price arrives.
        let seeded_price = candles.last().map(|c| c.close);

        // Session high starts at the greater of the historical DB record and
        // the seeded price. It only rises from here, so a new all-time high
        // set mid-session stays shown even after the price pulls back.
        let session_high = crate::modules::compute::price_stats::db_high()
            .into_iter()
            .chain(seeded_price)
            .fold(0.0_f64, f64::max);

        let all_time_high = crate::modules::compute::price_stats::fmt_high(session_high);
        // Compute the startup metrics and sidebar values from the seeded price
        // so the dashboard is fully populated when it first appears.
        let metrics = crate::modules::compute::metrics::compute(&candles, seeded_price);
        let subsidy_value =
            crate::modules::compute::price_stats::subsidy_value(seeded_price, current_subsidy_sat);
        let sats_per_usd = crate::modules::compute::price_stats::sats_per_usd(seeded_price);

        let mut state = Self {
            phase: AppPhase::Splash(SplashState::new(SplashState::DURATION_SECS)),
            selected_halving: None,
            yoy_selected: true,
            page_drawings: HashMap::new(),
            active_page: PageKey::Yoy,
            current_tip_height,
            current_subsidy_sat,
            mining_difficulty,
            next_halving_eta,
            blocks_to_next_halving,
            coins_issued,
            percentage_issued,
            remaining_issuance,
            live_price: seeded_price,
            ws_flash: None,
            subsidy_value,
            sats_per_usd,
            all_time_high,
            session_high,
            metrics,
            yoy_pl_sign: PLSign::NoChange,
            halving_pl_signs: vec![PLSign::NoChange; 33],
            current_halving: 0,
            selected_halving_eta: None,
            selected_halving_subsidy: None,
            subsidy_label: crate::modules::compute::halving_period::subsidy_btc_from_sat(
                current_subsidy_sat,
            ),
            yoy_candles: candles.clone(),
            line_chart_state: LineChartState::new(candles),
            volume_sync_start: Instant::now(),
            show_calmar_dialog: false,
            show_about_dialog: false,
            pending_resize: None,
        };
        Self::refresh_halving_signs(&mut state);
        state
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    HalvingSelected(u32),
    YoYSelected,
    Tick,
    LivePrice(f64),
    /// Periodic tick that expires an active websocket flash after 1 second.
    WsFlashTick,
    NewDay(i64),
    CalmarClicked,
    CloseCalmarDialog,
    AboutClicked,
    OpenGithub,
    CloseAboutDialog,
    SelectAVWAP,
    SelectRange,
    /// The window was resized; records the target size for debounced
    /// application (kept cheap so dragging stays smooth).
    WindowResized(iced::Size),
    /// Fires while a resize is pending; applies the scale factor once the
    /// resize events settle.
    ResizePoll,
    /// Advances splash progress by the elapsed time since start.
    SplashTick,
}

/// How often the resize poll ticks while a resize is pending.
const RESIZE_POLL_INTERVAL_MS: u64 = 30;

/// How long after the last resize event before the scale factor is applied.
const RESIZE_SETTLE_MS: std::time::Duration = std::time::Duration::from_millis(120);

fn subscription(state: &Halvora) -> Subscription<Message> {
    let mut subs = vec![
        iced::time::every(std::time::Duration::from_mins(10)).map(|_| Message::Tick),
        crate::modules::api::bit_stamp::ws::live_price().map(Message::LivePrice),
        crate::modules::compute::midnight_rollover::detect().map(Message::NewDay),
        iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size)),
    ];

    // While a websocket price flash is active, tick frequently so it can expire
    // after ~1 second and the Spot Price value returns to its normal color.
    if state.ws_flash.is_some() {
        subs.push(iced::time::every(Duration::from_millis(100)).map(|_| Message::WsFlashTick));
    }

    // While a window resize is pending, poll so the scale factor can be applied
    // once the resize events settle. The poll stays off while idle.
    if state.pending_resize.is_some() {
        subs.push(
            iced::time::every(Duration::from_millis(RESIZE_POLL_INTERVAL_MS))
                .map(|_| Message::ResizePoll),
        );
    }

    // While the splash or the main fade-in is active, drive progress on a
    // fast fixed interval (~125fps) to keep the fade-in, fade-out, and
    // progress bar animations smooth.
    if matches!(state.phase, AppPhase::Splash(_) | AppPhase::MainFadeIn(_)) {
        subs.push(iced::time::every(Duration::from_millis(8)).map(|_| Message::SplashTick));
    }

    Subscription::batch(subs)
}

fn update(state: &mut Halvora, message: Message) {
    // Handle the splash / fade-in tick on every frame.
    if matches!(message, Message::SplashTick) {
        // Splash phase: wait until the scale is resolved (first real window
        // size), then advance progress and move to the dashboard fade-in.
        if let AppPhase::Splash(s) = &mut state.phase {
            // Do not start the timer until the true window size is known, so
            // the splash never renders (nor advances) at a wrong scale.
            if !s.is_ready() {
                return;
            }
            // Set the start time on the first tick, then measure progress
            // relative to it.
            let elapsed = if let Some(t) = s.start_time() {
                t.elapsed().as_secs_f32()
            } else {
                s.mark_started(Instant::now());
                0.0
            };
            s.advance(elapsed);
            if s.is_finished() {
                state.phase =
                    AppPhase::MainFadeIn(MainFadeInState::new(MainFadeInState::FADE_IN_SECS));
            }
            return;
        }

        // Main fade-in phase: advance the overlay opacity, then show fully.
        if let AppPhase::MainFadeIn(f) = &mut state.phase {
            // Set the start time on the first tick; opacity() reads elapsed
            // time internally.
            if f.start_time().is_none() {
                f.mark_started(Instant::now());
            }
            if f.is_finished() {
                state.phase = AppPhase::Main;
            }
            return;
        }
    }

    match message {
        Message::HalvingSelected(n) => {
            // Save the current page's drawings and load this halving's own set.
            Halvora::switch_page(state, PageKey::Halving(n));
            state.selected_halving = Some(n);
            state.yoy_selected = false;
            // Show the block range in the top-left only for started halvings
            // (past or current), determined from the live halving data.
            state
                .line_chart_state
                .block_range
                .set(if n <= state.current_halving {
                    crate::modules::compute::halving_period::halving_block_range(n)
                } else {
                    None
                });
            // Record the ETA and subsidy for a future halving page.
            state.selected_halving_eta = Some(crate::modules::compute::halving_eta::halving_eta(
                state.current_tip_height,
                n,
            ));
            state.selected_halving_subsidy =
                Some(crate::modules::compute::halving_period::halving_subsidy_btc(n));
            // The metric bar shows this halving's subsidy.
            state.subsidy_label = crate::modules::compute::halving_period::halving_subsidy_btc(n);
            // Load this halving period's candles. Future halvings return an
            // empty set, so `metrics::compute` naturally produces dashes.
            // Use set_candles so the drawing tool and drawings are preserved.
            let mut candles = crate::modules::compute::halving_period::halving_period_candles(n);
            // Use the in-memory websocket price for today's partial candle so
            // the chart reflects the live price immediately instead of the
            // last DB-synced close (which only updates on the next tick).
            if let Some(price) = state.live_price {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs() as i64);
                let today_midnight = now - (now % 86_400);
                if let Some(last) = candles.last_mut()
                    && last.timestamp == today_midnight
                {
                    last.close = price;
                    if price > last.high {
                        last.high = price;
                    }
                    if price < last.low {
                        last.low = price;
                    }
                }
            }
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
            // Save the current page's drawings and load the YOY page's own set.
            Halvora::switch_page(state, PageKey::Yoy);
            state.yoy_selected = true;
            state.selected_halving = None;
            // YOY is not a halving period, so never show a block range.
            state.line_chart_state.block_range.set(None);
            state.selected_halving_eta = None;
            state.selected_halving_subsidy = None;
            // Metric bar shows the current tip subsidy on YOY.
            state.subsidy_label = crate::modules::compute::halving_period::subsidy_btc_from_sat(
                state.current_subsidy_sat,
            );
            // Restore the cached YOY candles and recompute metrics for them.
            // Use set_candles so drawings are preserved across page switches.
            state
                .line_chart_state
                .set_candles(state.yoy_candles.clone());
            state.metrics = crate::modules::compute::metrics::compute(
                &state.line_chart_state.candles,
                state.live_price,
            );
        }
        Message::Tick => {
            crate::modules::api::mempool::rest::halve_blocks::fetch_and_store();
            state.current_tip_height = db_accessor::load_tip_height();
            state.current_subsidy_sat = db_accessor::load_current_subsidy();
            state.mining_difficulty = db_accessor::load_mining_difficulty();
            // Keep the metric bar subsidy current for the active page.
            if let Some(n) = state.selected_halving {
                state.subsidy_label =
                    crate::modules::compute::halving_period::halving_subsidy_btc(n);
            } else {
                state.subsidy_label = crate::modules::compute::halving_period::subsidy_btc_from_sat(
                    state.current_subsidy_sat,
                );
            }
            state.next_halving_eta =
                crate::modules::compute::halving_eta::next_halving_eta(state.current_tip_height);
            state.blocks_to_next_halving =
                crate::modules::compute::halving_eta::blocks_to_next_halving(
                    state.current_tip_height,
                );
            // Keep the selected halving's ETA and subsidy current. The tip
            // advances and the live price may change.
            if let Some(n) = state.selected_halving {
                state.selected_halving_eta = Some(
                    crate::modules::compute::halving_eta::halving_eta(state.current_tip_height, n),
                );
                state.selected_halving_subsidy =
                    Some(crate::modules::compute::halving_period::halving_subsidy_btc(n));
            }
            state.coins_issued =
                crate::modules::compute::coins_issued::coins_issued(state.current_tip_height);
            state.percentage_issued =
                crate::modules::compute::coins_issued::percentage_issued(state.current_tip_height);
            state.remaining_issuance =
                crate::modules::compute::coins_issued::remaining_issuance(state.current_tip_height);
            // The tip may have advanced into a new live halving, so refresh
            // all signs and re-detect the live halving.
            Halvora::refresh_halving_signs(state);

            // Hourly volume sync for today's partial candle (with 1h cooldown at startup).
            if state.volume_sync_start.elapsed() >= Duration::from_hours(1) {
                crate::modules::api::bit_stamp::candle_sync::update_today_volume();
                let candles = crate::modules::compute::year_over_year::trailing_365_candles();
                state.yoy_candles = candles;
                state.volume_sync_start = Instant::now();
                // Only refresh the active page when YOY is selected; halving
                // pages keep their empty candle set and dash metrics.
                if state.yoy_selected {
                    state
                        .line_chart_state
                        .set_candles(state.yoy_candles.clone());
                    state.metrics = crate::modules::compute::metrics::compute(
                        &state.line_chart_state.candles,
                        Halvora::metric_current_price(state),
                    );
                }
            }
        }
        Message::WsFlashTick => {
            // Expire the websocket flash once its 1-second window has elapsed.
            if let Some(flash) = &state.ws_flash
                && !flash.is_active(std::time::Instant::now())
            {
                state.ws_flash = None;
                state.line_chart_state.ws_flash.set(None);
            }
        }
        Message::LivePrice(price) => {
            let flash = crate::modules::ui::ws_flash::WsFlash::from_tick(state.live_price, price);
            state.ws_flash = flash;
            state.line_chart_state.ws_flash.set(flash);
            state.live_price = Some(price);
            state.subsidy_value = crate::modules::compute::price_stats::subsidy_value(
                Some(price),
                state.current_subsidy_sat,
            );
            state.sats_per_usd = crate::modules::compute::price_stats::sats_per_usd(Some(price));
            // Raise the session high if the live price sets a new record; it
            // never falls back, so the all-time high stays shown even if the
            // price pulls back below the DB record.
            state.session_high = state.session_high.max(price);
            state.all_time_high =
                crate::modules::compute::price_stats::fmt_high(state.session_high);

            // Update today's candle close/high/low in the cached YOY data.
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs() as i64);
            let today_midnight = now - (now % 86_400);
            if let Some(last) = state.yoy_candles.last_mut()
                && last.timestamp == today_midnight
            {
                if price > last.high {
                    last.high = price;
                }
                if price < last.low {
                    last.low = price;
                }
                last.close = price;
            }

            // Update the active page's chart state. This covers both YOY and
            // the live halving period (which ends at today). Completed halving
            // periods have no today candle, so the update is a no-op for them.
            if state.yoy_selected {
                state
                    .line_chart_state
                    .set_candles(state.yoy_candles.clone());
            }
            if let Some(last) = state.line_chart_state.candles.last_mut()
                && last.timestamp == today_midnight
            {
                if price > last.high {
                    last.high = price;
                }
                if price < last.low {
                    last.low = price;
                }
                last.close = price;
            }
            state.metrics = crate::modules::compute::metrics::compute(
                &state.line_chart_state.candles,
                Halvora::metric_current_price(state),
            );
            // Update the YOY and live halving button colors with the new price.
            Halvora::refresh_live_signs(state);
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
        Message::AboutClicked => {
            state.show_about_dialog = true;
            state.line_chart_state.dialog_open.set(true);
        }
        Message::OpenGithub => {
            open_url("https://github.com/DOCKPORT/Halvora");
        }
        Message::CloseAboutDialog => {
            state.show_about_dialog = false;
            state.line_chart_state.dialog_open.set(false);
        }
        Message::CloseCalmarDialog => {
            state.show_calmar_dialog = false;
            state.line_chart_state.dialog_open.set(false);
        }
        Message::SelectAVWAP => {
            state
                .line_chart_state
                .drawing_mode
                .set(crate::modules::ui::line_chart::state::DrawingMode::AVWAP);
        }
        Message::SelectRange => {
            state
                .line_chart_state
                .drawing_mode
                .set(crate::modules::ui::line_chart::state::DrawingMode::Range);
        }
        Message::WindowResized(size) => {
            // During the splash, this is the first opportunity to learn the
            // true window size. Apply the scale factor immediately (no
            // debounce) and mark the splash ready, so the splash first
            // renders at the correct scale — no startup jump.
            if let AppPhase::Splash(s) = &mut state.phase {
                if !s.is_ready() {
                    crate::modules::ui::scaling::Scaling::global()
                        .set_window_size(size.width, size.height);
                    s.mark_ready();
                }
                return;
            }
            // Record the latest size and when it arrived. The actual scale
            // factor is applied later by ResizePoll once the events settle,
            // so the expensive `sp` re-layout does not run every frame.
            state.pending_resize = Some((size, std::time::Instant::now()));
        }
        Message::ResizePoll => {
            // Apply the pending resize once the user pauses the drag.
            if let Some((size, last)) = state.pending_resize
                && last.elapsed() >= RESIZE_SETTLE_MS
            {
                crate::modules::ui::scaling::Scaling::global()
                    .set_window_size(size.width, size.height);
                state.pending_resize = None;
            }
        }
        Message::SplashTick => {
            // Handled above; unreachable once the phase is Main.
        }
        Message::NewDay(_ts) => {
            // Midnight rollover — fetch the new day's candle and refresh the cache.
            crate::modules::api::bit_stamp::candle_sync::fetch_and_store();
            let candles = crate::modules::compute::year_over_year::trailing_365_candles();
            state.yoy_candles = candles;
            state.volume_sync_start = Instant::now();
            // The new day may change which period is live, so refresh all signs.
            Halvora::refresh_halving_signs(state);
            // Only refresh the active page when YOY is selected; halving
            // pages keep their empty candle set and dash metrics.
            if state.yoy_selected {
                state
                    .line_chart_state
                    .set_candles(state.yoy_candles.clone());
                state.metrics = crate::modules::compute::metrics::compute(
                    &state.line_chart_state.candles,
                    state.live_price,
                );
            }
        }
    }
}

fn view(state: &Halvora) -> Element<'_, Message> {
    // Show the splash screen while the app is in its splash phase.
    if let AppPhase::Splash(splash_state) = &state.phase {
        return splash::view(splash_state);
    }

    // The overlay alpha during the main fade-in; `None` when fully shown.
    let fade_in_opacity = match &state.phase {
        AppPhase::MainFadeIn(f) => Some(f.opacity()),
        _ => None,
    };

    // The active websocket price flash, used to highlight the changed digits.
    let spot_flash = state
        .ws_flash
        .as_ref()
        .filter(|f| f.is_active(std::time::Instant::now()));

    let main_content: Element<'_, Message> = row![
        halving_sidebar::view(
            state.selected_halving,
            state.yoy_selected,
            state.yoy_pl_sign,
            &state.halving_pl_signs,
        ),
        dashboard::view(
            state.selected_halving,
            state.yoy_selected,
            state.selected_halving_eta.as_deref(),
            state.selected_halving_subsidy.as_deref(),
            &state.subsidy_label,
            &state.line_chart_state,
            &state.metrics,
        ),
        blockchain_sidebar::view(
            state.current_tip_height,
            state.current_subsidy_sat,
            state.mining_difficulty,
            &state.next_halving_eta,
            &state.blocks_to_next_halving,
            &state.coins_issued,
            &state.percentage_issued,
            &state.remaining_issuance,
            state.live_price,
            spot_flash,
            &state.subsidy_value,
            &state.sats_per_usd,
            &state.all_time_high
        ),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    let mut display = main_content;
    if state.show_calmar_dialog {
        let overlay = calmar_dialog::view(&state.metrics);
        display = iced::widget::stack(vec![display, mouse_area(overlay).into()]).into();
        if let Some(opacity) = fade_in_opacity {
            display = fade_overlay(display, opacity);
        }
    }
    if state.show_about_dialog {
        let overlay = about_dialog::view();
        display = iced::widget::stack(vec![display, mouse_area(overlay).into()]).into();
    }
    if let Some(opacity) = fade_in_opacity {
        fade_overlay(display, opacity)
    } else {
        display
    }
}

/// Layers a dark, semi-transparent overlay on top of `content` to produce
/// a smooth fade-in. `opacity` is the overlay alpha (1.0 opaque → 0.0
/// transparent).
fn fade_overlay(content: Element<'_, Message>, opacity: f32) -> Element<'_, Message> {
    let overlay = container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(
                crate::modules::ui::theme::SPLASH_BACKGROUND.scale_alpha(opacity),
            )),
            ..Default::default()
        });
    iced::widget::stack(vec![content, overlay.into()]).into()
}

/// Open `url` in the system's default browser.
///
/// Uses `xdg-open` on Linux, `open` on macOS, and `cmd /c start` on Windows.
/// The command runs on a background thread so it does not block the UI thread.
fn open_url(url: &str) {
    let url = url.to_string();
    std::thread::spawn(move || {
        #[cfg(target_os = "windows")]
        let result = std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(&url)
            .spawn();
        #[cfg(target_os = "macos")]
        let result = std::process::Command::new("open").arg(&url).spawn();
        #[cfg(target_os = "linux")]
        let result = std::process::Command::new("xdg-open").arg(&url).spawn();

        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        let _ = result.map(|_| ());
    });
}
