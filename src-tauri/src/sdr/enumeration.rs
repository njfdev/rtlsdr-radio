use std::{panic, thread::sleep, time::Duration};

use rusb::{Context, DeviceList, UsbContext};
use serde::{Deserialize, Serialize};
use soapysdr::{enumerate, Args};
use struct_iterable::Iterable;
use tokio::task;

pub fn get_available_sdr_args() -> Result<Vec<AvailableSDRArgs>, ()> {
    let args = panic::catch_unwind(|| enumerate("driver=rtlsdr"));

    if args.is_err() || args.as_ref().unwrap().is_err() {
        return Err(());
    }

    Ok(args_to_available_sdr_args(args.unwrap().unwrap()))
}

pub fn register_available_sdrs_callback<F>(polling_rate: f32, callback: F)
where
    F: Fn(Vec<AvailableSDRArgs>) + Send + 'static,
{
    task::spawn({
        let polling_rate = polling_rate;
        async move {
            let libusb_context = Context::new().unwrap();

            let mut prev_dev_list: DeviceList<Context> = libusb_context.devices().unwrap();
            let mut is_first_call = true;

            // run until the application closes
            loop {
                let dev_list = libusb_context.devices().unwrap();

                if !are_device_lists_equal(&dev_list, &prev_dev_list) || is_first_call {
                    let args = get_available_sdr_args();

                    if args.is_ok() {
                        callback(args.unwrap());
                    }
                    prev_dev_list = dev_list;
                    is_first_call = false;
                }

                sleep(Duration::from_secs_f32(1.0 / polling_rate));
            }
        }
    });
}

#[derive(Serialize, Clone, Debug, PartialEq, Deserialize, Iterable)]
pub struct AvailableSDRArgs {
    pub driver: String,
    pub label: String,
    pub manufacturer: String,
    pub product: String,
    pub serial: String,
    pub tuner: String,
}

impl Into<Args> for AvailableSDRArgs {
    fn into(self) -> Args {
        let mut args = Args::new();
        for (arg_key, arg_value) in self.iter() {
            if let Some(arg_string) = arg_value.downcast_ref::<String>() {
                args.set(arg_key, arg_string.to_owned());
            }
        }
        args
    }
}

pub fn args_to_available_sdr_args(args: Vec<Args>) -> Vec<AvailableSDRArgs> {
    args.iter()
        .map(|args| AvailableSDRArgs {
            driver: args.get("driver").unwrap().to_string(),
            label: args.get("label").unwrap().to_string(),
            manufacturer: args.get("manufacturer").unwrap().to_string(),
            product: args.get("product").unwrap().to_string(),
            serial: args.get("serial").unwrap().to_string(),
            tuner: args.get("tuner").unwrap().to_string(),
        })
        .collect::<Vec<AvailableSDRArgs>>()
}

pub fn are_device_lists_equal(list1: &DeviceList<Context>, list2: &DeviceList<Context>) -> bool {
    if list1.len() != list2.len() {
        return false;
    }

    for (a, b) in list1.iter().zip(list2.iter()) {
        let a_desc = a.device_descriptor().unwrap();
        let b_desc = b.device_descriptor().unwrap();

        if a_desc.product_id() != b_desc.product_id() || a_desc.vendor_id() != b_desc.vendor_id() {
            return false;
        }
    }

    true
}
