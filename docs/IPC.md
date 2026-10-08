# Session D-Bus API — version 1

The current API is an unprivileged *read-only* transport. It is not the future privileged tuning IPC.

## Identity

| Field | Value |
| --- | --- |
| Bus | Session |
| Service | `io.github.chmodmasx.NvidiaControl` |
| Object | `/io/github/chmodmasx/NvidiaControl` |
| Interface | `io.github.chmodmasx.NvidiaControl1` |

## Methods

- `GetApiVersion() → u`: returns `1`
- `GetSnapshot() → s`: JSON array using the same schema as `nvidia-control-daemon --once`

Each snapshot is a list of records:
```json
[
  {
    "backend": "mock",
    "device": {"id": {"index": 0, "uuid": "GPU-MOCK-0000"}, "name": "NVIDIA Mock GPU"},
    "telemetry": {"temperature_c": 67, "power_watts": 318},
    "operating_limits": {"power": {"current_watts": 330}}
  }
]
```
The sample is abbreviated; the actual API includes all existing properties. `null` means unknown/not readable for a measurement; `unknown` is a capability evaluation state and differs from confirmed `unsupported`.

## Lifecycle

1. Start `nvidia-control-daemon --session` as the desktop user.
2. The daemon claims its bus name and serves requests on the session bus.
3. Qt requests snapshots asynchronously every 1000 ms, skipping requests if an earlier call has not returned.
4. If the service disappears, the UI clears stale measurements, reports disconnection and retries.
5. `--once` remains available for scripts and debugging.

Service activation and a systemd user unit are future packaging tasks, not prerequisites for the current development workflow.

## Versioning and permissions

- The interface name includes `1` for breaking changes.
- The version method makes client compatibility checks possible.
- All methods are read-only.
- No privileged operation should be added to this interface. Future writes belong to narrowly scoped, Polkit-authorized services; the GUI is never root.
- No hardware-specific read path exists in the Qt process. It deals only with the domain JSON objects.
- Current UI displays the first GPU even though the API returns all of them. Multi-GPU selection remains a separate UX task.
