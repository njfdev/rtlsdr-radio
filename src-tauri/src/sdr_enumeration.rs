use std::{panic, time::Duration};

use log::info;
use rusb::{Context, GlobalContext, Hotplug, HotplugBuilder, UsbContext};
use serde::Serialize;
use soapysdr::{enumerate, Args};
use tokio::time::sleep;

pub fn get_connected_sdr_args() -> Result<Vec<ConnectedSDRArgs>, ()> {
    let args = panic::catch_unwind(|| enumerate("driver=rtlsdr"));

    if args.is_err() || args.as_ref().unwrap().is_err() {
        return Err(());
    }

    Ok(args_to_connected_sdr_args(args.unwrap().unwrap()))
}

#[derive(Default)]
struct SdrConnectionHandler();

impl<T> Hotplug<T> for SdrConnectionHandler
where
    T: UsbContext,
{
    fn device_arrived(&mut self, device: rusb::Device<T>) {
        info!("Device Connected");
    }

    fn device_left(&mut self, device: rusb::Device<T>) {
        info!("Device Disconnected");
    }
}

pub fn register_connected_sdrs_callback<F>(polling_rate: f32, callback: F)
where
    F: Fn(Vec<ConnectedSDRArgs>) + Send + 'static,
{
    tokio::spawn(async move {
        let libusb_context = GlobalContext::default();

        let mut hotplug_builder = HotplugBuilder::new();
        hotplug_builder.register(libusb_context, Box::new(SdrConnectionHandler::default()));
        hotplug_builder.enumerate(true);

        let mut prev_args = get_connected_sdr_args();

        if prev_args.is_ok() {
            callback(prev_args.clone().unwrap());
        }

        // run until the application closes
        loop {
            sleep(Duration::from_secs_f32(1.0 / polling_rate)).await;

            let args = get_connected_sdr_args();

            if args != prev_args {
                if args.is_ok() {
                    callback(args.clone().unwrap());
                }
                prev_args = args;
            }
        }
    });
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ConnectedSDRArgs {
    pub driver: String,
    pub label: String,
    pub manufacturer: String,
    pub product: String,
    pub serial: String,
    pub tuner: String,
}

pub fn args_to_connected_sdr_args(args: Vec<Args>) -> Vec<ConnectedSDRArgs> {
    args.iter()
        .map(|args| ConnectedSDRArgs {
            driver: args.get("driver").unwrap().to_string(),
            label: args.get("label").unwrap().to_string(),
            manufacturer: args.get("manufacturer").unwrap().to_string(),
            product: args.get("product").unwrap().to_string(),
            serial: args.get("serial").unwrap().to_string(),
            tuner: args.get("tuner").unwrap().to_string(),
        })
        .collect::<Vec<ConnectedSDRArgs>>()
}
