use esp_idf_svc::hal::{gpio::PinDriver, prelude::Peripherals};

pub fn up() -> Result<(), anyhow::Error> {
    let peripherals = Peripherals::take()?;
    let mut motor_up = PinDriver::output(peripherals.pins.gpio25)?;
    
    motor_up.set_drive_strength(esp_idf_svc::hal::gpio::DriveStrength::I20mA)?;
    motor_up.set_level(esp_idf_svc::hal::gpio::Level::High)?;

    log::info!("motor level: {} {}", motor_up.pin(), motor_up.is_set_high());
    
    std::thread::sleep(std::time::Duration::from_secs(15));

    motor_up.set_level(esp_idf_svc::hal::gpio::Level::Low)?;
    Ok(())
}

pub fn down() -> Result<(), anyhow::Error> {
    let peripherals = Peripherals::take()?;
    let mut motor_down = PinDriver::output(peripherals.pins.gpio21)?;
    
    motor_down.set_drive_strength(esp_idf_svc::hal::gpio::DriveStrength::I20mA)?;
    motor_down.set_level(esp_idf_svc::hal::gpio::Level::High)?;

    log::info!("motor level: {} {}", motor_down.pin(), motor_down.is_set_high());
    
    std::thread::sleep(std::time::Duration::from_secs(15));

    motor_down.set_level(esp_idf_svc::hal::gpio::Level::Low)?;
    Ok(())
}