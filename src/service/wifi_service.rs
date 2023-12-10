use embedded_svc::wifi::ClientConfiguration;
use embedded_svc::wifi::Configuration;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_svc::wifi::WifiDeviceId;
use esp_idf_sys::EspError;
use log::error;
use log::warn;

use crate::config::config::WIFI_PASS;
use crate::config::config::WIFI_SSID;
pub fn reconnect_to_wifi_insistently_if_needed(wifi_driver: &mut EspWifi<'_>, one_shot: bool) {
    while wifi_driver.is_connected().is_err() || !wifi_driver.is_connected().unwrap() {
        warn!("reconnecting to WiFi...");
        if connect_to_wifi(wifi_driver, one_shot).is_err() {
            error!("failed to connect to the WiFi network");
        }
        if one_shot {
            break;
        }
        FreeRtos::delay_ms(100);
    }
}

pub fn connect_to_wifi(wifi_driver: &mut EspWifi<'_>, one_shot: bool) -> Result<(), EspError> {
    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.into(),
        password: WIFI_PASS.into(),
        ..Default::default()
    }))?;

    wifi_driver.start()?;
    wifi_driver.connect()?;
    let mut attempts = 0;
    while !wifi_driver.is_connected()? {
        let config = wifi_driver.get_configuration()?;
        warn!("Waiting for connection instauration {:?}", config);
        FreeRtos::delay_ms(100);
        attempts += 1;
        if one_shot && attempts > 5000 {
            break;
        }
        if attempts > 300000 {
            return Err(EspError::from(1).unwrap());
        }
    }
    Ok(())
}

pub fn get_mac_address(wifi: &mut EspWifi<'static>) -> String {
    let mav = wifi.driver().get_mac(WifiDeviceId::Sta).unwrap();
    let mac_address_obj = macaddr::MacAddr6::new(mav[0], mav[1], mav[2], mav[3], mav[4], mav[5]);
    let mac_address_value = mac_address_obj.to_string();
    mac_address_value
}
