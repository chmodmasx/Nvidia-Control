use nvidia_control_backend_mock::MockBackend;
use nvidia_control_backend_nvml::NvmlBackend;
use nvidia_control_core::{
    ControlError, GpuBackend, GpuDevice, GpuOperatingLimits, TelemetrySnapshot,
};
use serde::Serialize;
use std::process::ExitCode;

#[derive(Debug, Serialize)]
struct DeviceReport {
    backend: &'static str,
    device: GpuDevice,
    telemetry: TelemetrySnapshot,
    operating_limits: GpuOperatingLimits,
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

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let backend = select_backend()?;
    let backend_name = backend.backend_name();
    let mut reports = Vec::new();

    for device in backend.enumerate()? {
        let telemetry = backend.telemetry(&device.id)?;
        let operating_limits = backend.operating_limits(&device.id)?;
        reports.push(DeviceReport {
            backend: backend_name,
            device,
            telemetry,
            operating_limits,
        });
    }

    println!("{}", serde_json::to_string_pretty(&reports)?);
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nvidia-control-daemon: {error}");
            eprintln!(
                "hint: set NVIDIA_CONTROL_BACKEND=mock to run the prototype without NVIDIA hardware"
            );
            ExitCode::FAILURE
        }
    }
}
