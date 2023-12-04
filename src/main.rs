use std::str::FromStr;

use chrono::{DateTime, Datelike, FixedOffset, Timelike, Utc};
mod config;
use config::DEFAULT_ALARM_INTERVAL_SECONDS;
use cron::Schedule;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sntp;
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, EspWifi};
use esp_idf_svc::{
    hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals},
    sntp::SyncStatus,
};
use log::{error, info, warn};

use crate::config::{WIFI_PASS, WIFI_SSID};
fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();

    wifi_driver
        .set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: WIFI_SSID.into(),
            password: WIFI_PASS.into(),
            ..Default::default()
        }))
        .unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    while !wifi_driver.is_connected().unwrap() {
        let config = wifi_driver.get_configuration().unwrap();
        warn!("Waiting for station {:?}", config);
        FreeRtos::delay_ms(100);
    }

    // TODO retrieve MAC Address

    synchronize_clock();

    let mut buzzer1 = PinDriver::output(peripherals.pins.gpio5).unwrap();
    let mut buzzer2 = PinDriver::output(peripherals.pins.gpio15).unwrap();

    // TODO download configuration from server

    let schedule_result = Schedule::from_str(&config::DEFAULT_CRONTAB);
    let schedule;
    if schedule_result.is_err() {
        error!("invalid crontab value");
        panic!();
    } else {
        schedule = schedule_result.unwrap();
    }
    let offset_crontab = FixedOffset::east_opt(0).unwrap();
    let mut alarm = calculate_next_date_time(&schedule, &offset_crontab);

    let offset = FixedOffset::east_opt(1 * 60 * 60).unwrap();
    let mut is_calculated_next_date_time = false;
    loop {
        let now = Utc::now().with_timezone(&offset);

        if is_alarm(alarm, now) {
            bzzzzzzzzz(&mut buzzer1, &mut buzzer2);
            is_calculated_next_date_time = false;
        } else {
            FreeRtos::delay_ms(1000);
            info!("{:?} => {:?}", now, alarm);
            if !is_calculated_next_date_time {
                alarm = calculate_next_date_time(&schedule, &offset_crontab);
                is_calculated_next_date_time = true;
            }
            // TODO if wifi not connected, then try to connect, remember to allow alarm in any case
            // TODO sync with server every 3 seconds: retrieve configuration
        }
    }
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

fn is_alarm(alarm: DateTime<FixedOffset>, now: DateTime<FixedOffset>) -> bool {
    alarm.day() == now.day()
        && alarm.month() == now.month()
        && alarm.year() == now.year()
        && alarm.hour() == now.hour()
        && now.minute() >= alarm.minute()
        && now.minute() <= alarm.minute() + DEFAULT_ALARM_INTERVAL_SECONDS
}

fn synchronize_clock() {
    let sntp = sntp::EspSntp::new_default().unwrap();
    info!("SNTP initialized, waiting for status!");
    while sntp.get_sync_status() != SyncStatus::Completed {
        FreeRtos::delay_ms(100);
        warn!("wating for clock sync...");
    }
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
