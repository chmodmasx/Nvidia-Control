use nvidia_control_core::{
    AccessLevel, CapabilitySet, ClockLimitInfo, ControlError, FanLimitInfo, GpuBackend, GpuDevice,
    GpuId, GpuOperatingLimits, PowerLimitInfo, TelemetrySnapshot,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct MockBackend;

impl MockBackend {
    fn mock_device() -> GpuDevice {
        GpuDevice {
            id: GpuId {
                index: 0,
                uuid: "GPU-MOCK-0000".to_string(),
            },
            name: "NVIDIA Mock GPU".to_string(),
            driver_version: Some("mock".to_string()),
            vbios_version: Some("mock".to_string()),
            capabilities: CapabilitySet {
                telemetry: AccessLevel::ReadOnly,
                power_limit: AccessLevel::ReadWrite,
                clocks: AccessLevel::ReadWrite,
                fan_control: AccessLevel::ReadWrite,
                voltage_frequency_curve: AccessLevel::ExperimentalReadWrite,
                digital_vibrance: AccessLevel::ExperimentalReadWrite,
                dlss_overrides: AccessLevel::ReadWrite,
                reflex: AccessLevel::ReadWrite,
                smooth_motion: AccessLevel::Unsupported,
            },
        }
    }
}

impl GpuBackend for MockBackend {
    fn backend_name(&self) -> &'static str {
        "mock"
    }

    fn enumerate(&self) -> Result<Vec<GpuDevice>, ControlError> {
        Ok(vec![Self::mock_device()])
    }

    fn telemetry(&self, id: &GpuId) -> Result<TelemetrySnapshot, ControlError> {
        let device = Self::mock_device();

        if device.id != *id {
            return Err(ControlError::GpuNotFound(id.uuid.clone()));
        }

        Ok(TelemetrySnapshot {
            temperature_c: Some(67.0),
            hotspot_temperature_c: Some(78.0),
            memory_temperature_c: Some(82.0),
            gpu_util_percent: Some(84.0),
            memory_util_percent: Some(61.0),
            memory_used_bytes: Some(13_099_941_888),
            memory_total_bytes: Some(25_769_803_776),
            power_watts: Some(318.0),
            core_clock_mhz: Some(1815),
            memory_clock_mhz: Some(9751),
            fan_percent: Some(68.0),
        })
    }

    fn operating_limits(&self, id: &GpuId) -> Result<GpuOperatingLimits, ControlError> {
        if Self::mock_device().id != *id {
            return Err(ControlError::GpuNotFound(id.uuid.clone()));
        }

        Ok(GpuOperatingLimits {
            power: PowerLimitInfo {
                current_watts: Some(330.0),
                default_watts: Some(350.0),
                enforced_watts: Some(330.0),
                min_watts: Some(100.0),
                max_watts: Some(400.0),
            },
            clocks: ClockLimitInfo {
                max_graphics_mhz: Some(2100),
                max_memory_mhz: Some(10000),
                supported_application_memory_mhz: Some(vec![9501, 810]),
                graphics_clocks_for_memory_mhz: Some(9501),
                supported_application_graphics_mhz: Some(vec![2100, 1950, 1800]),
            },
            fans: FanLimitInfo {
                min_percent: Some(30),
                max_percent: Some(100),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_one_test_gpu() {
        let backend = MockBackend;
        let devices = backend.enumerate().expect("enumeration should work");

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id.uuid, "GPU-MOCK-0000");
    }

    #[test]
    fn exposes_test_limits() {
        let backend = MockBackend;
        let id = backend.enumerate().expect("enumerate")[0].id.clone();
        let limits = backend.operating_limits(&id).expect("read limits");
        assert_eq!(limits.power.current_watts, Some(330.0));
        assert_eq!(limits.clocks.graphics_clocks_for_memory_mhz, Some(9501));
    }

    #[test]
    fn limits_reject_unknown_gpu() {
        let result = MockBackend.operating_limits(&GpuId {
            index: 9,
            uuid: "missing".to_string(),
        });
        assert!(matches!(result, Err(ControlError::GpuNotFound(_))));
    }

    #[test]
    fn telemetry_rejects_unknown_gpu() {
        let backend = MockBackend;
        let result = backend.telemetry(&GpuId {
            index: 9,
            uuid: "missing".to_string(),
        });

        assert!(matches!(result, Err(ControlError::GpuNotFound(_))));
    }
}
