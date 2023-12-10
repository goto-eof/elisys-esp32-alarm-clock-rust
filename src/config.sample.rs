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
// I am alive endpoint
pub const DEFAULT_I_AM_ALIVE_ENDPOINT: &str = "";
// I am alive time interval
pub const DEFAULT_I_AM_ALIVE_INTERVAL_SECONDS: u32 = 30;
// configuration download endpoint
pub const DEFAULT_CONFIGURATION_URI: &str =
    "http://192.168.1.102:8080/api/v1/alarm-clock/configuration";
// configuration check cron
pub const CHECK_INTERVAL_CONFIGURATION_CRON: &str =
    "0   0-59   0-23      1-31       Jan-Dec  Mon,Tue,Wed,Thu,Fri,Sat,Sun          2023-2100";
pub const ENABLE_I_AM_ALIVE_ACK: bool = false;
// Device registration endpoint
pub const REGISTER_DEVICE_URL: &str = "http://192.168.1.102:8080/api/v1/device/register";
// Device name
pub const DEVICE_NAME: &str = "Alarm Clock";
// Device description
pub const DEVICE_DESCRIPTION: &str = "Alarm Clock Device";
// Device type
pub const DEVICE_TYPE: &str = "AlarmClock";
