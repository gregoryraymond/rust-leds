use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::gpio::*;

pub fn on() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio10)?;
    led.set_high()?;
    Ok(())
}

pub fn off() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio10)?;
    led.set_low()?;
    Ok(())
}