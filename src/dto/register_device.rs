use serde::Serialize;

#[derive(Serialize)]
pub struct RegisterDeviceDTO {
    #[serde(rename = "macAddress")]
    mac_address: String,
    #[serde(rename = "type")]
    device_type: String,
    name: String,
    description: String,
}

impl RegisterDeviceDTO {
    pub fn new(
        mac_address: String,
        device_type: String,
        name: String,
        description: String,
    ) -> RegisterDeviceDTO {
        RegisterDeviceDTO {
            mac_address,
            device_type,
            name,
            description,
        }
    }
}
