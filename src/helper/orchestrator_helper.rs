use chrono::{DateTime, FixedOffset};
use esp_idf_svc::wifi::EspWifi;

use super::date_helper::{
    calculate_next_scheduled_time, from_str_to_date_time_after, is_same_sec, is_same_time,
    is_same_time_sec,
};
use crate::config::config::{CHECK_INTERVAL_CONFIGURATION_CRON, DEFAULT_CONFIGURATION_URI};
use crate::helper::configuration_helper::get_default_configuration;
use crate::service::client_service::{get_configuration, register_device, send_i_am_alive};
use crate::service::clock_service::synchronize_clock_insistently_and_connect_wifi_if_necessary;
use crate::service::wifi_service::reconnect_to_wifi_insistently_if_needed;
use crate::ConfigurationResponse;
use log::{error, info, warn};

pub fn send_i_am_alive_if_necessary(
    i_am_alive_sent: &mut bool,
    second_number: u32,
    now: DateTime<FixedOffset>,
    mac_address: &String,
    configuration: &mut ConfigurationResponse,
    wifi_driver: &mut EspWifi<'static>,
) {
    if is_same_sec(second_number, now) && !*i_am_alive_sent {
        reconnect_to_wifi_insistently_if_needed(wifi_driver, true);
        let res = send_i_am_alive(mac_address, &configuration.i_am_alive_endpoint);
        if res.is_err() {
            error!("send i am alive ack failed");
        }
        warn!("i am alive sent :)");
        *i_am_alive_sent = true;
    }
    if !is_same_sec(second_number, now) && *i_am_alive_sent {
        *i_am_alive_sent = false;
    }
}

pub fn sync_system_clock_if_necessary(
    ntp_synchronized: &mut bool,
    ntp_sync_time: DateTime<FixedOffset>,
    now: DateTime<FixedOffset>,
    wifi_driver: &mut EspWifi<'static>,
) {
    if !*ntp_synchronized && is_same_time(ntp_sync_time, now) {
        synchronize_clock_insistently_and_connect_wifi_if_necessary(wifi_driver, true);
        *ntp_synchronized = true;
    } else if *ntp_synchronized && !is_same_time(ntp_sync_time, now) {
        *ntp_synchronized = false;
    }
}

pub fn try_register_device(mac_address: &String) {
    let register_device_result = register_device(mac_address);
    if register_device_result.is_err() {
        error!(
            "Failed to register the device: {:?}",
            register_device_result
        );
    } else {
        info!("Device registered successfully!");
    }
}

pub fn calculate_alarm_next_date_time(
    is_calculated_alarm_next_date_time: &mut bool,
    alarm: &mut DateTime<FixedOffset>,
    configuration: &ConfigurationResponse,
    offset: FixedOffset,
) {
    if !*is_calculated_alarm_next_date_time {
        *alarm = calculate_next_scheduled_time(&configuration.cron_list, &offset);
        *is_calculated_alarm_next_date_time = true;
    }
}

pub fn retrieve_config_if_necessary(
    downloaded: &mut bool,
    cron_time: &mut DateTime<FixedOffset>,
    offset: FixedOffset,
    now: DateTime<FixedOffset>,
    mac_address: &String,
    configuration: &mut ConfigurationResponse,
    wifi_driver: &mut EspWifi<'static>,
    is_calculated_alarm_next_date_time: &mut bool,
) {
    let old_cron_time = *cron_time;
    *cron_time = from_str_to_date_time_after(&now, CHECK_INTERVAL_CONFIGURATION_CRON, &offset);

    if is_same_time_sec(old_cron_time, now) && !*downloaded {
        reconnect_to_wifi_insistently_if_needed(wifi_driver, true);
        let configuration_result = get_configuration(DEFAULT_CONFIGURATION_URI, mac_address);
        warn!("configuration requested :)");
        *configuration = match configuration_result {
            Err(e) => get_default_configuration(e),
            Ok(data) => data,
        };

        *downloaded = true;
        *is_calculated_alarm_next_date_time = false;
    }
    if !is_same_time_sec(old_cron_time, now) && *downloaded {
        *downloaded = false;
    }
}

pub fn load_remote_configuration_or_default(
    mac_address: &String,
) -> crate::dto::config_response::Configuration {
    let configuration = match get_configuration(DEFAULT_CONFIGURATION_URI, mac_address) {
        Err(e) => get_default_configuration(e),
        Ok(data) => data,
    };
    configuration
}
