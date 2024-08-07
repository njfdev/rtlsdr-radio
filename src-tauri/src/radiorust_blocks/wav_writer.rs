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
pub struct WavWriterBlock<Flt> {
    receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
    sender_connector: SenderConnector<Signal<Complex<Flt>>>,
}

impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for WavWriterBlock<Flt> }
impl_block_trait! { <Flt> Producer<Signal<Complex<Flt>>> for WavWriterBlock<Flt> }

impl<Flt> WavWriterBlock<Flt>
where
    Flt: Float + Into<f64>,
{
    pub fn new(filepath: String, pass_along: bool) -> Self {
        let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();
        let (sender, sender_connector) = new_sender::<Signal<Complex<Flt>>>();

        let wav_writer: Arc<Mutex<Option<WavWriter<BufWriter<fs::File>>>>> =
            Arc::new(Mutex::new(None));

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
                        if wav_writer.clone().lock().unwrap().is_none() {
                            let wav_spec = WavSpec {
                                channels: 1,
                                sample_rate: sample_rate as u32,
                                bits_per_sample: 32,
                                sample_format: hound::SampleFormat::Float,
                            };
                            *(wav_writer.lock().unwrap()) =
                                Some(WavWriter::create(filepath.clone(), wav_spec).unwrap());
                        }
                        for sample in input_chunk.iter() {
                            let value = WavWriterBlock::calc_magnitude(sample);
                            wav_writer
                                .lock()
                                .unwrap()
                                .as_mut()
                                .unwrap()
                                .write_sample(value)
                                .unwrap();
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
