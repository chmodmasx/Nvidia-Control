# Hardware validation

This document records **observed** behavior of the existing read-only backends. It is not a cross-vendor compatibility guarantee or a license to enable privileged writes.

## NVIDIA GeForce RTX 3090 — driver 610.57.04 (Linux)

Validated by running `cargo run -p nvidia-control-daemon` on physical hardware under an ordinary desktop user account on **2026-10-08**.

- Backend: `nvml`
- GPU: GeForce RTX 3090 (24 GiB)
- NVIDIA driver: `610.57.04`
- VBIOS: `94.02.42.80.42`
- GPU identity/UUID: **intentionally not retained**
- Build and program invocation: **PASS**
- Unit tests: passed in the previous workstation run; the current hardware-limit run confirms the executable's read path

### Observed live telemetry

These are an instantaneous example, not recommended operating targets:

| Value | Observed |
| --- | ---: |
| GPU temperature | 70 °C |
| GPU utilization | 99% |
| GPU memory-controller utilization | 19% |
| Power draw | 336.112 W |
| Graphics clock | 1800 MHz |
| Memory clock | 9501 MHz |
| Fan telemetry | 48% |

Hotspot and VRAM junction temperatures currently return `null` because the corresponding extended backends are not implemented. `memory_util_percent` is controller activity, not VRAM capacity consumption.

### Readable power constraints

| NVML field | Result |
| --- | ---: |
| Current power limit | 350 W |
| Default power limit | 350 W |
| Enforced power limit | 350 W |
| Minimum power limit | 100 W |
| Maximum power limit | 350 W |

The current driver/VBIOS reports **350 W as the maximum configurable power limit through this NVML interface**. A higher setting must not be offered by the stable power-limit UI on this machine.

### Readable clock information

| NVML field | Result |
| --- | ---: |
| Maximum graphics clock | 2100 MHz |
| Maximum memory clock | 9751 MHz |
| Supported legacy application memory clocks | 9751, 9501, 5001, 810, 405 MHz |
| Graphics clock table queried for memory | 9751 MHz |
| Legacy application graphics clock table | 2100 MHz down to 210 MHz, listed in 15 MHz increments |

These are values *advertised by the driver*. They are not guaranteed sustained frequencies, voltage/frequency curve points, stability-test results, or proof that setting them is supported. The legacy application clock interface must not determine the future OC/undervolt implementation by default.

### Readable fan limits

The driver reports a fan setpoint range of **30–100%**. This is not proof that per-fan manual controls, a custom fan curve, or fan writes are supported with the current API, permissions, or display stack.

### Capability semantics observed

- `telemetry`: `read_only`
- `power_limit`: `read_only`
- `clocks`: `read_only`
- `fan_control`, `voltage_frequency_curve`, `digital_vibrance`, gaming features: `unknown` until the responsible adapters perform appropriate checks

### Follow-up validation requirements

1. Validate NVML behavior under additional driver generations and GPUs.
2. Probe an explicit fan-control backend without deriving write support from a fan-setpoint range.
3. Introduce a separate policy/authorization path before **any** tuning writes.
4. Check readback and safe restoration behavior before exposing a setting in the GUI.
5. Investigate hotspot/GDDR6X junction thermals through optional extended NVIDIA backends, without introducing a hard dependency.
