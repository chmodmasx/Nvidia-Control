//! Narrow, testable power-limit transaction policy.
//! No privilege management, NVML, QML, D-Bus or shell execution lives here.
use thiserror::Error;

const MAX_SAFE_MILLIWATTS: u32 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerBounds {
    pub minimum_mw: u32,
    pub maximum_mw: u32,
    pub factory_default_mw: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerPlan {
    pub current_mw: u32,
    pub target_mw: u32,
    pub bounds: PowerBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    Unchanged { milliwatts: u32 },
    Applied { previous_mw: u32, actual_mw: u32 },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PowerError {
    #[error("power settings unavailable: {0}")]
    Device(String),
    #[error("invalid power limits reported by driver")]
    InvalidBounds,
    #[error("GPU power limit changed since the request: expected {expected_mw} mW, actual {actual_mw} mW")]
    Stale { expected_mw: u32, actual_mw: u32 },
    #[error("requested {requested_mw} mW is outside device range {min_mw}..={max_mw} mW")]
    OutOfRange { requested_mw: u32, min_mw: u32, max_mw: u32 },
    #[error("requested {requested_mw} mW exceeds factory default {default_mw} mW; over-default tuning is disabled")]
    OverFactoryDefault { requested_mw: u32, default_mw: u32 },
    #[error("readback or write failure: {cause}; previous setting was restored")]
    Restored { cause: String },
    #[error("CRITICAL: power state could not be restored to {previous_mw} mW after: {cause}; restoration error: {restore_error}")]
    RestoreFailed { previous_mw: u32, cause: String, restore_error: String },
}

pub trait PowerDevice {
    fn current_limit_mw(&mut self) -> Result<u32, String>;
    fn bounds(&mut self) -> Result<PowerBounds, String>;
    fn set_limit_mw(&mut self, new_limit_mw: u32) -> Result<(), String>;
}

pub fn validate(
    current_mw: u32,
    expected_mw: u32,
    target_mw: u32,
    bounds: PowerBounds,
) -> Result<PowerPlan, PowerError> {
    if bounds.minimum_mw == 0
        || bounds.minimum_mw > bounds.maximum_mw
        || bounds.maximum_mw > MAX_SAFE_MILLIWATTS
        || bounds.factory_default_mw < bounds.minimum_mw
        || bounds.factory_default_mw > bounds.maximum_mw
        || current_mw < bounds.minimum_mw
        || current_mw > bounds.maximum_mw
    {
        return Err(PowerError::InvalidBounds);
    }

    if expected_mw != current_mw {
        return Err(PowerError::Stale { expected_mw, actual_mw: current_mw });
    }
    if target_mw < bounds.minimum_mw || target_mw > bounds.maximum_mw {
        return Err(PowerError::OutOfRange {
            requested_mw: target_mw,
            min_mw: bounds.minimum_mw,
            max_mw: bounds.maximum_mw,
        });
    }
    if target_mw > bounds.factory_default_mw {
        return Err(PowerError::OverFactoryDefault {
            requested_mw: target_mw,
            default_mw: bounds.factory_default_mw,
        });
    }

    Ok(PowerPlan { current_mw, target_mw, bounds })
}

// Attempt to restore whenever a write may have occurred. Always verify the
// restoration via a new readback instead of assuming a successful setter call.
fn restore<D: PowerDevice>(device: &mut D, previous_mw: u32, cause: String) -> PowerError {
    let restored = device.set_limit_mw(previous_mw)
        .and_then(|()| device.current_limit_mw())
        .and_then(|actual| {
            if actual == previous_mw {
                Ok(())
            } else {
                Err(format!("restoration readback was {actual} mW"))
            }
        });

    match restored {
        Ok(()) => PowerError::Restored { cause },
        Err(restore_error) => PowerError::RestoreFailed {
            previous_mw,
            cause,
            restore_error,
        },
    }
}

/// Executes one time-bounded transaction with compare-and-set semantics,
/// driver-provided bounds, a factory-default safety ceiling, and readback.
/// A failed/mismatched setter triggers best-effort rollback and verification.
/// Never called by the unprivileged session daemon.
pub fn apply<D: PowerDevice>(
    device: &mut D, expected_mw: u32, target_mw: u32,
) -> Result<ApplyOutcome, PowerError> {
    let current_mw = device.current_limit_mw().map_err(PowerError::Device)?;
    let bounds = device.bounds().map_err(PowerError::Device)?;
    let plan = validate(current_mw, expected_mw, target_mw, bounds)?;

    if plan.target_mw == plan.current_mw {
        return Ok(ApplyOutcome::Unchanged { milliwatts: current_mw });
    }

    if let Err(err) = device.set_limit_mw(plan.target_mw) {
        // A setter can report failure after a partial state change. If the
        // readback cannot prove the original setting, attempt restoration.
        if matches!(device.current_limit_mw(), Ok(value) if value == current_mw) {
            return Err(PowerError::Device(err));
        }
        return Err(restore(device, current_mw, err));
    }

    match device.current_limit_mw() {
        Ok(actual) if actual == plan.target_mw => Ok(ApplyOutcome::Applied {
            previous_mw: current_mw,
            actual_mw: actual,
        }),
        Ok(actual) => Err(restore(
            device, current_mw,
            format!("readback was {actual} mW instead of {} mW", plan.target_mw),
        )),
        Err(err) => Err(restore(
            device, current_mw,
            format!("readback failed: {err}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct FakeDevice {
        current: u32,
        bounds: PowerBounds,
        writes: Vec<u32>,
        fail_on_write: bool,
        ignore_first_write: bool,
        fail_on_restore: bool,
    }
    impl FakeDevice {
        fn new() -> Self {
            Self {
                current: 350_000,
                bounds: PowerBounds { minimum_mw: 100_000, maximum_mw: 400_000, factory_default_mw: 350_000 },
                writes: Vec::new(),
                fail_on_write: false,
                ignore_first_write: false,
                fail_on_restore: false,
            }
        }
    }
    impl PowerDevice for FakeDevice {
        fn current_limit_mw(&mut self) -> Result<u32, String> { Ok(self.current) }
        fn bounds(&mut self) -> Result<PowerBounds, String> { Ok(self.bounds) }
        fn set_limit_mw(&mut self, target: u32) -> Result<(), String> {
            self.writes.push(target);
            if self.fail_on_restore && self.writes.len() > 1 {
                return Err("restore rejected".into());
            }
            if self.fail_on_write { return Err("driver rejected".into()); }
            if self.ignore_first_write && self.writes.len() == 1 { return Ok(()); }
            self.current = target;
            Ok(())
        }
    }

    #[test]
    fn applies_with_readback() {
        let mut device = FakeDevice::new();
        assert_eq!(apply(&mut device, 350_000, 250_000).unwrap(), ApplyOutcome::Applied {
            previous_mw: 350_000, actual_mw: 250_000
        });
        assert_eq!(device.writes, vec![250_000]);
    }
    #[test]
    fn unchanged_never_writes() {
        let mut d = FakeDevice::new();
        assert_eq!(apply(&mut d, 350_000, 350_000).unwrap(), ApplyOutcome::Unchanged { milliwatts: 350_000 });
        assert!(d.writes.is_empty());
    }
    #[test]
    fn stale_or_outside_bounds_never_writes() {
        let mut d = FakeDevice::new();
        assert!(matches!(apply(&mut d, 300_000, 250_000), Err(PowerError::Stale { .. })));
        assert!(matches!(apply(&mut d, 350_000, 50_000), Err(PowerError::OutOfRange { .. })));
        assert!(matches!(apply(&mut d, 350_000, 450_000), Err(PowerError::OutOfRange { .. })));
        assert!(d.writes.is_empty());
    }
    #[test]
    fn forbids_above_factory_default_even_if_driver_supports_it() {
        let mut d = FakeDevice::new();
        assert!(matches!(apply(&mut d, 350_000, 375_000), Err(PowerError::OverFactoryDefault { .. })));
        assert!(d.writes.is_empty());
    }
    #[test]
    fn bad_driver_limits_never_write() {
        let mut d = FakeDevice::new();
        d.bounds.minimum_mw = 0;
        assert_eq!(apply(&mut d, 350_000, 250_000), Err(PowerError::InvalidBounds));
        assert!(d.writes.is_empty());
    }
    #[test]
    fn mismatched_readback_restores_previous_limit() {
        let mut d = FakeDevice::new();
        d.ignore_first_write = true;
        assert!(matches!(apply(&mut d, 350_000, 250_000), Err(PowerError::Restored { .. })));
        assert_eq!(d.current, 350_000);
        assert_eq!(d.writes, vec![250_000, 350_000]);
    }
    #[test]
    fn rollback_failure_is_explicitly_reported() {
        let mut d = FakeDevice::new();
        d.ignore_first_write = true;
        d.fail_on_restore = true;
        assert!(matches!(apply(&mut d, 350_000, 250_000), Err(PowerError::RestoreFailed { .. })));
        assert_eq!(d.writes, vec![250_000, 350_000]);
    }
    #[test]
    fn rejected_write_with_unchanged_state_does_not_attempt_extra_write() {
        let mut d = FakeDevice::new();
        d.fail_on_write = true;
        assert!(matches!(apply(&mut d, 350_000, 250_000), Err(PowerError::Device(_))));
        assert_eq!(d.writes, vec![250_000]);
    }
}
