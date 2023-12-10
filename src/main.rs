use anyhow::Error as StandardError;
use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike, Utc};

use config::config::{DEFAULT_ALARM_INTERVAL_MINUTES, DEFAULT_CRONTAB, DEFAULT_TIMEZONE};
use cron::Schedule;
use embedded_svc::{http::client::Client as HttpClient, io::Write, utils::io};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sntp;
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, EspWifi, WifiDeviceId};
use esp_idf_svc::{
    hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals},
    sntp::SyncStatus,
};
use std::str::FromStr;
mod dto;
mod request_i_am_alive;

use crate::dto::config_cron_list_response::CronListResponse;
use crate::dto::register_device::RegisterDeviceDTO;
use crate::request_i_am_alive::RequestIAmAlive;
use config::config::{
    CHECK_INTERVAL_CONFIGURATION_CRON, DEFAULT_CONFIGURATION_URI, DEFAULT_I_AM_ALIVE_ENDPOINT,
    DEFAULT_I_AM_ALIVE_INTERVAL_SECONDS, DEVICE_DESCRIPTION, DEVICE_NAME, DEVICE_TYPE,
    ENABLE_I_AM_ALIVE_ACK, REGISTER_DEVICE_URL, WIFI_PASS, WIFI_SSID,
};
use dto::config_request::ConfigRequest;
use dto::config_response::Configuration as ConfigurationResponse;
use esp_idf_sys::EspError;
use log::{error, info, warn};
mod config;
fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();

    reconnect_to_wifi_insistently_if_needed(&mut wifi_driver, false);

    let mac_address = get_mac_address(&mut wifi_driver);
    info!("WiFi MAC Address: {:?}", mac_address);

    synchronize_clock_insistently_and_connect_wifi_if_necessary(&mut wifi_driver, false);

    let register_device_result = register_device(&mac_address);
    if register_device_result.is_err() {
        error!(
            "Failed to register the device: {:?}",
            register_device_result
        );
    } else {
        info!("Device registered successfully!");
    }

    let mut buzzer1 = PinDriver::output(peripherals.pins.gpio5).unwrap();
    let mut buzzer2 = PinDriver::output(peripherals.pins.gpio15).unwrap();
    buzzer1.set_low().ok();
    buzzer2.set_low().ok();

    let configuration_result = get_configuration(DEFAULT_CONFIGURATION_URI, &mac_address);
    let mut configuration = match configuration_result {
        Err(e) => get_default_configuration(e),
        Ok(data) => data,
    };

    let offset = FixedOffset::east_opt(configuration.timezone_seconds).unwrap();

    let mut alarm = calculate_next_scheduled_time(&configuration.cron_list, &offset);

    let mut is_calculated_alarm_next_date_time = false;
    let mut is_last_config_sync = false;
    let ntp_sync_time: DateTime<FixedOffset> = Utc
        .with_ymd_and_hms(2023, 1, 1, 0, 0, 0)
        .unwrap()
        .with_timezone(&offset);
    info!("NTP server synchronization at: {}", ntp_sync_time);
    let mut ntp_synchronized = false;
    let now = Utc::now().with_timezone(&offset);

    let mut cron_time =
        from_str_to_date_time_after(&now, CHECK_INTERVAL_CONFIGURATION_CRON, &offset);

    let mut i_am_alive_sent = false;
    let i_am_alive_cron_time = (now.second() + configuration.i_am_alive_interval_seconds) % 60;
    loop {
        let now = Utc::now().with_timezone(&offset);

        if is_bzzzzzzzzz_time(alarm, now) {
            bzzzzzzzzz(&mut buzzer1, &mut buzzer2);
            info!("bzzzzzzzz: {:?} => {:?}", now, alarm);
            is_calculated_alarm_next_date_time = false;
        } else {
            calculate_alarm_next_date_time(
                &mut is_calculated_alarm_next_date_time,
                &mut alarm,
                &configuration,
                offset,
            );

            sync_system_clock_if_necessary(
                &mut ntp_synchronized,
                ntp_sync_time,
                now,
                &mut wifi_driver,
            );

            sync_config_if_necessary(
                &mut is_last_config_sync,
                &mut cron_time,
                offset,
                now,
                &mac_address,
                &mut configuration,
                &mut wifi_driver,
            );

            if ENABLE_I_AM_ALIVE_ACK {
                send_i_am_alive_if_necessary(
                    &mut i_am_alive_sent,
                    i_am_alive_cron_time,
                    now,
                    &mac_address,
                    &mut configuration,
                    &mut wifi_driver,
                );
            }

            FreeRtos::delay_ms(100);
        }
    }
}

