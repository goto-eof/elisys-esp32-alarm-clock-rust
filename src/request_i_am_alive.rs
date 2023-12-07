use serde::Serialize;

#[derive(Serialize)]
#[warn(non_snake_case)]
pub struct RequestIAmAlive {
    #[serde(rename = "macAddress")]
    mac_address: String,
}

impl RequestIAmAlive {
    pub fn new(mac_address: String) -> RequestIAmAlive {
        RequestIAmAlive { mac_address }
    }
}
