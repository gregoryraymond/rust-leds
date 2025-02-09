mod http_client;
mod motor;
mod wifi;

use chrono::{DateTime, Duration, Local, NaiveDateTime, NaiveTime, TimeZone};
use esp_idf_sys::{esp_flash_init, esp_sleep_ext1_wakeup_mode_t, esp_timer_get_time, nvs_flash_erase, nvs_flash_init, ESP_ERR_NVS_NEW_VERSION_FOUND, ESP_ERR_NVS_NO_FREE_PAGES};
use log::info;
use motor::{down, up};
use std::time::SystemTime;

use crate::wifi::CONFIG;
use anyhow::{bail, Result};
use serde_json::Value;

use esp_idf_svc::{hal::{gpio::{PinDriver, Pull}, prelude::Peripherals}, sntp::{EspSntp, SyncStatus}, timer::EspTimer};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    unsafe {
        let err = nvs_flash_init();
        if err == ESP_ERR_NVS_NO_FREE_PAGES || err == ESP_ERR_NVS_NEW_VERSION_FOUND {
            nvs_flash_erase();
            let lerr = nvs_flash_init();
            info!("Reinit nvs_flash {}", lerr);
        }
    }

    if let Err(e) = logic() {
        log::error!("{}", e);
        return Err(e);
    }
    Ok(())
}

fn parse_sunset_time(sunset: String, dt_now: &DateTime<Local>) -> Result<DateTime<Local>> {
    // Example str: "sunset":"5:22:11 PM"
    let values: Vec<u32> = sunset
        .split(&['"', ':', ' '][..])
        .flat_map(|x| x.trim().parse::<u32>())
        .collect();
    let after_noon = match sunset.ends_with("PM") {
        true => 12,
        false => 0,
    };
    info!("parse_sunset_time {}, {:?}", sunset, values);
    let naive_time = NaiveTime::from_hms_opt(values[0] + after_noon, values[1], values[2])
        .ok_or(anyhow::anyhow!("Could not parse sunset time"))?;
    let naive_sunset = NaiveDateTime::new(dt_now.date_naive(), naive_time);
    Ok(match TimeZone::from_local_datetime(&Local, &naive_sunset) {
        chrono::offset::LocalResult::Single(x) => x,
        chrono::offset::LocalResult::Ambiguous(_, _) => bail!("Ambigious sunset time given"),
        chrono::offset::LocalResult::None => bail!("No sunset time given"),
    })
}

fn ntp_time_sync() -> Result<()> {
    let ntp = EspSntp::new_default()?;
    while ntp.get_sync_status() != SyncStatus::Completed {}
    Ok(())
}

fn parse_json_to_sunset_time(json: &String) -> Result<String> {
    let value: Value = serde_json::from_str(json)?;
    Ok(value["results"]["sunset"].to_string())
}

fn parse_json_to_sunrise_time(json: &String) -> Result<String> {
    let value: Value = serde_json::from_str(json)?;
    Ok(value["results"]["sunrise"].to_string())
}

fn logic() -> Result<()> {
    let app_config = CONFIG;



    let mut wifi = match wifi::connect(app_config) {
        Ok(x) => x,
        Err(e) => {
            bail!("Error connecting to wifi: {}", e);
        }
    };
    let json: String = match http_client::load() {
        Err(e) => {
            bail!("Error loading client: {}", e);
        }
        Ok(x) => x,
    };

    ntp_time_sync()?;

    wifi.disconnect()?;

    let now: DateTime<Local> = SystemTime::now().into();
    let sunset: DateTime<Local> = parse_sunset_time(parse_json_to_sunset_time(&json)?, &now)?;
    let sunrise: DateTime<Local> = parse_sunset_time(parse_json_to_sunrise_time(&json)?, &now)?;
    let offet_mins = Duration::minutes(60);
    if now < (sunset - offet_mins) && now > (sunset + offet_mins) {
        down()?;
    } else if now < (sunrise - offet_mins) && now > (sunrise + offet_mins) {
        up()?;
    } else {
        info!("Going into deep sleep for 2 hours.");
        unsafe { esp_idf_sys::esp_deep_sleep((Duration::hours(2).num_microseconds().unwrap() as i64).try_into().unwrap()); }
    }

    Ok(())
}
