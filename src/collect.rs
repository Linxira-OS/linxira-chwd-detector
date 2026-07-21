// SPDX-License-Identifier: GPL-3.0-only

use crate::{CpuEvidence, DmiEvidence, Evidence, PciDevice, Warning};
use std::fs;

const PCI_ROOT: &str = "/sys/bus/pci/devices";
const CPU_INFO: &str = "/proc/cpuinfo";
const DMI_ROOT: &str = "/sys/devices/virtual/dmi/id";

pub(crate) fn collect() -> (Evidence, Vec<Warning>) {
    let mut warnings = Vec::new();
    let pci = collect_pci(&mut warnings);
    let dmi = DmiEvidence {
        system_vendor: read_optional(
            &format!("{DMI_ROOT}/sys_vendor"),
            "dmi.system_vendor",
            &mut warnings,
        ),
        product_name: read_optional(
            &format!("{DMI_ROOT}/product_name"),
            "dmi.product_name",
            &mut warnings,
        ),
        chassis_type: read_optional(
            &format!("{DMI_ROOT}/chassis_type"),
            "dmi.chassis_type",
            &mut warnings,
        ),
    };
    let cpu = collect_cpu(&mut warnings);
    let virtualization = infer_virtualization(&dmi, &pci);
    if virtualization.is_none() && dmi.system_vendor.is_none() && dmi.product_name.is_none() {
        warnings.push(Warning {
            source: "virtualization".to_string(),
            message: "could not infer VM type because DMI evidence is unavailable".to_string(),
        });
    }

    (Evidence { pci, dmi, cpu, virtualization }, warnings)
}

fn collect_pci(warnings: &mut Vec<Warning>) -> Vec<PciDevice> {
    let entries = match fs::read_dir(PCI_ROOT) {
        Ok(entries) => entries,
        Err(error) => {
            warnings.push(io_warning("pci", error));
            return Vec::new();
        }
    };
    let mut devices = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warnings.push(io_warning("pci", error));
                continue;
            }
        };
        let bus_id = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        let values = (
            fs::read_to_string(path.join("class")),
            fs::read_to_string(path.join("vendor")),
            fs::read_to_string(path.join("device")),
        );
        match values {
            (Ok(class_id), Ok(vendor_id), Ok(device_id)) => devices.push(PciDevice {
                bus_id,
                class_id: class_id.trim().to_string(),
                vendor_id: vendor_id.trim().to_string(),
                device_id: device_id.trim().to_string(),
            }),
            _ => warnings.push(Warning {
                source: format!("pci.{bus_id}"),
                message: "ignored device with unreadable identity evidence".to_string(),
            }),
        }
    }
    devices
}

fn collect_cpu(warnings: &mut Vec<Warning>) -> CpuEvidence {
    let content = match fs::read_to_string(CPU_INFO) {
        Ok(content) => content,
        Err(error) => {
            warnings.push(io_warning("cpu", error));
            return CpuEvidence::default();
        }
    };
    let field = |name: &str| {
        content.lines().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            (key.trim() == name).then(|| value.trim().to_string())
        })
    };
    CpuEvidence { vendor: field("vendor_id"), family: field("cpu family"), model: field("model") }
}

fn read_optional(path: &str, source: &str, warnings: &mut Vec<Warning>) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(value) if !value.trim().is_empty() => Some(value.trim().to_string()),
        Ok(_) => {
            warnings.push(Warning {
                source: source.to_string(),
                message: "evidence was empty".to_string(),
            });
            None
        }
        Err(error) => {
            warnings.push(io_warning(source, error));
            None
        }
    }
}

fn io_warning(source: &str, error: std::io::Error) -> Warning {
    Warning { source: source.to_string(), message: format!("evidence unavailable: {error}") }
}

pub(crate) fn infer_virtualization(dmi: &DmiEvidence, pci: &[PciDevice]) -> Option<String> {
    let dmi_text = format!(
        "{} {}",
        dmi.system_vendor.as_deref().unwrap_or_default(),
        dmi.product_name.as_deref().unwrap_or_default()
    )
    .to_ascii_lowercase();

    if dmi_text.contains("vmware") {
        return Some("vmware".to_string());
    }
    if dmi_text.contains("virtualbox") || dmi_text.contains("innotek") {
        return Some("oracle".to_string());
    }
    if dmi_text.contains("qemu") || dmi_text.contains("kvm") || dmi_text.contains("bochs") {
        return Some("kvm".to_string());
    }
    if dmi_text.contains("hyper-v") || dmi_text.contains("virtual machine") {
        return Some("microsoft".to_string());
    }
    if dmi_text.contains("xen") {
        return Some("xen".to_string());
    }

    if pci.iter().any(|device| device.vendor_id.eq_ignore_ascii_case("80ee")) {
        Some("oracle".to_string())
    } else if pci.iter().any(|device| device.vendor_id.eq_ignore_ascii_case("15ad")) {
        Some("vmware".to_string())
    } else if pci.iter().any(|device| {
        matches!(device.vendor_id.to_ascii_lowercase().as_str(), "1af4" | "1b36" | "1234")
    }) {
        Some("kvm".to_string())
    } else {
        None
    }
}
