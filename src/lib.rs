// Copyright (C) 2023-2026 Vladislav Nepogodin
// Copyright (C) 2026 Linxira contributors
// SPDX-License-Identifier: GPL-3.0-only

mod collect;
mod matching;

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SCHEMA_VERSION: u32 = 1;
pub const UPSTREAM_CHWD_VERSION: &str = "1.23.0";

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Evidence {
    #[serde(default)]
    pub pci: Vec<PciDevice>,
    #[serde(default)]
    pub dmi: DmiEvidence,
    #[serde(default)]
    pub cpu: CpuEvidence,
    #[serde(default)]
    pub virtualization: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PciDevice {
    pub bus_id: String,
    pub class_id: String,
    pub vendor_id: String,
    pub device_id: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct DmiEvidence {
    #[serde(default)]
    pub system_vendor: Option<String>,
    #[serde(default)]
    pub product_name: Option<String>,
    #[serde(default)]
    pub chassis_type: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct CpuEvidence {
    #[serde(default)]
    pub vendor: Option<String>,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Warning {
    pub source: String,
    pub message: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct DetectorMetadata {
    pub name: &'static str,
    pub version: &'static str,
    pub upstream_chwd: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct DetectionReport {
    pub schema_version: u32,
    pub detector: DetectorMetadata,
    pub evidence: Evidence,
    pub profile_ids: Vec<String>,
    pub warnings: Vec<Warning>,
}

pub fn collect_and_detect() -> DetectionReport {
    let (evidence, warnings) = collect::collect();
    detect(evidence, warnings)
}

pub fn detect(mut evidence: Evidence, mut warnings: Vec<Warning>) -> DetectionReport {
    evidence.pci = evidence
        .pci
        .into_iter()
        .filter_map(|device| normalize_device(device, &mut warnings))
        .collect();
    evidence.pci.sort_by(|left, right| left.bus_id.cmp(&right.bus_id));
    evidence.virtualization = evidence
        .virtualization
        .map(|value| value.to_ascii_lowercase())
        .or_else(|| collect::infer_virtualization(&evidence.dmi, &evidence.pci));

    let mut profile_ids = matching::matching_profile_ids(&evidence);
    add_cpu_profile(&evidence.cpu, &mut profile_ids);
    profile_ids.sort_unstable();
    profile_ids.dedup();

    warnings.sort();
    warnings.dedup();

    DetectionReport {
        schema_version: SCHEMA_VERSION,
        detector: DetectorMetadata {
            name: "linxira-chwd-detector",
            version: env!("CARGO_PKG_VERSION"),
            upstream_chwd: UPSTREAM_CHWD_VERSION,
        },
        evidence,
        profile_ids,
        warnings,
    }
}

fn normalize_device(mut device: PciDevice, warnings: &mut Vec<Warning>) -> Option<PciDevice> {
    device.class_id = match normalize_hex(&device.class_id, true) {
        Some(value) => value,
        None => return malformed_device(&device.bus_id, "class_id", warnings),
    };
    device.vendor_id = match normalize_hex(&device.vendor_id, false) {
        Some(value) => value,
        None => return malformed_device(&device.bus_id, "vendor_id", warnings),
    };
    device.device_id = match normalize_hex(&device.device_id, false) {
        Some(value) => value,
        None => return malformed_device(&device.bus_id, "device_id", warnings),
    };
    if device.bus_id.trim().is_empty() {
        return malformed_device("<empty>", "bus_id", warnings);
    }
    Some(device)
}

fn malformed_device(bus_id: &str, field: &str, warnings: &mut Vec<Warning>) -> Option<PciDevice> {
    warnings.push(Warning {
        source: format!("pci.{bus_id}"),
        message: format!("ignored device with malformed {field}"),
    });
    None
}

fn normalize_hex(value: &str, class_id: bool) -> Option<String> {
    let value = value.trim().strip_prefix("0x").unwrap_or(value.trim());
    let expected = if class_id { [4, 6].as_slice() } else { [4].as_slice() };
    if !expected.contains(&value.len()) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let normalized = value.to_ascii_lowercase();
    Some(if class_id { normalized[..4].to_string() } else { normalized })
}

fn add_cpu_profile(cpu: &CpuEvidence, profile_ids: &mut Vec<String>) {
    match cpu.vendor.as_deref().map(str::to_ascii_lowercase).as_deref() {
        Some("genuineintel") => profile_ids.push("cpu.intel".to_string()),
        Some("authenticamd") => profile_ids.push("cpu.amd".to_string()),
        _ => {}
    }
}

pub fn unique_graphics_vendors(evidence: &Evidence) -> BTreeSet<&str> {
    evidence
        .pci
        .iter()
        .filter(|device| matches!(device.class_id.as_str(), "0300" | "0302" | "0380"))
        .map(|device| device.vendor_id.as_str())
        .collect()
}
