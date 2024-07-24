pub mod custom_radiorust_blocks {
    use biquad::{coefficients, Biquad, Coefficients, DirectForm1, ToHertz};
    use radiorust::{
        flow::{new_receiver, new_sender, Message, ReceiverConnector, SenderConnector},
        impl_block_trait,
        numbers::Float,
        prelude::{ChunkBufPool, Complex},
        signal::Signal,
    };
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
                            // if a lowpass filter isn't already created, create one with sample_rate info
                            if coeffs.is_none() {
                                // create a lowpass filter with biquad
                                // no AM broadcast should have frequencies higher than 15khz
                                let cutoff_hz = 15_000.0;
                                coeffs = Some(
                                    Coefficients::<f64>::from_params(
                                        biquad::Type::LowPass,
                                        (sample_rate as f64).hz(),
                                        (cutoff_hz as f64).hz(),
                                        0.707,
                                    )
                                    .unwrap(),
                                );
                                filter = Some(DirectForm1::<f64>::new(coeffs.unwrap()));
                                println!("Filter coefficients set: {:?}", coeffs);
                            }

                            let mut output_chunk = buf_pool.get_with_capacity(input_chunk.len());

                            // get the magnitude for each sample
                            for &sample in input_chunk.iter() {
                                let magnitude = AmDemod::calc_magnitude(sample);

                                // apply the lowpass filter (easy way to demodulate am)
                                let filtered_magnitude = filter.unwrap().run(magnitude.into());
                                output_chunk.push(Complex {
                                    re: Flt::from(filtered_magnitude).unwrap(),
                                    im: Flt::from(0.0).unwrap(),
                                });
                            }

                            // Print some samples for debugging
                            for i in 0..10 {
                                //println!("Output sample {}: {:?}", i, output_chunk[i]);
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
}
