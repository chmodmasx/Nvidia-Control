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
    /// The active backends have not established whether this feature is available.
    Unknown,
    Unsupported,
    ReadOnly,
    ReadWrite,
    ExperimentalReadWrite,
}

impl AccessLevel {
    pub const fn is_available(self) -> bool {
        matches!(
            self,
            Self::ReadOnly | Self::ReadWrite | Self::ExperimentalReadWrite
        )
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
            telemetry: AccessLevel::Unknown,
            power_limit: AccessLevel::Unknown,
            clocks: AccessLevel::Unknown,
            fan_control: AccessLevel::Unknown,
            voltage_frequency_curve: AccessLevel::Unknown,
            digital_vibrance: AccessLevel::Unknown,
            dlss_overrides: AccessLevel::Unknown,
            reflex: AccessLevel::Unknown,
            smooth_motion: AccessLevel::Unknown,
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

/// Read-only device constraints and the supported legacy application-clock table.
/// This is separate from rapidly changing telemetry and implies no write permission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GpuOperatingLimits {
    pub power: PowerLimitInfo,
    pub clocks: ClockLimitInfo,
    pub fans: FanLimitInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PowerLimitInfo {
    pub current_watts: Option<f32>,
    pub default_watts: Option<f32>,
    pub enforced_watts: Option<f32>,
    pub min_watts: Option<f32>,
    pub max_watts: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ClockLimitInfo {
    pub max_graphics_mhz: Option<u32>,
    pub max_memory_mhz: Option<u32>,
    /// Legacy application-clock memory settings: not a guarantee of write support.
    pub supported_application_memory_mhz: Option<Vec<u32>>,
    /// Which memory frequency was used for the graphics-clock query below.
    pub graphics_clocks_for_memory_mhz: Option<u32>,
    /// Possible legacy application graphics-clock settings for that memory clock.
    pub supported_application_graphics_mhz: Option<Vec<u32>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FanLimitInfo {
    /// Readable requested fan-speed range; not confirmation of write privileges.
    pub min_percent: Option<u32>,
    pub max_percent: Option<u32>,
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

    fn operating_limits(&self, id: &GpuId) -> Result<GpuOperatingLimits, ControlError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_limit_readings_serialize_as_null() {
        let json = serde_json::to_value(GpuOperatingLimits::default()).expect("serialize limits");
        assert!(json["power"]["min_watts"].is_null());
        assert!(json["clocks"]["supported_application_memory_mhz"].is_null());
        assert!(json["fans"]["max_percent"].is_null());
    }

    #[test]
    fn experimental_access_is_writable() {
        assert!(AccessLevel::ExperimentalReadWrite.can_write());
        assert!(!AccessLevel::ReadOnly.can_write());
    }

    #[test]
    fn unknown_is_serialized_as_unknown() {
        let json = serde_json::to_string(&AccessLevel::Unknown).expect("serialize unknown");
        assert_eq!(json, "\"unknown\"");
    }

    #[test]
    fn unsupported_access_is_not_available() {
        assert!(!AccessLevel::Unsupported.is_available());
        assert!(!AccessLevel::Unknown.is_available());
        assert!(!AccessLevel::Unknown.can_write());
        assert!(AccessLevel::ReadOnly.is_available());
    }
}
