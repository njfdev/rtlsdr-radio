use serde::{Deserialize, Serialize};

// these are all the expected types for type safety of the ADS-B DB API

pub const API_VERSION: u8 = 0;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AircraftIcaoDetails {
    // the r# is used the escape the "type" reserved keyword in Rust
    pub r#type: String,
    pub icao_type: String,
    pub manufacturer: String,
    pub mode_s: String,
    pub registration: String,
    pub registered_owner_country_iso_name: String,
    pub registered_owner_country_name: String,
    pub registered_owner_operator_flag_code: Option<String>,
    pub registered_owner: String,
    pub url_photo: Option<String>,
    pub url_photo_thumbnail: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AircraftIcaoResponse {
    pub aircraft: AircraftIcaoDetails,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiIcaoLookupResponse {
    KnownAircraft { response: AircraftIcaoResponse },
    UnknownAircraft { response: String },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Airline {
    pub name: String,
    pub icao: String,
    pub iata: Option<String>,
    pub country: String,
    pub country_iso: String,
    pub callsign: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Airport {
    pub country_iso_name: String,
    pub country_name: String,
    pub elevation: u32,
    pub iata_code: String,
    pub icao_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub municipality: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlightRoute {
    pub callsign: String,
    pub callsign_icao: Option<String>,
    pub callsign_iata: Option<String>,
    pub airline: Option<Airline>,
    pub origin: Airport,
    pub destination: Airport,
}

#[derive(Deserialize, Debug)]
pub struct CallsignResponse {
    pub flightroute: FlightRoute,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiCallsignLookupResponse {
    KnownCallsign { response: CallsignResponse },
    UnknownCallsign { response: String },
}
