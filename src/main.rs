use std::str::FromStr;

use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike, Utc};
mod config;
use config::{DEFAULT_ALARM_INTERVAL_MINUTES, DEFAULT_CRONTAB, DEFAULT_TIMEZONE};
use cron::Schedule;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sntp;
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, EspWifi, WifiDeviceId};
use esp_idf_svc::{
    hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals},
    sntp::SyncStatus,
};
use esp_idf_sys::EspError;
use log::{error, info, warn};

use crate::config::{WIFI_PASS, WIFI_SSID};
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

    let mut buzzer1 = PinDriver::output(peripherals.pins.gpio5).unwrap();
    let mut buzzer2 = PinDriver::output(peripherals.pins.gpio15).unwrap();
    buzzer1.set_low().ok();
    buzzer2.set_low().ok();

    // TODO download configuration from server

    let configuration_crontab = DEFAULT_CRONTAB;
    let configuration_timezone = DEFAULT_TIMEZONE;

    let offset = FixedOffset::east_opt(configuration_timezone).unwrap();

    let mut alarm = calculate_next_scheduled_time(configuration_crontab, &offset);

    let mut is_calculated_next_date_time = false;

    let ntp_sync_time: DateTime<FixedOffset> = Utc
        .with_ymd_and_hms(2023, 1, 1, 0, 0, 0)
        .unwrap()
        .with_timezone(&offset);
    info!("NTP server synchronization at: {}", ntp_sync_time);
    let mut ntp_synchronized = false;
    loop {
        let now = Utc::now().with_timezone(&offset);

        if is_bzzzzzzzzz_time(alarm, now) {
            bzzzzzzzzz(&mut buzzer1, &mut buzzer2);
            info!("{:?} => {:?}", now, alarm);
            is_calculated_next_date_time = false;
        } else {
            FreeRtos::delay_ms(1000);
            info!("{:?} => {:?}", now, alarm);
            if !is_calculated_next_date_time {
                alarm = calculate_next_scheduled_time(configuration_crontab, &offset);
                is_calculated_next_date_time = true;
            }
            reconnect_to_wifi_insistently_if_needed(&mut wifi_driver, true);
            if !ntp_synchronized && is_same_time(ntp_sync_time, now) {
                synchronize_clock_insistently_and_connect_wifi_if_necessary(&mut wifi_driver, true);
                ntp_synchronized = true;
            } else if ntp_synchronized && !is_same_time(ntp_sync_time, now) {
                ntp_synchronized = false;
            }
            // TODO sync with configuration server every 3 seconds: retrieve configuration
        }
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
    configuration_crontab: &[&str],
    offset: &FixedOffset,
) -> DateTime<FixedOffset> {
    let now = Utc::now().with_timezone(offset);
    let mut lower_date_time = from_str_to_date_time(configuration_crontab[0], offset);
    for cron_string in configuration_crontab.iter() {
        let processed = from_str_to_date_time(cron_string, offset);
        info!("--> processed: {}, now: {}", processed, now);
        if processed < lower_date_time && processed >= now {
            lower_date_time = processed;
        }
    }
    info!("--> selected: {}", lower_date_time);
    lower_date_time
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
        && now.minute() <= alarm.minute() + DEFAULT_ALARM_INTERVAL_MINUTES
}

fn is_same_time(alarm: DateTime<FixedOffset>, now: DateTime<FixedOffset>) -> bool {
    alarm.hour() == now.hour() && now.minute() >= alarm.minute() && now.minute() <= alarm.minute()
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

fn get_mac_address(wifi: &mut EspWifi<'static>) -> String {
    let mav = wifi.driver().get_mac(WifiDeviceId::Sta).unwrap();
    let mac_address_obj = macaddr::MacAddr6::new(mav[0], mav[1], mav[2], mav[3], mav[4], mav[5]);
    let mac_address_value = mac_address_obj.to_string();
    mac_address_value
}
