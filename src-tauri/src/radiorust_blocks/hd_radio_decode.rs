use std::{
    ffi::{c_char, c_void, CStr},
    sync::{Arc, Mutex},
};

use crate::{
    modes::*,
    nrsc5::{
        bindings::{nrsc5_event_t, NRSC5_EVENT_ID3},
        Nrsc5,
    },
};
use radiorust::{
    flow::{new_receiver, new_sender, ReceiverConnector, SenderConnector},
    impl_block_trait,
    numbers::Float,
    prelude::{ChunkBufPool, Complex},
    signal::Signal,
};
use tauri::ipc::Channel;
use tokio::spawn;

unsafe extern "C" fn nrsc5_custom_callback(event: *const nrsc5_event_t, opaque: *mut c_void) {
    if (*event).event == NRSC5_EVENT_ID3 && (*event).__bindgen_anon_1.id3.program == 0 {
        let title_ptr: *const c_char = (*event).__bindgen_anon_1.id3.title;
        if !title_ptr.is_null() {
            let title = CStr::from_ptr(title_ptr).to_string_lossy();
            println!("Title: {}", title);
        }
    }
}

pub struct HdRadioDecode<Flt> {
    receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
    sender_connector: SenderConnector<Signal<Complex<Flt>>>,
}

impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for HdRadioDecode<Flt> }
impl_block_trait! { <Flt> Producer<Signal<Complex<Flt>>> for HdRadioDecode<Flt> }

impl<Flt> HdRadioDecode<Flt>
where
    Flt: Float + Into<f64>,
{
    pub fn new(pass_along: bool) -> Self {
        let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();
        let (sender, sender_connector) = new_sender::<Signal<Complex<Flt>>>();

        let mut buf_pool = ChunkBufPool::<Complex<Flt>>::new();

        let nrsc5_decoder = Nrsc5::new(Some(nrsc5_custom_callback));

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
                        nrsc5_decoder.pipe_samples(
                            Self::convert_complex_to_iq_samples(
                                input_chunk.iter().map(|value| value.clone()),
                            )
                            .as_slice(),
                        );

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

    fn convert_complex_to_iq_samples<'a>(iter: impl Iterator<Item = Complex<Flt>>) -> Vec<i16> {
        let mut iq_samples = Vec::new();

        for sample in iter {
            let i = sample.re; // Real part (I)
            let q = sample.im; // Imaginary part (Q)

            // Convert to i16 and push to the output vector (assuming we want to keep the precision)
            iq_samples.push((i.to_f32().unwrap() * 32767.0) as i16);
            iq_samples.push((q.to_f32().unwrap() * 32767.0) as i16);
        }

        iq_samples
    }
}
