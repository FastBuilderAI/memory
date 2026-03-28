use reqwest::Client;
use serde::Serialize;
use std::env;
use std::time::Duration;
use sysinfo::System;

#[derive(Serialize)]
struct TelemetryPayload {
    license_key: String,
    cpu_id: String,
    ip_address: Option<String>,
}

#[derive(serde::Deserialize)]
struct TelemetryResponse {
    valid: bool,
    message: String,
}

pub struct LicenseTelemetry;

impl LicenseTelemetry {
    pub fn ping() {
        let _ = dotenv::dotenv();

        let license_key = match env::var("FASTMEMORY_LICENSE_KEY") {
            Ok(key) if !key.trim().is_empty() => key.trim().to_string(),
            _ => {
                eprintln!("\x1b[33mWARN: No FastMemory Enterprise License found (FASTMEMORY_LICENSE_KEY is missing). Operating in community mode.\x1b[0m");
                return;
            }
        };

        let mut sys = System::new();
        sys.refresh_cpu_all();

        let hostname = System::host_name().unwrap_or_else(|| "unknown-host".to_string());
        
        let cpu_info = sys.cpus().first()
            .map(|c| c.vendor_id().to_string())
            .unwrap_or_else(|| "unknown-cpu".to_string());
            
        let cpu_id = format!("{}-{}", hostname, cpu_info);

        let payload = TelemetryPayload {
            license_key,
            cpu_id,
            ip_address: None,
        };

        std::thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new());

            let backend_url = env::var("FASTMEMORY_API_URL")
                .unwrap_or_else(|_| "http://localhost:3002/api/licenses/fastmemory/verify".to_string());

            match client.post(&backend_url).json(&payload).send() {
                Ok(resp) => {
                    if let Ok(json) = resp.json::<TelemetryResponse>() {
                        if !json.valid {
                            eprintln!("\x1b[31mWARN: FastMemory Enterprise License is INVALID or EXPIRED: {}\x1b[0m", json.message);
                        }
                    }
                },
                Err(_) => {
                    // Suppress connection errors
                }
            }
        });
    }
}
