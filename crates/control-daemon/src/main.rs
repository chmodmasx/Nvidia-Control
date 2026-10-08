use nvidia_control_backend_mock::MockBackend;
use nvidia_control_backend_nvml::NvmlBackend;
use nvidia_control_core::{
    ControlError, GpuBackend, GpuDevice, GpuOperatingLimits, TelemetrySnapshot,
};
use serde::Serialize;
use std::process::ExitCode;

const BUS_NAME: &str = "io.github.chmodmasx.NvidiaControl";
const OBJECT_PATH: &str = "/io/github/chmodmasx/NvidiaControl";

#[derive(Debug, Serialize)]
struct DeviceReport {
    backend: &'static str,
    device: GpuDevice,
    telemetry: TelemetrySnapshot,
    operating_limits: GpuOperatingLimits,
}

struct Service {
    backend: Box<dyn GpuBackend>,
}

impl Service {
    fn new(backend: Box<dyn GpuBackend>) -> Self {
        Self { backend }
    }

    fn snapshot(&self) -> Result<Vec<DeviceReport>, ControlError> {
        let name = self.backend.backend_name();
        let mut reports = Vec::new();

        for device in self.backend.enumerate()? {
            let telemetry = self.backend.telemetry(&device.id)?;
            let operating_limits = self.backend.operating_limits(&device.id)?;
            reports.push(DeviceReport {
                backend: name,
                device,
                telemetry,
                operating_limits,
            });
        }

        Ok(reports)
    }

    fn snapshot_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(&self.snapshot()?)?)
    }
}

struct SessionInterface {
    service: Service,
}

#[zbus::interface(name = "io.github.chmodmasx.NvidiaControl1")]
impl SessionInterface {
    /// Returns the same JSON array as --once, including all GPUs and capabilities.
    /// Contract version 1: read-only; future writes require a separate privileged service.
    fn get_snapshot(&self) -> zbus::fdo::Result<String> {
        self.service
            .snapshot_json()
            .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))
    }

    fn get_api_version(&self) -> u32 {
        1
    }
}

fn select_backend() -> Result<Box<dyn GpuBackend>, ControlError> {
    let requested = std::env::var("NVIDIA_CONTROL_BACKEND").unwrap_or_else(|_| "nvml".to_string());

    match requested.as_str() {
        "nvml" => Ok(Box::new(NvmlBackend::new()?)),
        "mock" => Ok(Box::new(MockBackend)),
        other => Err(ControlError::BackendUnavailable(format!(
            "unknown backend '{other}'; supported backends: nvml, mock"
        ))),
    }
}

fn run_once(backend: Box<dyn GpuBackend>) -> Result<(), Box<dyn std::error::Error>> {
    let report = Service::new(backend).snapshot()?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn run_session(backend: Box<dyn GpuBackend>) -> Result<(), Box<dyn std::error::Error>> {
    let interface = SessionInterface {
        service: Service::new(backend),
    };

    let _connection = zbus::blocking::connection::Builder::session()?
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, interface)?
        .build()?;

    eprintln!("nvidia-control-daemon: serving read-only D-Bus on {BUS_NAME}");
    loop {
        std::thread::park();
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let backend = select_backend()?;
    match std::env::args().nth(1).as_deref() {
        None | Some("--once") => run_once(backend),
        Some("--session") => run_session(backend),
        Some(arg) => Err(format!(
            "unknown argument '{arg}'; usage: nvidia-control-daemon [--once|--session]"
        )
        .into()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nvidia-control-daemon: {error}");
            eprintln!("hint: set NVIDIA_CONTROL_BACKEND=mock for testing without NVIDIA hardware");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_snapshot_has_expected_fields() {
        let service = Service::new(Box::new(MockBackend));
        let value: serde_json::Value =
            serde_json::from_str(&service.snapshot_json().expect("snapshot")).expect("JSON");
        assert_eq!(value[0]["backend"], "mock");
        assert_eq!(value[0]["device"]["name"], "NVIDIA Mock GPU");
        assert!(value[0]["telemetry"]["power_watts"].is_number());
        assert!(value[0]["operating_limits"]["power"]["min_watts"].is_number());
    }
}
