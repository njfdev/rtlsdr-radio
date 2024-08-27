use enumeration::AvailableSDRArgs;
use log::{error, info};
use serde::{Deserialize, Serialize, Serializer};
use soapysdr::Device;
use tauri::{AppHandle, Emitter, State};

use crate::AppState;

pub mod enumeration;

fn serialize_device<S>(dev: &SDRDeviceState, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match dev {
        SDRDeviceState::Available => {
            return serializer.serialize_bool(false);
        }
        _ => {
            return serializer.serialize_bool(true);
        }
    }
}

#[derive(Clone)]
pub enum SDRDeviceState {
    Available,
    Connected { dev: Device },
    InUse,
}

#[derive(Serialize, Clone)]
pub struct SDRState {
    pub args: AvailableSDRArgs,
    #[serde(serialize_with = "serialize_device")]
    pub dev: SDRDeviceState,
}

pub fn connect_to_sdr(
    args: AvailableSDRArgs,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), ()> {
    let dev_result = Device::new(args.clone());

    if dev_result.is_err() {
        error!("Could not connect to {}.", args.label);
        return Err(());
    }

    let dev = dev_result.unwrap();

    info!("Connected to {}!", args.label);

    // find sdr and add dev to it
    let mut sdrs = state.sdrs.lock().unwrap();
    let find_sdr_result = sdrs.iter_mut().find(|sdr| sdr.args == args);
    if find_sdr_result.is_some() {
        find_sdr_result.unwrap().dev = SDRDeviceState::Connected { dev };
    } else {
        // create new sdr state if not found
        sdrs.push(SDRState {
            args,
            dev: SDRDeviceState::Connected { dev },
        });
    }

    app.emit("sdr_states", sdrs.clone()).unwrap();

    Ok(())
}

pub fn disconnect_sdr(
    args: AvailableSDRArgs,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), &str> {
    // find sdr and add dev to it
    let mut sdrs = state.sdrs.lock().unwrap();
    let find_sdr_result = sdrs.iter_mut().find(|sdr| sdr.args == args);
    if find_sdr_result.is_none() {
        return Err("Could not find SDR to disconnect");
    }

    let sdr = find_sdr_result.unwrap();

    match sdr.dev.clone() {
        SDRDeviceState::Available => {
            return Err("SDR already disconnected");
        }
        SDRDeviceState::Connected { dev } => {
            sdr.dev = SDRDeviceState::Available;

            app.emit("sdr_states", sdrs.clone()).unwrap();
            return Ok(());
        }
        SDRDeviceState::InUse => {
            return Err("Can't disconnect SDR because it is in use");
        }
    }
}
