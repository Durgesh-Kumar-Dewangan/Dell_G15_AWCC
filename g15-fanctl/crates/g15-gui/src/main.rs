mod app;
mod dbus_client;
mod history_graph;
mod theme;
mod tray;

fn main() -> gtk::glib::ExitCode {
    app::run()
}
