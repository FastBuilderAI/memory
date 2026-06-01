/// Telemetry stub — FastMemory is fully open-source (MIT).
/// No license keys, no phone-home, no tracking.

pub struct LicenseTelemetry;

impl LicenseTelemetry {
    pub fn ping() -> Option<std::thread::JoinHandle<()>> {
        // No-op: FastMemory is MIT-licensed. No license verification needed.
        None
    }
}
