use serde::Serialize;

#[derive(Serialize)]
#[warn(non_snake_case)]
pub struct ConfigRequest {
    #[serde(rename = "macAddress")]
    mac_address: String,
}

impl ConfigRequest {
    pub fn new(mac_address: String) -> ConfigRequest {
        ConfigRequest { mac_address }
    }
}
