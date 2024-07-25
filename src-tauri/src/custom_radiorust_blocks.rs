pub mod custom_radiorust_blocks {
    use std::{
        f64::consts::PI,
        fmt::Debug,
        fs,
        io::BufWriter,
        sync::{Arc, Mutex},
    };

    use biquad::{
        self, Biquad, Coefficients, DirectForm1, DirectForm2Transposed, ToHertz, Type,
        Q_BUTTERWORTH_F32, Q_BUTTERWORTH_F64,
    };
    use fundsp::{
        hacker::{self, AudioNode, BiquadCoefs, BufferMut, BufferRef},
        F32x,
    };
    use hound::{WavSpec, WavWriter};
    use radiorust::{
        blocks,
        flow::{new_receiver, new_sender, Consumer, Message, ReceiverConnector, SenderConnector},
        impl_block_trait,
        numbers::Float,
        prelude::{ChunkBufPool, Complex},
        signal::Signal,
    };
    use rustfft::num_traits::ToPrimitive;
    use tauri::Window;
    use tokio::spawn;

    pub struct AmDemod<Flt> {
        receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
        sender_connector: SenderConnector<Signal<Complex<Flt>>>,
    }

    impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for AmDemod<Flt> }
    impl_block_trait! { <Flt> Producer<Signal<Complex<Flt>>> for AmDemod<Flt> }

    impl<Flt> AmDemod<Flt>
    where
        Flt: Float + Into<f64>,
    {
        pub fn new() -> Self {
            let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();
            let (sender, sender_connector) = new_sender::<Signal<Complex<Flt>>>();

            let mut buf_pool = ChunkBufPool::<Complex<Flt>>::new();

            let mut coeffs: Option<Coefficients<f64>> = None;
            let mut filter: Option<DirectForm1<f64>> = None;

            spawn(async move {
                loop {
                    let Ok(signal) = receiver.recv().await else {
                        return;
                    };
                    match signal {
                        Signal::Samples {
                            sample_rate,
                            chunk: input_chunk,
                        } => {
                            if coeffs.is_none() || filter.is_none() {
                                coeffs = Some(
                                    Coefficients::<f64>::from_params(
                                        Type::LowPass,
                                        ToHertz::hz(sample_rate),
                                        ToHertz::khz(15),
                                        Q_BUTTERWORTH_F64,
                                    )
                                    .unwrap(),
                                );

                                filter = Some(DirectForm1::<f64>::new(coeffs.unwrap()));
                            }
                            let mut output_chunk = buf_pool.get_with_capacity(input_chunk.len());

                            // get the magnitude for each sample
                            for &sample in input_chunk.iter() {
                                let magnitude = AmDemod::calc_magnitude(sample);

                                // center signal on 0
                                let centered_mag = magnitude - 1.0;

                                // run the lowpass filter
                                let filtered_magnitude = filter.unwrap().run(centered_mag);

                                output_chunk.push(Complex {
                                    re: Flt::from(filtered_magnitude).unwrap(),
                                    im: Flt::from(0.0).unwrap(),
                                });
                            }

                            let Ok(()) = sender
                                .send(Signal::Samples {
                                    sample_rate,
                                    chunk: output_chunk.finalize(),
                                })
                                .await
                            else {
                                return;
                            };
                        }
                        Signal::Event(event) => {
                            let Ok(()) = sender.send(Signal::Event(event)).await else {
                                return;
                            };
                        }
                    }
                }
            });
            Self {
                receiver_connector,
                sender_connector,
            }
        }

        fn calc_magnitude(c: Complex<Flt>) -> f64 {
            (c.re.powi(2) + c.im.powi(2)).sqrt().into()
        }
    }

    pub struct DownMixer<Flt> {
        receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
        sender_connector: SenderConnector<Signal<Complex<Flt>>>,
        freq: Flt,
    }

    impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for DownMixer<Flt> }
    impl_block_trait! { <Flt> Producer<Signal<Complex<Flt>>> for DownMixer<Flt> }

    impl<Flt> DownMixer<Flt>
    where
        Flt: Float,
    {
        pub fn new(freq: Flt) -> Self {
            let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();
            let (sender, sender_connector) = new_sender::<Signal<Complex<Flt>>>();

            let mut buf_pool = ChunkBufPool::<Complex<Flt>>::new();

            spawn(async move {
                loop {
                    let Ok(signal) = receiver.recv().await else {
                        return;
                    };
                    match signal {
                        Signal::Samples {
                            sample_rate,
                            chunk: input_chunk,
                        } => {
                            let mut output_chunk = buf_pool.get_with_capacity(input_chunk.len());

                            let mut i = 0;
                            // get the magnitude for each sample
                            for &sample in input_chunk.iter() {
                                let t = i as f64 / sample_rate;
                                let downmixed_value = sample.re.to_f64().unwrap()
                                    * (2.0 * PI * freq.to_f64().unwrap() * t).cos();

                                output_chunk.push(Complex {
                                    re: Flt::from(downmixed_value).unwrap(),
                                    im: Flt::from(0.0).unwrap(),
                                });
                                i += 1;
                            }

                            let Ok(()) = sender
                                .send(Signal::Samples {
                                    sample_rate,
                                    chunk: output_chunk.finalize(),
                                })
                                .await
                            else {
                                return;
                            };
                        }
                        Signal::Event(event) => {
                            let Ok(()) = sender.send(Signal::Event(event)).await else {
                                return;
                            };
                        }
                    }
                }
            });
            Self {
                receiver_connector,
                sender_connector,
                freq,
            }
        }
    }

    // constants for RDBS Decoding
    const RBDS_CARRIER_FREQ: f64 = 57_000.0;
    const RBDS_BANDWIDTH: f64 = 4_000.0;
    const RDBS_CLOCK_FREQ: f64 = RBDS_CARRIER_FREQ / 48.0; // as defined in the RDS spec

    pub struct RbdsDecode<Flt> {
        receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
        window: Window,
    }

    impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for RbdsDecode<Flt> }

    impl<Flt> RbdsDecode<Flt>
    where
        Flt: Float + Into<f64> + Into<f32>,
    {
        pub fn new(window: Window) -> Self {
            let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();

            // setup Wav file writer
            let mut wav_writer: Arc<Mutex<Option<WavWriter<BufWriter<fs::File>>>>> =
                Arc::new(Mutex::new(None));

            let mut bandpass_filter: Arc<Mutex<Option<hacker::Biquad<f64>>>> =
                Arc::new(Mutex::new(None));

            let mut lowpass_filter: Option<DirectForm1<f64>> = None;

            spawn(async move {
                loop {
                    let Ok(signal) = receiver.recv().await else {
                        return;
                    };
                    match signal {
                        Signal::Samples {
                            sample_rate,
                            chunk: input_chunk,
                        } => {
                            // Step 4: Save to WAV file for Testing
                            if wav_writer.clone().lock().unwrap().is_none() {
                                let wav_spec = WavSpec {
                                    channels: 1,
                                    sample_rate: sample_rate as u32,
                                    bits_per_sample: 32,
                                    sample_format: hound::SampleFormat::Float,
                                };
                                *(wav_writer.lock().unwrap()) =
                                    Some(WavWriter::create("rbds_output.wav", wav_spec).unwrap());
                            }
                            for sample in input_chunk.iter() {
                                wav_writer
                                    .lock()
                                    .unwrap()
                                    .as_mut()
                                    .unwrap()
                                    .write_sample(sample.re.to_f32().unwrap())
                                    .unwrap();
                            }

                            // unlike other blocks, this just "eats" the signal and does not pass it on
                        }
                        Signal::Event(event) => {}
                    }
                }
            });
            Self {
                receiver_connector,
                window,
            }
        }
    }
}
