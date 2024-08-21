use std::sync::{Arc, Mutex};

use biquad::{self, Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F64};

use radiorust::{
    flow::{new_receiver, new_sender, ReceiverConnector, SenderConnector},
    impl_block_trait,
    numbers::Float,
    prelude::{Chunk, ChunkBufPool, Complex},
    signal::Signal,
};
use tokio::spawn;

pub struct Pauseable<Flt> {
    receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
    sender_connector: SenderConnector<Signal<Complex<Flt>>>,
}

impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for Pauseable<Flt> }
impl_block_trait! { <Flt> Producer<Signal<Complex<Flt>>> for Pauseable<Flt> }

impl<Flt> Pauseable<Flt>
where
    Flt: Float + Into<f64>,
{
    pub fn new(is_paused: Arc<Mutex<bool>>) -> Self {
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

                        if *(is_paused.lock().unwrap()) {
                            for _ in 0..input_chunk.len() {
                                output_chunk.push(Complex::from(Flt::from_f64(0.0).unwrap()));
                            }
                        } else {
                            for sample in input_chunk.iter() {
                                output_chunk.push(*sample);
                            }
                        }

                        let Ok(()) = sender
                            .send(Signal::Samples {
                                sample_rate,
                                chunk: output_chunk.into(),
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
