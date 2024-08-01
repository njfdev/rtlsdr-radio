pub mod custom_radiorust_blocks {
    use std::{
        collections::VecDeque,
        f64::consts::PI,
        ops::{Range, RangeFrom},
    };

    use serde_json::json;

    use biquad::{self, Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F64};

    #[cfg(debug_assertions)]
    use hound::{WavSpec, WavWriter};
    #[cfg(debug_assertions)]
    use std::{
        fs,
        io::BufWriter,
        sync::{Arc, Mutex},
    };

    use nalgebra::{SMatrix, SVector};
    use radiorust::{
        flow::{new_receiver, new_sender, ReceiverConnector, SenderConnector},
        impl_block_trait,
        numbers::Float,
        prelude::{ChunkBuf, ChunkBufPool, Complex},
        signal::Signal,
    };
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
            }
        }
    }

    // constants for RDBS Decoding
    //const RBDS_CARRIER_FREQ: f64 = 57_000.0;
    //const RBDS_BANDWIDTH: f64 = 4_000.0;
    //const RBDS_CLOCK_FREQ: f64 = RBDS_CARRIER_FREQ / 48.0; // as defined in the RDS spec
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
        "Unassigned",
        "Unassigned",
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

    struct RbdsDecodeState {
        last_28_bits: VecDeque<u8>,
        are_blocks_synced: bool,
        last_block_offset_word: String,
        bits_since_last_block: u64,
        // stores the current group of blocks in the format of (block_data, block_type)
        current_block_group: Vec<(u32, String)>,
        rbds_state: RbdsState,
    }

    impl RbdsDecodeState {
        pub fn new() -> RbdsDecodeState {
            Self {
                last_28_bits: VecDeque::with_capacity(28),
                are_blocks_synced: false,
                last_block_offset_word: String::from(""),
                bits_since_last_block: 0,
                current_block_group: vec![],
                rbds_state: RbdsState::new(),
            }
        }
    }

    pub struct RbdsDecode<Flt> {
        receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
    }

    impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for RbdsDecode<Flt> }

    impl<Flt> RbdsDecode<Flt>
    where
        Flt: Float + Into<f64> + Into<f32>,
    {
        pub fn new(window: Window) -> Self {
            let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();

            // setup Wav file writer
            #[cfg(debug_assertions)]
            let wav_writer: Arc<Mutex<Option<WavWriter<BufWriter<fs::File>>>>> =
                Arc::new(Mutex::new(None));

            let mut last_sample_value: f64 = 0.0;

            let mut samples_moving_average: f64 = 0.0;

            let acceptable_timing_error: f64 = 0.75; // should be between 0.5 and 1, but closer to 1
            let mut is_clock_synced = false;
            let mut samples_since_last_clock: f64 = 0.0;
            let mut last_clock_value: f64 = 0.0;

            let desired_clock_freq = 57000.0 / 48.0;

            let mut buf_pool = ChunkBufPool::<f32>::new();

            let mut rbds_decode_state = RbdsDecodeState::new();

            let mut samples_since_crossing: u32 = 0;
            let mut last_digitized_bit: f64 = 0.0;

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
                            // use for saving to wav file
                            let mut smoothed_input_chunk =
                                buf_pool.get_with_capacity(input_chunk.len());
                            let mut bitstream_output_chunk =
                                buf_pool.get_with_capacity(input_chunk.len());
                            let mut decoded_output_chunk =
                                buf_pool.get_with_capacity(input_chunk.len());

                            // decoded bits from manchester encoded signal
                            let mut decoded_bits: Vec<u8> = vec![];

                            let desired_samples_length = sample_rate / desired_clock_freq;
                            // smoothing of input based on last ~0.1 milliseconds
                            let samples_smoothing_length = desired_samples_length / 8.0;
                            // filter out high frequency crossings (they should occur no faster than 1/8 of the clock frequency)
                            let crossing_smoothing_length = desired_samples_length / 8.0;
                            let buffer_time_between_clocks =
                                desired_samples_length * acceptable_timing_error;
                            // calculate the average clock rate by watch time between crossing 0
                            for sample in input_chunk.iter() {
                                samples_since_last_clock = samples_since_last_clock + 1.0;
                                samples_since_crossing = samples_since_crossing + 1;

                                let sample_value_raw_value = sample.re.to_f64().unwrap();

                                samples_moving_average = (samples_moving_average
                                    * ((samples_smoothing_length - 1.0)
                                        / samples_smoothing_length))
                                    + (sample_value_raw_value * (1.0 / samples_smoothing_length));

                                let sample_value = samples_moving_average;

                                smoothed_input_chunk.push(sample_value.clone() as f32);

                                let mut digitized_bit = 0.0;

                                if (samples_since_crossing as f64) < crossing_smoothing_length {
                                    digitized_bit = last_digitized_bit.clone();
                                } else {
                                    if sample_value.is_sign_positive() {
                                        digitized_bit = 1.0;
                                    } else if sample_value.is_sign_negative() {
                                        digitized_bit = -1.0;
                                    }
                                    last_digitized_bit = digitized_bit.clone();
                                }

                                bitstream_output_chunk.push(digitized_bit.clone() as f32);

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

                                            // make sure to process rbds data already received
                                            rbds_process_bits(
                                                &mut decoded_bits,
                                                &mut rbds_decode_state,
                                                window.clone(),
                                                true,
                                            );
                                            decoded_bits.clear();
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

                                            decoded_bits.push(decoded_bit);

                                            last_clock_value = digitized_bit.clone() as f64;
                                            samples_since_last_clock = 0.0;
                                        }
                                    }
                                }

                                last_sample_value = sample_value;
                            }

                            // process all bits recieved
                            rbds_process_bits(
                                &mut decoded_bits,
                                &mut rbds_decode_state,
                                window.clone(),
                                false,
                            );

                            // Step 4: Save to WAV file for Testing (if not in production)
                            #[cfg(debug_assertions)]
                            {
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
                            }

                            // unlike other blocks, this just "eats" the signal and does not pass it on
                        }
                        Signal::Event(_event) => {}
                    }
                }
            });
            Self { receiver_connector }
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

    fn generate_n_burst_mask(length: u8, burst_width: u8) -> Vec<u32> {
        let mut burst_error_mask: Vec<u32> = vec![];

        for i in (0)..(length - burst_width + 1) {
            let mask_width: u32 = ((1 << burst_width) - 1) as u32;
            let mask: u32 = mask_width << i as u32;
            burst_error_mask.push(mask);
        }

        burst_error_mask
    }

    fn attempt_error_correction(
        raw_data: u32,
        block_index: usize,
    ) -> Result<(u32, String, u16), ()> {
        let data = (raw_data >> 10) as u16;
        let received_crc = (raw_data & 0b11_1111_1111) as u16;

        let expected_crc = compute_crc(data);

        let block_index_starting_chars = ["A", "B", "C", "D"];
        // get valid block_offsets for block_index
        let valid_block_offsets: Vec<&(&str, u16, u16)> = RBDS_OFFSET_WORDS
            .iter()
            .filter(|(block_type, _, _)| {
                block_type.starts_with(block_index_starting_chars[block_index])
            })
            .collect();

        // try burst errors from 1-5
        for i in 1..=5 {
            let bit_mask = generate_n_burst_mask(26, i);
            for mask in bit_mask {
                for (offset_type, offset_bits, _offset_syndrome) in valid_block_offsets.clone() {
                    // flip bit specified in mask
                    let new_raw_data = (raw_data ^ mask);
                    // remove possible offset word
                    let corrected_data = new_raw_data ^ (*offset_bits as u32);
                    let new_data = (corrected_data >> 10) as u16;
                    let new_crc = (corrected_data & 0b11_1111_1111) as u16;

                    let new_expected_crc = compute_crc(new_data);
                    // if it matches, return the error corrected data
                    if new_expected_crc == new_crc {
                        return Ok((
                            new_raw_data,
                            (*offset_type).to_owned(),
                            (*offset_bits).clone(),
                        ));
                    }
                }
            }
        }

        Err(())
    }

    fn is_block_next(cur_offset_word: &str, last_block: &str, bits_since_last_block: &u64) -> bool {
        if (cur_offset_word == "A")
            || (((last_block == "A" && cur_offset_word == "B")
                || (last_block == "B" && cur_offset_word.starts_with("C"))
                || (last_block.starts_with("C") && cur_offset_word == "D"))
                && bits_since_last_block.clone() == 26)
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
        let syndrome = calculate_syndrome(bits);

        // find the block type with the smallest number of errors
        let possible_offset_words: Vec<&(&str, u16, u16)> = RBDS_OFFSET_WORDS
            .iter()
            .filter(|(_block_type, _offset_bits, expected_syndrome)| syndrome == *expected_syndrome)
            .collect();

        if possible_offset_words.len() == 0 {
            return Err(());
        }

        let (block_type, offset_bits, _syndrome) = possible_offset_words[0];

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

        let matrix_mul_result = data_vector.transpose() * rbds_parity_vector;

        let mut syndrome: u16 = 0b0;

        for (i, num) in matrix_mul_result.into_iter().rev().enumerate() {
            syndrome = syndrome + ((*num as u16 % 2) << i);
        }

        syndrome
    }

    fn send_rbds_data<T: serde::Serialize>(param_name: &str, data: T, window: Window) {
        let json_object: String = json!({
            param_name: data
        })
        .to_string();
        window
            .emit("rtlsdr_rbds", json_object.as_str())
            .expect("failed to emit event");
    }

    struct RbdsState {
        service_name: String,
        radio_text: String,
        radio_text_ab_flag: bool, // if switches from previous value, then clear radio_text
    }

    impl RbdsState {
        pub fn new() -> Self {
            Self {
                service_name: String::from(" ".repeat(8)),
                radio_text: String::from(" ".repeat(64)),
                radio_text_ab_flag: false,
            }
        }
    }

    fn process_rbds_group(
        group_data: Vec<(u32, String)>,
        rbds_state: &mut RbdsState,
        window: Window,
    ) {
        // group info
        let mut pi: u16 = 0; // program identification code
        let mut gtype: u8 = 0; // group type
        let mut b0: bool = false; // if true, block C repeats PIC, otherwise, block C is group specific data
        let mut tp: bool = false; // periodic traffic reports?
        let mut pty: usize = 0; // program type
        let mut g_data: u8 = 0; // group specific data (bits 4-0 of block B)

        // block data
        let mut block3_data: Option<u16> = None;
        let mut block4_data: u16 = 0;

        for (raw_data, block_type) in group_data.iter() {
            let data = (raw_data >> 10) as u16;

            match block_type.as_str() {
                "A" => {
                    pi = data;
                }
                "B" => {
                    gtype = ((data >> 12) & 0b1111) as u8;
                    b0 = if (data >> 11) & 0b1 == 1 { true } else { false };
                    tp = if (data >> 10) & 0b1 == 1 { true } else { false };
                    pty = ((data >> 5) & 0b11111) as usize;
                    g_data = (data & 0b11111) as u8;
                }
                "C" => {
                    if b0 {
                        pi = data
                    } else {
                        block3_data = Some(data);
                    }
                }
                "D" => {
                    block4_data = data;
                }
                _ => {
                    println!("Unexpected Block");
                    return;
                }
            }
        }

        // process blocks based on group type
        match gtype {
            // Program Service Name
            0b0000 => {
                // set the decoder control bit
                let decoder_control_bit_index = g_data & 0b11;
                let decoder_control_bit = if ((g_data >> 2) & 1) == 1 {
                    true
                } else {
                    false
                };
                let mut di_bit_name = "";

                match decoder_control_bit_index {
                    0 => di_bit_name = "di_is_stereo",
                    1 => di_bit_name = "di_is_binaural",
                    2 => di_bit_name = "di_is_compressed",
                    3 => di_bit_name = "di_is_pty_dynamic",
                    _ => {}
                }
                if di_bit_name.len() > 0 {
                    send_rbds_data(&di_bit_name, decoder_control_bit, window.clone());
                }

                // get and set the service_name characters
                let mut service_name_segment: String = String::from("");
                service_name_segment.push(((block4_data >> 8) & 0xff) as u8 as char);
                service_name_segment.push((block4_data & 0xff) as u8 as char);

                let char_starting_index = decoder_control_bit_index as usize * 2;
                let char_ending_index = char_starting_index + 2;

                // if indexes are not at char boundaries, assume error and reset string
                if !rbds_state
                    .service_name
                    .is_char_boundary(char_starting_index)
                    || !rbds_state.service_name.is_char_boundary(char_ending_index)
                {
                    rbds_state.service_name = String::from(" ".repeat(8));
                }

                rbds_state.service_name.replace_range(
                    Range {
                        start: char_starting_index,
                        end: char_ending_index,
                    },
                    &service_name_segment,
                );

                // get the music/speech flag (true = Music, false = speech)
                let ms_flag = if ((g_data >> 3) & 1) == 1 {
                    true
                } else {
                    false
                };

                // send rbds data to UI
                send_rbds_data(
                    "program_service_name",
                    rbds_state.service_name.clone(),
                    window.clone(),
                );
                send_rbds_data("ms_flag", ms_flag, window.clone());
            }
            // RadioText
            0b0010 => {
                let mut radio_text_segment = String::from("");

                if !b0 {
                    radio_text_segment.push(((block3_data.unwrap() >> 8) & 0xff) as u8 as char);
                    radio_text_segment.push((block3_data.unwrap() & 0xff) as u8 as char);
                }
                radio_text_segment.push(((block4_data >> 8) & 0xff) as u8 as char);
                radio_text_segment.push((block4_data & 0xff) as u8 as char);

                // if ab_flag changes, clear radio text
                let ab_flag = if ((g_data >> 4) & 1) == 1 {
                    true
                } else {
                    false
                };
                if ab_flag != rbds_state.radio_text_ab_flag {
                    rbds_state.radio_text.clear();
                    rbds_state.radio_text = String::from(" ".repeat(64));
                    rbds_state.radio_text_ab_flag = ab_flag;
                }

                let char_starting_index = (g_data & 0b1111) as usize * radio_text_segment.len();
                let char_ending_index = char_starting_index + radio_text_segment.len();

                rbds_state
                    .radio_text
                    .replace_range(char_starting_index..char_ending_index, &radio_text_segment);

                // send rbds data to UI
                send_rbds_data("radio_text", rbds_state.radio_text.clone(), window.clone());
            }
            _ => {}
        }

        // send rbds data to UI
        send_rbds_data(
            "program_type",
            RBDS_PTY_INDEX[pty].to_string(),
            window.clone(),
        );
    }

    fn rbds_process_bits(
        bit_stream: &mut Vec<u8>,
        rbds_decode_state: &mut RbdsDecodeState,
        window: Window,
        bit_stream_ending: bool,
    ) {
        for bit in bit_stream {
            rbds_decode_state.last_28_bits.push_back(*bit);
            rbds_decode_state.bits_since_last_block = rbds_decode_state.bits_since_last_block + 1;

            if rbds_decode_state.bits_since_last_block > 28 {
                rbds_decode_state.are_blocks_synced = false;
            }

            if rbds_decode_state.last_28_bits.len() >= 26 {
                let mut last_26_bits_u32 =
                    (bits_to_u32(rbds_decode_state.last_28_bits.clone().into()))
                        & 0b11_1111_1111_1111_1111_1111_1111;

                let mut offset_word_result = determine_offset_word(last_26_bits_u32);
                let mut is_error_corrected = false;

                // if offset_word_result is err and block sync is achieved, attempt error correction
                if offset_word_result.is_err() && rbds_decode_state.are_blocks_synced {
                    let new_data_result = attempt_error_correction(
                        last_26_bits_u32,
                        rbds_decode_state.current_block_group.len(),
                    );
                    if new_data_result.is_ok() {
                        let (new_data, new_offset_type, new_offset_bits) = new_data_result.unwrap();
                        last_26_bits_u32 = new_data;
                        offset_word_result = Ok((new_offset_type, new_offset_bits));
                        is_error_corrected = true;
                    }
                }

                // calculate and check CRC
                let data: u16 = (last_26_bits_u32 >> 10) as u16;
                let data_check_crc: u16 = (last_26_bits_u32 & 0b11_1111_1111) as u16;

                if offset_word_result.is_ok() {
                    let (offset_word, offset_bits) = offset_word_result.unwrap();
                    let received_crc = remove_offset_word(data_check_crc, offset_bits);
                    let computed_crc = compute_crc(data);

                    if (computed_crc == received_crc
                        && data != 0x0
                        && (rbds_decode_state.current_block_group.len() == 0
                            || offset_word != "A".to_owned())
                        && is_block_next(
                            &offset_word,
                            &rbds_decode_state.last_block_offset_word,
                            &rbds_decode_state.bits_since_last_block,
                        ))
                    {
                        if is_error_corrected {
                            println!("Error Corrected Block was Accepted!");
                        }
                        rbds_decode_state
                            .current_block_group
                            .push((last_26_bits_u32, offset_word.clone()));

                        // if 2 valid blocks in a row, then block synced has been achieved (as defined by RSD spec)
                        if rbds_decode_state.current_block_group.len() >= 2 {
                            rbds_decode_state.are_blocks_synced = true;
                        }

                        if rbds_decode_state.current_block_group.len() == 4 {
                            process_rbds_group(
                                rbds_decode_state.current_block_group.clone(),
                                &mut rbds_decode_state.rbds_state,
                                window.clone(),
                            );
                            rbds_decode_state.current_block_group.clear();
                        }

                        if rbds_decode_state.are_blocks_synced {
                            rbds_decode_state.last_28_bits.clear();
                        }
                    } else {
                        rbds_decode_state.are_blocks_synced = false;
                        rbds_decode_state.current_block_group.clear();
                    }

                    rbds_decode_state.last_block_offset_word = offset_word.clone();

                    rbds_decode_state.bits_since_last_block = 0;
                }
            }

            if rbds_decode_state.last_28_bits.len() > 28 {
                rbds_decode_state.last_28_bits.pop_front();
            }
        }

        // if bit stream is ending (in case of clock losing sync), then reset RBDS State
        if bit_stream_ending {
            *rbds_decode_state = RbdsDecodeState::new();
        }
    }
}
