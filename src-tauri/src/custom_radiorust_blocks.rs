pub mod custom_radiorust_blocks {
    use std::{
        f64::consts::PI,
        fmt::Debug,
        fs,
        io::BufWriter,
        sync::{Arc, Mutex},
    };

    use biquad::{
        Biquad, Coefficients, DirectForm1, DirectForm2Transposed, ToHertz, Type, Q_BUTTERWORTH_F32,
        Q_BUTTERWORTH_F64,
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

    // constants for RDBS Decoding
    const RDBS_CARRIER_FREQ: f64 = 57_000.0;
    const RDBS_CLOCK_FREQ: f64 = RDBS_CARRIER_FREQ / 48.0; // as defined in the RDS spec

    pub struct RbdsDecode<Flt> {
        receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
        window: Window,
    }

    impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for RbdsDecode<Flt> }

    impl<Flt> RbdsDecode<Flt>
    where
        Flt: Float + Into<f64>,
    {
        pub fn new(window: Window) -> Self {
            let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();

            // setup Wav file writer
            let mut wav_writer: Arc<Mutex<Option<WavWriter<BufWriter<fs::File>>>>> =
                Arc::new(Mutex::new(None));

            let mut bandpass_filter: Option<DirectForm2Transposed<f64>> = None;

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
                            // Step 1: Band-Pass Filter to "select" RBDS sub-carrier
                            if bandpass_filter.is_none() {
                                let filter_coefficients = Coefficients::<f64>::from_params(
                                    Type::BandPass,
                                    ToHertz::hz(sample_rate),
                                    ToHertz::hz(RDBS_CARRIER_FREQ),
                                    Q_BUTTERWORTH_F64,
                                );
                                bandpass_filter = Some(DirectForm2Transposed::<f64>::new(
                                    filter_coefficients.unwrap(),
                                ));
                            }
                            // NOTE: The input_chunk should already be FM demodulated, so the imaginary part of the Complex Number is 0 and can be ignored
                            let bandpassed_samples: Vec<f64> = input_chunk
                                .iter()
                                .map(|&sample| bandpass_filter.unwrap().run(sample.re.into()))
                                .collect();

                            // Step 2: Downmix to baseband
                            let downmixed_samples: Vec<f64> = bandpassed_samples
                                .iter()
                                .enumerate()
                                .map(|(i, &sample)| {
                                    let t = (i as f64 / sample_rate);
                                    sample * (2.0 * PI * RDBS_CARRIER_FREQ * t).cos()
                                })
                                .collect();

                            // Step 3: Apply Lowpass Filter to Remove High-Frequency Component (the 57KHz sub-carrier)
                            if lowpass_filter.is_none() {
                                let filter_coefficients = Coefficients::<f64>::from_params(
                                    Type::LowPass,
                                    ToHertz::hz(sample_rate),
                                    ToHertz::hz(RDBS_CARRIER_FREQ),
                                    Q_BUTTERWORTH_F64,
                                );
                                lowpass_filter =
                                    Some(DirectForm1::<f64>::new(filter_coefficients.unwrap()));
                            }
                            let lowpassed_samples: Vec<f64> = downmixed_samples
                                .iter()
                                .map(|sample| lowpass_filter.unwrap().run(*sample))
                                .collect();

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
                            for sample in lowpassed_samples {
                                wav_writer
                                    .lock()
                                    .unwrap()
                                    .as_mut()
                                    .unwrap()
                                    .write_sample(sample as f32)
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
