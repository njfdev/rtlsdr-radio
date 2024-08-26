use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use soapysdr::{enumerate, Args};
use soapysdr_sys::SoapySDRKwargs;
use tokio::time::sleep;

fn get_raw_connected_sdr_args() -> Vec<*mut SoapySDRKwargs> {
    enumerate("driver=rtlsdr")
        .unwrap()
        .iter_mut()
        .map(|arg| (*arg).as_raw())
        .collect()
}

pub fn connected_sdrs_hotplug_callback<F>(polling_rate: f32, callback: F)
where
    F: Fn(Vec<Args>) + Send + 'static,
{
    tokio::spawn(async move {
        callback(raw_args_vec_to_args_vec(get_raw_connected_sdr_args()));

        let prev_raw_args: Vec<*mut SoapySDRKwargs> = vec![];

        // run until the application closes
        loop {
            sleep(Duration::from_secs_f32(1.0 / polling_rate)).await;

            let args = get_raw_connected_sdr_args();

            if are_args_equal(args, prev_raw_args) {
                callback(raw_args_vec_to_args_vec(args));
                prev_raw_args = args;
            }
        }
    });
}

pub fn raw_args_vec_to_args_vec(raw_args: Vec<*mut SoapySDRKwargs>) -> Vec<Args> {
    raw_args
        .iter()
        .map(|raw_args| unsafe { Args::from_raw(**raw_args) })
        .collect()
}

pub fn args_to_string(args: Vec<Args>) -> String {
    format!(
        "{:?}",
        args.iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
    )
}

fn are_args_equal(args1: Vec<*mut SoapySDRKwargs>, args2: Vec<*mut SoapySDRKwargs>) -> bool {
    if args_to_string(raw_args_vec_to_args_vec(args1))
        == args_to_string(raw_args_vec_to_args_vec(args2))
    {
        return true;
    }

    false
}
