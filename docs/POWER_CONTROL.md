# M3 power-limit controls: current development stage

**Status:** the policy engine, one-shot NVML helper and optional Polkit packaging are implemented. Hardware writes and GUI apply are **not yet validated/enabled**. No privileged component is installed by the existing `scripts/install-user.sh` or launched during normal monitoring.

## Why a separate helper?

NVML's `set_power_management_limit` needs admin/root permissions on Linux. The session D-Bus daemon must remain unprivileged. The dedicated helper is intentionally narrow and **must never take arbitrary filesystem paths, commands or shell input**. A root-owned helper is necessary before a GUI may use Polkit safely; executing a user-writable binary via `pkexec` would be insecure.

## Safe testing: read-only inspect

After pulling updates, build the helper:

```bash
cargo test -p nvidia-control-power
cargo test -p nvidia-control-power-helper
cargo build -p nvidia-control-power-helper
```

Use the previously observed limits from the read-only NVML daemon to derive the **current** GPU UUID and power in whole watts. The arguments of `inspect` are **integer milliwatts**, not watts. Example for an RTX 3090 reporting current power limit 350 W:

```bash
./target/debug/nvidia-control-power-helper inspect GPU-YOUR-ACTUAL-UUID 350000 300000
```

This prints a validated plan only. It does not call a setter and needs no sudo. Replace the example UUID with the actual one from the device's current output. If the limit changed, the command refuses the stale `350000` value.

The helper enforces all of the following **again in the privileged process** before writing:

1. Exactly one `GPU-...` UUID matched to the physical NVML device.
2. Current mW must match the caller's expected current mW.
3. Driver-reported min/max constraints must be coherent and `<= 1,000,000 mW`.
4. Requested power must be inside the NVML range and **not above factory default**.
5. A successful setter must be confirmed by NVML readback.
6. A failed/mismatched setter must trigger a best-effort restoration and restoration readback. If rollback fails, the helper reports a **critical** failure and cannot guarantee the state.

**No actual write is guaranteed safe by automated tests.** Device behavior, permission and rollback must be validated on physical hardware in a controlled follow-up.

## Optional administrator installation (later, explicitly)

The script `scripts/install-power-helper.sh` is **not part of the normal application installation**. It requires a confirmation prompt and uses `sudo install` to copy the compiled release binary and policy to root-controlled paths.

```bash
cargo build --release -p nvidia-control-power-helper
bash scripts/install-power-helper.sh
```

Installed locations:

- `/usr/libexec/nvidia-control-power-helper` (root-owned executable, mode 0755).
- `/usr/share/polkit-1/actions/io.github.chmodmasx.nvidiacontrol.policy` (root-owned action, mode 0644).

The policy requires administrator authentication for each `apply` (no `auth_admin_keep`) and matches the fixed binary path plus first argument `apply`.

**Do not execute the helper from `target/debug`, `target/release` or `~/.local/bin` with `sudo` or `pkexec`.** Those are user-writable and are not a safe privileged installation.

An authenticated operation, once hardware validation has been explicitly approved, has the shape:

```text
pkexec /usr/libexec/nvidia-control-power-helper apply GPU-... EXPECTED_MW TARGET_MW
```

This is reference documentation, **not an instruction to run it now**. The helper changes one power limit per invocation; it does not save profiles or create a background writer. To uninstall privileged components separately, use `bash scripts/uninstall-power-helper.sh`.

## Known limitations and future gates

- `GetInventory` still reports `read_only` for power-limit capabilities. Read support **does not prove** write support. Actual write permission may depend on driver, kernel and GPU.
- This is not a fault-proof transaction if the driver crashes, the system loses power, or the helper is terminated mid-write. Rollback is best-effort after detectable failures only.
- Changes may last until GPU reset/driver reload; no persistence profile or timeout-based watchdog has been added.
- There is no GUI apply/revert workflow, explicit confirmation panel, progress feedback, or forced inventory refresh yet. Those must land before enabling normal end-user writes.
- Multiple concurrent administrative invocations are not serialized by a global lock yet; this needs addressing before exposing GUI control.
- Only NVML power management is in scope. Fan control and V/F tuning are separate adapters with their own safety requirements.
