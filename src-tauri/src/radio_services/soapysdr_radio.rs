use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use log::{debug, error};
use radiorust::{blocks::io::rf, prelude::*};
use soapysdr::Direction;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use tauri::{async_runtime, ipc::Channel, AppHandle, Emitter, Listener, Manager};
use tokio::{self, time};

use crate::{
    nrsc5::bindings::{NRSC5_SAMPLE_RATE_CS16_FM, NRSC5_SAMPLE_RATE_CU8},
    radiorust_blocks::{
        am_demod::AmDemod,
        better_cpal,
        hd_radio_decode::{HdRadioDecode, HdRadioState},
        pauseable::Pauseable,
        rbds_decode::{DownMixer, RbdsDecode, RbdsState},
        wav_writer::WavWriterBlock,
    },
    sdr::{enumeration::AvailableSDRArgs, get_sdr_dev, release_sdr_dev},
};

#[derive(serde::Deserialize, PartialEq)]
pub enum StreamType {
    FM = 0,
    AM = 1,
    HD = 2,
}

pub struct RtlSdrState(Arc<Mutex<RtlSdrData>>);
pub struct RtlSdrData {
    pub radio_stream_thread: Option<async_runtime::JoinHandle<()>>,
    pub shutdown_flag: Arc<AtomicBool>,
}

#[derive(serde::Deserialize)]
pub struct StreamSettings {
    freq: f64,
    volume: f64,
    gain: f64,
    sample_rate: f64,
    stream_type: StreamType,
    hd_radio_program: Option<u32>,
}

impl RtlSdrState {
    pub fn new() -> Self {
        RtlSdrState(Arc::new(Mutex::new(RtlSdrData {
            radio_stream_thread: None,
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        })))
    }

