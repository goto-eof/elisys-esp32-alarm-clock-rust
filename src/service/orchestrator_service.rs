use crate::{
    config::config::{CHECK_INTERVAL_CONFIGURATION_CRON, ENABLE_I_AM_ALIVE_ACK},
    helper::{
        date_helper::{
            calculate_next_scheduled_time, from_str_to_date_time_after, is_time_to_buzz,
        },
        orchestrator_helper::{
            calculate_alarm_next_date_time, load_remote_configuration_or_default,
            retrieve_config_if_necessary, send_i_am_alive_if_necessary,
            sync_system_clock_if_necessary, try_register_device,
        },
    },
    service::{
        clock_service::synchronize_clock_insistently_and_connect_wifi_if_necessary,
        peripheral_service::buzz,
        wifi_service::{get_mac_address, reconnect_to_wifi_insistently_if_needed},
    },
};
use chrono::{DateTime, FixedOffset, TimeZone, Timelike, Utc};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals},
    nvs::EspDefaultNvsPartition,
    wifi::EspWifi,
};
use log::info;
use log::warn;

pub fn orchestrate() {
    let peripherals = Peripherals::take().unwrap();

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();

    reconnect_to_wifi_insistently_if_needed(&mut wifi_driver, false);

    let mac_address = get_mac_address(&mut wifi_driver);

    synchronize_clock_insistently_and_connect_wifi_if_necessary(&mut wifi_driver, false);

    try_register_device(&mac_address);

    let mut buzzer1 = PinDriver::output(peripherals.pins.gpio5).unwrap();
    let mut buzzer2 = PinDriver::output(peripherals.pins.gpio15).unwrap();
    buzzer1.set_low().ok();
    buzzer2.set_low().ok();

    let mut configuration = load_remote_configuration_or_default(&mac_address);

    let user_timezone_offset = FixedOffset::east_opt(configuration.timezone_seconds).unwrap();

    let mut alarm = calculate_next_scheduled_time(&configuration.cron_list, &user_timezone_offset);

    let mut is_calculated_alarm_next_date_time = false;
    let mut is_last_config_sync = false;

    let ntp_sync_time: DateTime<FixedOffset> = Utc
        .with_ymd_and_hms(2023, 1, 1, 0, 0, 0)
        .unwrap()
        .with_timezone(&user_timezone_offset);

    let mut ntp_synchronized = false;
    let now = Utc::now().with_timezone(&user_timezone_offset);

    let mut cron_time = from_str_to_date_time_after(
        &now,
        CHECK_INTERVAL_CONFIGURATION_CRON,
        &user_timezone_offset,
    );

    let mut i_am_alive_sent = false;
    let i_am_alive_cron_time = (now.second() + configuration.i_am_alive_interval_seconds) % 60;
    loop {
        let now = Utc::now().with_timezone(&user_timezone_offset);

        if is_time_to_buzz(alarm, now) {
            buzz_buzz_buzz(
                &mut buzzer1,
                &mut buzzer2,
                now,
                alarm,
                &mut is_calculated_alarm_next_date_time,
            );
        } else {
            sync_data_if_needed(
                &mut is_calculated_alarm_next_date_time,
                &mut alarm,
                &mut configuration,
                user_timezone_offset,
                &mut ntp_synchronized,
                ntp_sync_time,
                now,
                &mut wifi_driver,
                &mut is_last_config_sync,
                &mut cron_time,
                &mac_address,
                &mut i_am_alive_sent,
                i_am_alive_cron_time,
            );

            FreeRtos::delay_ms(100);
        }
    }
}

fn sync_data_if_needed(
    is_calculated_alarm_next_date_time: &mut bool,
    alarm: &mut DateTime<FixedOffset>,
    configuration: &mut crate::dto::config_response::Configuration,
    user_timezone_offset: FixedOffset,
    ntp_synchronized: &mut bool,
    ntp_sync_time: DateTime<FixedOffset>,
    now: DateTime<FixedOffset>,
    wifi_driver: &mut EspWifi<'static>,
    is_last_config_sync: &mut bool,
    cron_time: &mut DateTime<FixedOffset>,
    mac_address: &String,
    i_am_alive_sent: &mut bool,
    i_am_alive_cron_time: u32,
) {
    info!(
        "calculated buzz time (now => buzz time): {:?} => {:?}",
        now, alarm
    );

    retrieve_config_if_necessary(
        is_last_config_sync,
        cron_time,
        user_timezone_offset,
        now,
        mac_address,
        configuration,
        wifi_driver,
        is_calculated_alarm_next_date_time
    );

    calculate_alarm_next_date_time(
        is_calculated_alarm_next_date_time,
        alarm,
        &*configuration,
        user_timezone_offset,
    );

    sync_system_clock_if_necessary(ntp_synchronized, ntp_sync_time, now, wifi_driver);

    if ENABLE_I_AM_ALIVE_ACK {
        send_i_am_alive_if_necessary(
            i_am_alive_sent,
            i_am_alive_cron_time,
            now,
            mac_address,
            configuration,
            wifi_driver,
        );
    }
}

fn buzz_buzz_buzz(
    buzzer1: &mut PinDriver<'_, esp_idf_svc::hal::gpio::Gpio5, esp_idf_svc::hal::gpio::Output>,
    buzzer2: &mut PinDriver<'_, esp_idf_svc::hal::gpio::Gpio15, esp_idf_svc::hal::gpio::Output>,
    now: DateTime<FixedOffset>,
    alarm: DateTime<FixedOffset>,
    is_calculated_alarm_next_date_time: &mut bool,
) {
    buzz(buzzer1, buzzer2);
    warn!("bzzzzzzzz: {:?} => {:?}", now, alarm);
    *is_calculated_alarm_next_date_time = false;
}
