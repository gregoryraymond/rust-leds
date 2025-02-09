mod http_client;
mod motor;
mod wifi;

use chrono::{DateTime, Duration, Local, NaiveDateTime, NaiveTime, TimeZone};
use esp_idf_sys::{nvs_flash_erase, nvs_flash_init, ESP_ERR_NVS_NEW_VERSION_FOUND, ESP_ERR_NVS_NO_FREE_PAGES};
use log::info;
use motor::{down, up};
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
        },
        Ok(x) => x,
    };

    ntp_time_sync()?;

    wifi.disconnect()?;

    info!("Going into deep sleep for 2 hours.");
    unsafe { esp_idf_sys::esp_deep_sleep(Duration::hours(2).num_microseconds().unwrap().try_into().unwrap()); }

    Ok(())
}
