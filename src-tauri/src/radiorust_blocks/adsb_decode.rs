use std::f32::consts::PI;

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

const AIS_CHARSET: &str = "?ABCDEFGHIJKLMNOPQRSTUVWXYZ????? ???????????????0123456789??????";

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

        // used just for testing
        #[cfg(debug_assertions)]
        {
            let example_messages = [
              "1000110110101100010000101101111101011000101001010010001110110101001001001111110111111011100000000100111111011101",
              "1000110110101100010000101101111110011001000100001111011000110011101110000100000001001100010011011000101010110011",
              "1000110110101100010000101101111101011000101001010100011101001111000101010110110111000010011010111111100101100111",
              "0101110110101100010000101101111100001001001110101011100010110010110100101010101101000101111011111110111000001111",
              "1000110110101100010000101101111110011001000100001111011000110011101110000011110001001101010110000011000010100001",
              "0101110110101100001011001001110011100011110111001101000100001110101001011010111001000011000101010010010000011011",
              "1000110110100000010111110010000110011011000001101011011010101111000110001001010000000000110010111100001100111111", // Airborne velocity (air speed)
            ];

            for message in example_messages {
                let mut message_vec: Vec<u8> = Vec::new();
                let mut current_byte = 0u8;
                let mut bit_index = 0;

                for (i, c) in message.chars().enumerate() {
                    if c == '1' {
                        current_byte |= 1 << (7 - bit_index);
                    }

                    bit_index += 1;

                    if bit_index == 8 || i == message.len() - 1 {
                        message_vec.push(current_byte);
                        current_byte = 0;
                        bit_index = 0;
                    }
                }

                let vec_to_string_vec: Vec<String> = message_vec
                    .iter_mut()
                    .map(|byte| format!("{:08b}", byte))
                    .collect();
                println!("{}", vec_to_string_vec.join(""));
                decode_modes_msg(message_vec);
            }
        }

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

        // If we reached this point and there are no errors, this is likely a Mode S message.
        if !(bits[0..msg_len].contains(&2)) {
            let result = perform_modes_crc(msg);

            if result.is_ok() {
                let fixed_msg = result.unwrap();
                decode_modes_msg(fixed_msg);
            }
        }
    }
}

#[derive(PartialEq)]
enum AltitudeType {
    GNSS,
    Barometer,
}

#[derive(PartialEq)]
enum AirspeedType {
    IAS, // indicated airspeed
    TAS, // true airspeed
}

