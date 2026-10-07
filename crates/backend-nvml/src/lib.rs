use nvidia_control_core::{
    AccessLevel, CapabilitySet, ControlError, GpuBackend, GpuDevice, GpuId, TelemetrySnapshot,
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
                ..CapabilitySet::default()
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

fn nvml_error(operation: &str, error: NvmlError) -> ControlError {
    ControlError::Backend(format!("NVML could not {operation}: {error}"))
}

#[cfg(test)]
mod tests {
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
