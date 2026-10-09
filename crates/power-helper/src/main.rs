//! One-shot, narrow privileged helper. Never run the read-only D-Bus daemon
//! or the Qt frontend as root. Privileged application requires an explicitly
//! installed, root-owned binary under /usr/libexec and a Polkit authorization.
use nvidia_control_power::{apply, validate, ApplyOutcome, PowerBounds, PowerDevice};
use nvml_wrapper::{Device, Nvml};
use std::error::Error;
use std::process::ExitCode;

struct NvmlPowerDevice<'a> {
    device: Device<'a>,
}

impl PowerDevice for NvmlPowerDevice<'_> {
    fn current_limit_mw(&mut self) -> Result<u32, String> {
        self.device.power_management_limit().map_err(|e| e.to_string())
    }

    fn bounds(&mut self) -> Result<PowerBounds, String> {
        let constraints = self
            .device
            .power_management_limit_constraints()
            .map_err(|e| e.to_string())?;
        let factory_default_mw = self
            .device
            .power_management_limit_default()
            .map_err(|e| e.to_string())?;
        Ok(PowerBounds {
            minimum_mw: constraints.min_limit,
            maximum_mw: constraints.max_limit,
            factory_default_mw,
        })
    }

    fn set_limit_mw(&mut self, target_mw: u32) -> Result<(), String> {
        self.device
            .set_power_management_limit(target_mw)
            .map_err(|e| e.to_string())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Inspect,
    Apply,
}

struct Arguments {
    mode: Mode,
    uuid: String,
    expected_mw: u32,
    requested_mw: u32,
}

fn parse_arguments(arguments: &[String]) -> Result<Arguments, String> {
    if arguments.len() != 4 {
        return Err("usage: nvidia-control-power-helper inspect|apply GPU-UUID EXPECTED_MW TARGET_MW".into());
    }
    let mode = match arguments[0].as_str() {
        "inspect" => Mode::Inspect,
        "apply" => Mode::Apply,
        _ => return Err("only inspect and apply operations are permitted".into()),
    };
    let uuid = &arguments[1];
    if uuid.len() < 5 || uuid.len() > 128 || !uuid.starts_with("GPU-")
        || !uuid.bytes().all(|value| value.is_ascii_alphanumeric() || value == b'-')
    {
        return Err("invalid GPU UUID".into());
    }

    let expected_mw = arguments[2]
        .parse::<u32>()
        .map_err(|_| "expected power must be an unsigned integer in mW")?;
    let requested_mw = arguments[3]
        .parse::<u32>()
        .map_err(|_| "requested power must be an unsigned integer in mW")?;
    Ok(Arguments { mode, uuid: uuid.clone(), expected_mw, requested_mw })
}

fn run(args: Arguments) -> Result<(), Box<dyn Error>> {
    // Even when started through Polkit, inspect may be run without root.
    // A manually invoked "apply" without authorization fails closed.
    if args.mode == Mode::Apply && unsafe { libc::geteuid() } != 0 {
        return Err("power changes require root via the installed Polkit helper".into());
    }

    let nvml = Nvml::init()?;
    let device = nvml.device_by_uuid(args.uuid.as_str())?;
    if device.uuid()? != args.uuid {
        return Err("resolved GPU UUID mismatch".into());
    }
    let mut power = NvmlPowerDevice { device };
    if args.mode == Mode::Inspect {
        let current = power.current_limit_mw()?;
        let bounds = power.bounds()?;
        let plan = validate(current, args.expected_mw, args.requested_mw, bounds)?;
        println!(
            "VALIDATED (no write): GPU={} current={}mW requested={}mW min={}mW max={}mW default={}mW",
            args.uuid, plan.current_mw, plan.target_mw,
            bounds.minimum_mw, bounds.maximum_mw, bounds.factory_default_mw
        );
        return Ok(());
    }

    match apply(&mut power, args.expected_mw, args.requested_mw)? {
        ApplyOutcome::Unchanged { milliwatts } => {
            println!("UNCHANGED GPU={} power={}mW", args.uuid, milliwatts);
        }
        ApplyOutcome::Applied { previous_mw, actual_mw } => {
            println!(
                "APPLIED GPU={} previous={}mW readback={}mW",
                args.uuid, previous_mw, actual_mw
            );
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let parsed = match parse_arguments(&args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("nvidia-control-power-helper: {message}");
            return ExitCode::FAILURE;
        }
    };
    match run(parsed) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nvidia-control-power-helper: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|x| (*x).to_string()).collect()
    }

    #[test]
    fn valid_inspect_does_not_require_hardware_to_parse() {
        let parsed = parse_arguments(&args(&["inspect", "GPU-aaaa-bbbb", "350000", "250000"])).unwrap();
        assert_eq!(parsed.mode, Mode::Inspect);
        assert_eq!(parsed.requested_mw, 250000);
    }

    #[test]
    fn valid_apply_can_be_parsed_but_not_executed_without_root() {
        let parsed = parse_arguments(&args(&["apply", "GPU-aaaa-bbbb", "350000", "250000"])).unwrap();
        assert_eq!(parsed.mode, Mode::Apply);
    }

    #[test]
    fn rejects_non_gpu_arguments() {
        assert!(parse_arguments(&args(&["apply", "../binary", "350000", "250000"])).is_err());
        assert!(parse_arguments(&args(&["shell", "GPU-a", "350000", "250000"])).is_err());
        assert!(parse_arguments(&args(&["apply", "GPU-a", "-1", "250000"])).is_err());
        assert!(parse_arguments(&args(&["apply", "GPU-a", "350000", "250.5"])).is_err());
        assert!(parse_arguments(&args(&["apply", "GPU-a", "350000"])).is_err());
    }
}