fn decode_modes_msg(msg: Vec<u8>) {
    let msg_type = msg[0] >> 3;
    let ca = msg[0] & 0b111; // responder capabilities

    println!("\n-------------------------");

    // extended squitter (a.k.a. ADS-B)
    if msg_type == 17 {
        let icao_address = ((msg[1] as u32) << 16) | ((msg[2] as u32) << 8) | msg[3] as u32;

        println!("Decoded ICAO Address: {:#06x}", icao_address);

        // the ADS-B message is bytes 5-11 (4-10 as indexes)
        let me: &[u8] = &msg[4..10];
        let type_code = me[0] >> 3;

        println!(
            "ADS-B bytes: {:?}",
            me.iter()
                .map(|byte| format!("{:08b}", byte))
                .collect::<Vec<String>>()
        );

        match type_code {
            // Aircraft identification
            1..=4 => {
                println!("Mode S msg Type: Aircraft identification");
            }
            // Surface position
            5..=8 => {
                println!("Mode S msg Type: Surface position");
            }
            // Airborne position (barometric altitude)
            9..=18 => {
                println!("Mode S msg Type: Airborne position (barometric altitude)");
            }
            // Airborne velocities
            19 => {
                println!("Mode S msg Type: Airborne velocity");
                let subtype = me[0] & 0b111;
                let intent_change_flag = if me[1] >> 7 == 1 { true } else { false };
                let ifr_capability = if (me[1] >> 6) & 1 == 1 { true } else { false };
                let subtype_specific_data = ((me[1] as u32 & 0b111) << 19)
                    | ((me[2] as u32) << 11)
                    | ((me[3] as u32) << 3)
                    | (me[4] as u32 >> 5);
                let vertical_rate_source = if (me[4] >> 3) & 1 == 1 {
                    AltitudeType::Barometer
                } else {
                    AltitudeType::GNSS
                };
                // 1 means down and 0 means up
                let vertical_rate_sign = if (me[4] >> 3) & 1 == 1 { -1 } else { 0 };
                let vertical_rate_raw = ((me[4] as u16 & 0b111) << 6) | (me[5] as u16 >> 2);

                if vertical_rate_raw != 0 {
                    let vertical_rate =
                        ((vertical_rate_raw as isize) - 1) * 64 * (vertical_rate_sign as isize);
                    println!("Vertical Velocity: {} ft/min", vertical_rate);
                } else {
                    println!("Vertical Velocity: N/A");
                }

                // subtype 1/3 -> subsonic, subtype 2/4 -> supersonic
                let speed_multiplier = if subtype % 2 == 0 { 4 } else { 1 };

                // decode subtype data
                match subtype {
                    // ground speed
                    1..=2 => {
                        // 0 -> West to East (1), 1 -> East to West (-1)
                        let ew_sign: i16 = if (subtype_specific_data >> 21) == 1 {
                            -1
                        } else {
                            1
                        };
                        let ew_velocity_raw = (subtype_specific_data >> 11) as u16 & 0b11_1111_1111;
                        let mut ew_velocity_abs: Option<u16> = None;
                        if ew_velocity_raw != 0 {
                            ew_velocity_abs = Some((ew_velocity_raw as u16 - 1) * speed_multiplier);
                        }

                        // 0 -> South to North (1), 1 -> North to South (-1)
                        let ns_sign: i16 = if ((subtype_specific_data >> 10) & 1) == 1 {
                            -1
                        } else {
                            1
                        };
                        let ns_velocity_raw = subtype_specific_data as u16 & 0b11_1111_1111;
                        let mut ns_velocity_abs: Option<u16> = None;
                        if ns_velocity_raw != 0 {
                            ns_velocity_abs = Some((ns_velocity_raw as u16 - 1) * speed_multiplier);
                        }

                        if ns_velocity_abs.is_some() && ew_velocity_abs.is_some() {
                            // calculate heading
                            let angle_rad = ((((ns_velocity_abs.unwrap() as i16 * -1) as f32)
                                .atan2((ew_velocity_abs.unwrap() as i16 * ew_sign) as f32))
                                / PI)
                                * 180.0;
                            println!("Heading (Relative to East): {:.2}°", angle_rad);
                        }

                        print!(
                            "Ground Speed ({}): ",
                            if subtype == 1 {
                                "subsonic"
                            } else {
                                "supersonic"
                            }
                        );
                        if ns_velocity_abs.is_some() && ew_velocity_abs.is_some() {
                            let real_speed = ((ns_velocity_abs.unwrap() as f32).powi(2)
                                + (ew_velocity_abs.unwrap() as f32).powi(2))
                            .sqrt();
                            println!("{} knots", real_speed);
                        } else {
                            println!()
                        }
                        if ns_velocity_abs.is_some() {
                            println!(
                                "  {} knots {}",
                                ns_velocity_abs.unwrap(),
                                if ns_sign > 0 { "North" } else { "South" }
                            );
                        }
                        if ew_velocity_abs.is_some() {
                            println!(
                                "  {} knots {}",
                                ew_velocity_abs.unwrap(),
                                if ew_sign > 0 { "East" } else { "West" }
                            );
                        }
                    }
                    // air speed
                    3..=4 => {
                        let is_magnetic_heading_included = if (subtype_specific_data >> 21) == 1 {
                            true
                        } else {
                            false
                        };
                        let magnetic_heading_raw =
                            (subtype_specific_data >> 11) as u16 & 0b11_1111_1111;
                        let mut magnetic_heading: Option<f32> = None;

                        if is_magnetic_heading_included {
                            magnetic_heading =
                                Some((magnetic_heading_raw as f32) * (360.0 / 1024.0));
                        }

                        let airspeed_type = if (subtype_specific_data >> 10) & 1 == 1 {
                            AirspeedType::TAS
                        } else {
                            AirspeedType::IAS
                        };
                        let airspeed_raw = subtype_specific_data as u16 & 0b11_1111_1111;
                        let mut airspeed: Option<u16> = None;

                        if airspeed_raw != 0 {
                            airspeed = Some((airspeed_raw - 1) * speed_multiplier);
                        }

                        println!(
                            "Magnetic Heading (Relative to North): {}",
                            if is_magnetic_heading_included {
                                format!("{:.2}°", magnetic_heading.unwrap())
                            } else {
                                "N/A".to_string()
                            }
                        );
                        println!(
                            "{} Airspeed: {}",
                            if airspeed_type == AirspeedType::TAS {
                                "True"
                            } else {
                                "Indicated"
                            },
                            if airspeed.is_some() {
                                format!("{} knots", airspeed.unwrap())
                            } else {
                                "N/A".to_string()
                            }
                        )
                    }
                    _ => {}
                }
            }
            // Airborne position (GNSS height)
            20..=22 => {
                println!("Mode S msg Type: Airborne position (GNSS height)");
            }
            // Reserved
            23..=27 => {
                println!("Mode S msg Type: Reserved");
            }
            // Aircraft status
            28 => {
                println!("Mode S msg Type: Aircraft status");
            }
            // Target state and status information
            29 => {
                println!("Mode S msg Type: Target state and status information");
            }
            // Aircraft operation status
            31 => {
                println!("Mode S msg Type: Aircraft operation status");
            }
            _ => {}
        }
    }

    println!("-------------------------\n");
}

