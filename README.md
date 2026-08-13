# Linxira CHWD Detector

Full CHWD 1.23.0 port: hardware detection and driver configuration tool for Linxira OS.

Two binaries are shipped (same code, different entry contracts):

| Binary | Contract |
|---|---|
| `chwd` | Full CHWD CLI: `list` / `autoconfigure` / `install` / `remove` (CachyOS-compatible) |
| `linxira-chwd-detector` | No-argument mode: emits a read-only JSON hardware report for `linxira-hardware-driver-manager` |

## Output contract (`linxira-chwd-detector`, no arguments)

The binary reads only these fixed Linux evidence locations:

- `/sys/bus/pci/devices/*/{class,vendor,device}`
- `/sys/devices/virtual/dmi/id/{sys_vendor,product_name,chassis_type}`
- `/proc/cpuinfo`

It writes one JSON document to standard output. `schema_version` versions the contract;
`detector.version` versions this implementation. Arrays are sorted for deterministic output.
Missing evidence is represented by `null` or an empty array and a structured warning.

Stable profile IDs currently emitted are:

- `cpu.amd`, `cpu.intel`
- `graphics.amd`, `graphics.intel`, `graphics.nvidia`, `graphics.hybrid`
- `vm.hyperv`, `vm.qemu`, `vm.virtualbox`, `vm.vmware`, `vm.xen`

These IDs describe detected hardware classes. They deliberately contain no package or mutation
policy; a separate manager owns any mapping from IDs to actions.

## CHWD CLI (`chwd`)

Any argument routes to the full CHWD CLI (CachyOS-compatible):

- `chwd --list` — list available profiles for detected devices
- `chwd --list-installed` / `chwd --list-all` — list installed / all profiles
- `chwd -a` / `chwd --autoconfigure` — detect hardware and install the best-matching profile
- `chwd -i <profile>` / `chwd --install <profile>` — install a driver profile
- `chwd -r <profile>` / `chwd --remove <profile>` — remove a driver profile

## Build and test

The crate targets Linux (requires `libpci-dev` and `libusb-dev` headers) but its fixture tests
are platform-independent:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build --release
./target/release/chwd -l
./target/release/linxira-chwd-detector
```

## License and provenance

This fork is GPL-3.0-only. The complete license is in [LICENSE](LICENSE), and upstream attribution
and the exact fork point are documented in [UPSTREAM.md](UPSTREAM.md). Git history was retained
from the local CHWD repository.
