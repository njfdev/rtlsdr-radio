pub mod airborne_pos;
pub mod airborne_vel;
pub mod aircraft_ident;

use airborne_pos::*;
use airborne_vel::*;
use aircraft_ident::*;

use super::AdsbState;

pub fn decode_adsb_msg(me: &[u8], adsb_state: &mut AdsbState) {
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
            decode_aircraft_ident(me, adsb_state);
        }
        // Surface position
        5..=8 => {
            println!("Mode S msg Type: Surface position");
        }
        // Airborne position (barometric altitude)
        9..=18 => {
            println!("Mode S msg Type: Airborne position (barometric altitude)");
            decode_aircraft_pos(me, adsb_state);
        }
        // Airborne velocities
        19 => {
            println!("Mode S msg Type: Airborne velocity");
            decode_airborne_vel(me, adsb_state);
        }
        // Airborne position (GNSS height)
        20..=22 => {
            println!("Mode S msg Type: Airborne position (GNSS height)");
            decode_aircraft_pos(me, adsb_state);
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
