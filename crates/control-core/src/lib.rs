use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GpuId {
    pub index: u32,
    pub uuid: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    Unsupported,
    ReadOnly,
    ReadWrite,
    ExperimentalReadWrite,
}

impl AccessLevel {
    pub const fn is_available(self) -> bool {
        !matches!(self, Self::Unsupported)
    }

    pub const fn can_write(self) -> bool {
        matches!(self, Self::ReadWrite | Self::ExperimentalReadWrite)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub telemetry: AccessLevel,
    pub power_limit: AccessLevel,
    pub clocks: AccessLevel,
    pub fan_control: AccessLevel,
    pub voltage_frequency_curve: AccessLevel,
    pub digital_vibrance: AccessLevel,
    pub dlss_overrides: AccessLevel,
    pub reflex: AccessLevel,
    pub smooth_motion: AccessLevel,
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self {
            telemetry: AccessLevel::Unsupported,
            power_limit: AccessLevel::Unsupported,
            clocks: AccessLevel::Unsupported,
            fan_control: AccessLevel::Unsupported,
            voltage_frequency_curve: AccessLevel::Unsupported,
            digital_vibrance: AccessLevel::Unsupported,
            dlss_overrides: AccessLevel::Unsupported,
            reflex: AccessLevel::Unsupported,
            smooth_motion: AccessLevel::Unsupported,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuDevice {
    pub id: GpuId,
    pub name: String,
    pub driver_version: Option<String>,
    pub vbios_version: Option<String>,
    pub capabilities: CapabilitySet,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TelemetrySnapshot {
    pub temperature_c: Option<f32>,
    pub hotspot_temperature_c: Option<f32>,
    pub memory_temperature_c: Option<f32>,
    pub gpu_util_percent: Option<f32>,
    pub memory_util_percent: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub power_watts: Option<f32>,
    pub core_clock_mhz: Option<u32>,
    pub memory_clock_mhz: Option<u32>,
    pub fan_percent: Option<f32>,
}

#[derive(Debug, Error)]
pub enum ControlError {
    #[error("backend is unavailable: {0}")]
    BackendUnavailable(String),

    #[error("GPU was not found: {0}")]
    GpuNotFound(String),

    #[error("operation is not supported: {0}")]
    Unsupported(String),

    #[error("backend operation failed: {0}")]
    Backend(String),
}

pub trait GpuBackend: Send + Sync {
    fn backend_name(&self) -> &'static str;

    fn enumerate(&self) -> Result<Vec<GpuDevice>, ControlError>;

    fn telemetry(&self, id: &GpuId) -> Result<TelemetrySnapshot, ControlError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn experimental_access_is_writable() {
        assert!(AccessLevel::ExperimentalReadWrite.can_write());
        assert!(!AccessLevel::ReadOnly.can_write());
    }

    #[test]
    fn unsupported_access_is_not_available() {
        assert!(!AccessLevel::Unsupported.is_available());
        assert!(AccessLevel::ReadOnly.is_available());
    }
}
