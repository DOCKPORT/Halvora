mod modules;

fn main() -> iced::Result {
    modules::app_data_dir::ensure();
    modules::api::mempool::rest::halve_blocks::fetch_and_store();
    modules::ui::mainwindow::application::run()
}
