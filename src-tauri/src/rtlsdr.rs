pub mod rtlsdr {
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
        time::Duration,
    };

    use blocks::modulation::FmDemod;
    use radiorust::{
        blocks::io::{audio::cpal::AudioPlayer, rf},
        prelude::*,
    };
    use soapysdr::Direction;
    use tauri::{async_runtime, Window};
    use tokio::{self, time};

    use crate::custom_radiorust_blocks::custom_radiorust_blocks::AmDemod;

    #[derive(serde::Deserialize, PartialEq)]
    pub enum StreamType {
        FM = 0,
        AM = 1,
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
    }

    impl RtlSdrState {
        pub fn new() -> Self {
            RtlSdrState(Arc::new(Mutex::new(RtlSdrData {
                radio_stream_thread: None,
                shutdown_flag: Arc::new(AtomicBool::new(false)),
            })))
        }

        pub fn start_stream(&self, window: Window, stream_settings: StreamSettings) {
            let rtlsdr_state = self.0.clone();
            let rtlsdr_state_clone = rtlsdr_state.clone();

            let shutdown_flag = rtlsdr_state.lock().unwrap().shutdown_flag.clone();

            // set defaults for FM Radio
            let mut freq_mul: f64 = 1_000_000.0;
            let mut required_bandwidth: f64 = 200_000.0;

            // if AM Radio, use KHz instead
            if stream_settings.stream_type == StreamType::AM {
                freq_mul = 1_000.0;
                required_bandwidth = 10_000.0;
            }

            rtlsdr_state.lock().unwrap().radio_stream_thread = Some(async_runtime::spawn_blocking(
                move || {
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(async move {
                            // connect to SDR
                            let rtlsdr_dev_result = soapysdr::Device::new("driver=rtlsdr");

                            if rtlsdr_dev_result.is_err() {
                                // notify frontend of error
                                window
                                    .emit("rtlsdr_err", "Could not connect to your RTL-SDR. Make sure it is plugged in!")
                                    .expect("failed to emit event");
                                window
                                    .emit("rtlsdr_status", "stopped")
                                    .expect("failed to emit event");

                                // remove the reference to the thread
                                drop(rtlsdr_state_clone.lock().unwrap().radio_stream_thread.take());
                                return;
                            }

                            let rtlsdr_dev = rtlsdr_dev_result.unwrap();

                            // set sample rate
                            let sample_rate = 1.024e6;
                            let _ = rtlsdr_dev.set_sample_rate(Direction::Rx, 0, sample_rate);

                            // set center frequency
                            let sdr_freq = stream_settings.freq * freq_mul;
                            println!("{}hz", sdr_freq);
                            rtlsdr_dev
                                .set_frequency(Direction::Rx, 0, sdr_freq, "")
                                .expect("Failed to set frequency");

                            // set the bandwidth
                            let _ = rtlsdr_dev.set_bandwidth(Direction::Rx, 0, 1.024e6);

                            // turn on direct sampling mode if in low frequencies
                            if stream_settings.stream_type == StreamType::AM {
                                // 0 -> disabled, 1 -> I-branch direct sampling, 2 -> Q-branch direct sampling
                                let _ = rtlsdr_dev.write_setting("direct_samp", "2");
                                let _ = rtlsdr_dev.write_setting("digital_agc", "true");
                            }

                            // start sdr rx stream
                            let rx_stream = rtlsdr_dev.rx_stream::<Complex<f32>>(&[0]).unwrap();
                            let sdr_rx = rf::soapysdr::SoapySdrRx::new(rx_stream, sample_rate);
                            sdr_rx.activate().await.unwrap();

                            // add frequency shifter
                            let freq_shifter = blocks::FreqShifter::<f32>::with_shift(0.0e6);
                            freq_shifter.feed_from(&sdr_rx);

                            // add downsampler
                            let downsample1 =
                                blocks::Downsampler::<f32>::new(16384, 384000.0, required_bandwidth);
                            downsample1.feed_from(&freq_shifter);

                            // add gain
                            let gain = blocks::GainControl::<f32>::new(stream_settings.gain);
                            gain.feed_from(&downsample1);

                            // add lowpass filter
                            let filter1 = blocks::Filter::new(|_, freq| {
                                if freq.abs() <= 100000.0 {
                                    Complex::from(1.0)
                                } else {
                                    Complex::from(0.0)
                                }
                            });

                            if stream_settings.stream_type == StreamType::FM {
                                // demodulate fm signal
                                let demodulator = blocks::modulation::FmDemod::<f32>::new(150000.0);
                                demodulator.feed_from(&gain);
                                filter1.feed_from(&demodulator);
                            } else if stream_settings.stream_type == StreamType::AM {
                                let demodulator = AmDemod::<f32>::new();
                                demodulator.feed_from(&gain);
                                filter1.feed_from(&demodulator);
                            }

                            // filter frequencies beyond normal human hearing range (20hz to 16 kHz)
                            let filter2 = blocks::filters::Filter::new_rectangular(|bin, freq| {
                                if bin.abs() >= 1 && freq.abs() >= 20.0 && freq.abs() <= 16000.0 {
                                    blocks::filters::deemphasis_factor(50e-6, freq)
                                } else {
                                    Complex::from(0.0)
                                }
                            });
                            filter2.feed_from(&filter1);

                            // downsample so the output device can play the audio
                            let downsample2 = blocks::Downsampler::<f32>::new(
                                4096,
                                stream_settings.sample_rate,
                                2.0 * (required_bandwidth / 10.0),
                            );
                            downsample2.feed_from(&filter2);

                            // add a volume block
                            let volume = blocks::GainControl::<f32>::new(stream_settings.volume);
                            volume.feed_from(&downsample2);

                            // add a buffer
                            let buffer = blocks::Buffer::new(0.0, 0.0, 0.0, 1.0);
                            buffer.feed_from(&volume);

                            // output the stream
                            let playback =
                                AudioPlayer::new(stream_settings.sample_rate, None).unwrap();
                            playback.feed_from(&buffer);

                            while !shutdown_flag.load(Ordering::SeqCst) {
                                // notify frontend that audio is playing
                                window
                                    .emit("rtlsdr_status", "running")
                                    .expect("failed to emit event");

                                time::sleep(Duration::from_millis(250)).await;
                            }
                        })
                },
            ));
        }

        pub async fn stop_stream(&self, window: Window) {
            if let Ok(mut rtl_sdr_data) = self.0.clone().lock() {
                rtl_sdr_data.shutdown_flag.store(true, Ordering::SeqCst);

                if let Some(thread) = rtl_sdr_data.radio_stream_thread.take() {
                    thread.await.expect("Failed to join thread");
                }

                rtl_sdr_data.shutdown_flag.store(false, Ordering::SeqCst);

                window
                    .emit("rtlsdr_status", Some("stopped"))
                    .expect("failed to emit event");
            } else {
                println!("Could not acquire lock immediately");
                return;
            }
        }

        pub fn is_playing(&self) -> bool {
            return self.0.clone().lock().unwrap().radio_stream_thread.is_some();
        }
    }
}