fn perform_modes_crc(msg: Vec<u8>) -> Result<Vec<u8>, ()> {
    let msg_type = msg[0] >> 3;
    let msg_bits = get_message_length(msg_type);

    // the crc is always the last 3 bytes
    let received_crc = ((msg[(msg_bits / 8) - 3] as u32) << 16)
        | ((msg[(msg_bits / 8) - 2] as u32) << 8)
        | msg[(msg_bits / 8) - 1] as u32;
    let computed_crc = compute_modes_crc(msg.clone(), msg_bits);

    if received_crc != computed_crc {
        return Err(());
    }

    print!("Valid Mode S Message Demodulated: ");
    for byte in msg.clone() {
        print!("{:08b}", byte);
    }
    println!();

    Ok(msg)
}

// Precalculated values for ADS-B checksum for each bit of the 112 bits.
const MODES_CHECKSUM_TABLE: [u32; 112] = [
    0x3935ea, 0x1c9af5, 0xf1b77e, 0x78dbbf, 0xc397db, 0x9e31e9, 0xb0e2f0, 0x587178, 0x2c38bc,
    0x161c5e, 0x0b0e2f, 0xfa7d13, 0x82c48d, 0xbe9842, 0x5f4c21, 0xd05c14, 0x682e0a, 0x341705,
    0xe5f186, 0x72f8c3, 0xc68665, 0x9cb936, 0x4e5c9b, 0xd8d449, 0x939020, 0x49c810, 0x24e408,
    0x127204, 0x093902, 0x049c81, 0xfdb444, 0x7eda22, 0x3f6d11, 0xe04c8c, 0x702646, 0x381323,
    0xe3f395, 0x8e03ce, 0x4701e7, 0xdc7af7, 0x91c77f, 0xb719bb, 0xa476d9, 0xadc168, 0x56e0b4,
    0x2b705a, 0x15b82d, 0xf52612, 0x7a9309, 0xc2b380, 0x6159c0, 0x30ace0, 0x185670, 0x0c2b38,
    0x06159c, 0x030ace, 0x018567, 0xff38b7, 0x80665f, 0xbfc92b, 0xa01e91, 0xaff54c, 0x57faa6,
    0x2bfd53, 0xea04ad, 0x8af852, 0x457c29, 0xdd4410, 0x6ea208, 0x375104, 0x1ba882, 0x0dd441,
    0xf91024, 0x7c8812, 0x3e4409, 0xe0d800, 0x706c00, 0x383600, 0x1c1b00, 0x0e0d80, 0x0706c0,
    0x038360, 0x01c1b0, 0x00e0d8, 0x00706c, 0x003836, 0x001c1b, 0xfff409, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x000000,
];

fn compute_modes_crc(msg: Vec<u8>, msg_bits: usize) -> u32 {
    let mut crc: u32 = 0;
    let offset = if (msg_bits == MODES_LONG_MSG_BITS) {
        0
    } else {
        MODES_LONG_MSG_BITS - MODES_SHORT_MSG_BITS
    };

    for i in 0..msg_bits {
        let byte = i / 8;
        let bit = i % 8;
        let bitmask = 1 << (7 - bit);

        if msg[byte] & bitmask > 0 {
            crc ^= MODES_CHECKSUM_TABLE[i + offset];
        }
    }

    crc
}

fn get_message_length(msg_type: u8) -> usize {
    if msg_type == 0 || msg_type == 4 || msg_type == 5 || msg_type == 11 {
        return MODES_SHORT_MSG_BITS;
    }

    MODES_LONG_MSG_BITS
}
