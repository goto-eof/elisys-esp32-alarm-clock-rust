use super::wifi_service::reconnect_to_wifi_insistently_if_needed;
use esp_idf_svc::sntp;
use esp_idf_svc::{hal::delay::FreeRtos, sntp::SyncStatus, wifi::EspWifi};
use log::info;
use log::warn;

pub fn synchronize_clock_insistently_and_connect_wifi_if_necessary(
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

pub fn synchronize_clock(one_shot: bool) -> Result<(), String> {
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
