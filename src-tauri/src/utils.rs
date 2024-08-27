use log::{debug, info};
use std::env;
use tauri::{App, Emitter, Manager};

use crate::{
    sdr::{enumeration::register_available_sdrs_callback, SDRDeviceState, SDRState},
    AppState,
};

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

        let state = app_handle.state::<AppState>();
        let mut sdrs = state.sdrs.lock().unwrap();

        // remove existing devices that no longer are available
        for existing_dev in sdrs.clone().iter() {
            let matching_args = args
                .iter()
                .find(|available_args| existing_dev.args == **available_args);
            if matching_args.is_none() {
                sdrs.retain(|sdr_state| sdr_state.args != existing_dev.args);
            }
        }

        // add args that don't have an existing device
        for available_arg in args.clone().iter() {
            if sdrs
                .iter()
                .find(|sdr_state| sdr_state.args == *available_arg)
                .is_none()
            {
                sdrs.push(SDRState {
                    args: available_arg.to_owned(),
                    dev: SDRDeviceState::Available,
                });
            }
        }

        let _ = app_handle.emit("sdr_states", sdrs.clone());
    });
}
