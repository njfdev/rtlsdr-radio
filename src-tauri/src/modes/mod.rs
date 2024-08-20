pub mod adsb;
pub mod adsb_db;
pub mod arla;
pub mod crc;
pub mod types;

use std::time::SystemTime;

use adsb::decode_adsb_msg;
use adsb_db::get_icao_details;
use arla::get_aircraft_registration;
use crc::perform_modes_crc;
use types::*;

pub async fn detect_modes_signal(m: Vec<u16>, modes_state: &mut ModeSState) {
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
                decode_modes_msg(fixed_msg, modes_state).await;
            }
        }
    }
}

pub async fn decode_modes_msg(msg: Vec<u8>, modes_state: &mut ModeSState) {
    let msg_type = msg[0] >> 3;
    // TODO: process this and send to frontend
    let _ca = msg[0] & 0b111; // responder capabilities

    println!(
        "Existing Known ICAO Addresses: {:?}",
        modes_state
            .aircraft
            .clone()
            .into_iter()
            .map(|state| format!("{:#x}", state.icao_address))
            .collect::<Vec<String>>()
    );

    println!("\n-------------------------");

    println!("Downlink Format: {}", msg_type);

    // extended squitter (a.k.a. ADS-B)
    if msg_type == 17 {
        let icao_address = ((msg[1] as u32) << 16) | ((msg[2] as u32) << 8) | msg[3] as u32;
        println!("Decoded ICAO Address: {:#06x}", icao_address);

        // check if data from airplane with ICAO address has already been received, otherwise add entry
        let mut cur_aircraft = modes_state
            .aircraft
            .iter_mut()
            .find(|aircraft| aircraft.icao_address == icao_address);
        if cur_aircraft.is_none() {
            modes_state.aircraft.push(AircraftState::new(icao_address));
            let new_aircraft = modes_state.aircraft.last_mut().unwrap();

            // fetch ICAO information for this new aircraft from ADS-B DB
            let icao_data_result = get_icao_details(new_aircraft.icao_address.clone()).await;
            if icao_data_result.is_ok() {
                new_aircraft.icao_details = Some(icao_data_result.unwrap());
            }

            // fetch Registration information for this new aircraft from arla
            let registration_result =
                get_aircraft_registration(new_aircraft.icao_address.clone()).await;
            println!("Fetching Registration Result");
            if registration_result.is_ok() {
                new_aircraft.registration = Some(registration_result.unwrap());
                println!("{:#?}", new_aircraft.registration);
            } else {
                println!("{}", registration_result.unwrap_err());
            }

            cur_aircraft = Some(new_aircraft);
        } else {
            // if we already know the airplane, update the timestamp since last message
            cur_aircraft.as_mut().unwrap().last_message_timestamp = SystemTime::now();
        }

        // the ADS-B message is bytes 5-11 (4-10 as indexes)
        let me: &[u8] = &msg[4..=10];

        decode_adsb_msg(me, &mut cur_aircraft.unwrap()).await;
    }

    println!("-------------------------\n");
}

fn get_message_length(msg_type: u8) -> usize {
    if msg_type == 0 || msg_type == 4 || msg_type == 5 || msg_type == 11 {
        return MODES_SHORT_MSG_BITS;
    }

    MODES_LONG_MSG_BITS
}
