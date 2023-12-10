# Elisys ESP32 Alarm Clock (Rust)

Elisys ESP32 Alarm Clock (Rust), as can be understood, is an alarm clock developed using Rust programming language for ESP32 devices. This project belongs to the suite of softwares called Elisys Home Automation. Take a look to [Elisys Home Automation server (Java)](https://github.com/goto-eof/elisys-home-automation-server-java) for more information.

In order to run the project please take a look to the `Configuration` section (else, it will not compile).

# How it works?

The final project involves the following behavior:
the application tries to connect to the WiFi network, in case of error, it will try until succeeds. Then it tries to synchronize the system clock, if error occurs, then it will retry to reconnect to the wifi and re-synchronize the system clock. If the device is connected to the WiFi network, and the clock is synchronized with the server, then Elisys ESP32 Alarm Clock, after registering the device on server and after downloading the configuration from the [Elisys Home Automation server (Java)](https://github.com/goto-eof/elisys-home-automation-server-java), will choose the nearest date time in a list of configuration chron strings and wait until the current time is equal to the nearest date time. In this case 2 GPIOs will be set to hight and to low in alternation (on the GPIOs could be connected 2 buzzers or 2 LEDs). Every day at 00:00 the application will try to synchronize the system clock with an NTP server. Moreover, Every 3 seconds the application will download the configuration from the server. Every 30 seconds the application will send an Ack to inform the server that it is alive.

# Configuration

The project contains a configuration file called `config.sample.rs`. Rename it to `config.rs`. Then change your preferences in that file and run the project.
I used the wonderful [`cron`](https://crates.io/crates/cron) crate, witch allows to customize the alarm date time.
For example here:

```
pub const DEFAULT_CRONTAB: &[&str; 2] = &[
    "0   45   8     1-31       Jan-Dec  Mon,Tue,Wed,Thu,Fri  2023-2100",
    "0    30   9     1-31       Jan-Dec  Mon,Tue,Wed,Thu,Fri  2023-2100",
];
pub const DEFAULT_TIMEZONE: i32 = 1 * 60 * 60;
```

we are saying that there are 2 alarms, the first one is at 9:45 (**nine** because the DEFAULT_TIMEZONE is +1h = 1 x 60 x 60) and occurs from Monday to Friday, every month, and every day of month, from 2023 to 2100 (i tried 2999, but cron throws an error).

# Hardware configuration

Here are the GPIOs and their description:

| GPIO | Description       |
| ---- | ----------------- |
| 5    | first buzzer/led  |
| 15   | second buzzer/led |

# Run it

If you are running Linux (Ubuntu) like me and have some configuration issues, please take a look [here](https://dodu.it/esp32-rust-configure-environment-linux-ubuntu/) for setting up the environment. Else, just execute:

```
cargo run
```

and hold the Boot button of your ESP32 DevKitC to install the software.

# Moreover

During my tests I had some issues with my WiFi network, so that i tried to adapt the code in a way to make the device always connected to internet.

If you found a bug, please ping me [here](https://andre-i.eu/#contactme).
