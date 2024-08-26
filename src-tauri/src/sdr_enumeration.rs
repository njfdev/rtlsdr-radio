use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use soapysdr::{enumerate, Args};
use tokio::time::sleep;

pub fn get_connected_sdr_args_string() -> Vec<String> {
    args_to_string(enumerate("driver=rtlsdr").unwrap())
}

pub fn register_connected_sdrs_callback<F>(polling_rate: f32, callback: F)
where
    F: Fn(Vec<Args>) + Send + 'static,
{
    tokio::spawn(async move {
        let mut prev_args = get_connected_sdr_args_string();

        callback(string_args_to_args(&prev_args));

        // run until the application closes
        loop {
            sleep(Duration::from_secs_f32(1.0 / polling_rate)).await;

            let args = get_connected_sdr_args_string();

            if args.join(", ") != prev_args.join(", ") {
                callback(string_args_to_args(&args));
                prev_args = args;
            }
        }
    });
}

pub fn string_args_to_args(args: &Vec<String>) -> Vec<Args> {
    args.iter()
        .map(|string| Args::from(string.as_str()))
        .collect()
}

pub fn args_to_string(args: Vec<Args>) -> Vec<String> {
    args.iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<String>>()
}
