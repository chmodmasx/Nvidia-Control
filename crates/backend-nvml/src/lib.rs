use nvidia_control_core::{
    AccessLevel, CapabilitySet, ClockLimitInfo, ControlError, FanLimitInfo, GpuBackend,
    GpuDevice, GpuId, GpuOperatingLimits, PowerLimitInfo, TelemetrySnapshot,
};
use nvml_wrapper::{
    enum_wrappers::device::{Clock, TemperatureSensor},
    error::NvmlError,
    Device, Nvml,
};

pub struct NvmlBackend {
    nvml: Nvml,
}

impl NvmlBackend {
    pub fn new() -> Result<Self, ControlError> {
        let nvml = Nvml::init().map_err(|error| {
            ControlError::BackendUnavailable(format!("NVML initialization failed: {error}"))
        })?;

        Ok(Self { nvml })
    }

    fn device_by_id(&self, id: &GpuId) -> Result<Device<'_>, ControlError> {
        self.nvml
            .device_by_uuid(id.uuid.as_str())
            .map_err(|error| match error {
                NvmlError::NotFound | NvmlError::InvalidArg => {
                    ControlError::GpuNotFound(id.uuid.clone())
                }
                other => nvml_error("open GPU by UUID", other),
            })
    }

    fn read_device(
        &self,
        index: u32,
        driver_version: Option<String>,
    ) -> Result<GpuDevice, ControlError> {
        let device = self
            .nvml
            .device_by_index(index)
            .map_err(|error| nvml_error("open GPU by index", error))?;

        let uuid = device
            .uuid()
            .map_err(|error| nvml_error("read GPU UUID", error))?;
        let name = device
            .name()
            .map_err(|error| nvml_error("read GPU name", error))?;

        Ok(GpuDevice {
            id: GpuId { index, uuid },
            name,
            driver_version,
            vbios_version: optional_metric(device.vbios_version())?,
            capabilities: CapabilitySet {
                telemetry: AccessLevel::ReadOnly,
                // A successful getter establishes read access, not write permission.
                power_limit: probe_read_access(device.power_management_limit()),
                clocks: probe_read_access(device.clock_info(Clock::Graphics)),
                // Fan telemetry cannot establish that fan control is writable.
                ..CapabilitySet::default()
            },
        })
    }

    fn read_operating_limits(&self, device: &Device<'_>) -> Result<GpuOperatingLimits, ControlError> {
        let constraints = optional_metric(device.power_management_limit_constraints())?;
        let memory_clocks = optional_metric(device.supported_memory_clocks())?;
        let selected_memory_clock = memory_clocks
            .as_ref()
            .and_then(|clocks| clocks.iter().copied().max());
        let graphics_clocks = selected_memory_clock
            .map(|memory_clock| optional_metric(device.supported_graphics_clocks(memory_clock)))
            .transpose()?
            .flatten();
        let fan_range = optional_metric(device.min_max_fan_speed())?;

        Ok(GpuOperatingLimits {
            power: PowerLimitInfo {
                current_watts: as_watts(optional_metric(device.power_management_limit())?),
                default_watts: as_watts(optional_metric(device.power_management_limit_default())?),
                enforced_watts: as_watts(optional_metric(device.enforced_power_limit())?),
                min_watts: constraints.as_ref().map(|limits| limits.min_limit as f32 / 1_000.0),
                max_watts: constraints.as_ref().map(|limits| limits.max_limit as f32 / 1_000.0),
            },
            clocks: ClockLimitInfo {
                max_graphics_mhz: optional_metric(device.max_clock_info(Clock::Graphics))?,
                max_memory_mhz: optional_metric(device.max_clock_info(Clock::Memory))?,
                supported_application_memory_mhz: memory_clocks,
                graphics_clocks_for_memory_mhz: selected_memory_clock.filter(|_| graphics_clocks.is_some()),
                supported_application_graphics_mhz: graphics_clocks,
            },
            fans: FanLimitInfo {
                min_percent: fan_range.as_ref().map(|range| range.0),
                max_percent: fan_range.as_ref().map(|range| range.1),
            },
        })
    }

    fn read_telemetry(&self, device: &Device<'_>) -> Result<TelemetrySnapshot, ControlError> {
        let temperature_c =
            optional_metric(device.temperature(TemperatureSensor::Gpu))?.map(|value| value as f32);

        let utilization = optional_metric(device.utilization_rates())?;
        let memory = optional_metric(device.memory_info())?;

        let power_watts =
            optional_metric(device.power_usage())?.map(|milliwatts| milliwatts as f32 / 1_000.0);

        let core_clock_mhz = optional_metric(device.clock_info(Clock::Graphics))?;
        let memory_clock_mhz = optional_metric(device.clock_info(Clock::Memory))?;
        let fan_percent = read_average_fan_percent(device)?;

        Ok(TelemetrySnapshot {
            temperature_c,
            hotspot_temperature_c: None,
            memory_temperature_c: None,
            gpu_util_percent: utilization.as_ref().map(|value| value.gpu as f32),
            memory_util_percent: utilization.as_ref().map(|value| value.memory as f32),
            memory_used_bytes: memory.as_ref().map(|value| value.used),
            memory_total_bytes: memory.as_ref().map(|value| value.total),
            power_watts,
            core_clock_mhz,
            memory_clock_mhz,
            fan_percent,
        })
    }
}

