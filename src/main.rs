#![no_main]

mod http_client;
mod leds;
mod wifi;

use chrono::{DateTime, Duration, Local, NaiveDateTime, NaiveTime, TimeZone};
use std::time::SystemTime;

use crate::wifi::CONFIG;
use anyhow::{bail, Result};
use serde_json::Value;

use esp_idf_svc::sntp::{EspSntp, SyncStatus};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    if let Err(e) = logic() {
        panic!("{}", e);
    }
    Ok(())
}

fn parse_sunset_time(sunset: String, dt_now: &DateTime<Local>) -> Result<DateTime<Local>> {
    // Example str: "sunset":"5:22:11 PM"
    let values: Vec<u32> = sunset
        .split(&[':', ' '])
        .flat_map(|x| x.parse::<u32>())
        .collect();
    let after_noon = match sunset.ends_with("PM") {
        true => 12,
        false => 0,
    };
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

fn parse_json_to_sunset_time(json: String) -> Result<String> {
    let value: Value = serde_json::from_str(&json)?;
    Ok(value["results"]["sunset"].to_string())
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
    let sunset: DateTime<Local> = parse_sunset_time(parse_json_to_sunset_time(json)?, &now)?;
    let offet_mins = Duration::minutes(60);
    if now < (sunset - offet_mins) && now > (sunset + offet_mins) {
        // Show LEDS
        let _ = leds::on();
    } else if now > (sunset + offet_mins) {
        // Sleep for a long time
    } else {
        // Sleep for a short time
    }

    Ok(())
}
