use std::{
    ffi::{c_char, c_void, CStr},
    ptr,
    sync::{Arc, Mutex},
};

use crate::{
    modes::*,
    nrsc5::{
        bindings::{
            nrsc5_event_t, NRSC5_EVENT_AUDIO, NRSC5_EVENT_ID3, NRSC5_EVENT_LOST_SYNC,
            NRSC5_EVENT_LOT, NRSC5_EVENT_SYNC, NRSC5_SAMPLE_RATE_AUDIO,
        },
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

pub struct HdRadioDecode<Flt> {
    receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
    sender_connector: SenderConnector<Signal<Complex<Flt>>>,
}

static mut AUDIO_SAMPLES: Vec<i16> = vec![];

unsafe extern "C" fn nrsc5_custom_callback(event: *const nrsc5_event_t, opaque: *mut c_void) {
    if (*event).event == NRSC5_EVENT_ID3 && (*event).__bindgen_anon_1.id3.program == 0 {
        let title_ptr: *const c_char = (*event).__bindgen_anon_1.id3.title;
        if !title_ptr.is_null() {
            let title = CStr::from_ptr(title_ptr).to_string_lossy();
            println!("Title: {}", title);
        }
    } else if (*event).event == NRSC5_EVENT_AUDIO && (*event).__bindgen_anon_1.audio.program == 0 {
        let data_ptr = (*event).__bindgen_anon_1.audio.data;
        let data_len = (*event).__bindgen_anon_1.audio.count as usize;
        // Safety: We assume that the data pointer is valid and has the correct length.
        let audio_data = std::slice::from_raw_parts(data_ptr, data_len);

        // update AUDIO_SAMPLES
        AUDIO_SAMPLES.extend_from_slice(audio_data);
    } else if (*event).event == NRSC5_EVENT_LOT && (*event).__bindgen_anon_1.audio.program == 0 {
        println!(
            "-----------------Name: {}",
            CStr::from_ptr((*event).__bindgen_anon_1.lot.name)
                .to_str()
                .unwrap()
        );
    } else if (*event).event == NRSC5_EVENT_SYNC && (*event).__bindgen_anon_1.audio.program == 0 {
        println!("Synced to Station");
    } else if (*event).event == NRSC5_EVENT_LOST_SYNC
        && (*event).__bindgen_anon_1.audio.program == 0
    {
        println!("Lost Sync to Station");
    } else if (*event).event == NRSC5_EVENT_LOT && (*event).__bindgen_anon_1.audio.program == 0 {
        println!("Synced to Station");
    }
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

        let nrsc5_decoder = Nrsc5::new(Some(nrsc5_custom_callback), ptr::null_mut());

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

                        let mut new_audio_samples: Vec<i16> = vec![];

                        unsafe {
                            if AUDIO_SAMPLES.len() == 0 {
                                continue;
                            }

                            println!("Audio Samples length: {}", AUDIO_SAMPLES.len());

                            new_audio_samples = AUDIO_SAMPLES.clone();
                            AUDIO_SAMPLES.clear();
                        }

                        let mut output_chunk = buf_pool.get();

                        for sample in new_audio_samples.iter() {
                            output_chunk.push(Complex {
                                re: Flt::from((*sample) as f32 / i16::MAX as f32).unwrap(),
                                im: Flt::from(0.0).unwrap(),
                            });
                        }

                        let Ok(()) = sender
                            .send(Signal::Samples {
                                sample_rate: NRSC5_SAMPLE_RATE_AUDIO as f64,
                                chunk: output_chunk.finalize(),
                            })
                            .await
                        else {
                            println!("Receiver is no longer available");
                            return;
                        };
                        // if pass_along {
                        //     let Ok(()) = sender
                        //         .send(Signal::Samples {
                        //             sample_rate,
                        //             chunk: input_chunk,
                        //         })
                        //         .await
                        //     else {
                        //         return;
                        //     };
                        // }
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
