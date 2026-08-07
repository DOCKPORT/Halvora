mod modules;

fn main() -> iced::Result {
    // Resolve the scale factor before any other startup work so the UI
    // renders correctly from the very first frame.
    modules::ui::scaling::Scaling::init();
    modules::app_data_dir::ensure();
    modules::api::mempool::rest::halve_blocks::fetch_and_store();
    modules::api::bit_stamp::candle_sync::fetch_and_store();
    modules::ui::mainwindow::application::run()
}