impl GpuBackend for NvmlBackend {
    fn backend_name(&self) -> &'static str {
        "nvml"
    }

    fn enumerate(&self) -> Result<Vec<GpuDevice>, ControlError> {
        let count = self
            .nvml
            .device_count()
            .map_err(|error| nvml_error("enumerate GPUs", error))?;
        let driver_version = optional_metric(self.nvml.sys_driver_version())?;

        (0..count)
            .map(|index| self.read_device(index, driver_version.clone()))
            .collect()
    }

    fn telemetry(&self, id: &GpuId) -> Result<TelemetrySnapshot, ControlError> {
        let device = self.device_by_id(id)?;
        self.read_telemetry(&device)
    }

    fn operating_limits(&self, id: &GpuId) -> Result<GpuOperatingLimits, ControlError> {
        let device = self.device_by_id(id)?;
        self.read_operating_limits(&device)
    }
}

fn as_watts(milliwatts: Option<u32>) -> Option<f32> {
    milliwatts.map(|value| value as f32 / 1_000.0)
}

fn read_average_fan_percent(device: &Device<'_>) -> Result<Option<f32>, ControlError> {
    let Some(fan_count) = optional_metric(device.num_fans())? else {
        return Ok(None);
    };

    if fan_count == 0 {
        return Ok(None);
    }

    let mut sum = 0u64;
    let mut samples = 0u32;

    for fan_index in 0..fan_count {
        if let Some(speed) = optional_metric(device.fan_speed(fan_index))? {
            sum += u64::from(speed);
            samples += 1;
        }
    }

    if samples == 0 {
        Ok(None)
    } else {
        Ok(Some(sum as f32 / samples as f32))
    }
}

fn optional_metric<T>(result: Result<T, NvmlError>) -> Result<Option<T>, ControlError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(
            NvmlError::NotSupported
            | NvmlError::NoPermission
            | NvmlError::FunctionNotFound
            | NvmlError::NoData,
        ) => Ok(None),
        Err(error) => Err(nvml_error("read telemetry", error)),
    }
}

/// A failed getter is not enough to classify the whole feature as unsupported:
/// another backend or permission context may expose it.
fn probe_read_access<T>(result: Result<T, NvmlError>) -> AccessLevel {
    if result.is_ok() {
        AccessLevel::ReadOnly
    } else {
        AccessLevel::Unknown
    }
}

fn nvml_error(operation: &str, error: NvmlError) -> ControlError {
    ControlError::Backend(format!("NVML could not {operation}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{as_watts, probe_read_access, AccessLevel, NvmlError};

    #[test]
    fn converts_power_limit_milliwatts_to_watts() {
        assert_eq!(as_watts(Some(350_000)), Some(350.0));
        assert_eq!(as_watts(None), None);
    }

    #[test]
    fn successful_probe_reports_read_only() {
        assert_eq!(probe_read_access(Ok(250_000u32)), AccessLevel::ReadOnly);
    }

    #[test]
    fn unsuccessful_probes_remain_unknown() {
        assert_eq!(
            probe_read_access::<u32>(Err(NvmlError::NotSupported)),
            AccessLevel::Unknown
        );
        assert_eq!(
            probe_read_access::<u32>(Err(NvmlError::NoPermission)),
            AccessLevel::Unknown
        );
    }

    #[test]
    fn converts_milliwatts_without_integer_truncation() {
        let milliwatts = 318_500u32;
        let watts = milliwatts as f32 / 1_000.0;

        assert_eq!(watts, 318.5);
    }

    #[test]
    fn averages_multiple_fans_as_float() {
        let values = [61u32, 68u32, 72u32];
        let average =
            values.iter().map(|value| u64::from(*value)).sum::<u64>() as f32 / values.len() as f32;

        assert!((average - 67.0).abs() < 0.01);
    }
}
