mod modules;

fn main() -> iced::Result {
    // Resolve the scale factor before any other startup work so the UI
    // renders correctly from the very first frame.
    modules::ui::scaling::Scaling::init();
    modules::app_data_dir::ensure();
    modules::desktop_entry::ensure();
    modules::api::bit_stamp::bitstamp_data::seed_if_missing();
    // The two network fetches (mempool halving blocks and Bitstamp candles)
    // run in the background once the iced app starts, so the splash screen
    // stays visible while they complete instead of the user staring at a
    // blank window.
    modules::ui::mainwindow::application::run()
}
