// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod custom_radiorust_blocks;
mod nrsc5;
mod rtlsdr;

use libloading::Library;
use nrsc5::nrsc5::Nrsc5State;
use rtlsdr::rtlsdr::{RtlSdrState, StreamSettings};
use std::{
    env,
    sync::{Arc, Mutex},
};
use tauri::{async_runtime::block_on, utils::platform::current_exe, AppHandle, State, Window};

struct AppState {
    nrsc5_state: Nrsc5State,
    rtl_sdr_state: Arc<Mutex<RtlSdrState>>,
}

#[tokio::main]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    // set env for SoapySDR modules
    //env::set_var(key, value);
    let current_exe_path = current_exe().unwrap();
    let current_exe_parent_dir = current_exe_path.parent().unwrap();
    let mut modules_path = current_exe_parent_dir.join("resources/lib/SoapySDR/modules0.8/");
    env::set_var("SOAPY_SDR_PLUGIN_PATH", modules_path.as_mut_os_str());

    // Determine the correct file extension for the shared library based on the OS
    let os_ext = if cfg!(target_os = "windows") {
        ".dll"
    } else if cfg!(target_os = "macos") {
        ".dylib"
    } else {
        ".so"
    };

    // load libusb shared library
    unsafe {
        let lib = Library::new(format!(
            "{}/resources/lib/libusb-1.0.0{}",
            current_exe_parent_dir.as_os_str().to_str().unwrap(),
            os_ext
        ))
        .expect("Failed to load shared library");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
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
fn start_stream(app: AppHandle, state: State<AppState>, stream_settings: StreamSettings) {
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
