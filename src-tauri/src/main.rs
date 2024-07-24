// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod custom_radiorust_blocks;
mod nrsc5;
mod rtlsdr;

use nrsc5::nrsc5::Nrsc5State;
use rtlsdr::rtlsdr::{RtlSdrState, StreamSettings};
use std::sync::{Arc, Mutex};
use tauri::{async_runtime::block_on, State, Window};

struct AppState {
    nrsc5_state: Nrsc5State,
    rtl_sdr_state: Arc<Mutex<RtlSdrState>>,
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .manage(AppState {
            nrsc5_state: Nrsc5State::new(),
            rtl_sdr_state: Arc::new(Mutex::new(RtlSdrState::new())),
        })
        .invoke_handler(tauri::generate_handler![
            start_nrsc5,
            stop_nrsc5,
            start_stream,
            stop_stream
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn start_nrsc5(window: Window, state: State<AppState>, fm_freq: String, channel: String) {
    if state.nrsc5_state.is_playing() {
        return;
    };
    state.nrsc5_state.start_thread(window, fm_freq, channel);
}

#[tauri::command]
fn stop_nrsc5(window: Window, state: State<AppState>) {
    state.nrsc5_state.stop_thread(window);
}

#[tauri::command]
fn start_stream(window: Window, state: State<AppState>, stream_settings: StreamSettings) {
    if state.rtl_sdr_state.lock().unwrap().is_playing() {
        return;
    };
    state
        .rtl_sdr_state
        .lock()
        .unwrap()
        .start_stream(window, stream_settings);
}

#[tauri::command]
async fn stop_stream(window: Window, state: State<'_, AppState>) -> Result<String, ()> {
    let rtlsdr_state_clone = state.rtl_sdr_state.clone();

    tokio::task::spawn_blocking(move || {
        block_on(rtlsdr_state_clone.lock().unwrap().stop_stream(window));
    })
    .await
    .unwrap();
    Ok("".to_string())
}
