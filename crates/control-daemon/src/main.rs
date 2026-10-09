use nvidia_control_backend_mock::{DualMockBackend, MockBackend};
use nvidia_control_backend_nvml::NvmlBackend;
use nvidia_control_core::{
    ControlError, GpuBackend, GpuDevice, GpuId, GpuOperatingLimits, TelemetrySnapshot,
};
use serde::Serialize;
use std::process::ExitCode;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const BUS_NAME: &str = "io.github.chmodmasx.NvidiaControl";
const OBJECT_PATH: &str = "/io/github/chmodmasx/NvidiaControl";
const INVENTORY_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Serialize)]
struct InventoryReport {
    backend: &'static str,
    device: GpuDevice,
    operating_limits: GpuOperatingLimits,
}

#[derive(Debug, Serialize)]
struct DeviceReport {
    backend: &'static str,
    device: GpuDevice,
    telemetry: TelemetrySnapshot,
    operating_limits: GpuOperatingLimits,
}

struct CachedInventory {
    loaded_at: Instant,
    devices: Vec<InventoryReport>,
}

struct Service {
    backend: Box<dyn GpuBackend>,
    inventory_cache: Mutex<Option<CachedInventory>>,
}

impl Service {
    fn new(backend: Box<dyn GpuBackend>) -> Self {
        Self {
            backend,
            inventory_cache: Mutex::new(None),
        }
    }

    // Device discovery, capabilities and operating limits are cached in the
    // daemon so several GUI clients do not repeat expensive NVML queries.
    fn inventory(&self) -> Result<Vec<InventoryReport>, ControlError> {
        let mut cache = self
            .inventory_cache
            .lock()
            .map_err(|_| ControlError::Backend("inventory cache lock poisoned".to_string()))?;

        if let Some(entry) = cache.as_ref() {
            if entry.loaded_at.elapsed() < INVENTORY_TTL {
                return Ok(entry.devices.clone());
            }
        }

        let name = self.backend.backend_name();
        let mut devices = Vec::new();
        for device in self.backend.enumerate()? {
            let operating_limits = self.backend.operating_limits(&device.id)?;
            devices.push(InventoryReport {
                backend: name,
                device,
                operating_limits,
            });
        }

        *cache = Some(CachedInventory {
            loaded_at: Instant::now(),
            devices: devices.clone(),
        });
        Ok(devices)
    }

    // Hot path: just the requested GPU's telemetry. No enumeration, static
    // capability probing or legacy application clock tables are read here.
    fn telemetry(&self, id: &GpuId) -> Result<TelemetrySnapshot, ControlError> {
        self.backend.telemetry(id)
    }

    fn snapshot(&self) -> Result<Vec<DeviceReport>, ControlError> {
        self.inventory()?
            .into_iter()
            .map(|item| {
                let telemetry = self.telemetry(&item.device.id)?;
                Ok(DeviceReport {
                    backend: item.backend,
                    device: item.device,
                    telemetry,
                    operating_limits: item.operating_limits,
                })
            })
            .collect()
    }

    fn snapshot_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(&self.snapshot()?)?)
    }

    fn inventory_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(&self.inventory()?)?)
    }

    fn telemetry_json(&self, id: &GpuId) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(&self.telemetry(id)?)?)
    }
}

struct SessionInterface {
    service: Service,
}

#[zbus::interface(name = "io.github.chmodmasx.NvidiaControl1")]
impl SessionInterface {
    /// Backwards-compatible full JSON report. Static data uses a 60s cache.
    fn get_snapshot(&self) -> zbus::fdo::Result<String> {
        self.service
            .snapshot_json()
            .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))
    }

    /// Enumerated GPUs, read capabilities and limits; no telemetry.
    fn get_inventory(&self) -> zbus::fdo::Result<String> {
        self.service
            .inventory_json()
            .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))
    }

    /// Read one GPU's changing sensor values without recomputing inventory.
    fn get_telemetry(&self, index: u32, uuid: String) -> zbus::fdo::Result<String> {
        self.service
            .telemetry_json(&GpuId { index, uuid })
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
        "mock-dual" => Ok(Box::new(DualMockBackend)),
        other => Err(ControlError::BackendUnavailable(format!(
            "unknown backend '{other}'; supported backends: nvml, mock, mock-dual"
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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[derive(Default)]
    struct ReadCounts {
        enumerate: AtomicUsize,
        limits: AtomicUsize,
        telemetry: AtomicUsize,
    }

    struct CountingBackend(Arc<ReadCounts>);

    impl GpuBackend for CountingBackend {
        fn backend_name(&self) -> &'static str {
            "counting-mock"
        }

        fn enumerate(&self) -> Result<Vec<GpuDevice>, ControlError> {
            self.0.enumerate.fetch_add(1, Ordering::SeqCst);
            MockBackend.enumerate()
        }

        fn telemetry(&self, id: &GpuId) -> Result<TelemetrySnapshot, ControlError> {
            self.0.telemetry.fetch_add(1, Ordering::SeqCst);
            MockBackend.telemetry(id)
        }

        fn operating_limits(&self, id: &GpuId) -> Result<GpuOperatingLimits, ControlError> {
            self.0.limits.fetch_add(1, Ordering::SeqCst);
            MockBackend.operating_limits(id)
        }
    }

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

    #[test]
    fn polling_telemetry_does_not_reread_static_hardware_data() {
        let counts = Arc::new(ReadCounts::default());
        let service = Service::new(Box::new(CountingBackend(Arc::clone(&counts))));
        let id = MockBackend.enumerate().expect("mock GPU")[0].id.clone();

        let inventory: serde_json::Value =
            serde_json::from_str(&service.inventory_json().expect("inventory")).expect("JSON");
        assert!(inventory[0].get("telemetry").is_none());
        assert_eq!(inventory[0]["device"]["id"]["uuid"], id.uuid);

        // A second inventory request should use the daemon-side TTL cache.
        service.inventory_json().expect("cached inventory");

        for _ in 0..10 {
            let telemetry: serde_json::Value =
                serde_json::from_str(&service.telemetry_json(&id).expect("telemetry"))
                    .expect("JSON");
            assert!(telemetry["gpu_util_percent"].is_number());
            assert!(telemetry.get("operating_limits").is_none());
        }

        assert_eq!(counts.enumerate.load(Ordering::SeqCst), 1);
        assert_eq!(counts.limits.load(Ordering::SeqCst), 1);
        assert_eq!(counts.telemetry.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn backwards_compatible_snapshot_reuses_static_cache() {
        let counts = Arc::new(ReadCounts::default());
        let service = Service::new(Box::new(CountingBackend(Arc::clone(&counts))));
        service.snapshot_json().expect("first snapshot");
        service.snapshot_json().expect("second snapshot");

        assert_eq!(counts.enumerate.load(Ordering::SeqCst), 1);
        assert_eq!(counts.limits.load(Ordering::SeqCst), 1);
        assert_eq!(counts.telemetry.load(Ordering::SeqCst), 2);
    }
}
