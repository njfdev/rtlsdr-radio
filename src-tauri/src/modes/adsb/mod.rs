pub mod airborne_vel;

use crate::modes::adsb::airborne_vel::decode_airborne_vel;

pub fn decode_adsb_msg(me: &[u8]) {
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
            decode_airborne_vel(me);
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
