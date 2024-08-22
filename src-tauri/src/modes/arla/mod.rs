use types::*;

pub mod types;

pub async fn get_aircraft_registration(icao_address: u32) -> Result<RegistrationObject, String> {
    let api_url = format!(
        "https://arla.njf.dev/api/v{}/faa/registration/{:06x}",
        API_VERSION, icao_address
    );
    let response = reqwest::get(&api_url).await;

    if response.is_err() {
        return Err(response.unwrap_err().to_string());
    }

    // debug!("Raw data: {:?}", response.unwrap().text().await);
    // return Err(String::from(""));

    let json_data = response
        .unwrap()
        .json::<ApiAircraftRegistrationLookup>()
        .await;

    if json_data.is_err() {
        return Err(json_data.unwrap_err().to_string());
    }

    match json_data.unwrap() {
        ApiAircraftRegistrationLookup::KnownRegistration { registration } => {
            return Ok(registration)
        }
        ApiAircraftRegistrationLookup::LookupError { error: _, message } => return Err(message),
    }
}
