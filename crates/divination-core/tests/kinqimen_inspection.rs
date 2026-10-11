use serde_json::Value;

#[test]
fn pinned_kinqimen_inspection_never_authorizes_execution_or_hourly_charts() {
    let inspection: Value = serde_json::from_str(include_str!(
        "../../../data/research/kinqimen-source-inspection.json"
    ))
    .unwrap();
    let registry: Value = serde_json::from_str(include_str!(
        "../../../data/research/open-source-reference-implementations.json"
    ))
    .unwrap();
    let hourly: Value = serde_json::from_str(include_str!(
        "../../../data/research/qimen-hourly-duty-movement-candidates.json"
    ))
    .unwrap();
    let runtime: Value =
        serde_json::from_str(include_str!("../../../data/rules.seed.json")).unwrap();
    for flag in [
        "full_archive_identity_closed",
        "third_party_code_executed",
        "source_code_vendored",
        "direct_code_copy_allowed",
        "crosscheck_execution_allowed",
        "runtime_ingestion_allowed",
        "rule_evidence_allowed",
        "hourly_movement_implementation_allowed",
    ] {
        assert_eq!(inspection[flag], false, "{flag}");
    }
    assert_eq!(inspection["selected_source_bytes_verified"], true);
    let files = inspection["selected_files"].as_array().unwrap();
    assert_eq!(files.len(), 3);
    for file in files {
        assert!(file["bytes"].as_u64().unwrap() > 0);
        let hash = file["sha256"].as_str().unwrap();
        assert_eq!(hash.len(), 64);
        assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    assert_eq!(inspection["tree_observation"]["truncated"], false);
    let paths = inspection["tree_observation"]["observed_paths"]
        .as_array()
        .unwrap();
    assert_eq!(paths.len(), 19);
    assert!(!paths.iter().any(|value| value
        .as_str()
        .unwrap()
        .to_ascii_lowercase()
        .contains("license")));
    assert_eq!(
        inspection["license_audit"]["linked_license_http_status"],
        404
    );
    assert_eq!(inspection["license_audit"]["license_closed"], false);
    assert_eq!(
        inspection["license_audit"]["redistribution_allowed_by_project_gate"],
        false
    );
    assert_eq!(
        inspection["facsimile_attempt"]["file_bytes_obtained"],
        false
    );
    assert_eq!(
        inspection["facsimile_attempt"]["page_visually_reviewed"],
        false
    );
    assert!(inspection["facsimile_attempt"]["source_sha256"].is_null());
    assert!(inspection["trusted_expected_outputs"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(inspection["rule_ids"].as_array().unwrap().is_empty());
    let project = registry["projects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|project| project["project_id"] == "kinqimen")
        .unwrap();
    assert_eq!(
        project["inspection_record"]["inspection_id"],
        inspection["inspection_id"]
    );
    assert_eq!(
        project["inspection_record"]["commit_sha"],
        inspection["commit_sha"]
    );
    assert_eq!(project["source_pin"]["crosscheck_execution_allowed"], false);
    assert_eq!(hourly["implementation_allowed"], false);
    assert_eq!(hourly["runtime_rule_seed_allowed"], false);
    assert!(runtime["rules"].as_array().unwrap().is_empty());
}
