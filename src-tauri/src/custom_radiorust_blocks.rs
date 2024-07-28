pub mod custom_radiorust_blocks {
    use std::{
        collections::VecDeque,
        f64::consts::PI,
        fmt::Debug,
        fs,
        io::BufWriter,
        sync::{Arc, Mutex},
    };

    use biquad::{
        self, Biquad, Coefficients, DirectForm1, DirectForm2Transposed, ToHertz, Type,
        Q_BUTTERWORTH_F32, Q_BUTTERWORTH_F64,
    };
    use fundsp::{
        hacker::{self, AudioNode, BiquadCoefs, BufferMut, BufferRef},
        typenum::int::Z0,
        F32x,
    };
    use hound::{WavSpec, WavWriter};
    use nalgebra::{SMatrix, SVector, Vector1, VectorN};
    use radiorust::{
        blocks,
        flow::{new_receiver, new_sender, Consumer, Message, ReceiverConnector, SenderConnector},
        impl_block_trait,
        numbers::Float,
        prelude::{ChunkBuf, ChunkBufPool, Complex},
        signal::Signal,
    };
    use rustfft::num_traits::{Signed, ToBytes, ToPrimitive};
    use tauri::Window;
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
                            if coeffs.is_none() || filter.is_none() {
                                coeffs = Some(
                                    Coefficients::<f64>::from_params(
                                        Type::LowPass,
                                        ToHertz::hz(sample_rate),
                                        ToHertz::khz(15),
                                        Q_BUTTERWORTH_F64,
                                    )
                                    .unwrap(),
                                );

                                filter = Some(DirectForm1::<f64>::new(coeffs.unwrap()));
                            }
                            let mut output_chunk = buf_pool.get_with_capacity(input_chunk.len());

                            // get the magnitude for each sample
                            for &sample in input_chunk.iter() {
                                let magnitude = AmDemod::calc_magnitude(sample);

                                // center signal on 0
                                let centered_mag = magnitude - 1.0;

                                // run the lowpass filter
                                let filtered_magnitude = filter.unwrap().run(centered_mag);

                                output_chunk.push(Complex {
                                    re: Flt::from(filtered_magnitude).unwrap(),
                                    im: Flt::from(0.0).unwrap(),
                                });
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

    pub struct DownMixer<Flt> {
        receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
        sender_connector: SenderConnector<Signal<Complex<Flt>>>,
        freq: Flt,
    }

    impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for DownMixer<Flt> }
    impl_block_trait! { <Flt> Producer<Signal<Complex<Flt>>> for DownMixer<Flt> }

    impl<Flt> DownMixer<Flt>
    where
        Flt: Float,
    {
        pub fn new(freq: Flt) -> Self {
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

                            let mut i = 0;
                            // get the magnitude for each sample
                            for &sample in input_chunk.iter() {
                                let t = i as f64 / sample_rate;
                                let downmixed_value = sample.re.to_f64().unwrap()
                                    * (2.0 * PI * freq.to_f64().unwrap() * t).cos();

                                output_chunk.push(Complex {
                                    re: Flt::from(downmixed_value).unwrap(),
                                    im: Flt::from(0.0).unwrap(),
                                });
                                i += 1;
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
                freq,
            }
        }
    }

    // constants for RDBS Decoding
    const RBDS_CARRIER_FREQ: f64 = 57_000.0;
    const RBDS_BANDWIDTH: f64 = 4_000.0;
    const RBDS_CLOCK_FREQ: f64 = RBDS_CARRIER_FREQ / 48.0; // as defined in the RDS spec
    const RBDS_CRC_POLYNOMIAL: u16 = 0b10110111001; // As defined by RDS spec: x^10 + x^8 + x^7 + x^5 + x^4 + x^3 + 1
    const RBDS_CRC_ALGO: crc::Algorithm<u16> = crc::Algorithm {
        width: 10,
        poly: RBDS_CRC_POLYNOMIAL,
        init: 0x0000,
        refin: false,
        refout: false,
        xorout: 0x0000,
        check: 0x0079,
        residue: 0x0000,
    };
    const RBDS_OFFSET_WORDS: [(&str, u16, u16); 6] = [
        // Block type, offset word bits, expected syndrome bits
        ("A", 0b0011111100, 0b1111011000),
        ("B", 0b0110011000, 0b1111010100),
        ("C", 0b0101101000, 0b1001011100),
        ("C'", 0b1101010000, 0b1111001100),
        ("D", 0b0110110100, 0b1001011000),
        ("E", 0b0000000000, 0b0000000000),
    ];
    const RBDS_PTY_INDEX: [&str; 32] = [
        "Undefined",
        "News",
        "Information",
        "Sports",
        "Talk",
        "Rock",
        "Classic Rock",
        "Adult Hits",
        "Soft Rock",
        "Top 40",
        "Country",
        "Oldies",
        "Soft Music",
        "Nostalgia",
        "Jazz",
        "Classical",
        "Rhythm and Blues",
        "Soft rhythm and Blues",
        "Language",
        "Religious Music",
        "Religious Talk",
        "Personality",
        "Public",
        "College",
        "Spanish Talk",
        "Spanish Music",
        "Hip Hop",
        "Undefined",
        "Undefined",
        "Weather",
        "Emergency Test",
        "Emergency",
    ];
    type Matrix26x10 = SMatrix<u8, 26, 10>;
    type Vector26 = SVector<u8, 26>;
    // 26x10 matrix row slice
    const RBDS_PARITY_CHECK_MATRIX: [u8; 260] = [
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 1, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 1, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 1, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 1, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 1, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 1, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, 1, //
        1, 0, 1, 1, 0, 1, 1, 1, 0, 0, //
        0, 1, 0, 1, 1, 0, 1, 1, 1, 0, //
        0, 0, 1, 0, 1, 1, 0, 1, 1, 1, //
        1, 0, 1, 0, 0, 0, 0, 1, 1, 1, //
        1, 1, 1, 0, 0, 1, 1, 1, 1, 1, //
        1, 1, 0, 0, 0, 1, 0, 0, 1, 1, //
        1, 1, 0, 1, 0, 1, 0, 1, 0, 1, //
        1, 1, 0, 1, 1, 1, 0, 1, 1, 0, //
        0, 1, 1, 0, 1, 1, 1, 0, 1, 1, //
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1, //
        1, 1, 1, 1, 0, 1, 1, 1, 0, 0, //
        0, 1, 1, 1, 1, 0, 1, 1, 1, 0, //
        0, 0, 1, 1, 1, 1, 0, 1, 1, 1, //
        1, 0, 1, 0, 1, 0, 0, 1, 1, 1, //
        1, 1, 1, 0, 0, 0, 1, 1, 1, 1, //
        1, 1, 0, 0, 0, 1, 1, 0, 1, 1, //
    ];

    pub struct RbdsDecode<Flt> {
        receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
        window: Window,
    }

    impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for RbdsDecode<Flt> }

    impl<Flt> RbdsDecode<Flt>
    where
        Flt: Float + Into<f64> + Into<f32>,
    {
        pub fn new(window: Window) -> Self {
            let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();

            // setup Wav file writer
            let mut wav_writer: Arc<Mutex<Option<WavWriter<BufWriter<fs::File>>>>> =
                Arc::new(Mutex::new(None));

            let mut last_sample_value: f64 = 0.0;

            let mut samples_moving_average: f64 = 0.0;

            let mut acceptable_timing_error: f64 = 0.65; // should be between 0.5 and 1, but closer to 1
            let mut is_clock_synced = false;
            let mut samples_since_last_clock: f64 = 0.0;
            let mut last_clock_value: f64 = 0.0;

            let desired_clock_freq = 57000.0 / 48.0;

            let mut buf_pool = ChunkBufPool::<f32>::new();

            let mut last_26_bits: VecDeque<u8> = VecDeque::with_capacity(26);

            let mut last_block_offset_word: String = "".to_owned();

            println!(
                "Calculated Syndrome: {:#010b}",
                calculate_syndrome(0b0000000000000000_0000000000)
            );

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
                            let mut smoothed_input_chunk =
                                buf_pool.get_with_capacity(input_chunk.len());
                            let mut bitstream_output_chunk =
                                buf_pool.get_with_capacity(input_chunk.len());
                            let mut decoded_output_chunk =
                                buf_pool.get_with_capacity(input_chunk.len());

                            let desired_samples_length = sample_rate / desired_clock_freq;
                            let samples_smoothing_length = desired_samples_length / 8.0;
                            let mut buffer_time_between_clocks =
                                desired_samples_length * acceptable_timing_error;
                            // calculate the average clock rate by watch time between crossing 0
                            for sample in input_chunk.iter() {
                                samples_since_last_clock = samples_since_last_clock + 1.0;

                                let sample_value_raw_value = sample.re.to_f64().unwrap();

                                samples_moving_average = (samples_moving_average
                                    * ((samples_smoothing_length - 1.0)
                                        / samples_smoothing_length))
                                    + (sample_value_raw_value * (1.0 / samples_smoothing_length));

                                let sample_value = samples_moving_average;

                                smoothed_input_chunk.push(sample_value.clone() as f32);

                                let mut digitized_bit = 0.0;

                                if sample_value.is_sign_positive() {
                                    digitized_bit = 1.0;
                                } else if sample_value.is_sign_negative() {
                                    digitized_bit = -1.0;
                                }

                                bitstream_output_chunk.push(digitized_bit.clone());

                                if is_crossing(last_sample_value, sample_value) {
                                    // If clock is not synced, go until a suitable change for clock is found
                                    if !is_clock_synced {
                                        add_n_to_buffer(
                                            &mut decoded_output_chunk,
                                            0.0,
                                            samples_since_last_clock,
                                        );

                                        if !(samples_since_last_clock < buffer_time_between_clocks)
                                            && !(samples_since_last_clock
                                                > (desired_samples_length * 2.0
                                                    - buffer_time_between_clocks))
                                        {
                                            // clock is within acceptable range
                                            is_clock_synced = true;
                                            println!("Clock is synced!");
                                        }
                                        samples_since_last_clock = 0.0;
                                    } else {
                                        // if clock is synced and clock is expected, run clock logic
                                        if samples_since_last_clock
                                            > (buffer_time_between_clocks * 2.0
                                                + desired_samples_length)
                                        {
                                            // if clock is way slower than expected, assume sync error and restart clock sync process
                                            is_clock_synced = false;
                                            add_n_to_buffer(
                                                &mut decoded_output_chunk,
                                                0.0,
                                                samples_since_last_clock,
                                            );
                                            samples_since_last_clock = 0.0;
                                            println!("\nLost clock sync!");
                                        } else if samples_since_last_clock
                                            > buffer_time_between_clocks
                                        {
                                            let decoded_bit;
                                            if last_clock_value == (digitized_bit as f64) {
                                                decoded_bit = 0;
                                                add_n_to_buffer(
                                                    &mut decoded_output_chunk,
                                                    -1.0,
                                                    samples_since_last_clock,
                                                );
                                            } else {
                                                decoded_bit = 1;
                                                add_n_to_buffer(
                                                    &mut decoded_output_chunk,
                                                    1.0,
                                                    samples_since_last_clock,
                                                );
                                            }

                                            last_26_bits.push_front(decoded_bit);

                                            if last_26_bits.len() > 26 {
                                                last_26_bits.pop_back();

                                                let last_26_bits_u32 =
                                                    bits_to_u32(last_26_bits.clone().into());

                                                // calculate and check CRC
                                                let data: u16 = (last_26_bits_u32 >> 10) as u16;
                                                let data_check_crc: u16 =
                                                    (last_26_bits_u32 & 0b11_1111_1111) as u16;

                                                let offset_word_result =
                                                    determine_offset_word(last_26_bits_u32);

                                                if offset_word_result.is_ok() {
                                                    let (offset_word, offset_bits) =
                                                        offset_word_result.unwrap();
                                                    let received_crc = remove_offset_word(
                                                        data_check_crc,
                                                        offset_bits,
                                                    );
                                                    let computed_crc = compute_crc(data);

                                                    if computed_crc == received_crc
                                                        && data != 0x0
                                                        && is_block_next(
                                                            &offset_word,
                                                            &last_block_offset_word,
                                                        )
                                                    {
                                                        last_block_offset_word =
                                                            offset_word.clone();
                                                        /*
                                                        println!(
                                                            "Actual: {}, Computed: {}, Offset Word: {}, Data: {:#b}, Syndrome: {}",
                                                            received_crc,
                                                            computed_crc,
                                                            offset_word,
                                                            last_26_bits_u32,
                                                            calculate_syndrome(last_26_bits_u32)
                                                        ); */

                                                        if offset_word == "B".to_owned() {
                                                            let pty: usize =
                                                                ((data >> 5) & 0b11111) as usize;
                                                            println!(
                                                                "Program Type: {}",
                                                                RBDS_PTY_INDEX[pty]
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                            // 01101
                                            last_clock_value = digitized_bit.clone() as f64;
                                            samples_since_last_clock = 0.0;
                                        }
                                    }
                                }

                                last_sample_value = sample_value;
                            }

                            // Step 4: Save to WAV file for Testing
                            if wav_writer.clone().lock().unwrap().is_none() {
                                let wav_spec = WavSpec {
                                    channels: 4,
                                    sample_rate: sample_rate as u32,
                                    bits_per_sample: 32,
                                    sample_format: hound::SampleFormat::Float,
                                };
                                *(wav_writer.lock().unwrap()) = Some(
                                    WavWriter::create("../rbds_output.wav", wav_spec).unwrap(),
                                );
                            }
                            for (i, sample) in input_chunk.iter().enumerate() {
                                wav_writer
                                    .lock()
                                    .unwrap()
                                    .as_mut()
                                    .unwrap()
                                    .write_sample(sample.re.to_f32().unwrap())
                                    .unwrap();
                                wav_writer
                                    .lock()
                                    .unwrap()
                                    .as_mut()
                                    .unwrap()
                                    .write_sample(smoothed_input_chunk[i])
                                    .unwrap();
                                wav_writer
                                    .lock()
                                    .unwrap()
                                    .as_mut()
                                    .unwrap()
                                    .write_sample(bitstream_output_chunk[i])
                                    .unwrap();
                                let decoded_bit_result = decoded_output_chunk.get(i);
                                let mut decoded_bit: f32 = 0.0;
                                if decoded_bit_result.is_some() {
                                    decoded_bit = *decoded_bit_result.unwrap();
                                }
                                wav_writer
                                    .lock()
                                    .unwrap()
                                    .as_mut()
                                    .unwrap()
                                    .write_sample(decoded_bit)
                                    .unwrap();
                            }

                            // unlike other blocks, this just "eats" the signal and does not pass it on
                        }
                        Signal::Event(event) => {}
                    }
                }
            });
            Self {
                receiver_connector,
                window,
            }
        }
    }

    fn is_crossing(last: f64, new: f64) -> bool {
        !((last.is_sign_positive() && new.is_sign_positive())
            || (last.is_sign_negative() && new.is_sign_negative()))
    }

    fn add_n_to_buffer(buffer: &mut ChunkBuf<f32>, value: f32, length: f64) {
        let mut new_data = vec![value; length as usize];
        buffer.append(&mut new_data);
    }

    fn compute_crc(data: u16) -> u16 {
        let crc = crc::Crc::<u16>::new(&RBDS_CRC_ALGO);
        let mut digest = crc.digest();
        digest.update(&data.to_be_bytes());
        digest.finalize()
    }

    fn is_block_next(cur_offset_word: &str, last_block: &str) -> bool {
        if (cur_offset_word == "A")
            || (last_block == "A" && cur_offset_word == "B")
            || (last_block == "B" && cur_offset_word.starts_with("C"))
            || (last_block.starts_with("C") && cur_offset_word == "D")
        {
            return true;
        }

        false
    }

    fn remove_offset_word(recieved_checkword: u16, offset_word: u16) -> u16 {
        recieved_checkword ^ offset_word
    }

    fn bits_to_u32(bits: Vec<u8>) -> u32 {
        let mut value: u32 = 0;
        for bit in bits.iter().take(32) {
            value = (value << 1) | (*bit as u32);
        }
        value
    }

    fn determine_offset_word(bits: u32) -> Result<(String, u16), ()> {
        let data = (bits >> 10) as u16;
        let checkword = (bits & 0b11_1111_1111) as u16;

        let syndrome = calculate_syndrome(bits);

        // find the block type with the smallest number of errors
        let possible_offset_words: Vec<&(&str, u16, u16)> = RBDS_OFFSET_WORDS
            .iter()
            .filter(|(block_type, offset_bits, expected_syndrome)| syndrome == *expected_syndrome)
            .collect();

        if possible_offset_words.len() == 0 {
            return Err(());
        }

        let (block_type, offset_bits, syndrome) = possible_offset_words[0];

        Ok(((block_type.to_owned()).to_owned(), *offset_bits))
    }

    fn u32_to_bits(n: u32, length: usize) -> Vec<u8> {
        let mut bits = Vec::with_capacity(length);
        for i in (0..length).rev() {
            bits.push(((n >> i) & 1) as u8);
        }
        bits
    }

    fn calculate_syndrome(recieved_data: u32) -> u16 {
        let rbds_parity_vector = Matrix26x10::from_row_slice(&RBDS_PARITY_CHECK_MATRIX);
        let data_vector = Vector26::from_row_slice(u32_to_bits(recieved_data, 26).as_mut_slice());

        let matrix_mul_result = (data_vector.transpose() * rbds_parity_vector);

        let mut syndrome: u16 = 0b0;

        for (i, num) in matrix_mul_result.into_iter().rev().enumerate() {
            syndrome = syndrome + ((*num as u16 % 2) << i);
        }

        syndrome
    }
}
