use anyhow::{bail, Result};
use log::info;
use core::str;
use std::sync::LazyLock;
use esp_idf_svc::{
    http::client::{Method, Configuration as HttpConfiguration, EspHttpConnection},
    tls::X509,
    io::Read
};
use embedded_svc::http::client::Client;

static CERTIFICATE: LazyLock<Vec<u8>> = std::sync::LazyLock::new(|| {
    let mut c = include_bytes!("cert.crt").to_vec();
    c.append(&mut vec![0_u8]);
    c
});

pub fn load() -> Result<String> {
    get("https://api.sunrisesunset.io/json?lat=-35.2820012&lng=149.128998")
}

fn get(url: impl AsRef<str>) -> Result<String> {
    let connection = EspHttpConnection::new(&HttpConfiguration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        client_certificate: Some(X509::pem_until_nul(CERTIFICATE.as_slice())),
        ..Default::default()
    })?;
    // ANCHOR_END: connection
    let mut client = Client::wrap(connection);

    // 2. Open a GET request to `url`
    let headers = [("accept", "text/plain")];
    let request = client.request(Method::Get, url.as_ref(), &headers)?;

    // 3. Submit write request and check the status code of the response.
    // Successful http status codes are in the 200..=299 range.
    let response = request.submit()?;
    let status = response.status();

    info!("Response code: {}\n", status);

    match status {
        200..=299 => {
            // 4. if the status is OK, read response data chunk by chunk into a buffer and print it until done
            //
            // NB. see http_client.rs for an explanation of the offset mechanism for handling chunks that are
            // split in the middle of valid UTF-8 sequences. This case is encountered a lot with the given
            // example URL.
            let mut buf = [0_u8; 256];
            let mut offset = 0;
            let mut reader = response;
            let mut total_str: String = String::new();
            loop {
                match Read::read(&mut reader, &mut buf[offset..]) {
                    Ok(size) => {
                        if size == 0 {
                            return Ok(total_str);
                        }
                        // 5. try converting the bytes into a Rust (UTF-8) string and print it
                        let size_plus_offset = size + offset;
                        match str::from_utf8(&buf[..size_plus_offset]) {
                            Ok(text) => {
                                total_str += text;
                                offset = 0;
                            }
                            Err(error) => {
                                let valid_up_to = error.valid_up_to();
                                unsafe {
                                    print!("{}", str::from_utf8_unchecked(&buf[..valid_up_to]));
                                }
                                buf.copy_within(valid_up_to.., 0);
                                offset = size_plus_offset - valid_up_to;
                            }
                        }
                    }
                    _ => {
                        return Ok(total_str);
                    }
                }
            }
        }
        _ => bail!("Unexpected response code: {}", status),
    }
}
