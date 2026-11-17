#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod hook;
mod layout;
mod overlay;
mod settings;
mod sound;
mod system;
mod tray;
mod util;
mod win;

fn main() {
    let code = app::run();
    std::process::exit(code);
}
