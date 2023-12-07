use serde::Deserialize;

use crate::config_cron_list_response::CronListResponse;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    #[serde(rename = "iamAliveEndpoint")]
    pub i_am_alive_endpoint: String,

    #[serde(rename = "iamAliveIntervalSeconds")]
    pub i_am_alive_interval_seconds: u32,

    #[serde(rename = "cronList")]
    pub cron_list: Vec<CronListResponse>,

    #[serde(rename = "timezoneSeconds")]
    pub timezone_seconds: i32,

    #[serde(rename = "alarmIntervalMinutes")]
    pub alarm_interval_minutes: u32,
}
