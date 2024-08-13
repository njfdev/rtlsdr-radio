use serde::Deserialize;

// these are all the expected types for type safety of the ADS-B DB API

pub const API_VERSION: u8 = 0;

#[derive(Deserialize, Debug)]
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
