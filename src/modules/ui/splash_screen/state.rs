use std::time::Instant;

/// State for the fixed-duration splash screen.
///
/// Progress advances from 0.0 to 1.0 over a fixed total duration.
pub struct SplashState {
    /// Progress from 0.0 (start) to 1.0 (done).
    progress: f32,
    /// The fixed total splash duration in seconds.
    duration_secs: f32,
    /// The instant the splash started, used to compute elapsed time.
    start_time: Option<Instant>,
    /// Whether the true window size (and thus the scale factor) is known.
    /// Until true, the splash must not render scale-dependent content.
    ready: bool,
}

impl SplashState {
    /// Fade-in duration in seconds; the splash ramps from transparent to
    /// fully visible over this window at the start.
    pub const FADE_IN_SECS: f32 = 0.5;
    /// The splash duration in seconds; the splash stays fully visible for
    /// this long before the fade-out begins.
    pub const DURATION_SECS: f32 = 5.0;
    /// Fade-out duration in seconds.
    pub const FADE_OUT_SECS: f32 = 0.8;

    /// The total time from start until the splash is done, including the
    /// fade-in and fade-out.
    pub const TOTAL_SECS: f32 = Self::FADE_IN_SECS + Self::DURATION_SECS + Self::FADE_OUT_SECS;

    /// Creates a new splash state with the given total duration.
    pub fn new(duration_secs: f32) -> Self {
        Self {
            progress: 0.0,
            duration_secs,
            start_time: None,
            ready: false,
        }
    }

    /// Marks the splash ready, i.e. the true window size is known and the
    /// scale factor is resolved. Called once on the first resize event.
    pub fn mark_ready(&mut self) {
        self.ready = true;
    }

    /// Returns whether the splash is ready to render scale-dependent content.
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Records the splash start instant.
    pub fn mark_started(&mut self, start: Instant) {
        self.start_time = Some(start);
    }

    /// Advances progress based on the elapsed seconds since start.
    ///
    /// Progress is clamped to the range 0.0..=1.0.
    pub fn advance(&mut self, elapsed_secs: f32) {
        self.progress = (elapsed_secs / self.duration_secs).clamp(0.0, 1.0);
    }

    /// Returns true once the splash has fully elapsed, including the fade-out.
    pub fn is_finished(&self) -> bool {
        self.elapsed_secs() >= Self::TOTAL_SECS
    }

    /// The current progress in the range 0.0..=1.0.
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// The start instant of the splash, if it has started.
    pub fn start_time(&self) -> Option<Instant> {
        self.start_time
    }

    /// The number of seconds elapsed since the splash started.
    ///
    /// Returns `0.0` when the splash has not started yet.
    pub fn elapsed_secs(&self) -> f32 {
        self.start_time
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0)
    }

    /// The current opacity in the range 0.0..=1.0.
    ///
    /// Opacity ramps from 0 to 1 over the fade-in window, holds at 1.0 for
    /// the full display duration, then ramps to 0 over the fade-out window.
    pub fn opacity(&self) -> f32 {
        let elapsed = self.elapsed_secs();
        if elapsed <= Self::FADE_IN_SECS {
            // Fade-in window at the start.
            (elapsed / Self::FADE_IN_SECS).clamp(0.0, 1.0)
        } else if elapsed <= Self::FADE_IN_SECS + self.duration_secs {
            // Fully visible during the display duration.
            1.0
        } else {
            // Fade-out window after the display duration ends.
            ((Self::FADE_IN_SECS + self.duration_secs + Self::FADE_OUT_SECS - elapsed)
                / Self::FADE_OUT_SECS)
                .clamp(0.0, 1.0)
        }
    }
}

/// State for the main dashboard fade-in transition.
///
/// A full-screen dark overlay fades from opaque to transparent over a short
/// duration, so the dashboard appears smoothly after the splash.
pub struct MainFadeInState {
    /// The fade-in duration in seconds.
    duration_secs: f32,
    /// The instant the fade-in started.
    start_time: Option<Instant>,
}

impl MainFadeInState {
    /// The main dashboard fade-in duration in seconds.
    pub const FADE_IN_SECS: f32 = 1.0;

    /// Creates a new main fade-in state.
    pub fn new(duration_secs: f32) -> Self {
        Self {
            duration_secs,
            start_time: None,
        }
    }

    /// Records the fade-in start instant.
    pub fn mark_started(&mut self, start: Instant) {
        self.start_time = Some(start);
    }

    /// The start instant of the fade-in, if it has started.
    pub fn start_time(&self) -> Option<Instant> {
        self.start_time
    }

    /// The number of seconds elapsed since the fade-in started.
    ///
    /// Returns `0.0` when the fade-in has not started yet.
    pub fn elapsed_secs(&self) -> f32 {
        self.start_time
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0)
    }

    /// The current overlay opacity in the range 0.0..=1.0.
    ///
    /// Opacity ramps from 1.0 to 0.0 over the fade-in duration, revealing the
    /// dashboard underneath.
    pub fn opacity(&self) -> f32 {
        (1.0 - self.elapsed_secs() / self.duration_secs).clamp(0.0, 1.0)
    }

    /// Returns true once the fade-in has fully completed.
    pub fn is_finished(&self) -> bool {
        self.elapsed_secs() >= self.duration_secs
    }
}
