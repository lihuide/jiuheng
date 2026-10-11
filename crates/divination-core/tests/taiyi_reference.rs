mod support {
    pub mod taiyi_reference;
}

use divination_core::{taiyi_seventy_two_board_frame_from_count, TaiyiDun};
use serde_json::Value;
use support::taiyi_reference::{compare, fixture};

#[test]
fn all_144_static_reference_boards_have_exactly_the_reviewed_four_differences() {
    let reference = fixture();
    let report = compare(&reference);
    assert_eq!(report["summary"], reference["expected_summary"]);
    assert_eq!(report["differences"], reference["documented_differences"]);
    assert_eq!(report["runtime_rule_count"], 0);
    assert_eq!(report["third_party_code_executed"], false);
    assert_eq!(report["ancient_evidence_verified"], false);
}

#[test]
fn reference_identity_is_linked_to_existing_pin_and_never_to_runtime_rules() {
    let reference = fixture();
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../data/research/reference-source-manifest.json"
    ))
    .unwrap();
    let project = manifest["projects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|project| project["project_id"] == "kintaiyi")
        .unwrap();
    let config = project["selected_files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == reference["source"]["path"])
        .unwrap();
    assert_eq!(reference["commit_sha"], project["commit_sha"]);
    assert_eq!(reference["source"]["sha256"], config["sha256"]);
    assert_eq!(reference["source"]["bytes"], config["bytes"]);
    assert_eq!(
        project["execution_gate"]["crosscheck_execution_allowed"],
        false
    );
    assert_eq!(reference["license"]["expression"], "MIT");
    let license = project["selected_files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == "LICENSE")
        .unwrap();
    assert_eq!(
        reference["license"]["original_manifest_bytes"],
        license["bytes"]
    );
    assert_eq!(
        reference["license"]["original_manifest_sha256"],
        license["sha256"]
    );
    assert_eq!(
        project["static_literal_comparison"]["compared_board_count"],
        144
    );
    assert_eq!(
        project["static_literal_comparison"]["different_cell_count"],
        4
    );
    assert_eq!(
        project["static_literal_comparison"]["third_party_code_execution_allowed"],
        false
    );
    assert_eq!(reference["extraction"]["module_imported"], false);
    assert_eq!(reference["extraction"]["pickle_deserialized"], false);
    assert_eq!(
        reference["extraction"]["third_party_function_executed"],
        false
    );
    assert_eq!(reference["extraction"]["ancient_evidence_verified"], false);
    assert_eq!(
        reference["extraction"]["retained_columns"],
        serde_json::json!(["main_count", "guest_count"])
    );
    for flag in [
        "runtime_ingestion_allowed",
        "rule_evidence_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert_eq!(reference[flag], false, "unsafe research flag: {flag}");
    }
    assert_eq!(reference["runtime_rule_count"], 0);
}

#[test]
fn comparison_is_deterministic_and_reference_drift_changes_its_report() {
    let mut reference = fixture();
    let original = compare(&reference);
    assert_eq!(compare(&reference), original);
    reference["yang"][0][0] = Value::from(8);
    let changed = compare(&reference);
    assert_ne!(changed["differences"], original["differences"]);
    assert_eq!(changed["summary"]["different_cells"], 5);
}

#[test]
fn reference_count_cycles_never_turn_exact_multiples_into_zero() {
    for dun in [TaiyiDun::Yang, TaiyiDun::Yin] {
        for count in 1..=72 {
            let first = taiyi_seventy_two_board_frame_from_count(count, dun).unwrap();
            for cycle in [1, 2, 9] {
                let repeated =
                    taiyi_seventy_two_board_frame_from_count(count + 72 * cycle, dun).unwrap();
                assert_eq!(repeated.position_in_72, first.position_in_72);
                assert_eq!(repeated.main_count, first.main_count);
                assert_eq!(repeated.guest_count, first.guest_count);
                assert!(!repeated.interpretation_eligible);
            }
        }
    }
}
