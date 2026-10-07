use nvidia_control_backend_mock::MockBackend;
use nvidia_control_core::{ControlError, GpuBackend, GpuDevice, TelemetrySnapshot};
use serde::Serialize;
use std::process::ExitCode;

#[derive(Debug, Serialize)]
struct DeviceReport {
    device: GpuDevice,
    telemetry: TelemetrySnapshot,
}

fn select_backend() -> Result<Box<dyn GpuBackend>, ControlError> {
    let requested =
        std::env::var("NVIDIA_CONTROL_BACKEND").unwrap_or_else(|_| "mock".to_string());

    match requested.as_str() {
        "mock" => Ok(Box::new(MockBackend)),
        other => Err(ControlError::BackendUnavailable(format!(
            "backend '{other}' is not implemented yet"
        ))),
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let backend = select_backend()?;
    let mut reports = Vec::new();

    for device in backend.enumerate()? {
        let telemetry = backend.telemetry(&device.id)?;
        reports.push(DeviceReport { device, telemetry });
    }

    println!("{}", serde_json::to_string_pretty(&reports)?);
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nvidia-control-daemon: {error}");
            ExitCode::FAILURE
        }
    }
}
