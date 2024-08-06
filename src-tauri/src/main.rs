#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    rtlsdr_radio_lib::run();
}
