# Session D-Bus API — version 1

This is the unprivileged **read-only** session service. Privileged tuning will use a separate authorized service.

## Identity

| Property | Value |
| --- | --- |
| Bus | Session |
| Service | `io.github.chmodmasx.NvidiaControl` |
| Object | `/io/github/chmodmasx/NvidiaControl` |
| Interface | `io.github.chmodmasx.NvidiaControl1` |

## Methods

| Method | D-Bus signature | Contents | Cadence |
| --- | --- | --- | --- |
| `GetApiVersion` | `() → u` | Version `1` | On connection |
| `GetInventory` | `() → s` | JSON array of `backend`, `device`, `operating_limits` | Startup/reconnection, then 60s |
| `GetTelemetry` | `(u, s) → s` | Single GPU's telemetry JSON; arguments index and UUID | Every 1s |
| `GetSnapshot` | `() → s` | Legacy complete JSON array; still supported | On demand |

The interface name and version remain unchanged because the new methods are additive; existing `GetSnapshot` clients continue working.

Example abbreviated `GetInventory`:
```json
[
  {
    "backend": "mock",
    "device": {
      "id": {"index": 0, "uuid": "GPU-MOCK-0000"},
      "name": "NVIDIA Mock GPU"
    },
    "operating_limits": {"power": {"current_watts": 330.0}}
  }
]
```

Example abbreviated `GetTelemetry(0, "GPU-MOCK-0000")`:
```json
{"temperature_c": 67.0, "power_watts": 318.0, "gpu_util_percent": 84.0}
```

The real responses contain all domain fields. Missing metrics return `null`. The mock's write-capability flags are test fixtures, not evidence of real permissions.

Example CLI queries when using the mock backend:

```bash
busctl --user call io.github.chmodmasx.NvidiaControl \
  /io/github/chmodmasx/NvidiaControl \
  io.github.chmodmasx.NvidiaControl1 GetInventory

busctl --user call io.github.chmodmasx.NvidiaControl \
  /io/github/chmodmasx/NvidiaControl \
  io.github.chmodmasx.NvidiaControl1 GetTelemetry us 0 GPU-MOCK-0000
```

## Caching and reconnection

- **Daemon:** caches enumerated GPUs, capability probes and static operating limits for 60 seconds. Multiple clients share this cache.
- **GUI:** reads the inventory on startup, after reconnection, and every 60 seconds. It reads only the selected GPU's live telemetry every second.
- **Performance:** `GetTelemetry` calls only the backend `telemetry` method, without enumeration, power-limit probes or clock-table queries.
- **Async requests:** inventory and telemetry have independent in-flight guards. Qt discards replies from a previous connection generation and clears measurements when disconnected.
- **Selection:** currently shows the first GPU and preserves that UUID across inventory refresh. Multi-GPU selection is planned.
- **Legacy:** `GetSnapshot` remains fully supported and now reuses the cached inventory.

The inventory TTL also means a device removed while the daemon is running can remain listed until the next refresh. Hotplug-aware invalidation remains a future task.

## Contract guarantees

- Nothing exposed by the current bus service can change hardware.
- `unknown` is not equivalent to confirmed `unsupported`.
- `memory_util_percent` measures memory-controller activity rather than VRAM occupancy.
- Additional methods are additive within interface version 1; breaking changes require a new interface version.
- Per-user D-Bus activation and a Type=dbus systemd user unit can be installed with `bash scripts/install-user.sh` (see [ACTIVATION.md](ACTIVATION.md)). This does not modify the read-only API contract. Manual `--session` remains available for development.
