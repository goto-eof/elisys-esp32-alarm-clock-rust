use crate::ConfigurationResponse;
use crate::{
    config::config::{DEVICE_DESCRIPTION, DEVICE_NAME, DEVICE_TYPE, REGISTER_DEVICE_URL},
    dto::{
        config_request::ConfigRequest, register_device::RegisterDeviceDTO,
        request_i_am_alive::RequestIAmAlive,
    },
};
use anyhow::Error as StandardError;
use embedded_svc::{http::client::Client as HttpClient, io::Write, utils::io};
use esp_idf_svc::http::client::EspHttpConnection;
use log::{error, info};

pub fn register_device(mac_address: &str) -> anyhow::Result<(), anyhow::Error> {
    let client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

    let payload = serde_json::to_string(&RegisterDeviceDTO::new(
        mac_address.to_owned(),
        DEVICE_TYPE.into(),
        DEVICE_NAME.into(),
        DEVICE_DESCRIPTION.into(),
    ))
    .unwrap();
    let payload = payload.as_bytes();

    info!("trying to send data...");
    let result = post_request(payload, client, REGISTER_DEVICE_URL);
    info!("data sent? {}", !result.is_err());
    return match result {
        Err(e) => Err(e.into()),
        Ok(_) => Ok(()),
    };
}

pub fn send_i_am_alive(mac_address: &str, url: &str) -> anyhow::Result<(), anyhow::Error> {
    let client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
    let payload = serde_json::to_string(&RequestIAmAlive::new(mac_address.to_owned())).unwrap();
    let payload = payload.as_bytes();

    info!("trying to send is alive ack...");
    let result = post_request(payload, client, url);
    info!("ack sent? {}", !result.is_err());
    return match result {
        Err(e) => Err(e.into()),
        Ok(_) => Ok(()),
    };
}

pub fn get_configuration(
    configuration_uri: &str,
    mac_address: &str,
) -> anyhow::Result<ConfigurationResponse, anyhow::Error> {
    let client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
    let payload = serde_json::to_string(&ConfigRequest::new(mac_address.to_owned())).unwrap();
    let payload = payload.as_bytes();

    info!("[config downloader]: trying to get remote configuration...");
    let result = post_request(payload, client, configuration_uri);
    info!(
        "[config downloader]: configuration retrieved with success? {}",
        !result.is_err()
    );

    match result {
        Ok(body_string) => {
            let configuration: Result<ConfigurationResponse, serde_json::Error> =
                serde_json::from_str(&body_string);
            info!("{:?}", configuration);

            if configuration.is_err() {
                let err = configuration.err().unwrap();
                error!(
            "[config downloader]: error while trying to parse the configuration response: {}",
            &err
        );
                return Err(err.into());
            }

            let configuration = configuration.unwrap();
            info!(
                "[config downloader]: Remote configuration loaded successfully: {:?}",
                configuration
            );
            return Ok(configuration);
        }
        Err(e) => {
            error!("[config downloader]: Error decoding response body: {}", e);
            return Err(e.into());
        }
    }
}

fn post_request(
    payload: &[u8],
    mut client: HttpClient<EspHttpConnection>,
    url: &str,
) -> Result<String, StandardError> {
    let content_length_header = format!("{}", payload.len());
    let headers = [
        ("content-type", "application/json"),
        ("content-length", &*content_length_header),
    ];

    let request = client.post(url, &headers);

    if request.is_err() {
        let message = format!("connection error: {:?}", request.err());
        error!("{}", message);
        return Err(StandardError::msg(message));
    }
    let mut request = request.unwrap();

    if request.write_all(payload).is_err() {
        let message = format!("connection error while trying to write all");
        error!("{}", message);
        return Err(StandardError::msg(message));
    }
    if request.flush().is_err() {
        let message = format!("connection error while trying to flush");
        error!("{}", message);
        return Err(StandardError::msg(message));
    }
    info!("-> POST {}", url);
    let response = request.submit();
    if response.is_err() {
        let message = format!("connection error while trying to read response");
        error!("{}", message);
        return Err(StandardError::msg(message));
    }
    let mut response = response.unwrap();

    let status = response.status();
    info!("<- {}", status);
    let mut buf = [0u8; 4086];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0);

    if bytes_read.is_err() {
        let message = format!(
            "connection error while trying to read response: {:?}",
            bytes_read.err()
        );
        error!("{}", message);
        return Err(StandardError::msg(message));
    } else {
        let bytes_read = bytes_read.unwrap();
        return match std::str::from_utf8(&buf[0..bytes_read]) {
            Err(e) => Err(StandardError::msg(format!("{:?}", e))),
            Ok(str) => {
                info!("received message: {}", str);
                Ok(str.to_owned())
            }
        };
    }
}
