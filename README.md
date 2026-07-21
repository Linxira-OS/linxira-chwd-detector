# Linxira CHWD Detector

`linxira-chwd-detector` is a small, read-only hardware detector derived from CHWD 1.23.0.
It emits stable profile IDs and versioned JSON for a separate Linxira manager. It does not
install packages, invoke commands, alter profiles, write system state, or accept paths or other
arguments.

## Output contract

The binary reads only these fixed Linux evidence locations:

- `/sys/bus/pci/devices/*/{class,vendor,device}`
- `/sys/devices/virtual/dmi/id/{sys_vendor,product_name,chassis_type}`
- `/proc/cpuinfo`

It writes one JSON document to standard output. `schema_version` versions the contract;
`detector.version` versions this implementation. Arrays are sorted for deterministic output.
Missing evidence is represented by `null` or an empty array and a structured warning. The binary
accepts no command-line arguments, package names, commands, or filesystem paths.

Stable profile IDs currently emitted are:

- `cpu.amd`, `cpu.intel`
- `graphics.amd`, `graphics.intel`, `graphics.nvidia`, `graphics.hybrid`
- `vm.hyperv`, `vm.qemu`, `vm.virtualbox`, `vm.vmware`, `vm.xen`

These IDs describe detected hardware classes. They deliberately contain no package or mutation
policy; a separate manager owns any mapping from IDs to actions.

## Build and test

The crate targets Linux but its fixture tests are platform-independent:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build --release
./target/release/linxira-chwd-detector
```

## License and provenance

This fork is GPL-3.0-only. The complete license is in [LICENSE](LICENSE), and upstream attribution
and the exact fork point are documented in [UPSTREAM.md](UPSTREAM.md). Git history was retained
from the local CHWD repository.
