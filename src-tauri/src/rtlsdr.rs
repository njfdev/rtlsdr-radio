pub mod rtlsdr {
    use std::{
        ops::DerefMut,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
        thread,
        time::Duration,
    };

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use radiorust::{
        blocks::{
            io::{audio::cpal::AudioPlayer, rf},
            modulation::FmDemod,
            FreqShifter,
        },
        prelude::*,
    };
    use soapysdr::Direction;
    use tauri::{async_runtime, Window};
    use tokio::{self, time};

    pub struct RtlSdrState(Arc<Mutex<RtlSdrData>>);
    pub struct RtlSdrData {
        pub radio_stream_thread: Option<async_runtime::JoinHandle<()>>,
        pub shutdown_flag: Arc<AtomicBool>,
    }

    #[derive(serde::Deserialize)]
    pub struct StreamSettings {
        fm_freq: f64,
        volume: f64,
        sample_rate: f64,
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

            let shutdown_flag = rtlsdr_state.lock().unwrap().shutdown_flag.clone();

            rtlsdr_state.lock().unwrap().radio_stream_thread =
                Some(async_runtime::spawn_blocking(move || {
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(async move {
                            // connect to SDR
                            let mut rtlsdr_dev = soapysdr::Device::new("driver=rtlsdr").unwrap();

                            // set sample rate
                            let sample_rate = 1.024e6;
                            rtlsdr_dev.set_sample_rate(Direction::Rx, 0, sample_rate);

                            // set center frequency
                            let sdr_freq = stream_settings.fm_freq * 1_000_000.0;
                            rtlsdr_dev
                                .set_frequency(Direction::Rx, 0, sdr_freq, "")
                                .expect("Failed to set frequency");

                            // set the bandwidth
                            rtlsdr_dev.set_bandwidth(Direction::Rx, 0, 1.024e6);

                            // start sdr rx stream
                            let rx_stream = rtlsdr_dev.rx_stream::<Complex<f32>>(&[0]).unwrap();
                            let sdr_rx = rf::soapysdr::SoapySdrRx::new(rx_stream, sample_rate);
                            sdr_rx.activate().await.unwrap();

                            // add frequency shifter
                            let freq_shifter = blocks::FreqShifter::<f32>::with_shift(0.0e6);
                            freq_shifter.feed_from(&sdr_rx);

                            // add downsampler
                            let downsample1 =
                                blocks::Downsampler::<f32>::new(16384, 384000.0, 200000.0);
                            downsample1.feed_from(&freq_shifter);

                            // add lowpass filter
                            let filter1 = blocks::Filter::new(|_, freq| {
                                if freq.abs() <= 100000.0 {
                                    Complex::from(1.0)
                                } else {
                                    Complex::from(0.0)
                                }
                            });
                            filter1.feed_from(&downsample1);

                            // demodulate fm signal
                            let demodulator = blocks::modulation::FmDemod::<f32>::new(150000.0);
                            demodulator.feed_from(&filter1);

                            // filter frequencies beyond normal human hearing range (20hz to 16 kHz)
                            let filter2 = blocks::filters::Filter::new_rectangular(|bin, freq| {
                                if bin.abs() >= 1 && freq.abs() >= 20.0 && freq.abs() <= 16000.0 {
                                    blocks::filters::deemphasis_factor(50e-6, freq)
                                } else {
                                    Complex::from(0.0)
                                }
                            });
                            filter2.feed_from(&demodulator);

                            // downsample so the output device can play the audio
                            let downsample2 = blocks::Downsampler::<f32>::new(
                                4096,
                                stream_settings.sample_rate,
                                2.0 * 20000.0,
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

                            // notify frontend that audio is playing
                            window
                                .emit("rtlsdr_status", "running")
                                .expect("failed to emit event");

                            while !shutdown_flag.load(Ordering::SeqCst) {
                                time::sleep(Duration::from_millis(100)).await;
                            }
                        })
                }));
        }

        pub async fn stop_stream(&self, window: Window) {
            if let Ok(mut rtlSdrData) = self.0.clone().lock() {
                rtlSdrData.shutdown_flag.store(true, Ordering::SeqCst);

                if let Some(thread) = rtlSdrData.radio_stream_thread.take() {
                    thread.await.expect("Failed to join thread");
                }

                rtlSdrData.shutdown_flag.store(false, Ordering::SeqCst);

                window
                    .emit("rtlsdr_status", Some("stopped"))
                    .expect("failed to emit event");
            } else {
                println!("Could not acquire lock immediately");
                return;
            }
        }
    }
}
