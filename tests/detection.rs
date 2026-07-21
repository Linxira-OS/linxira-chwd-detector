use linxira_chwd_detector::{DetectionReport, Evidence, SCHEMA_VERSION, detect};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    evidence: Evidence,
    expected_profile_ids: Vec<String>,
}

const FIXTURES: &[&str] = &[
    include_str!("fixtures/intel.json"),
    include_str!("fixtures/amd.json"),
    include_str!("fixtures/nvidia.json"),
    include_str!("fixtures/hybrid.json"),
    include_str!("fixtures/vm.json"),
];

#[test]
fn detects_expected_profiles_from_fixtures() {
    for source in FIXTURES {
        let fixture: Fixture = serde_json::from_str(source).expect("valid fixture");
        let report = detect(fixture.evidence, Vec::new());
        assert_eq!(report.schema_version, SCHEMA_VERSION);
        assert_eq!(report.profile_ids, fixture.expected_profile_ids);
        assert_stable_json(&report);
    }
}

#[test]
fn malformed_evidence_is_ignored_with_a_warning() {
    let fixture: Fixture = serde_json::from_str(include_str!("fixtures/malformed-evidence.json"))
        .expect("valid fixture");
    let report = detect(fixture.evidence, Vec::new());

    assert_eq!(report.profile_ids, fixture.expected_profile_ids);
    assert!(report.evidence.pci.is_empty());
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(report.warnings[0].source, "pci.0000:00:02.0");
}

#[test]
fn malformed_fixture_input_is_rejected_without_detection() {
    let result = serde_json::from_str::<Fixture>(include_str!("fixtures/malformed-input.json"));
    assert!(result.is_err());
}

#[test]
fn missing_dmi_and_vm_evidence_does_not_panic() {
    let report = detect(Evidence::default(), Vec::new());
    assert!(report.evidence.dmi.product_name.is_none());
    assert!(report.evidence.virtualization.is_none());
    assert!(report.profile_ids.is_empty());
}

#[test]
fn emits_stable_hyperv_profile_without_a_virtual_gpu() {
    let evidence =
        Evidence { virtualization: Some("microsoft".to_string()), ..Evidence::default() };
    let report = detect(evidence, Vec::new());
    assert_eq!(report.profile_ids, vec!["vm.hyperv"]);
}

fn assert_stable_json(report: &DetectionReport) {
    let first = serde_json::to_string(report).expect("serialize report");
    let second = serde_json::to_string(report).expect("serialize report again");
    assert_eq!(first, second);
}
