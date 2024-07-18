// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod nrsc5;
mod rtlsdr;

use std::{os::macos::raw::stat, thread};
use nrsc5::nrsc5::Nrsc5State;
use rtlsdr::rtlsdr::RtlSdrState;
use soapysdr::Device;
use tauri::{api::process::{Command, CommandEvent}, App, Manager, State, Window};

struct AppState {
  nrsc5State: Nrsc5State,
  rtlSdrState: RtlSdrState
}

fn main() {
  tauri::Builder::default()
    .manage(AppState { nrsc5State: Nrsc5State::new(), rtlSdrState: RtlSdrState::new() })
    .invoke_handler(tauri::generate_handler![start_nrsc5, stop_nrsc5, start_fm_stream])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn start_nrsc5(window: Window, state: State<AppState>, fm_freq: String, channel: String) {
  state.nrsc5State.start_thread(window, fm_freq, channel);
}

#[tauri::command]
fn stop_nrsc5(window: Window, state: State<AppState>) {
  state.nrsc5State.stop_thread(window);
}

#[tauri::command]
fn start_fm_stream(window: Window, state: State<AppState>) {
  state.rtlSdrState.start_stream(window, "101.5".to_owned());
}