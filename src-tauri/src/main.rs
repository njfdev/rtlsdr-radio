// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod nrsc5;

use std::thread;
use nrsc5::nrsc5::Nrsc5State;
use tauri::{api::process::{Command, CommandEvent}, Manager, State, Window};

fn main() {
  tauri::Builder::default()
    .manage(Nrsc5State::new())
    .invoke_handler(tauri::generate_handler![start_nrsc5, stop_nrsc5])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn start_nrsc5(window: Window, state: State<Nrsc5State>, fm_freq: String, channel: String) {
  state.start_thread(window, fm_freq, channel);
}

#[tauri::command]
fn stop_nrsc5(window: Window, state: State<Nrsc5State>) {
  state.stop_thread(window);
}