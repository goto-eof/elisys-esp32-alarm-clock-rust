pub const WIFI_SSID: &str = "";
pub const WIFI_PASS: &str = "";
// should be retrieved from server
pub const DEFAULT_CRONTAB: &[&str; 2] = &[
    "0   45   8     1-31       Jan-Dec  Mon,Tue,Wed,Thu,Fri  2023-2100",
    "0    30   9     1-31       Jan-Dec  Mon,Tue,Wed,Thu,Fri  2023-2100",
];
// should be retrieved from server
pub const DEFAULT_ALARM_INTERVAL_MINUTES: u32 = 1;
// user timezone
pub const DEFAULT_TIMEZONE: i32 = 1 * 60 * 60;
