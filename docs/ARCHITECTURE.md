# Architecture

## Goal

Nvidia-Control must remain maintainable even when NVIDIA changes an undocumented API, a compositor changes its integration surface, or a distribution changes its package-management workflow.

The application therefore treats every external integration as a replaceable backend rather than allowing hardware- or desktop-specific code to spread through the UI and daemon.

## Process model

```text
+----------------------------+
| Qt 6 / QML GUI             |
| unprivileged               |
+-------------+--------------+
              |
              | stable D-Bus contract
              v
+-------------+--------------+
| nvidia-control-daemon      |
| policy + orchestration     |
+-------------+--------------+
              |
       +------+-------------------------------+
       |                                      |
       v                                      v
+------+-------+                      +-------+--------+
| safe/read    |                      | privileged     |
| backends     |                      | write adapters |
+--------------+                      +----------------+
```

The GUI is a client. It must not contain NVML, NVKMS, NVAPI, package-manager or Proton implementation details.

## Core contracts

`control-core` owns the domain model:

- GPU identity
- capability discovery
- telemetry
- feature availability
- errors
- backend traits

It does **not** know how those values are obtained.

A backend translates one external technology into the core model. This lets us replace or disable a backend without changing consumers.

## Backend boundaries

### backend-nvml

Primary supported NVIDIA hardware backend.

Responsibilities:

- enumerate GPUs
- stable identity/UUID
- temperature
- utilization
- VRAM usage
- clocks
- power telemetry
- read-only power-limit constraints (current, default, enforced and allowed range)
- maximum clocks and legacy application-clock combinations (read-only diagnostics)
- fan setpoint range when the driver exposes it
- supported stable tuning operations

Writes must be added one operation at a time and guarded by explicit capabilities.

### backend-nvapi

Experimental Linux NvAPI adapter.

Intended for functionality exposed by `libnvidia-api.so` that is not available through stable NVML, including V/F functionality where verified.

Rules:

- never make it a hard dependency
- runtime capability detection
- driver-version guards
- failures cannot disable NVML functionality
- no undocumented call is exposed directly to the UI

### backend-nvkms

Experimental display/NVIDIA-specific adapter.

Potential responsibilities include Digital Vibrance and NVIDIA-specific display controls that genuinely belong below the compositor.

It must not duplicate controls owned by the Wayland compositor.

### backend-display

Desktop/compositor adapters:

```text
backend-display
  kde/
  gnome/
  wlroots/
  cosmic/
```

Responsibilities may include resolution, refresh rate, HDR and VRR when those operations are compositor-owned.

### backend-proton

Gaming integration layer.

Responsibilities:

- discover Steam/Proton titles
- manage per-game launch environment
- DXVK-NVAPI settings
- NGX updater integration
- DLSS SR/RR/FG overrides
- DLAA
- custom scaling
- Reflex
- Smooth Motion capability/configuration where supported

The game-profile data model should not depend on Steam so Heroic, Lutris and other launchers can reuse it.

### backend-capture

Capture pipeline behind a common interface:

- PipeWire screen/game capture
- NVENC encode
- recording
- screenshots
- ring-buffer Instant Replay

### backend-packages

Distribution-specific driver/package management.

Adapters are isolated by distribution:

```text
apt
dnf
pacman
rpm-ostree
nix
```

The project must not install NVIDIA's `.run` package as a generic update mechanism.

## Session IPC and Qt frontend

A concrete, versioned read-only D-Bus interface is documented in [IPC.md](IPC.md). The Rust daemon exposes backward-compatible `GetSnapshot` plus `GetInventory` and `GetTelemetry` on the **session bus**. Inventory (GPU IDs, capabilities, power/clock/fan constraints) is cached by the daemon for 60 seconds and refreshed by the UI once per minute. The hot path reads only the selected GPU's telemetry once per second. Both calls are asynchronous and independent. The GUI never executes NVIDIA APIs or commands. See [IPC.md](IPC.md).

## Frontend telemetry history

The independent `ui/src/TelemetryHistory.{h,cpp}` module subscribes to successful `TelemetryBridge::telemetryReceived` events. It keeps timestamped in-memory telemetry samples for at most **60 minutes and 3601 entries**, without modifying the daemon, NVML, or the D-Bus v1 schema.

