use divination_core::{
    taiyi_annual_frame_from_traditional_year_candidate, TaiyiYearEpochProfile,
    TAIYI_TRADITIONAL_YEAR_EPOCH_ALGORITHM_VERSION,
};
use serde_json::Value;

#[test]
fn epoch_research_matches_core_without_releasing_runtime_rules() {
    let research: Value = serde_json::from_str(include_str!(
        "../../../data/research/taiyi-traditional-year-epoch-candidates.json"
    ))
    .unwrap();
    let parent: Value = serde_json::from_str(include_str!(
        "../../../data/research/taiyi-year-position-candidates.json"
    ))
    .unwrap();
    assert_eq!(
        research["source_identity"]["sha256"],
        parent["source_identity"]["raw_files"][0]["sha256"]
    );
    assert_eq!(research["source_identity"]["facsimile_verified"], false);
    assert_eq!(
        research["source_identity"]["independent_review_completed"],
        false
    );
    assert_eq!(
        research["candidate"]["algorithm_version"],
        TAIYI_TRADITIONAL_YEAR_EPOCH_ALGORITHM_VERSION
    );
    assert_eq!(research["candidate"]["profile_has_default"], false);
    let profile: TaiyiYearEpochProfile =
        serde_json::from_value(research["candidate"]["profile_id"].clone()).unwrap();
    for anchor in research["machine_test_anchors"].as_array().unwrap() {
        let year = i32::try_from(anchor["year"].as_i64().unwrap()).unwrap();
        let candidate = taiyi_annual_frame_from_traditional_year_candidate(year, profile).unwrap();
        assert_eq!(
            candidate.accumulated_year_count,
            anchor["count"].as_u64().unwrap()
        );
        assert!(!candidate.rule_eligible);
        assert!(!candidate.annual_frame.rule_eligible);
    }
    for flag in [
        "civil_date_input_allowed",
        "automatic_year_boundary_selection_allowed",
        "runtime_epoch_selection_allowed",
        "runtime_ingestion_allowed",
        "complete_board_generation_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert_eq!(research[flag], false, "unsafe research flag {flag}");
    }
    for attempt in research["facsimile_acquisition_attempts"]
        .as_array()
        .unwrap()
    {
        assert_eq!(attempt["actual_original_bytes_retrieved"], false);
        assert_eq!(attempt["actual_original_sha256"], Value::Null);
    }
    assert_eq!(research["runtime_rule_count"], 0);
    let runtime: Value =
        serde_json::from_str(include_str!("../../../data/rules.seed.json")).unwrap();
    assert!(runtime["rules"].as_array().unwrap().is_empty());
}
