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
    pub fn ping() -> Option<std::thread::JoinHandle<()>> {
        let _ = dotenv::dotenv();

        let license_key = match env::var("FASTMEMORY_LICENSE_KEY") {
            Ok(key) if !key.trim().is_empty() => key.trim().to_string(),
            _ => {
                eprintln!("\x1b[33mWARN: No FastMemory Enterprise License found (FASTMEMORY_LICENSE_KEY is missing). Operating in community mode.\x1b[0m");
                "community_edition".to_string()
            }
        };

        let mut sys = System::new();
        sys.refresh_cpu_all();

        let hostname = System::host_name().unwrap_or_else(|| "unknown-host".to_string());
        
        let cpu_info = sys.cpus().first()
            .map(|c| c.vendor_id().to_string())
            .unwrap_or_else(|| "unknown-cpu".to_string());
            
        let cpu_id = format!("{}-{}", hostname, cpu_info);

        Some(std::thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new());

            let mut ip_stack = String::new();
            if let Ok(resp) = client.get("https://api.ipify.org").timeout(std::time::Duration::from_secs(2)).send() {
                if let Ok(ip) = resp.text() {
                    ip_stack = ip;
                }
            }

            if ip_stack.is_empty() {
                let networks = sysinfo::Networks::new_with_refreshed_list();
                let mut ips = Vec::new();
                for (name, data) in &networks {
                    if name.contains("lo") { continue; }
                    for ip in data.ip_networks() {
                        ips.push(format!("{}|{}", name, ip));
                    }
                }
                ip_stack = ips.join(", ");
                if ip_stack.is_empty() {
                    ip_stack = "unknown".to_string();
                }
            }

            let payload = TelemetryPayload {
                license_key,
                cpu_id,
                ip_address: Some(ip_stack),
            };

            let backend_url = env::var("FASTMEMORY_API_URL")
                .unwrap_or_else(|_| "https://api.fastbuilder.ai/api/licenses/fastmemory/verify".to_string());

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
        }))
    }
}
