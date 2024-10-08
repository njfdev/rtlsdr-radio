use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use blocks::Rechunker;
use log::error;
use radiorust::{blocks::io::rf, prelude::*};
use soapysdr::Direction;
use tauri::{async_runtime, ipc::Channel, AppHandle, Emitter};
use tokio::{self, time};

use crate::{
    modes::types::ModeSState,
    radiorust_blocks::adsb_decode::AdsbDecode,
    sdr::{enumeration::AvailableSDRArgs, get_sdr_dev, release_sdr_dev},
};

pub struct AdsbDecoderState(Arc<Mutex<AdsbDecoderData>>);
pub struct AdsbDecoderData {
    pub decode_thread: Option<async_runtime::JoinHandle<()>>,
    pub shutdown_flag: Arc<AtomicBool>,
}

#[derive(serde::Deserialize)]
pub struct StreamSettings {}

impl AdsbDecoderState {
    pub fn new() -> Self {
        AdsbDecoderState(Arc::new(Mutex::new(AdsbDecoderData {
            decode_thread: None,
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        })))
    }

    // TODO: use these stream settings and allow users to modify ADS-B decode settings
    pub fn start_decoding(
        &self,
        app: AppHandle,
        _stream_settings: StreamSettings,
        sdr_args: AvailableSDRArgs,
        modes_channel: Channel<ModeSState>,
    ) {
        let adbs_decoder_state = self.0.clone();
        let adbs_decoder_state_clone = adbs_decoder_state.clone();

        let shutdown_flag = adbs_decoder_state.lock().unwrap().shutdown_flag.clone();

        adbs_decoder_state_clone.lock().unwrap().decode_thread =
            Some(async_runtime::spawn_blocking(move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(async move {
                        // get SDR
                        let rtlsdr_dev_result = get_sdr_dev(app.clone(), sdr_args);

                        if rtlsdr_dev_result.is_err() {
                            // notify frontend of error
                            app.emit("rtlsdr_err", unsafe {
                                rtlsdr_dev_result.unwrap_err_unchecked()
                            })
                            .expect("failed to emit event");
                            app.emit("rtlsdr_status", "stopped")
                                .expect("failed to emit event");

                            // remove the reference to the thread
                            drop(adbs_decoder_state.lock().unwrap().decode_thread.take());
                            return;
                        }

                        let (rtlsdr_dev, sdr_args) = rtlsdr_dev_result.unwrap();

                        // set sample rate (the clock is 1MHz, so we need at least 2MHz sample rate, which the RTL-SDR can barely do)
                        let sample_rate = 2e6;
                        let _ = rtlsdr_dev.set_sample_rate(Direction::Rx, 0, sample_rate);

                        // set center frequency
                        rtlsdr_dev
                            .set_frequency(Direction::Rx, 0, 1090.0 * 1_000_000.0, "")
                            .expect("Failed to set frequency");

                        // make sure direct sampling is disabled
                        let _ = rtlsdr_dev.write_setting("direct_samp", "0");

                        // enable automatic gain mode
                        rtlsdr_dev
                            .set_gain_mode(Direction::Rx, 0, true)
                            .expect("Failed to set automatic gain");

                        // set the bandwidth
                        let _ = rtlsdr_dev.set_bandwidth(Direction::Rx, 0, sample_rate / 2.0);

                        // start sdr rx stream
                        let rx_stream = rtlsdr_dev.rx_stream::<Complex<f32>>(&[0]).unwrap();
                        let sdr_rx = rf::soapysdr::SoapySdrRx::new(rx_stream, sample_rate);
                        sdr_rx.activate().await.unwrap();

                        let rechunker = Rechunker::new((sample_rate).round() as usize);
                        rechunker.feed_from(&sdr_rx);

                        // add buffer to discard samples that take long than 1 second to be processed by ADS-B decode (to prevent slowdowns)
                        let buffer = blocks::Buffer::new(0.0, 0.0, 0.0, 0.1);
                        buffer.feed_from(&rechunker);

                        let adsb_decode = AdsbDecode::new(modes_channel, false);
                        adsb_decode.feed_from(&buffer);

                        // let wavwriter = WavWriterBlock::new(
                        //     String::from("../adsb_output.wav"),
                        //     false,
                        //     Some(10.0),
                        // );
                        // wavwriter.feed_from(&adsb_decode);

                        while !shutdown_flag.load(Ordering::SeqCst) {
                            // notify frontend that decoding is happening
                            app.emit("adsb_status", "running")
                                .expect("failed to emit event");

                            time::sleep(Duration::from_millis(250)).await;
                        }

                        // release the SDR
                        release_sdr_dev(app, rtlsdr_dev, sdr_args).unwrap();
                    })
            }));
    }

    pub async fn stop_decoding(&self, app: AppHandle) {
        if let Ok(mut adsb_decoder_data) = self.0.clone().lock() {
            adsb_decoder_data
                .shutdown_flag
                .store(true, Ordering::SeqCst);

            if let Some(thread) = adsb_decoder_data.decode_thread.take() {
                thread.await.expect("Failed to join thread");
            }

            adsb_decoder_data
                .shutdown_flag
                .store(false, Ordering::SeqCst);

            app.emit("adsb_status", Some("stopped"))
                .expect("failed to emit event");
        } else {
            error!("Could not acquire lock immediately");
            return;
        }
    }

    pub fn is_running(&self) -> bool {
        return self.0.clone().lock().unwrap().decode_thread.is_some();
    }
}
