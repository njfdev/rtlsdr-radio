// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod modes;
mod radio_services;
mod radiorust_blocks;
mod sdr;
mod utils;

use log::info;
use modes::types::ModeSState;
use radio_services::{
    nrsc5::Nrsc5State,
    soapysdr_adsb::{self, AdsbDecoderState},
    soapysdr_radio::{self, RtlSdrState},
};
use radiorust_blocks::rbds_decode::RbdsState;
use sdr::{enumeration::AvailableSDRArgs, SDRState};
use serde::Serialize;
use std::{
    env,
    sync::{Arc, Mutex},
};
use tauri::{async_runtime::block_on, ipc::Channel, AppHandle, Manager, State};
use utils::{setup_callbacks, setup_dependencies};

struct AppState {
    nrsc5_state: Nrsc5State,
    rtl_sdr_state: Arc<Mutex<RtlSdrState>>,
    adsb_state: Arc<Mutex<AdsbDecoderState>>,
    sdrs: Arc<Mutex<Vec<SDRState>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            nrsc5_state: Nrsc5State::new(),
            rtl_sdr_state: Arc::new(Mutex::new(RtlSdrState::new())),
            adsb_state: Arc::new(Mutex::new(AdsbDecoderState::new())),
            sdrs: Arc::new(Mutex::new(vec![])),
        }
    }
}

#[tokio::main]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .setup(|app| {
            setup_dependencies(app);
            setup_callbacks(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_nrsc5,
            stop_nrsc5,
            start_stream,
            stop_stream,
            start_adsb_decoding,
            stop_adsb_decoding,
            get_sdr_states,
            connect_to_sdr,
            disconnect_sdr
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
    sdr_args: AvailableSDRArgs,
    rbds_channel: Channel<RbdsState>,
) {
    if state.rtl_sdr_state.lock().unwrap().is_playing() {
        return;
    };
    state
        .rtl_sdr_state
        .lock()
        .unwrap()
        .start_stream(app, stream_settings, sdr_args, rbds_channel);
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
    sdr_args: AvailableSDRArgs,
    modes_channel: Channel<ModeSState>,
) {
    if state.adsb_state.lock().unwrap().is_running() {
        return;
    };
    state
        .adsb_state
        .lock()
        .unwrap()
        .start_decoding(app, stream_settings, sdr_args, modes_channel);
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

#[tauri::command]
async fn get_sdr_states(state: State<'_, AppState>) -> Result<serde_json::Value, ()> {
    let sdrs = state.sdrs.lock().unwrap();

    Ok(sdrs
        .clone()
        .serialize(serde_json::value::Serializer)
        .unwrap())
}

#[tauri::command]
async fn connect_to_sdr(
    app: AppHandle,
    state: State<'_, AppState>,
    args: AvailableSDRArgs,
) -> Result<(), ()> {
    info!("Connecting to {}", args.label);

    let result = sdr::connect_to_sdr(args, app);

    if result.is_err() {
        return Err(());
    }

    Ok(())
}

#[tauri::command]
async fn disconnect_sdr(
    app: AppHandle,
    state: State<'_, AppState>,
    args: AvailableSDRArgs,
) -> Result<(), &str> {
    info!("Disconnecting from {}", args.label);

    let result = sdr::disconnect_sdr(args, app, state);

    return result;
}
