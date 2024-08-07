use std::{
    collections::btree_map::Range,
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

        let mut processing_buf_pool = ChunkBufPool::<f32>::new();
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
                        let mut processing_chunk =
                            processing_buf_pool.get_with_capacity(input_chunk.len());

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

                            processing_chunk.push(value);
                        }

                        detect_modes_signal(processing_chunk.to_vec());

                        if pass_along {
                            let mut output_chunk = buf_pool.get_with_capacity(input_chunk.len());

                            for value in processing_chunk.iter() {
                                output_chunk.push(Complex::from(Flt::from(*value).unwrap()));
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

fn detect_modes_signal(m: Vec<f32>) {
    /* Go through each sample, and see if it and the following 9 samples match the Mode S preamble.
     *
     * The Mode S preamble is made of impulses with a width of 0.5 microseconds, and each sample is 0.5 microseconds
     * wide (as determined by the sample rate of 2MHz). This means each sample should be equal to 1 bit.
     *
     * This is what the preamble (1010000101) should look like (taken from dump1090 comments):
     * 0   -----------------
     * 1   -
     * 2   ------------------
     * 3   --
     * 4   -
     * 5   --
     * 6   -
     * 7   ------------------
     * 8   --
     * 9   -------------------
     */

    for i in 0..(m.len() - 9) {
        // First, check if the relations between samples matches. We can skip it if it doesn't.
        if !(m[i] > m[i+1] &&  // 1
            m[i+1] < m[i+2] && // 0
            m[i+2] > m[i+3] && // 1
            m[i+3] < m[i] &&   // 0
            m[i+4] < m[i] &&   // 0
            m[i+5] < m[i] &&   // 0
            m[i+6] < m[i] &&   // 0
            m[i+7] > m[i+8] && // 1
            m[i+8] < m[i+9] && // 0
            m[i+9] > m[i+6]/* 1 */)
        {
            continue;
        }
        println!("Found possible preamble!");
    }
}
