use enumeration::AvailableSDRArgs;
use log::{error, info};
use soapysdr::Device;
use tauri::AppHandle;

pub mod enumeration;

pub fn connect_to_sdr(args: AvailableSDRArgs, app: AppHandle) -> Result<(), ()> {
    let dev_result = Device::new(args.clone());

    if dev_result.is_err() {
        error!("Could not connect to {}.", args.label);
        return Err(());
    }

    info!("Connected to {}!", args.label);

    Ok(())
}