The `HistoryChart.qml` component queries that store for the selected 5, 15 or 60 minute window and plots only while visible. Six metrics are available: GPU load, GPU temperature, power, used VRAM (converted from bytes to GiB for display only), graphics clock and memory clock. An absent reading is treated as a discontinuity, and gaps longer than 3.5 seconds do not become misleading interpolated curves. Restarting the UI clears its local history; reconnecting to the same GPU preserves recorded samples, while a GPU identity change resets it. Unit tests exercise bounded retention, window selection, missing data and reset behavior.

History is a **presentation-side concern** and remains independent of backend abstractions.

## On-demand service activation

The optional [per-user installer](ACTIVATION.md) installs the read-only Rust daemon under `~/.local/bin`, a `Type=dbus` systemd user unit and a matching D-Bus session service file containing `SystemdService=` plus the executable `Exec=` fallback. The frontend initiates a normal `GetInventory` call; systemd (when integrated with the desktop session bus) starts the daemon only on demand. The unit is **not enabled** at login, and neither the unit nor the GUI runs as root. Distribution packaging is separate from this per-user development installation. Persistent storage or exported time-series data, if needed, should be a separate optional module rather than a requirement for telemetry.

`--once` remains the CLI diagnostic mode, and `--session` hosts the read-only service. No root privileges or Polkit are required for either current path.

## M3 privileged power-limit boundary

The normal session D-Bus service and Qt client remain unprivileged and read-only. `control-power` is a pure Rust library containing the limited power-change transaction: compare expected/current, validate NVML limits and factory ceiling, write, verify readback, and attempt rollback with its own readback if verification fails.

The separate `nvidia-control-power-helper` is the **only** new executable that knows about `set_power_management_limit`. It accepts only `inspect` (read-only) or `apply` (root-only). It identifies a GPU by UUID, not caller-controlled device index, initializes NVML itself, and rechecks current values and limits before any set operation. Failed setter/readback may leave unknown state if restoration also fails; such failures must be treated as critical and displayed explicitly.

The helper is not part of the user's normal systemd session unit. Optional installation uses an explicit root-owned executable under `/usr/libexec`, a root-owned Polkit action requiring `auth_admin` for each apply, and no retained authorization. A helper installed into `~/.local/bin` or built in a user-writable checkout **must never be executed as root**. Future Qt authenticated calls should use only the fixed root-owned helper path.

This milestone has **no actual hardware write validation yet**. See [POWER_CONTROL.md](POWER_CONTROL.md).

## Privilege model

Read-only telemetry should remain unprivileged whenever the kernel/driver allows it.

Operations such as power limits, clocks, fan policies or package changes are routed through narrowly scoped daemon methods and authorized with Polkit.

The GUI itself never runs as root.

## Capability model

The UI must never infer support from a GPU marketing name alone.

Each feature reports an access level:

- `unknown`: feature availability has not been established by the active backends (default)
- `unsupported`: confirmed unavailable by relevant capability checks
- `read_only`
- `read_write`
- `experimental_read_write`

Reading a setting through NVML **does not prove write access**. In the current read-only backend, a successful getter reports `read_only`. An unsuccessful getter remains `unknown` because another backend, driver revision or permission context might still provide the feature. Do not promote capabilities to writable based on card model or a successful read.

The final capability is computed from the complete environment where relevant:

```text
GPU
+ driver
+ backend availability
+ compositor
+ game/runtime
= exposed capability
```

Example: Smooth Motion may exist in the installed driver but remain unavailable on unsupported GPU generations.

## Failure isolation

A failed experimental NvAPI probe must not prevent:

- GPU telemetry
- game discovery
- display management
- package management

Each backend should eventually expose its own health state so the GUI can explain a degraded feature instead of failing globally.

## Prototype contract

The prototype now implements:

1. the core domain model and NVML/mock backend traits;
2. read-only GPU identification, telemetry and operating limits;
3. a versioned read-only session D-Bus service;
4. a Qt 6/QML dashboard that uses only the D-Bus API;
5. CI checks for Rust, D-Bus mock smoke test and Qt compilation.

The frontend is an initial shell with live history graphs. Physical hardware validation of the new history page, multi-GPU selection, packaging and privileged controls remain open.

NVML read-only telemetry has now been validated on a physical RTX 3090 with driver 610.57.04. Read access to power-limit settings and graphics clocks is dynamically probed. A separate `GpuOperatingLimits` snapshot exposes power constraints, max clocks, optional legacy application-clock tables and fan ranges. These readings do not imply permission to write. Privileged writes come later.
