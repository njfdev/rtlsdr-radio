use std::{
    fs,
    io::BufWriter,
    sync::{Arc, Mutex},
};

use biquad::{self, Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F64};

use hound::{WavSpec, WavWriter};
use radiorust::{
    flow::{new_receiver, new_sender, ReceiverConnector, SenderConnector},
    impl_block_trait,
    numbers::Float,
    prelude::{ChunkBufPool, Complex},
    signal::Signal,
};
use tokio::spawn;

/// A custom radiorust block that saves the input stream to a wav file at the specified path. You can enabled the pass_along argument to pass along samples, so it can be between blocks.
pub struct AdsbDecode<Flt> {
    receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
    sender_connector: SenderConnector<Signal<Complex<Flt>>>,
}

impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for AdsbDecode<Flt> }
impl_block_trait! { <Flt> Producer<Signal<Complex<Flt>>> for AdsbDecode<Flt> }

impl<Flt> AdsbDecode<Flt>
where
    Flt: Float + Into<f64>,
{
    pub fn new(pass_along: bool) -> Self {
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

                        for sample in input_chunk.iter() {
                            let abs_sample = Complex {
                                re: if sample.re.to_f32().unwrap() > 0.0 {
                                    sample.re
                                } else {
                                    -sample.re
                                },
                                im: if sample.im.to_f32().unwrap() > 0.0 {
                                    sample.im
                                } else {
                                    -sample.im
                                },
                            };
                            let value = AdsbDecode::calc_magnitude(&abs_sample);

                            output_chunk.push(Complex::from(Flt::from(value).unwrap()));
                        }

                        if pass_along {
                            let Ok(()) = sender
                                .send(Signal::Samples {
                                    sample_rate,
                                    chunk: input_chunk,
                                })
                                .await
                            else {
                                return;
                            };
                        }
                    }
                    Signal::Event(event) => {
                        if pass_along {
                            let Ok(()) = sender.send(Signal::Event(event)).await else {
                                return;
                            };
                        }
                    }
                }
            }
        });
        Self {
            receiver_connector,
            sender_connector,
        }
    }

    fn calc_magnitude(c: &Complex<Flt>) -> f32 {
        (c.re.powi(2) + c.im.powi(2)).sqrt().to_f32().unwrap()
    }
}
