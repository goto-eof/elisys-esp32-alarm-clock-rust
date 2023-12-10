use esp_idf_svc::hal::{delay::FreeRtos, gpio::PinDriver};

pub fn buzz(
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