fn calculate_alarm_next_date_time(
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

fn sync_config_if_necessary(
    downloaded: &mut bool,
    cron_time: &mut DateTime<FixedOffset>,
    offset: FixedOffset,
    now: DateTime<FixedOffset>,
    mac_address: &String,
    configuration: &mut ConfigurationResponse,
    wifi_driver: &mut EspWifi<'static>,
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
    }
    if !is_same_time_sec(old_cron_time, now) && *downloaded {
        *downloaded = false;
    }
}

fn send_i_am_alive_if_necessary(
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

fn send_i_am_alive(mac_address: &str, url: &str) -> anyhow::Result<(), anyhow::Error> {
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

fn sync_system_clock_if_necessary(
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

fn reconnect_to_wifi_insistently_if_needed(wifi_driver: &mut EspWifi<'_>, one_shot: bool) {
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

fn connect_to_wifi(wifi_driver: &mut EspWifi<'_>, one_shot: bool) -> Result<(), EspError> {
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

fn calculate_next_scheduled_time(
    configuration_crontab: &Vec<CronListResponse>,
    offset: &FixedOffset,
) -> DateTime<FixedOffset> {
    if configuration_crontab.len() > 0 {
        let now = Utc::now().with_timezone(offset);
        let mut lower_date_time =
            from_str_to_date_time(configuration_crontab.get(0).unwrap().cron.as_str(), offset);
        for cron in configuration_crontab.iter() {
            let processed = from_str_to_date_time(cron.cron.as_str(), offset);
            info!("--> processed: {}, now: {}", processed, now);
            if processed < lower_date_time && processed >= now {
                lower_date_time = processed;
            }
        }
        info!("--> selected: {}", lower_date_time);
        return lower_date_time;
    }
    let offset = FixedOffset::east_opt(-60).unwrap();
    return Utc::now().with_timezone(&offset);
}

fn from_str_to_date_time(cron_string: &str, offset_crontab: &FixedOffset) -> DateTime<FixedOffset> {
    let schedule_result = Schedule::from_str(&cron_string);
    let schedule;
    if schedule_result.is_err() {
        error!("invalid crontab value");
        panic!();
    } else {
        schedule = schedule_result.unwrap();
    }
    calculate_next_date_time(&schedule, offset_crontab)
}

fn from_str_to_date_time_after(
    date_time: &DateTime<FixedOffset>,
    cron_string: &str,
    offset_crontab: &FixedOffset,
) -> DateTime<FixedOffset> {
    let schedule_result = Schedule::from_str(&cron_string);
    let schedule;
    if schedule_result.is_err() {
        error!("invalid crontab value");
        panic!();
    } else {
        schedule = schedule_result.unwrap();
    }
    calculate_next_date_time2(date_time, &schedule, offset_crontab)
}
fn bzzzzzzzzz(
    buzzer1: &mut PinDriver<'_, esp_idf_svc::hal::gpio::Gpio5, esp_idf_svc::hal::gpio::Output>,
    buzzer2: &mut PinDriver<'_, esp_idf_svc::hal::gpio::Gpio15, esp_idf_svc::hal::gpio::Output>,
) {
    buzzer1.set_high().ok();
    FreeRtos::delay_ms(100);
    buzzer1.set_low().ok();
    buzzer2.set_high().ok();
    FreeRtos::delay_ms(100);
    buzzer2.set_low().ok();
}

fn is_bzzzzzzzzz_time(alarm: DateTime<FixedOffset>, now: DateTime<FixedOffset>) -> bool {
    alarm.day() == now.day()
        && alarm.month() == now.month()
        && alarm.year() == now.year()
        && alarm.hour() == now.hour()
        && now.minute() >= alarm.minute()
        && now.minute() <= alarm.minute() + (DEFAULT_ALARM_INTERVAL_MINUTES)
}

fn is_same_time(alarm: DateTime<FixedOffset>, now: DateTime<FixedOffset>) -> bool {
    alarm.hour() == now.hour() && now.minute() == alarm.minute()
}

fn is_same_time_sec(alarm: DateTime<FixedOffset>, now: DateTime<FixedOffset>) -> bool {
    alarm.hour() == now.hour() && now.minute() == alarm.minute() && now.second() == alarm.second()
}

fn is_same_sec(second_number: u32, now: DateTime<FixedOffset>) -> bool {
    now.second() == second_number
}

fn synchronize_clock_insistently_and_connect_wifi_if_necessary(
    wifi: &mut EspWifi<'static>,
    one_shot: bool,
) {
    while synchronize_clock(one_shot).is_err() {
        FreeRtos::delay_ms(100);
        if one_shot {
            break;
        }
        reconnect_to_wifi_insistently_if_needed(wifi, one_shot);
    }
}

fn synchronize_clock(one_shot: bool) -> Result<(), String> {
    let sntp = sntp::EspSntp::new_default();
    if sntp.is_err() {
        return Err("Sync error".into());
    }
    let sntp = sntp.unwrap();
    info!("SNTP initialized, waiting for status!");
    let mut attempts = 0;
    while sntp.get_sync_status() != SyncStatus::Completed {
        FreeRtos::delay_ms(100);
        warn!("waiting for clock synchronization...");
        attempts += 1;
        if one_shot && attempts > 3000 {
            break;
        }
        if attempts > 300000 {
            return Err("clock sync: to many attempts".into());
        }
    }
    Ok(())
}

fn calculate_next_date_time(schedule: &Schedule, offset: &FixedOffset) -> DateTime<FixedOffset> {
    schedule
        .upcoming(Utc)
        .take(1)
        .into_iter()
        .last()
        .unwrap()
        .with_timezone(offset)
}

fn calculate_next_date_time2(
    after: &DateTime<FixedOffset>,
    schedule: &Schedule,
    offset: &FixedOffset,
) -> DateTime<FixedOffset> {
    schedule
        .after(after)
        .take(1)
        .into_iter()
        .last()
        .unwrap()
        .with_timezone(offset)
}

fn get_mac_address(wifi: &mut EspWifi<'static>) -> String {
    let mav = wifi.driver().get_mac(WifiDeviceId::Sta).unwrap();
    let mac_address_obj = macaddr::MacAddr6::new(mav[0], mav[1], mav[2], mav[3], mav[4], mav[5]);
    let mac_address_value = mac_address_obj.to_string();
    mac_address_value
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

pub fn get_default_configuration(e: StandardError) -> ConfigurationResponse {
    error!(
        "Error while trying to load configuration from remote server: {:?}",
        e
    );
    ConfigurationResponse {
        i_am_alive_endpoint: DEFAULT_I_AM_ALIVE_ENDPOINT.to_owned(),
        i_am_alive_interval_seconds: DEFAULT_I_AM_ALIVE_INTERVAL_SECONDS,
        cron_list: DEFAULT_CRONTAB
            .map(|item| CronListResponse {
                cron: item.to_owned(),
                description: "alarm".to_owned(),
            })
            .to_vec(),
        timezone_seconds: DEFAULT_TIMEZONE,
        alarm_interval_minutes: DEFAULT_ALARM_INTERVAL_MINUTES,
    }
}

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
