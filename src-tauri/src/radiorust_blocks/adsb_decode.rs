use radiorust::{
    flow::{new_receiver, new_sender, ReceiverConnector, SenderConnector},
    impl_block_trait,
    numbers::Float,
    prelude::{ChunkBufPool, Complex},
    signal::Signal,
};
use tokio::spawn;

const MODES_LONG_MSG_BITS: usize = 112;
const MODES_SHORT_MSG_BITS: usize = 56;
const MODES_PREAMBLE_US: usize = 8; // preamble length in microseconds

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

        let mut processing_buf_pool = ChunkBufPool::<u16>::new();
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
                            let u8_value = (value * (65536.0)).round() as u16;

                            processing_chunk.push(u8_value);
                        }

                        detect_modes_signal(processing_chunk.to_vec());

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

fn detect_modes_signal(m: Vec<u16>) {
    /* Go through each sample, and see if it and the following 9 samples match the start of the Mode S preamble.
     *
     * The Mode S preamble is made of impulses with a width of 0.5 microseconds, and each sample is 0.5 microseconds
     * wide (as determined by the sample rate of 2MHz). This means each sample should be equal to 1 bit.
     *
     * This is what the start of the preamble (1010000101) should look like (taken from dump1090 comments):
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

    for i in 0..(m.len() - (MODES_PREAMBLE_US * 2 + MODES_LONG_MSG_BITS * 2)) {
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

        /* Now, check if the samples between the spikes are below the average of the spikes.
         * We don't want to test bits next to the spikes as they could be out of phase.
         *
         * The final bits of the preamble (10-15) are also low, so we need to check those as well,
         * but only the ones not next to high signals (11-14).
         */
        let avg_spike =
            (((m[i] as u32 + m[i + 2] as u32 + m[i + 7] as u32 + m[i + 9] as u32) as f64) / 4.0)
                .round() as u16;
        if m[i + 4] >= avg_spike
            || m[i + 5] >= avg_spike
            || m[i + 11] >= avg_spike
            || m[i + 12] >= avg_spike
            || m[i + 13] >= avg_spike
            || m[i + 14] >= avg_spike
        {
            continue;
        }

        let mut bits: Vec<u16> = vec![];

        // Decode the next 112 bits (regardless of message length/type)
        for j in (0..(MODES_LONG_MSG_BITS * 2)).step_by(2) {
            // get the start and end signal of the current cycle data (make sure to skip preamble)
            let start = m[i + j + MODES_PREAMBLE_US * 2];
            let end = m[i + j + MODES_PREAMBLE_US * 2 + 1];
            // the delta (difference) is use to calculate bit values and detect errors
            let mut delta = start as i32 - end as i32;
            if delta < 0 {
                delta = -delta;
            }

            if delta < 256 {
                // if change is small, it is probably equal to the last bit
                let last_value = bits.get(j);
                if last_value.is_none() {
                    bits.push(2);
                } else {
                    bits.push(last_value.unwrap().clone());
                }
            } else if start == end {
                // if 2 adjacent samples have the same magnitude, it is probably an error/noise
                bits.push(2);
            } else if start > end {
                bits.push(1)
            } else {
                bits.push(0);
            }
        }

        let mut msg: Vec<u8> = vec![];

        // pack bits into bytes
        for i in (0..MODES_LONG_MSG_BITS).step_by(8) {
            msg.push(
                (bits[i] << 7
                    | bits[i + 1] << 6
                    | bits[i + 2] << 5
                    | bits[i + 3] << 4
                    | bits[i + 4] << 3
                    | bits[i + 5] << 2
                    | bits[i + 6] << 1
                    | bits[i + 7]) as u8,
            )
        }

        // get the message type to determine the message length
        let msg_type = msg[0] >> 3;
        let msg_len = get_message_length(msg_type);

        // Verify that the high and low bits are different enough to consider this a signal and not noise
        let mut delta = 0;
        for j in (0..(msg_len * 2)).step_by(2) {
            delta += (m[i + j + MODES_PREAMBLE_US * 2] as i32
                - m[i + j + MODES_PREAMBLE_US * 2 + 1] as i32)
                .abs() as usize;
        }
        delta /= msg_len * 4;

        if delta < 10 * 255 {
            continue;
        }

        // If we reached this point and there are no errors, this is likely a valid Mode S message.
        if !(bits[0..msg_len].contains(&2)) {
            print!("Msg Data: ");
            for byte in msg {
                print!("{:08b}", byte);
            }
            println!();
        }
    }
}

fn get_message_length(msg_type: u8) -> usize {
    if msg_type == 0 || msg_type == 4 || msg_type == 5 || msg_type == 11 {
        return MODES_SHORT_MSG_BITS;
    }

    MODES_LONG_MSG_BITS
}