    pub fn start_stream(
        &self,
        app: AppHandle,
        stream_settings: StreamSettings,
        default_sdr_args: AvailableSDRArgs,
        rbds_channel: Channel<RbdsState>,
        hd_radio_channel: Channel<HdRadioState>,
    ) {
        let rtlsdr_state = self.0.clone();
        let rtlsdr_state_clone = rtlsdr_state.clone();

        let shutdown_flag = rtlsdr_state.lock().unwrap().shutdown_flag.clone();

        // set defaults for FM Radio
        let mut freq_mul: f64 = 1_000_000.0;
        let mut freq_offset: f64 = 0.0;
        let mut required_bandwidth: f64 = 200_000.0;
        let mut sample_rate = 1.000e6;
        let mut downsampled_rate = 336000.0;

        // if AM Radio, use KHz instead
        if stream_settings.stream_type == StreamType::AM {
            freq_mul = 1_000.0;
            required_bandwidth = 10_000.0;
        }

        // TODO: properly use both sidebands of HD Radio signal
        // if HD Radio, focus in on the lower sideband
        if stream_settings.stream_type == StreamType::HD {
            required_bandwidth = 400_000.0;
            downsampled_rate = NRSC5_SAMPLE_RATE_CS16_FM;
        }

        rtlsdr_state.lock().unwrap().radio_stream_thread =
            Some(async_runtime::spawn_blocking(move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(async move {
                        // get SDR
                        let rtlsdr_dev_result = get_sdr_dev(app.clone(), default_sdr_args);

                        if rtlsdr_dev_result.is_err() {
                            // notify frontend of error
                            app.emit("rtlsdr_err", unsafe {
                                rtlsdr_dev_result.unwrap_err_unchecked()
                            })
                            .expect("failed to emit event");
                            app.emit("rtlsdr_status", "stopped")
                                .expect("failed to emit event");

                            // remove the reference to the thread
                            drop(
                                rtlsdr_state_clone
                                    .lock()
                                    .unwrap()
                                    .radio_stream_thread
                                    .take(),
                            );
                            return;
                        }

                        // setup media controls
                        #[cfg(not(target_os = "windows"))]
                        let hwnd = None;

                        #[cfg(target_os = "windows")]
                        let hwnd = {
                            let webview_window = app.get_webview_window("main").unwrap();
                            let native_window = webview_window.hwnd().unwrap();
                            let hwnd = native_window.0;

                            Some(hwnd)
                        };

                        let config = PlatformConfig {
                            dbus_name: "dev.njf.RTL_SDR_Radio",
                            display_name: "RTL-SDR Radio",
                            hwnd,
                        };

                        let mut controls = MediaControls::new(config).unwrap();
                        let _ = controls.set_playback(MediaPlayback::Playing { progress: None });

                        let resource_dir = app.path().resource_dir().unwrap();
                        let icon_url = format!(
                            "file://{}/resources/AppIcon.png",
                            resource_dir.as_os_str().to_str().unwrap()
                        );
                        let radio_type_name =
                            Some(if stream_settings.stream_type == StreamType::FM {
                                "FM Radio"
                            } else if stream_settings.stream_type == StreamType::HD {
                                "HD Radio"
                            } else {
                                "AM Radio"
                            });
                        let _ = controls.set_metadata(MediaMetadata {
                            title: radio_type_name,
                            cover_url: Some(icon_url.as_str()),
                            ..Default::default()
                        });

                        let is_paused = Arc::new(Mutex::new(false));
                        let is_paused_clone = is_paused.clone();

                        let controls_arc = Arc::new(Mutex::new(controls));
                        let controls_clone = controls_arc.clone();

                        // The closure must be Send and have a static lifetime.
                        {
                            controls_arc
                                .lock()
                                .unwrap()
                                .attach(move |event: MediaControlEvent| {
                                    let mut is_paused_locked = is_paused_clone.lock().unwrap();
                                    match event {
                                        MediaControlEvent::Pause => {
                                            *is_paused_locked = true;
                                        }
                                        MediaControlEvent::Play => {
                                            *is_paused_locked = false;
                                        }
                                        MediaControlEvent::Toggle => {
                                            *is_paused_locked =
                                                if *is_paused_locked { false } else { true };
                                        }
                                        _ => {
                                            debug!("Unhandled Media Control: {:?}", event);
                                        }
                                    }
                                    let mut locked_controls = controls_clone.lock().unwrap();
                                    if *is_paused_locked {
                                        let _ = locked_controls
                                            .set_playback(MediaPlayback::Paused { progress: None });
                                    } else {
                                        let _ =
                                            locked_controls.set_playback(MediaPlayback::Playing {
                                                progress: None,
                                            });
                                    }
                                })
                                .unwrap();
                        }

                        let (rtlsdr_dev, sdr_args) = rtlsdr_dev_result.unwrap();

                        // set sample rate
                        let _ = rtlsdr_dev.set_sample_rate(Direction::Rx, 0, sample_rate);

                        // set center frequency
                        let sdr_freq = stream_settings.freq * freq_mul + freq_offset;
                        debug!("{}hz", sdr_freq);
                        rtlsdr_dev
                            .set_frequency(Direction::Rx, 0, sdr_freq, "")
                            .expect("Failed to set frequency");

                        // set the bandwidth
                        let _ = rtlsdr_dev.set_bandwidth(Direction::Rx, 0, 1.000e6);

                        // start sdr rx stream
                        let rx_stream = rtlsdr_dev.rx_stream::<Complex<f32>>(&[0]).unwrap();
                        let sdr_rx = rf::soapysdr::SoapySdrRx::new(rx_stream, sample_rate);
                        sdr_rx.activate().await.unwrap();

                        // turn on direct sampling mode if in low frequencies
                        if stream_settings.stream_type == StreamType::AM {
                            // 0 -> disabled, 1 -> I-branch direct sampling, 2 -> Q-branch direct sampling
                            let _ = rtlsdr_dev.write_setting("direct_samp", "2");
                        } else {
                            let _ = rtlsdr_dev.write_setting("direct_samp", "0");
                        }

                        // disable automatic gain mode on RTL-SDR (does not work that well)
                        if sdr_args.driver == "rtlsdr" {
                            rtlsdr_dev
                                .set_gain_mode(Direction::Rx, 0, false)
                                .expect("Failed to set automatic gain");
                        } else {
                            rtlsdr_dev
                                .set_gain_mode(Direction::Rx, 0, true)
                                .expect("Failed to set automatic gain");
                        }

                        // set a predetermined gain value
                        // TODO: figure out a better automatic way to set this
                        // if SDRPlay, use better special RF Gain control
                        if sdr_args.driver == "sdrplay" {
                            let _ = rtlsdr_dev.write_setting(
                                "rfgain_sel",
                                stream_settings.gain.round().to_string().as_str(),
                            );
                        } else {
                            rtlsdr_dev
                                .set_gain(Direction::Rx, 0, stream_settings.gain)
                                .expect("Failed to set a gain value");
                        }

                        // add frequency shifter
                        let freq_shifter = blocks::FreqShifter::<f32>::with_shift(0.0e6);
                        freq_shifter.feed_from(&sdr_rx);

                        // add downsampler
                        let downsample1 = blocks::Downsampler::<f32>::new(
                            16384,
                            downsampled_rate,
                            required_bandwidth,
                        );
                        downsample1.feed_from(&freq_shifter);

                        // add lowpass filter
                        let filter1 = blocks::Filter::new(|_, freq| {
                            if freq.abs() <= 100000.0 {
                                Complex::from(1.0)
                            } else {
                                Complex::from(0.0)
                            }
                        });
                        if !(stream_settings.stream_type == StreamType::HD) {
                            filter1.feed_from(&downsample1);
                        }

                        let pauser = Pauseable::new(is_paused);

                        if stream_settings.stream_type == StreamType::FM {
                            // demodulate fm signal
                            let demodulator = blocks::modulation::FmDemod::<f32>::new(150000.0);
                            demodulator.feed_from(&filter1);
                            pauser.feed_from(&demodulator);

                            // add a buffer
                            let rbds_buffer = blocks::Buffer::new(0.0, 0.0, 0.0, 5.0);
                            rbds_buffer.feed_from(&demodulator);

                            // upper sideband
                            const RBDS_FREQ: f64 = 57_000.0;
                            const RBDS_BANDWIDTH: f64 = 2_000.0;

                            // Step 1. apply bandpass signal to 57Khz with bandwidth of 4KHz for RBDS decoding
                            let rbds_bandpass_filter = blocks::Filter::new(|_, freq| {
                                if freq.abs() >= (RBDS_FREQ + 1000.0 - (RBDS_BANDWIDTH / 2.0))
                                    && freq.abs() <= (RBDS_FREQ + 1000.0 + (RBDS_BANDWIDTH / 2.0))
                                {
                                    Complex::from(1.0)
                                } else {
                                    Complex::from(0.0)
                                }
                            });
                            rbds_bandpass_filter.feed_from(&rbds_buffer);

                            // Step 2. downmix the signal
                            let rbds_downmixer = DownMixer::<f32>::new(RBDS_FREQ as f32);
                            rbds_downmixer.feed_from(&rbds_bandpass_filter);

                            // Step 3. remove high-frequency data and very-low frequency data
                            let rbds_lowpass_filter = blocks::Filter::new(|_, freq| {
                                if freq.abs() >= 10.0 && freq.abs() <= (RBDS_FREQ) {
                                    Complex::from(1.0)
                                } else {
                                    Complex::from(0.0)
                                }
                            });
                            rbds_lowpass_filter.feed_from(&rbds_downmixer);

                            let controls_clone2 = controls_arc.clone();
                            // add rbds decoder to output FM stream
                            let rdbs_decoder =
                                RbdsDecode::<f32>::new(rbds_channel, move |radiotext: String| {
                                    let _ = controls_clone2.lock().unwrap().set_metadata(
                                        MediaMetadata {
                                            title: Some(&radiotext),
                                            artist: radio_type_name,
                                            cover_url: Some(icon_url.as_str()),
                                            ..Default::default()
                                        },
                                    );
                                });
                            rdbs_decoder.feed_from(&rbds_lowpass_filter);
                        } else if stream_settings.stream_type == StreamType::AM {
                            let demodulator = AmDemod::<f32>::new();
                            demodulator.feed_from(&filter1);
                            pauser.feed_from(&demodulator);
                        } else if stream_settings.stream_type == StreamType::HD {
                            let controls_clone2 = controls_arc.clone();

                            let hd_radio_decoder = HdRadioDecode::<f32>::new(
                                stream_settings.hd_radio_program.unwrap(),
                                true,
                                move |state: HdRadioState| {
                                    let thumbnail_base64 = state.thumbnail_data.clone();
                                    let cover_url = if state.clone().thumbnail_data.is_some() {
                                        thumbnail_base64.unwrap()
                                    } else {
                                        icon_url.clone()
                                    };
                                    let _ = controls_clone2.lock().unwrap().set_metadata(
                                        MediaMetadata {
                                            title: Some(&state.title),
                                            artist: Some(&state.artist),
                                            album: Some(&state.album),
                                            cover_url: Some(cover_url.as_str()),
                                            ..Default::default()
                                        },
                                    );

                                    //println!("HD Radio State: {:#?}", state);

                                    hd_radio_channel.send(state);
                                },
                            );
                            hd_radio_decoder.feed_from(&downsample1);

                            // let test_recorder = WavWriterBlock::<f32>::new(
                            //     "nrsc5_test_direct_output.wav".to_string(),
                            //     false,
                            //     None,
                            // );
                            // test_recorder.feed_from(&hd_radio_decoder);

                            pauser.feed_from(&hd_radio_decoder);

                            app.clone().listen("radio_update_settings", move |event| {
                                if let Ok(new_settings) =
                                    serde_json::from_str::<StreamSettings>(&event.payload())
                                {
                                    if hd_radio_decoder.get()
                                        != new_settings.hd_radio_program.unwrap()
                                    {
                                        hd_radio_decoder
                                            .set(new_settings.hd_radio_program.unwrap());
                                    }
                                }
                            });
                        }

                        // filter frequencies beyond normal human hearing range (20hz to 16 kHz)
                        let filter2 = blocks::filters::Filter::new_rectangular(|bin, freq| {
                            if bin.abs() >= 1 && freq.abs() >= 20.0 && freq.abs() <= 16000.0 {
                                blocks::filters::deemphasis_factor(50e-6, freq)
                            } else {
                                Complex::from(0.0)
                            }
                        });
                        if stream_settings.stream_type != StreamType::HD {
                            filter2.feed_from(&pauser);
                        }

                        // downsample so the output device can play the audio
                        let downsample2 = blocks::Downsampler::<f32>::new(
                            4096,
                            stream_settings.sample_rate,
                            stream_settings.sample_rate / 2.0,
                        );
                        downsample2.feed_from(&filter2);

                        // add a volume block
                        let volume = blocks::GainControl::<f32>::new(stream_settings.volume);
                        if stream_settings.stream_type == StreamType::HD {
                            volume.feed_from(&pauser);
                        } else {
                            volume.feed_from(&downsample2);
                        }

                        // add a buffer
                        let buffer = blocks::Buffer::new(0.0, 0.0, 0.0, 1.0);
                        buffer.feed_from(&volume);

                        // output the stream
                        let playback = better_cpal::AudioPlayer::new(
                            stream_settings.sample_rate,
                            None,
                            2,
                            Some(if stream_settings.stream_type == StreamType::HD {
                                false
                            } else {
                                true
                            }),
                        )
                        .unwrap();
                        playback.feed_from(&buffer);

                        let sdr_clone = rtlsdr_dev.clone();
                        let args_clone = sdr_args.clone();
                        app.listen("radio_update_settings", move |event| {
                            if let Ok(new_settings) =
                                serde_json::from_str::<StreamSettings>(&event.payload())
                            {
                                if volume.get() != new_settings.volume {
                                    volume.set(new_settings.volume);
                                }
                                let sdr_freq = new_settings.freq * freq_mul;
                                if sdr_clone.frequency(Direction::Rx, 0).unwrap() != sdr_freq {
                                    // set center frequency
                                    sdr_clone
                                        .set_frequency(Direction::Rx, 0, sdr_freq, "")
                                        .expect("Failed to set new frequency");
                                }
                                if args_clone.driver == "sdrplay" {
                                    let _ = sdr_clone.write_setting(
                                        "rfgain_sel",
                                        new_settings.gain.round().to_string().as_str(),
                                    );
                                } else {
                                    if sdr_clone.gain(Direction::Rx, 0).unwrap()
                                        != new_settings.gain
                                    {
                                        // set center frequency
                                        sdr_clone
                                            .set_gain(Direction::Rx, 0, new_settings.gain)
                                            .expect("Failed to set new gain");
                                    }
                                }
                            }
                        });

                        let prefix: &str;

                        if stream_settings.stream_type == StreamType::FM {
                            prefix = "fm";
                        } else if stream_settings.stream_type == StreamType::AM {
                            prefix = "am";
                        } else {
                            prefix = "hd";
                        }

                        while !shutdown_flag.load(Ordering::SeqCst) {
                            // notify frontend that audio is playing
                            app.emit("rtlsdr_status", format!("{}_{}", prefix, "running"))
                                .expect("failed to emit event");

                            time::sleep(Duration::from_millis(250)).await;
                        }

                        // release the SDR
                        release_sdr_dev(app, rtlsdr_dev, sdr_args).unwrap();
                    })
            }));
    }

    pub async fn stop_stream(&self, app: AppHandle) {
        if let Ok(mut rtl_sdr_data) = self.0.clone().lock() {
            rtl_sdr_data.shutdown_flag.store(true, Ordering::SeqCst);

            if let Some(thread) = rtl_sdr_data.radio_stream_thread.take() {
                thread.await.expect("Failed to join thread");
            }

            rtl_sdr_data.shutdown_flag.store(false, Ordering::SeqCst);

            app.emit("rtlsdr_status", Some("stopped"))
                .expect("failed to emit event");
        } else {
            error!("Could not acquire lock immediately");
            return;
        }
    }

    pub fn is_playing(&self) -> bool {
        return self.0.clone().lock().unwrap().radio_stream_thread.is_some();
    }
}
