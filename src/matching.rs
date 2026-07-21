// Adapted from CHWD 1.23.0 src/data.rs hardware-ID matching.
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Evidence, PciDevice, unique_graphics_vendors};

struct HardwareId {
    class_ids: &'static [&'static str],
    vendor_ids: &'static [&'static str],
    device_ids: &'static [&'static str],
}

struct Profile {
    id: &'static str,
    environment_types: &'static [&'static str],
    hardware_ids: &'static [HardwareId],
}

const GRAPHICS_CLASSES: &[&str] = &["0300", "0302", "0380"];
const VGA_CLASS: &[&str] = &["0300"];
const ANY_DEVICE: &[&str] = &["*"];

const PROFILES: &[Profile] = &[
    Profile {
        id: "graphics.amd",
        environment_types: &[],
        hardware_ids: &[HardwareId {
            class_ids: GRAPHICS_CLASSES,
            vendor_ids: &["1002"],
            device_ids: ANY_DEVICE,
        }],
    },
    Profile {
        id: "graphics.intel",
        environment_types: &[],
        hardware_ids: &[HardwareId {
            class_ids: GRAPHICS_CLASSES,
            vendor_ids: &["8086"],
            device_ids: ANY_DEVICE,
        }],
    },
    Profile {
        id: "graphics.nvidia",
        environment_types: &[],
        hardware_ids: &[HardwareId {
            class_ids: GRAPHICS_CLASSES,
            vendor_ids: &["10de"],
            device_ids: ANY_DEVICE,
        }],
    },
    Profile {
        id: "vm.qemu",
        environment_types: &["kvm", "qemu"],
        hardware_ids: &[HardwareId {
            class_ids: VGA_CLASS,
            vendor_ids: &["1af4", "1b36", "1013", "1234"],
            device_ids: ANY_DEVICE,
        }],
    },
    Profile {
        id: "vm.virtualbox",
        environment_types: &["oracle"],
        hardware_ids: &[HardwareId {
            class_ids: VGA_CLASS,
            vendor_ids: &["80ee"],
            device_ids: ANY_DEVICE,
        }],
    },
    Profile {
        id: "vm.vmware",
        environment_types: &["vmware"],
        hardware_ids: &[HardwareId {
            class_ids: VGA_CLASS,
            vendor_ids: &["15ad"],
            device_ids: ANY_DEVICE,
        }],
    },
];

pub(crate) fn matching_profile_ids(evidence: &Evidence) -> Vec<String> {
    let mut matches = PROFILES
        .iter()
        .filter(|profile| environment_matches(profile, evidence))
        .filter(|profile| !get_all_devices_of_profile(&evidence.pci, profile).is_empty())
        .map(|profile| profile.id.to_string())
        .collect::<Vec<_>>();

    if let Some(profile_id) = evidence.virtualization.as_deref().and_then(vm_profile_id) {
        matches.push(profile_id.to_string());
    }

    if unique_graphics_vendors(evidence).len() > 1 {
        matches.push("graphics.hybrid".to_string());
    }
    matches
}

fn vm_profile_id(environment: &str) -> Option<&'static str> {
    match environment {
        "kvm" | "qemu" => Some("vm.qemu"),
        "oracle" => Some("vm.virtualbox"),
        "vmware" => Some("vm.vmware"),
        "microsoft" => Some("vm.hyperv"),
        "xen" => Some("vm.xen"),
        _ => None,
    }
}

fn environment_matches(profile: &Profile, evidence: &Evidence) -> bool {
    profile.environment_types.is_empty()
        || evidence
            .virtualization
            .as_deref()
            .is_some_and(|value| profile.environment_types.contains(&value))
}

fn get_all_devices_of_profile(devices: &[PciDevice], profile: &Profile) -> Vec<usize> {
    let mut found_indices = Vec::new();

    for hardware_id in profile.hardware_ids {
        let mut found_device = false;
        for (index, device) in devices.iter().enumerate() {
            if id_matches(hardware_id.class_ids, &device.class_id)
                && id_matches(hardware_id.vendor_ids, &device.vendor_id)
                && id_matches(hardware_id.device_ids, &device.device_id)
            {
                found_device = true;
                found_indices.push(index);
            }
        }
        if !found_device {
            return Vec::new();
        }
    }

    found_indices
}

fn id_matches(allowed: &[&str], actual: &str) -> bool {
    allowed.iter().any(|value| *value == "*" || value.eq_ignore_ascii_case(actual))
}
