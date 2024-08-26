use log::{debug, info};
use std::env;
use tauri::{App, Emitter, Manager};

use crate::sdr_enumeration::register_available_sdrs_callback;

pub fn setup_dependencies(app: &mut App) {
    let resource_dir = app.path().resource_dir().unwrap();

    // set env for SoapySDR modules
    let modules_path = resource_dir.join("resources/lib/SoapySDR/modules0.8/");
    let mut modules_path_str = modules_path.to_str().unwrap();
    /* On windows, \\?\ is a valid prefix to a path, however, it prevents SoapySDR
     * from loading the correct path, so we remove it if it exists.
     */
    if modules_path_str.starts_with("\\\\?\\") {
        modules_path_str = &modules_path_str[4..];
    }
    env::set_var("SOAPY_SDR_PLUGIN_PATH", modules_path_str);
    debug!(
        "SoapySDR Plugin Path: {}",
        env::var("SOAPY_SDR_PLUGIN_PATH").unwrap()
    );
}

pub fn setup_callbacks(app: &mut App) {
    let app_handle = app.app_handle().clone();
    register_available_sdrs_callback(5.0, move |args| {
        info!("Available SDR Details: {:?}", args);
        let _ = app_handle.emit("available_sdrs", args);
    });
}
