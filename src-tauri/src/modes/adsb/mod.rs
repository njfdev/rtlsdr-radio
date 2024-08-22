pub mod airborne_pos;
pub mod airborne_vel;
pub mod aircraft_ident;

use airborne_pos::*;
use airborne_vel::*;
use aircraft_ident::*;
use log::debug;

use super::{AdsbState, AircraftState};

pub async fn decode_adsb_msg(me: &[u8], aircraft: &mut AircraftState) {
    let type_code = me[0] >> 3;

    debug!(
        "ADS-B bytes: {:?}",
        me.iter()
            .map(|byte| format!("{:08b}", byte))
            .collect::<Vec<String>>()
    );

    match type_code {
        // Aircraft identification
        1..=4 => {
            debug!("Mode S msg Type: Aircraft identification");
            decode_aircraft_ident(me, aircraft).await;
        }
        // Surface position
        5..=8 => {
            debug!("Mode S msg Type: Surface position");
        }
        // Airborne position (barometric altitude)
        9..=18 => {
            debug!("Mode S msg Type: Airborne position (barometric altitude)");
            decode_aircraft_pos(me, &mut aircraft.adsb_state);
        }
        // Airborne velocities
        19 => {
            debug!("Mode S msg Type: Airborne velocity");
            decode_airborne_vel(me, &mut aircraft.adsb_state);
        }
        // Airborne position (GNSS height)
        20..=22 => {
            debug!("Mode S msg Type: Airborne position (GNSS height)");
            decode_aircraft_pos(me, &mut aircraft.adsb_state);
        }
        // Reserved
        23..=27 => {
            debug!("Mode S msg Type: Reserved");
        }
        // Aircraft status
        28 => {
            debug!("Mode S msg Type: Aircraft status");
        }
        // Target state and status information
        29 => {
            debug!("Mode S msg Type: Target state and status information");
        }
        // Aircraft operation status
        31 => {
            debug!("Mode S msg Type: Aircraft operation status");
        }
        _ => {}
    }
}
