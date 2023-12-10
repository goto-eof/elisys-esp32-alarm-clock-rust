use std::str::FromStr;

use chrono::{DateTime, Datelike, FixedOffset, Timelike, Utc};
use cron::Schedule;
use log::{error, info};

use crate::{
    config::config::DEFAULT_ALARM_INTERVAL_MINUTES,
    dto::config_cron_list_response::CronListResponse,
};
pub fn from_str_to_date_time(
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
    calculate_next_date_time(&schedule, offset_crontab)
}

pub fn from_str_to_date_time_after(
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

pub fn is_same_time(alarm: DateTime<FixedOffset>, now: DateTime<FixedOffset>) -> bool {
    alarm.hour() == now.hour() && now.minute() == alarm.minute()
}

pub fn is_same_time_sec(alarm: DateTime<FixedOffset>, now: DateTime<FixedOffset>) -> bool {
    alarm.hour() == now.hour() && now.minute() == alarm.minute() && now.second() == alarm.second()
}

pub fn is_same_sec(second_number: u32, now: DateTime<FixedOffset>) -> bool {
    now.second() == second_number
}

pub fn calculate_next_date_time(
    schedule: &Schedule,
    offset: &FixedOffset,
) -> DateTime<FixedOffset> {
    schedule
        .upcoming(Utc)
        .take(1)
        .into_iter()
        .last()
        .unwrap()
        .with_timezone(offset)
}

pub fn calculate_next_date_time2(
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

pub fn calculate_next_scheduled_time(
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

pub fn is_time_to_buzz(alarm: DateTime<FixedOffset>, now: DateTime<FixedOffset>) -> bool {
    alarm.day() == now.day()
        && alarm.month() == now.month()
        && alarm.year() == now.year()
        && alarm.hour() == now.hour()
        && now.minute() >= alarm.minute()
        && now.minute() <= alarm.minute() + (DEFAULT_ALARM_INTERVAL_MINUTES)
}
