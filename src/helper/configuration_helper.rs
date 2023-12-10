use crate::ConfigurationResponse;
use crate::{
    config::config::{
        DEFAULT_ALARM_INTERVAL_MINUTES, DEFAULT_CRONTAB, DEFAULT_I_AM_ALIVE_ENDPOINT,
        DEFAULT_I_AM_ALIVE_INTERVAL_SECONDS, DEFAULT_TIMEZONE,
    },
    dto::config_cron_list_response::CronListResponse,
};
use log::error;

pub fn get_default_configuration(e: anyhow::Error) -> ConfigurationResponse {
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
