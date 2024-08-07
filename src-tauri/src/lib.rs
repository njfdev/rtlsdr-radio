// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod radio_services;
mod radiorust_blocks;
mod utils;

use radio_services::{
    nrsc5::Nrsc5State,
    soapysdr_adsb::{self, AdsbDecoderState},
    soapysdr_radio::{self, RtlSdrState},
};
use std::{
    env,
    sync::{Arc, Mutex},
};
use tauri::{async_runtime::block_on, AppHandle, State};
use utils::utils::setup_dependencies;

struct AppState {
    nrsc5_state: Nrsc5State,
    rtl_sdr_state: Arc<Mutex<RtlSdrState>>,
    adsb_state: Arc<Mutex<AdsbDecoderState>>,
}

#[tokio::main]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::Builder::default()
        .setup(|app| {
            setup_dependencies(app);
            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            nrsc5_state: Nrsc5State::new(),
            rtl_sdr_state: Arc::new(Mutex::new(RtlSdrState::new())),
            adsb_state: Arc::new(Mutex::new(AdsbDecoderState::new())),
        })
        .invoke_handler(tauri::generate_handler![
            start_nrsc5,
            stop_nrsc5,
            start_stream,
            stop_stream,
            start_adsb_decoding,
            stop_adsb_decoding
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn start_nrsc5(app: AppHandle, state: State<AppState>, fm_freq: String, channel: String) {
    if state.nrsc5_state.is_playing() {
        return;
    };
    state.nrsc5_state.start_thread(app, fm_freq, channel);
}

#[tauri::command]
fn stop_nrsc5(app: AppHandle, state: State<AppState>) {
    state.nrsc5_state.stop_thread(app);
}

#[tauri::command]
fn start_stream(
    app: AppHandle,
    state: State<AppState>,
    stream_settings: soapysdr_radio::StreamSettings,
) {
    if state.rtl_sdr_state.lock().unwrap().is_playing() {
        return;
    };
    state
        .rtl_sdr_state
        .lock()
        .unwrap()
        .start_stream(app, stream_settings);
}

#[tauri::command]
async fn stop_stream(app: AppHandle, state: State<'_, AppState>) -> Result<String, ()> {
    let rtlsdr_state_clone = state.rtl_sdr_state.clone();

    tokio::task::spawn_blocking(move || {
        block_on(rtlsdr_state_clone.lock().unwrap().stop_stream(app));
    })
    .await
    .unwrap();
    Ok("".to_string())
}

#[tauri::command]
fn start_adsb_decoding(
    app: AppHandle,
    state: State<AppState>,
    stream_settings: soapysdr_adsb::StreamSettings,
) {
    if state.adsb_state.lock().unwrap().is_running() {
        return;
    };
    state
        .adsb_state
        .lock()
        .unwrap()
        .start_decoding(app, stream_settings);
}

#[tauri::command]
async fn stop_adsb_decoding(app: AppHandle, state: State<'_, AppState>) -> Result<String, ()> {
    let adsb_state_clone = state.adsb_state.clone();

    tokio::task::spawn_blocking(move || {
        block_on(adsb_state_clone.lock().unwrap().stop_decoding(app));
    })
    .await
    .unwrap();
    Ok("".to_string())
}
