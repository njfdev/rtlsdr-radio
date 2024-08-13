use std::time::Duration;

use crate::modes::*;
use radiorust::{
    flow::{new_receiver, new_sender, ReceiverConnector, SenderConnector},
    impl_block_trait,
    numbers::Float,
    prelude::{ChunkBufPool, Complex},
    signal::Signal,
};
use tauri::{async_runtime::block_on, AppHandle, Emitter};
use tokio::spawn;
use types::ModeSState;

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
    pub fn new(app: AppHandle, pass_along: bool) -> Self {
        let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();
        let (sender, sender_connector) = new_sender::<Signal<Complex<Flt>>>();

        let mut processing_buf_pool = ChunkBufPool::<u16>::new();
        let mut buf_pool = ChunkBufPool::<Complex<Flt>>::new();

        let mut modes_state = ModeSState::new();

        // used just for testing
        // #[cfg(debug_assertions)]
        // {
        //     let example_messages = [
        //       "1000110110101100010000101101111101011000101001010010001110110101001001001111110111111011100000000100111111011101",
        //       "1000110110101100010000101101111110011001000100001111011000110011101110000100000001001100010011011000101010110011",
        //       "1000110110101100010000101101111101011000101001010100011101001111000101010110110111000010011010111111100101100111",
        //       "0101110110101100010000101101111100001001001110101011100010110010110100101010101101000101111011111110111000001111",
        //       "1000110110101100010000101101111110011001000100001111011000110011101110000011110001001101010110000011000010100001",
        //       "0101110110101100001011001001110011100011110111001101000100001110101001011010111001000011000101010010010000011011",
        //       "1000110110100000010111110010000110011011000001101011011010101111000110001001010000000000110010111100001100111111", // Airborne velocity (air speed)
        //       "1000110110101100000011111111100111100001000010100011001000000000000000000000000000000000000101111101011101101000", // Aircraft Status
        //       "0101110110101100000011111111100100110010000000001011011101001001101101110010010111010001110100010111000010001101",
        //       "1000110110100000000001001110100011101010001000010100100001010101111011110101110000001000100010100010010001001001", // Target and Status Information
        //       "1000110101001000010000001101011000100000001011001100001101110001110000110010110011100000010101110110000010011000", // Aircraft Identification
        //       "1000110110101100000011111111100101011000101111110000011101100001100111010110001111001101110110111101001100001101", // Airborne position
        //       "1000110101000000011000100001110101011000110000111000011001000011010111001100010000010010011010010010101011010110", // Airborne position (CPR Odd)
        //       "1000110101000000011000100001110101011000110000111000001011010110100100001100100010101100001010000110001110100111", // Airborne position (CPR Even)
        //       "1000110101000000011000100001110101011000110000111000001011010110100100001100100010101100001010000110001110100111", // Airborne position (locally unambiguous)
        //       "1000110110100100001000111100010010011001000010011011111100011101000100000000100001011010100111000011110000101011", // Vertical Velocity
        //     ];

        //     for message in example_messages {
        //         let mut message_vec: Vec<u8> = Vec::new();
        //         let mut current_byte = 0u8;
        //         let mut bit_index = 0;

        //         for (i, c) in message.chars().enumerate() {
        //             if c == '1' {
        //                 current_byte |= 1 << (7 - bit_index);
        //             }

        //             bit_index += 1;

        //             if bit_index == 8 || i == message.len() - 1 {
        //                 message_vec.push(current_byte);
        //                 current_byte = 0;
        //                 bit_index = 0;
        //             }
        //         }

        //         let vec_to_string_vec: Vec<String> = message_vec
        //             .iter_mut()
        //             .map(|byte| format!("{:08b}", byte))
        //             .collect();
        //         println!("{}", vec_to_string_vec.join(""));
        //         decode_modes_msg(message_vec, &mut modes_state);
        //         app.emit("modes_state", modes_state.clone()).unwrap();
        //     }
        // }

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
                            let u8_value = (value * (65536.0)).round() as u16;

                            processing_chunk.push(u8_value);
                        }

                        detect_modes_signal(processing_chunk.to_vec(), &mut modes_state).await;

                        // filter out aircraft seen over 60 seconds ago
                        modes_state.aircraft = modes_state
                            .aircraft
                            .clone()
                            .into_iter()
                            .filter(|a| {
                                a.last_message_timestamp.elapsed().unwrap()
                                    < Duration::from_secs(60)
                            })
                            .collect();

                        // send update of data (whether new or not)
                        app.emit("modes_state", modes_state.clone()).unwrap();

                        if pass_along {
                            let mut output_chunk = buf_pool.get_with_capacity(input_chunk.len());

                            for value in processing_chunk.iter() {
                                output_chunk.push(Complex::from(
                                    Flt::from(((*value) as f32) / 65536.0).unwrap(),
                                ));
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
