// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod nrsc5;
mod rtlsdr;

use nrsc5::nrsc5::Nrsc5State;
use rtlsdr::rtlsdr::{RtlSdrState, StreamSettings};
use std::{
    ops::Deref,
    os::macos::raw::stat,
    sync::{Arc, Mutex},
    thread,
};
use tauri::{
    api::process::{Command, CommandEvent},
    async_runtime::block_on,
    App, Manager, State, Window,
};

struct AppState {
    nrsc5State: Nrsc5State,
    rtlSdrState: Arc<Mutex<RtlSdrState>>,
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .manage(AppState {
            nrsc5State: Nrsc5State::new(),
            rtlSdrState: Arc::new(Mutex::new(RtlSdrState::new())),
        })
        .invoke_handler(tauri::generate_handler![
            start_nrsc5,
            stop_nrsc5,
            start_fm_stream,
            stop_fm_stream
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn start_nrsc5(window: Window, state: State<AppState>, fm_freq: String, channel: String) {
    if state.nrsc5State.is_playing() {
        return;
    };
    state.nrsc5State.start_thread(window, fm_freq, channel);
}

#[tauri::command]
fn stop_nrsc5(window: Window, state: State<AppState>) {
    state.nrsc5State.stop_thread(window);
}

#[tauri::command]
fn start_fm_stream(window: Window, state: State<AppState>, stream_settings: StreamSettings) {
    if state.rtlSdrState.lock().unwrap().is_playing() {
        return;
    };
    state
        .rtlSdrState
        .lock()
        .unwrap()
        .start_stream(window, stream_settings);
}

#[tauri::command]
async fn stop_fm_stream(window: Window, state: State<'_, AppState>) -> Result<String, ()> {
    let rtlsdr_state_clone = state.rtlSdrState.clone();

    tokio::task::spawn_blocking(move || {
        block_on(rtlsdr_state_clone.lock().unwrap().stop_stream(window));
    })
    .await
    .unwrap();
    Ok("".to_string())
}
