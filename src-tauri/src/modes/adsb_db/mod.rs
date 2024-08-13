use types::*;

pub mod types;

pub async fn get_icao_details(icao_address: u32) -> Result<AircraftIcaoDetails, String> {
    let api_url = format!(
        "https://api.adsbdb.com/v{}/aircraft/{:06x}",
        API_VERSION, icao_address
    );
    let response = reqwest::get(&api_url).await;

    if response.is_err() {
        return Err(response.unwrap_err().to_string());
    }

    let json_data = response.unwrap().json::<ApiIcaoLookupResponse>().await;

    if json_data.is_err() {
        return Err(json_data.unwrap_err().to_string());
    }

    match json_data.unwrap() {
        ApiIcaoLookupResponse::KnownAircraft { response } => return Ok(response.aircraft),
        ApiIcaoLookupResponse::UnknownAircraft { response } => return Err(response),
    }
}
