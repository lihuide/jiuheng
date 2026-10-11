use std::collections::BTreeSet;

use divination_core::{
    five_visible_planet_fit_overlap_diagnostics, five_visible_planet_geometric_observations,
    liuyao_directional_five_element_relation, liuyao_eight_palace_audit,
    liuyao_fanyin_trigram_partner, liuyao_hidden_spirits_audit, liuyao_najia_audit,
    liuyao_progress_retreat_identity, liuyao_sanhe_groups, liuyao_six_combination_partner,
    liuyao_six_opposition_partner, liuyao_six_relatives_audit, liuyao_six_spirits_audit,
    liuyao_xun_void_branches, meihua_mutual_raw_audit, qizheng_siyu_identity_audit,
    taiyi_annual_frame_from_accumulated_count, taiyi_seventy_two_board_frame_from_count,
    taiyi_year_position_from_accumulated_count, ten_god_for_visible_stems, EarthlyBranch,
    FiveElement, HeavenlyStem, LineType, LiuyaoDirectionalFiveElementRelation,
    LiuyaoProgressRetreatIdentity, SexagenaryDesignation, TaiyiDun, TaiyiGate, TaiyiGeneralPalace,
    TenGodIdentity, Trigram, FOUR_PILLARS_VISIBLE_STEM_TEN_GODS_ALGORITHM_VERSION,
    JPL_LONG_RANGE_PLANETARY_ALGORITHM_VERSION, LIUYAO_BRANCH_PAIR_RELATIONS_ALGORITHM_VERSION,
    LIUYAO_CALENDAR_MOVING_ALGORITHM_VERSION, LIUYAO_CALENDAR_RELATIONS_ALGORITHM_VERSION,
    LIUYAO_EIGHT_PALACE_ALGORITHM_VERSION, LIUYAO_FANYIN_FUYIN_ALGORITHM_VERSION,
    LIUYAO_HIDDEN_SPIRITS_ALGORITHM_VERSION, LIUYAO_NAJIA_ALGORITHM_VERSION,
    LIUYAO_PROGRESS_RETREAT_ALGORITHM_VERSION, LIUYAO_SANHE_ALGORITHM_VERSION,
    LIUYAO_SIX_RELATIVES_ALGORITHM_VERSION, LIUYAO_SIX_SPIRITS_ALGORITHM_VERSION,
    MEIHUA_MUTUAL_RAW_ALGORITHM_VERSION, QIZHENG_SIYU_IDENTITY_ALGORITHM_VERSION,
    TAIYI_ACCUMULATED_YEAR_POSITION_ALGORITHM_VERSION, TAIYI_ANNUAL_FRAME_ALGORITHM_VERSION,
    TAIYI_ANNUAL_FRAME_V1_ALGORITHM_VERSION, TAIYI_ANNUAL_FRAME_V2_ALGORITHM_VERSION,
    TAIYI_ANNUAL_FRAME_V3_ALGORITHM_VERSION, TAIYI_ANNUAL_FRAME_V4_ALGORITHM_VERSION,
    TAIYI_ANNUAL_FRAME_V5_ALGORITHM_VERSION, TAIYI_ANNUAL_FRAME_V6_ALGORITHM_VERSION,
    TAIYI_ANNUAL_FRAME_V7_ALGORITHM_VERSION, TAIYI_ANNUAL_FRAME_V8_ALGORITHM_VERSION,
    TAIYI_ANNUAL_GATE_BOARDS_ALGORITHM_VERSION, TAIYI_BARRIER_RAW_OBSERVATIONS_ALGORITHM_VERSION,
    TAIYI_MECHANICAL_RELATIONS_ALGORITHM_VERSION, TAIYI_MECHANICAL_RELATIONS_V1_ALGORITHM_VERSION,
    TAIYI_MECHANICAL_RELATIONS_V2_ALGORITHM_VERSION,
    TAIYI_MECHANICAL_RELATIONS_V3_ALGORITHM_VERSION, TAIYI_SEVENTY_TWO_BOARD_ALGORITHM_VERSION,
    TAIYI_SEVENTY_TWO_BOARD_V1_ALGORITHM_VERSION, TAIYI_SEVENTY_TWO_BOARD_V2_ALGORITHM_VERSION,
    TAIYI_SEVENTY_TWO_BOARD_V3_ALGORITHM_VERSION,
};
use serde_json::Value;

const YARROW_RESEARCH: &str = include_str!("../../../data/research/yarrow-casting-candidates.json");
const MEIHUA_RESEARCH: &str =
    include_str!("../../../data/research/meihua-time-casting-candidates.json");
const MEIHUA_MUTUAL_RESEARCH: &str =
    include_str!("../../../data/research/meihua-mutual-raw-candidates.json");
const HEXAGRAM_CATALOG_RESEARCH: &str =
    include_str!("../../../data/research/zhouyi-hexagram-structure-catalog.json");
const FOUR_PILLARS_VISIBLE_STEM_TEN_GODS_RESEARCH: &str =
    include_str!("../../../data/research/four-pillars-visible-stem-ten-gods-candidates.json");
const LIUYAO_EIGHT_PALACE_RESEARCH: &str =
    include_str!("../../../data/research/liuyao-eight-palace-host-response-candidates.json");
const LIUYAO_NAJIA_RESEARCH: &str =
    include_str!("../../../data/research/liuyao-najia-stem-branch-candidates.json");
const LIUYAO_SIX_RELATIVES_RESEARCH: &str =
    include_str!("../../../data/research/liuyao-six-relatives-candidates.json");
const LIUYAO_SIX_SPIRITS_RESEARCH: &str =
    include_str!("../../../data/research/liuyao-six-spirits-candidates.json");
const LIUYAO_HIDDEN_SPIRITS_RESEARCH: &str =
    include_str!("../../../data/research/liuyao-hidden-spirits-candidates.json");
const LIUYAO_CALENDAR_MOVING_RESEARCH: &str =
    include_str!("../../../data/research/liuyao-calendar-moving-candidates.json");
const LIUYAO_CALENDAR_RELATIONS_RESEARCH: &str =
    include_str!("../../../data/research/liuyao-xun-void-calendar-relations-candidates.json");
const LIUYAO_BRANCH_PAIR_RELATIONS_RESEARCH: &str =
    include_str!("../../../data/research/liuyao-six-combination-opposition-pair-candidates.json");
const LIUYAO_PROGRESS_RETREAT_RESEARCH: &str =
    include_str!("../../../data/research/liuyao-progress-retreat-candidates.json");
const LIUYAO_FANYIN_FUYIN_RESEARCH: &str =
    include_str!("../../../data/research/liuyao-fanyin-fuyin-candidates.json");
const LIUYAO_SANHE_RESEARCH: &str =
    include_str!("../../../data/research/liuyao-sanhe-candidates.json");
const ZIWEI_RESEARCH: &str =
    include_str!("../../../data/research/ziwei-life-body-palace-candidates.json");
const ZIWEI_BUREAU_RESEARCH: &str =
    include_str!("../../../data/research/ziwei-palace-stems-bureau-candidates.json");
const ZIWEI_STAR_ANCHOR_RESEARCH: &str =
    include_str!("../../../data/research/ziwei-star-anchor-candidates.json");
const ZIWEI_FOURTEEN_STARS_RESEARCH: &str =
    include_str!("../../../data/research/ziwei-fourteen-main-stars-candidates.json");
const ZIWEI_FOUR_AUXILIARIES_RESEARCH: &str =
    include_str!("../../../data/research/ziwei-four-auxiliary-stars-candidates.json");
const QIZHENG_SIYU_IDENTITY_RESEARCH: &str =
    include_str!("../../../data/research/qizheng-siyu-identity-candidates.json");
const QIZHENG_PLANETARY_RESEARCH: &str =
    include_str!("../../../data/research/qizheng-five-planets-jpl-approx.json");
const QIZHENG_PLANETARY_FIT_OVERLAP_RESEARCH: &str =
    include_str!("../../../data/research/qizheng-five-planets-jpl-fit-overlap.json");
const CHINESE_ASTROLOGY_MANSION_RESEARCH: &str =
    include_str!("../../../data/research/chinese-astrology-twenty-eight-mansions-catalog.json");
const CHINESE_ASTROLOGY_ENCLOSURE_LUMINARY_RESEARCH: &str = include_str!(
    "../../../data/research/chinese-astrology-three-enclosures-seven-luminaries-catalog.json"
);
const QIMEN_JU_RESEARCH: &str =
    include_str!("../../../data/research/qimen-solar-term-three-yuan-ju-candidates.json");
const QIMEN_EARTH_PLATE_RESEARCH: &str =
    include_str!("../../../data/research/qimen-earth-plate-candidates.json");
const QIMEN_HOUR_XUN_RESEARCH: &str =
    include_str!("../../../data/research/qimen-hour-xun-head-candidates.json");
const QIMEN_DUTY_IDENTITY_RESEARCH: &str =
    include_str!("../../../data/research/qimen-duty-identity-candidates.json");
const QIMEN_HOURLY_DUTY_MOVEMENT_RESEARCH: &str =
    include_str!("../../../data/research/qimen-hourly-duty-movement-candidates.json");
const OPEN_SOURCE_REFERENCE_RESEARCH: &str =
    include_str!("../../../data/research/open-source-reference-implementations.json");
const REFERENCE_SOURCE_MANIFEST: &str =
    include_str!("../../../data/research/reference-source-manifest.json");
const REFERENCE_CROSSCHECK_FIXTURES: &str =
    include_str!("../../../data/research/reference-crosscheck-fixtures.json");
const TAIYI_YEAR_POSITION_RESEARCH: &str =
    include_str!("../../../data/research/taiyi-year-position-candidates.json");
const TAIYI_ANNUAL_FRAME_RESEARCH: &str =
    include_str!("../../../data/research/taiyi-annual-frame-candidates.json");
const TAIYI_EYES_COUNTS_RESEARCH: &str =
    include_str!("../../../data/research/taiyi-eyes-counts-candidates.json");
const TAIYI_GENERALS_RESEARCH: &str =
    include_str!("../../../data/research/taiyi-generals-candidates.json");
const TAIYI_EIGHT_GATE_DUTY_RESEARCH: &str =
    include_str!("../../../data/research/taiyi-eight-gate-duty-candidates.json");
const TAIYI_MECHANICAL_RELATIONS_RESEARCH: &str =
    include_str!("../../../data/research/taiyi-mechanical-relations-candidates.json");
const TAIYI_SEVENTY_TWO_BOARD_RESEARCH: &str =
    include_str!("../../../data/research/taiyi-seventy-two-board-candidates.json");
const TAIYI_ANNUAL_GATE_BOARDS_RESEARCH: &str =
    include_str!("../../../data/research/taiyi-annual-gate-boards-candidates.json");
const TAIYI_NEAR_RING_OPPOSITION_RESEARCH: &str =
    include_str!("../../../data/research/taiyi-near-ring-opposition-candidates.json");
const TAIYI_STRIKE_PRESSURE_RESEARCH: &str =
    include_str!("../../../data/research/taiyi-strike-pressure-candidates.json");
const TAIYI_BARRIER_RAW_OBSERVATIONS_RESEARCH: &str =
    include_str!("../../../data/research/taiyi-barrier-raw-observations-candidates.json");
const RUNTIME_RULE_SEED: &str = include_str!("../../../data/rules.seed.json");

fn required_bool(value: &Value, key: &str) -> bool {
    value[key]
        .as_bool()
        .unwrap_or_else(|| panic!("research field `{key}` must be a boolean"))
}

fn required_array<'a>(value: &'a Value, key: &str) -> &'a Vec<Value> {
    value[key]
        .as_array()
        .unwrap_or_else(|| panic!("research field `{key}` must be an array"))
}

fn assert_object_key_absent(value: &Value, forbidden_key: &str) {
    match value {
        Value::Object(object) => {
            assert!(
                !object.contains_key(forbidden_key),
                "research data must not contain object key `{forbidden_key}`"
            );
            for child in object.values() {
                assert_object_key_absent(child, forbidden_key);
            }
        }
        Value::Array(array) => {
            for child in array {
                assert_object_key_absent(child, forbidden_key);
            }
        }
        _ => {}
    }
}

fn assert_lower_hex(value: &str, expected_len: usize) {
    assert_eq!(value.len(), expected_len);
    assert!(
        value
            .chars()
            .all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character)),
        "expected {expected_len} lowercase hexadecimal characters: {value}"
    );
}

#[test]
fn blocked_yarrow_research_cannot_enter_the_runtime_rule_seed() {
    let research: Value = serde_json::from_str(YARROW_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "executable_seed_eligible"));
    assert!(!required_bool(&research, "runtime_imported"));
    assert_eq!(research["evidence_gate_summary"]["decision"], "blocked");
    assert_eq!(research["evidence_gate_summary"]["runtime_rule_count"], 0);

    for lead in required_array(&research, "blocked_source_leads") {
        assert_eq!(lead["admissible_as_facsimile_evidence"], false);
    }

    let mut forbidden_rule_ids = BTreeSet::new();
    for candidate in required_array(&research, "candidates") {
        assert_eq!(candidate["may_execute"], false);
        for id in required_array(candidate, "formal_rule_ids") {
            forbidden_rule_ids.insert(
                id.as_str()
                    .expect("formal rule identifiers must be strings")
                    .to_owned(),
            );
        }
    }
    assert!(
        forbidden_rule_ids.is_empty(),
        "blocked research must not have formal Rule IDs"
    );

    let runtime_rules = required_array(&runtime, "rules");
    assert_eq!(
        runtime_rules.len(),
        research["evidence_gate_summary"]["runtime_rule_count"]
            .as_u64()
            .expect("runtime_rule_count must be an unsigned integer") as usize,
        "research gate count and runtime rule seed disagree"
    );
}

#[test]
fn blocked_meihua_textual_candidate_cannot_enter_the_runtime_rule_seed() {
    let research: Value = serde_json::from_str(MEIHUA_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(
        research["status"],
        "mechanical_textual_candidate_blocked_from_runtime_rules"
    );
    assert_eq!(
        research["implemented_candidate"]["forbidden_use"],
        "Rule ID、体用、生克、吉凶、应期、中间信号或综合裁决"
    );
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn meihua_raw_mutual_trigrams_stay_separate_and_non_executable() {
    let research: Value =
        serde_json::from_str(MEIHUA_MUTUAL_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(
        research["candidate"]["algorithm_version"],
        MEIHUA_MUTUAL_RAW_ALGORITHM_VERSION
    );
    assert_eq!(
        research["candidate"]["all_primary_structures_and_moving_lines_verified"],
        384
    );
    let raw = meihua_mutual_raw_audit(0, 0b000100).unwrap();
    assert!(raw.primary_is_pure_qian_or_kun);
    assert_eq!(raw.primary_lower_from_lines_2_3_4, Trigram::Kun);
    assert_eq!(raw.changed_lower_from_lines_2_3_4, Trigram::Kan);
    assert_eq!(raw.selected_received_basis, None);
    assert!(!raw.rule_eligible);
    for key in [
        "hexagram_name_assigned",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(&research["candidate"], key));
    }
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_hexagram_catalog_cannot_be_mistaken_for_runtime_rules() {
    let research: Value =
        serde_json::from_str(HEXAGRAM_CATALOG_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(
        research["status"],
        "mechanical_structure_catalog_not_divination_rules"
    );
    assert_eq!(
        research["implemented_catalog"]["interpretation_eligible"],
        false
    );
    assert_eq!(
        research["implemented_catalog"]["forbidden_use"],
        "Rule ID、卦辞爻辞解释、吉凶、应期、中间信号或综合裁决"
    );
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn four_pillars_visible_stem_ten_gods_cover_all_pairs_and_stay_non_executable() {
    let research: Value = serde_json::from_str(FOUR_PILLARS_VISIBLE_STEM_TEN_GODS_RESEARCH)
        .expect("valid four-pillars visible-stem ten-god research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let candidate = &research["candidate"];

    assert_eq!(research["schema_version"], 2);
    assert_eq!(research["method"], "four_pillars");
    assert_eq!(
        candidate["algorithm_version"],
        FOUR_PILLARS_VISIBLE_STEM_TEN_GODS_ALGORITHM_VERSION
    );
    assert_eq!(candidate["all_day_stem_by_other_stem_pairs_checked"], 100);
    assert_eq!(
        candidate["each_ten_god_identity_occurrences_in_complete_table"],
        10
    );

    let jia_table = candidate["jia_day_visible_stem_sequence"]
        .as_object()
        .expect("Jia-day table must be an object");
    assert_eq!(jia_table.len(), HeavenlyStem::ALL.len());
    for other in HeavenlyStem::ALL {
        assert_eq!(
            jia_table[other.zh_name()],
            ten_god_for_visible_stems(HeavenlyStem::Jia, other).zh_name()
        );
    }

    let mut counts = [0_usize; 10];
    for day_master in HeavenlyStem::ALL {
        for other in HeavenlyStem::ALL {
            counts[ten_god_for_visible_stems(day_master, other).index()] += 1;
        }
    }
    assert_eq!(counts, [10; TenGodIdentity::ALL.len()]);
    assert!(required_bool(candidate, "visible_stem_identities_assigned"));
    assert!(required_bool(
        candidate,
        "late_zi_parallel_day_boundaries_saved"
    ));
    assert!(candidate["late_zi_selected_convention"].is_null());
    for key in [
        "hidden_stems_included",
        "hidden_stem_identities_assigned",
        "strength_assessed",
        "pattern_selected",
        "useful_god_selected",
        "luck_cycles_calculated",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(candidate, key));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn liuyao_eight_palace_host_response_matches_core_and_stays_non_executable() {
    let research: Value = serde_json::from_str(LIUYAO_EIGHT_PALACE_RESEARCH)
        .expect("valid Liuyao eight-palace research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "liuyao");
    assert_eq!(
        research["candidate"]["algorithm_version"],
        LIUYAO_EIGHT_PALACE_ALGORITHM_VERSION
    );
    assert_eq!(
        required_array(&research["candidate"], "palace_order").len(),
        8
    );
    assert_eq!(
        required_array(&research["candidate"], "stage_order").len(),
        8
    );
    let host_lines = required_array(&research["candidate"], "host_lines");
    let response_lines = required_array(&research["candidate"], "response_lines");
    let received = research["candidate"]["received_hexagram_sequences"]
        .as_object()
        .expect("received palace sequences must be an object");
    assert_eq!(received.len(), 8);

    let mut palace_stages = BTreeSet::new();
    for bits in 0..64 {
        let audit = liuyao_eight_palace_audit(bits).unwrap();
        assert!(palace_stages.insert((audit.palace.zh_name().to_owned(), audit.stage_sequence)));
        let sequence = received[audit.palace.zh_name()]
            .as_array()
            .expect("each palace sequence must be an array");
        assert_eq!(sequence.len(), 8);
        assert_eq!(
            sequence[audit.stage_sequence as usize],
            audit.hexagram.received_sequence
        );
        assert_eq!(host_lines[audit.stage_sequence as usize], audit.host_line);
        assert_eq!(
            response_lines[audit.stage_sequence as usize],
            audit.response_line
        );
        assert!(!audit.na_jia_assigned);
        assert!(!audit.six_relatives_assigned);
        assert!(!audit.six_spirits_assigned);
        assert!(!audit.interpretation_eligible);
        assert!(!audit.rule_eligible);
    }
    assert_eq!(palace_stages.len(), 64);
    for flag in [
        "na_jia_assigned",
        "six_relatives_assigned",
        "six_spirits_assigned",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(&research["candidate"], flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn liuyao_najia_tables_match_all_lines_and_stay_non_executable() {
    let research: Value =
        serde_json::from_str(LIUYAO_NAJIA_RESEARCH).expect("valid Liuyao Na Jia research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let candidate = &research["candidate"];

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "liuyao");
    assert_eq!(
        candidate["algorithm_version"],
        LIUYAO_NAJIA_ALGORITHM_VERSION
    );
    let pure_tables = candidate["pure_hexagram_lines_bottom_up"]
        .as_object()
        .expect("pure hexagram Na Jia tables must be an object");
    assert_eq!(pure_tables.len(), 8);
    for trigram in Trigram::ALL_IN_RECEIVED_NUMBER_ORDER {
        let pure = trigram.bits_bottom_up() | (trigram.bits_bottom_up() << 3);
        let audit = liuyao_najia_audit(pure).unwrap();
        let expected = pure_tables[trigram.zh_name()]
            .as_array()
            .expect("pure trigram Na Jia table must be an array");
        assert_eq!(expected.len(), 6);
        for (line, expected) in audit.lines_bottom_up.iter().zip(expected) {
            assert_eq!(expected, &line.designation_zh());
        }
    }

    let mut line_count = 0;
    for bits in 0..64 {
        let audit = liuyao_najia_audit(bits).unwrap();
        line_count += audit.lines_bottom_up.len();
        assert_eq!(audit.lines_bottom_up.len(), 6);
        assert!(audit.na_jia_stems_assigned);
        assert!(audit.na_jia_branches_assigned);
        assert!(audit.line_five_elements_assigned);
        assert!(audit.host_response_bound);
        assert_eq!(
            audit
                .lines_bottom_up
                .iter()
                .filter(|line| line.is_host)
                .count(),
            1
        );
        assert_eq!(
            audit
                .lines_bottom_up
                .iter()
                .filter(|line| line.is_response)
                .count(),
            1
        );
        assert!(!audit.six_relatives_assigned);
        assert!(!audit.six_spirits_assigned);
        assert!(!audit.hidden_spirits_assigned);
        assert!(!audit.interpretation_eligible);
        assert!(!audit.rule_eligible);
    }
    assert_eq!(line_count, 384);
    for flag in [
        "six_relatives_assigned",
        "six_spirits_assigned",
        "hidden_spirits_assigned",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(candidate, flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn liuyao_six_relative_tables_match_all_lines_and_stay_non_executable() {
    let research: Value = serde_json::from_str(LIUYAO_SIX_RELATIVES_RESEARCH)
        .expect("valid Liuyao six-relatives research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let candidate = &research["candidate"];

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "liuyao");
    assert_eq!(
        candidate["algorithm_version"],
        LIUYAO_SIX_RELATIVES_ALGORITHM_VERSION
    );
    assert!(required_bool(candidate, "palace_element_is_self"));

    let tables = candidate["palace_element_relatives"]
        .as_object()
        .expect("palace-element six-relative tables must be an object");
    assert_eq!(tables.len(), 5);
    let mut line_count = 0;
    let mut observed = BTreeSet::new();
    for bits in 0..64 {
        let audit = liuyao_six_relatives_audit(bits).unwrap();
        assert_eq!(audit.lines_bottom_up.len(), 6);
        assert!(audit.six_relatives_assigned);
        for line in audit.lines_bottom_up {
            let expected =
                &tables[audit.palace_element.zh_name()][line.najia.branch_five_element.zh_name()];
            assert_eq!(expected, line.relationship.zh_name());
            observed.insert(line.relationship.zh_name());
            line_count += 1;
        }
        assert!(!audit.six_spirits_assigned);
        assert!(!audit.hidden_spirits_assigned);
        assert!(!audit.useful_god_selected);
        assert!(!audit.strength_assessed);
        assert!(!audit.interpretation_eligible);
        assert!(!audit.rule_eligible);
    }
    assert_eq!(line_count, 384);
    assert_eq!(
        observed,
        BTreeSet::from(["兄弟", "妻财", "子孙", "官鬼", "父母"])
    );
    assert!(required_bool(candidate, "six_relatives_assigned"));
    for flag in [
        "six_spirits_assigned",
        "hidden_spirits_assigned",
        "useful_god_selected",
        "strength_assessed",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(candidate, flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn liuyao_six_spirit_tables_match_all_stems_and_stay_non_executable() {
    let research: Value = serde_json::from_str(LIUYAO_SIX_SPIRITS_RESEARCH)
        .expect("valid Liuyao six-spirits research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let candidate = &research["candidate"];

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "liuyao");
    assert_eq!(
        candidate["algorithm_version"],
        LIUYAO_SIX_SPIRITS_ALGORITHM_VERSION
    );
    assert_eq!(candidate["day_convention"], "beijing_civil_midnight");
    let tables = candidate["lines_bottom_up_by_day_stem_group"]
        .as_object()
        .expect("day-stem six-spirit tables must be an object");
    assert_eq!(tables.len(), 6);

    let mut line_count = 0;
    for bits in 0..64 {
        for stem in HeavenlyStem::ALL {
            let group = match stem {
                HeavenlyStem::Jia | HeavenlyStem::Yi => "甲乙",
                HeavenlyStem::Bing | HeavenlyStem::Ding => "丙丁",
                HeavenlyStem::Wu => "戊",
                HeavenlyStem::Ji => "己",
                HeavenlyStem::Geng | HeavenlyStem::Xin => "庚辛",
                HeavenlyStem::Ren | HeavenlyStem::Gui => "壬癸",
            };
            let expected = tables[group]
                .as_array()
                .expect("six-spirit line table must be an array");
            let audit = liuyao_six_spirits_audit(bits, stem).unwrap();
            assert_eq!(audit.day_convention, "beijing_civil_midnight");
            assert_eq!(audit.lines_bottom_up.len(), expected.len());
            for (line, expected) in audit.lines_bottom_up.iter().zip(expected) {
                assert_eq!(expected, line.spirit.zh_name());
            }
            assert!(audit.six_spirits_assigned);
            assert!(!audit.hidden_spirits_assigned);
            assert!(!audit.useful_god_selected);
            assert!(!audit.strength_assessed);
            assert!(!audit.interpretation_eligible);
            assert!(!audit.rule_eligible);
            line_count += audit.lines_bottom_up.len();
        }
    }
    assert_eq!(line_count, 3_840);
    for flag in [
        "hidden_spirits_assigned",
        "useful_god_selected",
        "strength_assessed",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(candidate, flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn liuyao_hidden_spirit_candidates_match_palace_anchors_and_stay_non_executable() {
    let research: Value = serde_json::from_str(LIUYAO_HIDDEN_SPIRITS_RESEARCH)
        .expect("valid Liuyao hidden-spirits research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let candidate = &research["candidate"];

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "liuyao");
    assert_eq!(
        candidate["algorithm_version"],
        LIUYAO_HIDDEN_SPIRITS_ALGORITHM_VERSION
    );
    assert_eq!(candidate["source_hexagram"], "current_palace_pure_hexagram");
    assert!(required_bool(candidate, "same_line_alignment"));
    let anchors = candidate["qian_palace_anchors"]
        .as_object()
        .expect("Qian-palace hidden-spirit anchors must be an object");
    assert_eq!(anchors.len(), 4);

    let mut matched_anchors = BTreeSet::new();
    for bits in 0..64 {
        let audit = liuyao_hidden_spirits_audit(bits).unwrap();
        let name = &audit.current.najia.eight_palace.hexagram.name_zh_simplified;
        if let Some(expected) = anchors.get(name) {
            matched_anchors.insert(name.clone());
            let expected = expected.as_array().expect("anchor must be an array");
            assert_eq!(audit.hidden_lines_bottom_up.len(), expected.len());
            for (line, expected) in audit.hidden_lines_bottom_up.iter().zip(expected) {
                assert_eq!(line.line_number_bottom_up, expected["line"]);
                assert_eq!(line.hidden_line.najia.designation_zh(), expected["hidden"]);
                assert_eq!(
                    line.hidden_line.relationship.zh_name(),
                    expected["relative"]
                );
            }
        }
        for missing in &audit.missing_relatives {
            assert!(audit
                .hidden_lines_bottom_up
                .iter()
                .any(|line| line.hidden_line.relationship == *missing));
        }
        assert!(audit.hidden_spirits_assigned);
        assert!(!audit.flying_hidden_interactions_assessed);
        assert!(!audit.useful_god_selected);
        assert!(!audit.strength_assessed);
        assert!(!audit.interpretation_eligible);
        assert!(!audit.rule_eligible);
    }
    assert_eq!(
        matched_anchors,
        BTreeSet::from(["剥", "姤", "观", "遯"].map(str::to_owned))
    );
    for flag in [
        "flying_hidden_interactions_assessed",
        "useful_god_selected",
        "strength_assessed",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(candidate, flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn liuyao_month_day_and_moving_candidates_stay_mechanical_and_non_executable() {
    let research: Value = serde_json::from_str(LIUYAO_CALENDAR_MOVING_RESEARCH)
        .expect("valid Liuyao calendar/moving research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let candidate = &research["candidate"];

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "liuyao");
    assert_eq!(
        candidate["algorithm_version"],
        LIUYAO_CALENDAR_MOVING_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["month_convention"],
        "locked_tt_solar_jie_span_with_lichun_year_stem"
    );
    assert_eq!(candidate["day_convention"], "beijing_civil_midnight");
    assert_eq!(candidate["checked_combination_count"], 4096);
    assert!(required_bool(
        candidate,
        "all_four_to_the_six_line_type_combinations_checked"
    ));
    assert!(required_bool(candidate, "moving_line_transformed_najia"));
    assert!(required_bool(
        candidate,
        "transformed_relative_uses_primary_palace_element"
    ));

    let line_values = candidate["line_values"]
        .as_object()
        .expect("line value table must be an object");
    let expected = [
        (LineType::OldYin, "老阴", "阴", "阳"),
        (LineType::YoungYang, "少阳", "阳", "阳"),
        (LineType::YoungYin, "少阴", "阴", "阴"),
        (LineType::OldYang, "老阳", "阳", "阴"),
    ];
    assert_eq!(line_values.len(), expected.len());
    for (line_type, name, primary, changed) in expected {
        let row = &line_values[&line_type.value().to_string()];
        assert_eq!(row["name"], name);
        assert_eq!(row["moving"], line_type.is_moving());
        assert_eq!(row["primary"], primary);
        assert_eq!(row["changed"], changed);
    }
    for flag in [
        "month_day_line_interactions_assessed",
        "useful_god_selected",
        "strength_assessed",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(candidate, flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn liuyao_xun_void_and_calendar_relations_stay_raw_and_non_executable() {
    let research: Value = serde_json::from_str(LIUYAO_CALENDAR_RELATIONS_RESEARCH)
        .expect("valid Liuyao xun/calendar relations research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let candidate = &research["candidate"];

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "liuyao");
    assert_eq!(
        candidate["algorithm_version"],
        LIUYAO_CALENDAR_RELATIONS_ALGORITHM_VERSION
    );
    assert_eq!(candidate["opposition_offset_in_twelve_branch_cycle"], 6);
    assert_eq!(candidate["checked_primary_line_observation_count"], 23040);
    assert_eq!(
        required_array(candidate, "observed_layers"),
        &[
            Value::String("primary_visible".to_owned()),
            Value::String("moving_transformed".to_owned())
        ]
    );
    assert_eq!(required_array(candidate, "observed_flags").len(), 7);
    assert_eq!(
        candidate["all_five_by_five_directional_relations_checked"],
        25
    );
    assert_eq!(
        required_array(candidate, "directional_five_element_relations").len(),
        LiuyaoDirectionalFiveElementRelation::ALL.len()
    );
    let elements = [
        FiveElement::Wood,
        FiveElement::Fire,
        FiveElement::Earth,
        FiveElement::Metal,
        FiveElement::Water,
    ];
    let mut relation_cells = BTreeSet::new();
    for source in elements {
        for target in elements {
            relation_cells.insert(format!(
                "{}:{}:{}",
                source.zh_name(),
                target.zh_name(),
                liuyao_directional_five_element_relation(source, target).zh_name()
            ));
        }
    }
    assert_eq!(relation_cells.len(), 25);

    let expected = [
        (0, "甲子", [EarthlyBranch::Xu, EarthlyBranch::Hai]),
        (10, "甲戌", [EarthlyBranch::Shen, EarthlyBranch::You]),
        (20, "甲申", [EarthlyBranch::Wu, EarthlyBranch::Wei]),
        (30, "甲午", [EarthlyBranch::Chen, EarthlyBranch::Si]),
        (40, "甲辰", [EarthlyBranch::Yin, EarthlyBranch::Mao]),
        (50, "甲寅", [EarthlyBranch::Zi, EarthlyBranch::Chou]),
    ];
    let table = candidate["xun_void_table"]
        .as_object()
        .expect("xun void table must be an object");
    assert_eq!(table.len(), expected.len());
    for (cycle_index, xun_name, expected_branches) in expected {
        let day = SexagenaryDesignation::from_cycle_index(cycle_index);
        let (start, branches) = liuyao_xun_void_branches(day);
        assert_eq!(start.zh_name(), xun_name);
        assert_eq!(branches, expected_branches);
        assert_eq!(
            table[xun_name]
                .as_array()
                .expect("xun pair must be an array")
                .iter()
                .map(|value| value.as_str().expect("branch must be a string"))
                .collect::<Vec<_>>(),
            expected_branches.map(EarthlyBranch::zh_name)
        );
    }
    assert!(required_bool(candidate, "raw_relations_identified"));
    for flag in [
        "relation_effects_assessed",
        "useful_god_selected",
        "strength_assessed",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(candidate, flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn liuyao_six_combination_and_opposition_pairs_stay_raw_and_non_executable() {
    use EarthlyBranch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};

    let research: Value = serde_json::from_str(LIUYAO_BRANCH_PAIR_RELATIONS_RESEARCH)
        .expect("valid Liuyao branch-pair research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let candidate = &research["candidate"];

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "liuyao");
    assert_eq!(
        candidate["algorithm_version"],
        LIUYAO_BRANCH_PAIR_RELATIONS_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["all_primary_line_type_combinations_verified"],
        4096
    );
    assert_eq!(
        candidate["primary_visible_pair_observations_verified"],
        61440
    );
    assert_eq!(
        candidate["moving_transformation_pair_observations_verified"],
        12288
    );
    assert_eq!(required_array(candidate, "comparison_scopes").len(), 3);

    let expected_combinations = [
        (Zi, Chou),
        (Yin, Hai),
        (Mao, Xu),
        (Chen, You),
        (Si, Shen),
        (Wu, Wei),
    ];
    let expected_oppositions = [
        (Zi, Wu),
        (Chou, Wei),
        (Yin, Shen),
        (Mao, You),
        (Chen, Xu),
        (Si, Hai),
    ];
    let combination_table = required_array(candidate, "six_combination_pairs");
    let opposition_table = required_array(candidate, "six_opposition_pairs");
    assert_eq!(combination_table.len(), 6);
    assert_eq!(opposition_table.len(), 6);
    for (index, (first, second)) in expected_combinations.into_iter().enumerate() {
        assert_eq!(liuyao_six_combination_partner(first), second);
        assert_eq!(liuyao_six_combination_partner(second), first);
        assert_eq!(combination_table[index][0], first.zh_name());
        assert_eq!(combination_table[index][1], second.zh_name());
    }
    for (index, (first, second)) in expected_oppositions.into_iter().enumerate() {
        assert_eq!(liuyao_six_opposition_partner(first), second);
        assert_eq!(liuyao_six_opposition_partner(second), first);
        assert_eq!(opposition_table[index][0], first.zh_name());
        assert_eq!(opposition_table[index][1], second.zh_name());
    }
    for flag in [
        "whole_hexagram_six_combination_classified",
        "whole_hexagram_six_opposition_classified",
        "combination_or_opposition_effect_assessed",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(candidate, flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn liuyao_progress_retreat_identities_stay_explicit_raw_and_non_executable() {
    use EarthlyBranch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};

    let research: Value = serde_json::from_str(LIUYAO_PROGRESS_RETREAT_RESEARCH)
        .expect("valid Liuyao progress/retreat research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let candidate = &research["candidate"];

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "liuyao");
    assert_eq!(
        candidate["algorithm_version"],
        LIUYAO_PROGRESS_RETREAT_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["all_primary_line_type_combinations_verified"],
        4096
    );
    assert_eq!(
        candidate["moving_transformation_observations_verified"],
        12288
    );
    assert_eq!(candidate["explicit_directed_pair_count"], 14);

    let expected_progress = [
        (Hai, Zi),
        (Yin, Mao),
        (Si, Wu),
        (Shen, You),
        (Chou, Chen),
        (Chen, Wei),
        (Wei, Xu),
    ];
    let expected_retreat = [
        (Zi, Hai),
        (Mao, Yin),
        (Wu, Si),
        (You, Shen),
        (Chen, Chou),
        (Wei, Chen),
        (Xu, Wei),
    ];
    let progress_table = required_array(candidate, "progress_pairs");
    let retreat_table = required_array(candidate, "retreat_pairs");
    assert_eq!(progress_table.len(), 7);
    assert_eq!(retreat_table.len(), 7);
    for (index, (primary, transformed)) in expected_progress.into_iter().enumerate() {
        assert_eq!(progress_table[index][0], primary.zh_name());
        assert_eq!(progress_table[index][1], transformed.zh_name());
        assert_eq!(
            liuyao_progress_retreat_identity(primary, transformed),
            LiuyaoProgressRetreatIdentity::Progress
        );
    }
    for (index, (primary, transformed)) in expected_retreat.into_iter().enumerate() {
        assert_eq!(retreat_table[index][0], primary.zh_name());
        assert_eq!(retreat_table[index][1], transformed.zh_name());
        assert_eq!(
            liuyao_progress_retreat_identity(primary, transformed),
            LiuyaoProgressRetreatIdentity::Retreat
        );
    }
    assert_eq!(
        required_array(candidate, "deliberately_unclassified_pairs").len(),
        2
    );
    assert_eq!(
        liuyao_progress_retreat_identity(Xu, Chou),
        LiuyaoProgressRetreatIdentity::Unclassified
    );
    assert_eq!(
        liuyao_progress_retreat_identity(Chou, Xu),
        LiuyaoProgressRetreatIdentity::Unclassified
    );
    assert!(required_bool(candidate, "raw_identity_assigned"));
    for flag in [
        "progression_or_retreat_effect_assessed",
        "useful_god_selected",
        "strength_assessed",
        "timing_assessed",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(candidate, flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn liuyao_fanyin_fuyin_prerequisites_stay_raw_and_non_executable() {
    let research: Value = serde_json::from_str(LIUYAO_FANYIN_FUYIN_RESEARCH)
        .expect("valid Liuyao Fanyin/Fuyin research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let candidate = &research["candidate"];

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "liuyao");
    assert_eq!(
        candidate["algorithm_version"],
        LIUYAO_FANYIN_FUYIN_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["branch_opposition_offset_in_twelve_branch_cycle"],
        6
    );
    assert_eq!(
        candidate["all_primary_line_type_combinations_verified"],
        4096
    );
    assert_eq!(candidate["moving_line_observations_verified"], 12288);
    assert!(required_bool(
        candidate,
        "scope_candidate_requires_actual_moving_line"
    ));

    let expected = [
        (Trigram::Qian, Trigram::Xun),
        (Trigram::Kan, Trigram::Li),
        (Trigram::Zhen, Trigram::Dui),
        (Trigram::Kun, Trigram::Gen),
    ];
    let table = required_array(candidate, "received_direction_opposition_pairs");
    assert_eq!(table.len(), expected.len());
    for (index, (first, second)) in expected.into_iter().enumerate() {
        assert_eq!(table[index][0], first.zh_name());
        assert_eq!(table[index][1], second.zh_name());
        assert_eq!(liuyao_fanyin_trigram_partner(first), second);
        assert_eq!(liuyao_fanyin_trigram_partner(second), first);
    }
    for branch in EarthlyBranch::ALL {
        assert_eq!(
            liuyao_six_opposition_partner(branch),
            EarthlyBranch::ALL[((branch.index() + 6) % 12) as usize]
        );
    }
    let scopes = required_array(candidate, "scopes");
    assert_eq!(scopes.len(), 2);
    assert_eq!(scopes[0]["scope"], "inner");
    assert_eq!(
        scopes[0]["line_numbers_bottom_up"],
        serde_json::json!([1, 2, 3])
    );
    assert_eq!(scopes[1]["scope"], "outer");
    assert_eq!(
        scopes[1]["line_numbers_bottom_up"],
        serde_json::json!([4, 5, 6])
    );
    assert_eq!(
        required_array(candidate, "raw_scope_prerequisites").len(),
        3
    );
    assert_eq!(
        required_array(candidate, "moving_line_prerequisites").len(),
        2
    );
    assert!(required_bool(candidate, "raw_prerequisites_identified"));
    for flag in [
        "whole_hexagram_fanyin_classified",
        "whole_hexagram_fuyin_classified",
        "relation_effects_assessed",
        "useful_god_selected",
        "strength_assessed",
        "timing_assessed",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(candidate, flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn liuyao_sanhe_member_occurrences_stay_separated_and_non_executable() {
    let research: Value =
        serde_json::from_str(LIUYAO_SANHE_RESEARCH).expect("valid Liuyao Sanhe research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let candidate = &research["candidate"];

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "liuyao");
    assert_eq!(
        candidate["algorithm_version"],
        LIUYAO_SANHE_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["all_primary_line_type_combinations_verified"],
        4096
    );
    assert_eq!(candidate["moving_line_occurrences_verified"], 12288);
    assert_eq!(candidate["total_source_occurrences_verified"], 45056);
    assert!(required_bool(candidate, "raw_union_presence_is_audit_only"));

    let expected = [
        (
            FiveElement::Water,
            [EarthlyBranch::Shen, EarthlyBranch::Zi, EarthlyBranch::Chen],
        ),
        (
            FiveElement::Wood,
            [EarthlyBranch::Hai, EarthlyBranch::Mao, EarthlyBranch::Wei],
        ),
        (
            FiveElement::Fire,
            [EarthlyBranch::Yin, EarthlyBranch::Wu, EarthlyBranch::Xu],
        ),
        (
            FiveElement::Metal,
            [EarthlyBranch::Si, EarthlyBranch::You, EarthlyBranch::Chou],
        ),
    ];
    assert_eq!(liuyao_sanhe_groups(), expected);
    let groups = required_array(candidate, "groups_birth_prosperity_tomb");
    assert_eq!(groups.len(), expected.len());
    let mut seen = [false; 12];
    for (index, (element, branches)) in expected.into_iter().enumerate() {
        assert_eq!(groups[index]["element"], element.zh_name());
        assert_eq!(required_array(&groups[index], "branches").len(), 3);
        for (member_index, branch) in branches.into_iter().enumerate() {
            assert_eq!(groups[index]["branches"][member_index], branch.zh_name());
            assert!(!seen[branch.index() as usize]);
            seen[branch.index() as usize] = true;
        }
    }
    assert!(seen.into_iter().all(|value| value));
    assert_eq!(required_array(candidate, "occurrence_layers").len(), 4);
    assert!(required_bool(candidate, "raw_occurrence_inventory_built"));
    for flag in [
        "source_scope_activation_requirements_classified",
        "any_sanhe_formation_classified",
        "combination_effects_assessed",
        "useful_god_selected",
        "strength_assessed",
        "timing_assessed",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(candidate, flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_ziwei_palaces_cannot_enter_the_runtime_rule_seed() {
    let research: Value = serde_json::from_str(ZIWEI_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(
        research["status"],
        "mechanical_received_text_candidate_blocked_from_runtime_rules"
    );
    assert_eq!(
        research["implemented_candidate"]["interpretation_eligible"],
        false
    );
    assert_eq!(
        research["implemented_candidate"]["selected_hour_convention"],
        Value::Null
    );
    assert_eq!(research["evidence_gate_summary"]["decision"], "blocked");
    assert_eq!(research["evidence_gate_summary"]["runtime_rule_count"], 0);
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_ziwei_bureau_candidates_cannot_enter_the_runtime_rule_seed() {
    let research: Value = serde_json::from_str(ZIWEI_BUREAU_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(
        research["status"],
        "mechanical_received_bureau_candidate_blocked_from_runtime_rules"
    );
    assert_eq!(
        research["implemented_candidate"]["interpretation_eligible"],
        false
    );
    assert_eq!(
        research["implemented_candidate"]["selected_year_stem_convention"],
        Value::Null
    );
    assert_eq!(
        research["implemented_candidate"]["selected_hour_convention"],
        Value::Null
    );
    assert_eq!(research["evidence_gate_summary"]["decision"], "blocked");
    assert_eq!(research["evidence_gate_summary"]["runtime_rule_count"], 0);
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_ziwei_star_anchor_cannot_enter_the_runtime_rule_seed() {
    let research: Value =
        serde_json::from_str(ZIWEI_STAR_ANCHOR_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(research["formal_rule_count"], 0);
    assert_eq!(research["candidate"]["interpretation_eligible"], false);
    assert_eq!(research["candidate"]["rule_eligible"], false);
    assert_eq!(
        research["candidate"]["selected_year_convention"],
        Value::Null
    );
    assert_eq!(
        research["candidate"]["selected_hour_convention"],
        Value::Null
    );
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_ziwei_fourteen_stars_cannot_enter_the_runtime_rule_seed() {
    let research: Value =
        serde_json::from_str(ZIWEI_FOURTEEN_STARS_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(research["formal_rule_count"], 0);
    assert_eq!(research["candidate"]["star_count"], 14);
    assert_eq!(research["candidate"]["interpretation_eligible"], false);
    assert_eq!(research["candidate"]["rule_eligible"], false);
    assert_eq!(
        research["candidate"]["selected_year_convention"],
        Value::Null
    );
    assert_eq!(
        research["candidate"]["selected_hour_convention"],
        Value::Null
    );
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_ziwei_four_auxiliaries_cannot_enter_the_runtime_rule_seed() {
    let research: Value =
        serde_json::from_str(ZIWEI_FOUR_AUXILIARIES_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(research["formal_rule_count"], 0);
    assert_eq!(research["candidate"]["star_count"], 4);
    assert_eq!(research["candidate"]["interpretation_eligible"], false);
    assert_eq!(research["candidate"]["rule_eligible"], false);
    assert_eq!(
        research["candidate"]["selected_hour_convention"],
        Value::Null
    );
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_mansion_catalog_cannot_enter_the_runtime_rule_seed() {
    let research: Value =
        serde_json::from_str(CHINESE_ASTROLOGY_MANSION_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(
        research["status"],
        "mechanical_received_mansion_identity_catalog_blocked_from_runtime_rules"
    );
    assert_eq!(
        required_array(&research["implemented_catalog"], "output_order").len(),
        28
    );
    assert_eq!(
        required_array(&research["implemented_catalog"], "quadrants").len(),
        4
    );
    assert_eq!(
        research["implemented_catalog"]["angular_boundary_model_selected"],
        false
    );
    assert_eq!(
        research["implemented_catalog"]["celestial_positions_calculated"],
        false
    );
    assert_eq!(
        research["implemented_catalog"]["interpretation_eligible"],
        false
    );
    assert_eq!(research["implemented_catalog"]["rule_eligible"], false);
    assert_eq!(research["evidence_gate_summary"]["decision"], "blocked");
    assert_eq!(research["evidence_gate_summary"]["runtime_rule_count"], 0);
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_enclosure_and_luminary_catalogs_cannot_enter_the_runtime_rule_seed() {
    let research: Value = serde_json::from_str(CHINESE_ASTROLOGY_ENCLOSURE_LUMINARY_RESEARCH)
        .expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(
        research["status"],
        "mechanical_received_enclosure_luminary_identity_catalog_blocked_from_runtime_rules"
    );
    assert_eq!(
        required_array(&research["implemented_catalog"], "three_enclosures").len(),
        3
    );
    assert_eq!(
        required_array(&research["implemented_catalog"], "seven_luminaries").len(),
        7
    );
    assert_eq!(
        research["implemented_catalog"]["enclosure_boundary_model_selected"],
        false
    );
    assert_eq!(
        research["implemented_catalog"]["angular_boundary_model_selected"],
        false
    );
    assert_eq!(
        research["implemented_catalog"]["celestial_positions_calculated"],
        false
    );
    assert_eq!(
        research["implemented_catalog"]["interpretation_eligible"],
        false
    );
    assert_eq!(research["implemented_catalog"]["rule_eligible"], false);
    assert_eq!(research["evidence_gate_summary"]["decision"], "blocked");
    assert_eq!(research["evidence_gate_summary"]["runtime_rule_count"], 0);
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn qizheng_siyu_identity_catalog_matches_core_and_cannot_enter_runtime_rules() {
    let research: Value = serde_json::from_str(QIZHENG_SIYU_IDENTITY_RESEARCH)
        .expect("valid Qizheng Siyu identity research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");
    let audit = qizheng_siyu_identity_audit();

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "qizheng_siyu");
    assert_eq!(
        research["status"],
        "mechanical_received_eleven_luminary_identity_catalog_blocked_from_runtime_rules"
    );
    assert_eq!(
        research["candidate"]["algorithm_version"],
        QIZHENG_SIYU_IDENTITY_ALGORITHM_VERSION
    );
    let seven = required_array(&research["candidate"], "seven_governors");
    let four = required_array(&research["candidate"], "four_residuals");
    assert_eq!(seven.len(), audit.seven_governors.len());
    assert_eq!(four.len(), audit.four_residuals.len());
    for (stored, current) in seven.iter().zip(&audit.seven_governors) {
        assert_eq!(stored, &Value::String(current.received_zh_name.clone()));
    }
    for (stored, current) in four.iter().zip(&audit.four_residuals) {
        assert_eq!(
            stored["normalized_name"],
            Value::String(current.normalized_zh_name.clone())
        );
        assert_eq!(stored["modern_astronomical_role"], Value::Null);
    }
    for flag in [
        "residual_astronomical_roles_selected",
        "planetary_position_algorithm_selected",
        "mansion_boundary_model_selected",
        "twelve_house_system_selected",
        "positions_calculated",
        "natal_chart_generated",
        "interpretation_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(&research["candidate"], flag));
    }
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(research["review_state"]["formal_rule_count"], 0);
    assert!(!required_array(&research, "forbidden_uses").is_empty());
    assert!(!required_array(&research, "unresolved").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn qizheng_planetary_approximation_is_research_only() {
    let research: Value = serde_json::from_str(QIZHENG_PLANETARY_RESEARCH).unwrap();
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).unwrap();
    let observations = five_visible_planet_geometric_observations(2_451_545.0).unwrap();
    assert_eq!(research["method"], "qizheng_siyu");
    assert_eq!(
        research["candidate"]["algorithm_version"],
        JPL_LONG_RANGE_PLANETARY_ALGORITHM_VERSION
    );
    assert_eq!(required_array(&research["candidate"], "bodies").len(), 5);
    assert_eq!(observations.len(), 5);
    for observation in observations {
        assert_eq!(
            observation.algorithm_version,
            JPL_LONG_RANGE_PLANETARY_ALGORITHM_VERSION
        );
        assert!(!observation.apparent_geocentric_position);
        assert!(!observation.historical_mansion_eligible);
        assert!(!observation.divination_rule_eligible);
    }
    for flag in [
        "apparent_geocentric_position",
        "historical_mansion_eligible",
        "natal_chart_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(&research["candidate"], flag));
    }
    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(research["formal_rule_count"], 0);
    assert!(required_array(&runtime, "rules").is_empty());
    assert_object_key_absent(&research, "formal_rule_ids");
}

#[test]
fn qizheng_two_fit_overlap_is_reproducible_but_not_a_rule_or_accuracy_bound() {
    let research: Value = serde_json::from_str(QIZHENG_PLANETARY_FIT_OVERLAP_RESEARCH).unwrap();
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).unwrap();
    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "qizheng_siyu");
    assert_eq!(research["comparison"]["sample_count"], 20);
    for epoch in required_array(&research["comparison"], "epochs") {
        let jd = epoch["jd_tt"].as_f64().unwrap();
        let diagnostics = five_visible_planet_fit_overlap_diagnostics(jd).unwrap();
        let largest = diagnostics
            .iter()
            .max_by(|left, right| {
                left.angular_disagreement_deg
                    .total_cmp(&right.angular_disagreement_deg)
            })
            .unwrap();
        assert_eq!(largest.planet, divination_core::VisiblePlanet::Saturn);
        let pinned = epoch["largest_angular_disagreement_deg"].as_f64().unwrap();
        assert!((largest.angular_disagreement_deg - pinned).abs() < 1.0e-7);
        assert!(diagnostics
            .iter()
            .all(|d| !d.independent_ephemeris_validation
                && !d.historical_mansion_eligible
                && !d.divination_rule_eligible));
    }
    for flag in [
        "independent_ephemeris_validation",
        "historical_mansion_eligible",
        "signal_eligible",
        "rule_eligible",
    ] {
        assert!(!required_bool(&research["comparison"], flag));
    }
    assert_eq!(
        research["external_validation_attempt"]["reference_vectors_recorded"],
        0
    );
    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(research["formal_rule_count"], 0);
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_qimen_ju_candidates_cannot_enter_the_runtime_rule_seed() {
    let research: Value = serde_json::from_str(QIMEN_JU_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(
        research["status"],
        "mechanical_received_solar_term_three_yuan_ju_candidate_blocked_from_runtime_rules"
    );
    assert_eq!(
        research["implemented_candidate"]["selected_day_convention"],
        Value::Null
    );
    assert_eq!(
        research["implemented_candidate"]["transmission_calendar_adjustment_selected"],
        Value::Null
    );
    assert_eq!(
        research["implemented_candidate"]["received_bureau_numbers_upper_middle_lower"]["处暑"],
        Value::Null
    );
    assert_eq!(research["implemented_candidate"]["plate_calculated"], false);
    assert_eq!(
        research["implemented_candidate"]["interpretation_eligible"],
        false
    );
    assert_eq!(research["implemented_candidate"]["rule_eligible"], false);
    assert_eq!(research["evidence_gate_summary"]["decision"], "blocked");
    assert_eq!(research["evidence_gate_summary"]["runtime_rule_count"], 0);
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_qimen_earth_plate_cannot_enter_the_runtime_rule_seed() {
    let research: Value =
        serde_json::from_str(QIMEN_EARTH_PLATE_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(
        research["status"],
        "mechanical_received_three_wonders_six_instruments_earth_plate_blocked_from_runtime_rules"
    );
    assert_eq!(
        required_array(&research["implemented_candidate"], "nine_palaces_by_number").len(),
        9
    );
    assert_eq!(
        required_array(&research["implemented_candidate"], "placement_sequence").len(),
        9
    );
    assert_eq!(
        research["implemented_candidate"]["heaven_plate_calculated"],
        false
    );
    assert_eq!(
        research["implemented_candidate"]["stars_gates_spirits_calculated"],
        false
    );
    assert_eq!(
        research["implemented_candidate"]["interpretation_eligible"],
        false
    );
    assert_eq!(research["implemented_candidate"]["rule_eligible"], false);
    assert_eq!(research["evidence_gate_summary"]["decision"], "blocked");
    assert_eq!(research["evidence_gate_summary"]["runtime_rule_count"], 0);
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_qimen_hour_xun_cannot_enter_the_runtime_rule_seed() {
    let research: Value =
        serde_json::from_str(QIMEN_HOUR_XUN_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(
        research["status"],
        "mechanical_received_hour_xun_head_hidden_instrument_palace_blocked_from_runtime_rules"
    );
    assert_eq!(
        required_array(&research["implemented_candidate"], "ten_hour_xun_groups").len(),
        6
    );
    assert_eq!(
        research["implemented_candidate"]["selected_hour_convention"],
        Value::Null
    );
    assert_eq!(
        research["implemented_candidate"]["duty_chief_calculated"],
        false
    );
    assert_eq!(
        research["implemented_candidate"]["duty_envoy_calculated"],
        false
    );
    assert_eq!(
        research["implemented_candidate"]["interpretation_eligible"],
        false
    );
    assert_eq!(research["implemented_candidate"]["rule_eligible"], false);
    assert_eq!(research["evidence_gate_summary"]["decision"], "blocked");
    assert_eq!(research["evidence_gate_summary"]["runtime_rule_count"], 0);
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn mechanical_qimen_duty_identity_cannot_enter_the_runtime_rule_seed() {
    let research: Value =
        serde_json::from_str(QIMEN_DUTY_IDENTITY_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert_eq!(
        research["status"],
        "mechanical_early_song_fixed_star_gate_duty_identity_blocked_from_runtime_rules"
    );
    assert_eq!(
        required_array(
            &research["implemented_candidate"],
            "fixed_stars_by_palace_one_to_nine"
        )
        .len(),
        9
    );
    assert_eq!(
        research["implemented_candidate"]["fixed_gates_by_palace"]["中五"],
        Value::Null
    );
    assert_eq!(
        research["implemented_candidate"]["center_hosting_convention"],
        "early_song_center_five_hosts_kun_two.v1"
    );
    assert_eq!(
        research["implemented_candidate"]["duty_chief_hourly_palace_calculated"],
        false
    );
    assert_eq!(
        research["implemented_candidate"]["duty_envoy_hourly_palace_calculated"],
        false
    );
    assert_eq!(
        research["implemented_candidate"]["interpretation_eligible"],
        false
    );
    assert_eq!(research["implemented_candidate"]["rule_eligible"], false);
    assert_eq!(research["evidence_gate_summary"]["decision"], "blocked");
    assert_eq!(research["evidence_gate_summary"]["runtime_rule_count"], 0);
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn unverified_qimen_hourly_duty_movement_cannot_be_implemented_or_released() {
    let research: Value =
        serde_json::from_str(QIMEN_HOURLY_DUTY_MOVEMENT_RESEARCH).expect("valid research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert!(!required_bool(&research, "implementation_allowed"));
    assert_eq!(
        research["status"],
        "early_text_hourly_duty_movement_candidate_blocked_from_implementation_and_runtime_rules"
    );
    assert_eq!(
        research["candidate"]["selected_algorithm_version"],
        Value::Null
    );
    assert_eq!(
        research["candidate"]["duty_chief_hourly_palace_calculated"],
        false
    );
    assert_eq!(
        research["candidate"]["duty_envoy_hourly_palace_calculated"],
        false
    );
    assert_eq!(research["candidate"]["rule_eligible"], false);
    assert_eq!(research["evidence_gate_summary"]["decision"], "blocked");
    assert_eq!(research["evidence_gate_summary"]["implementation_count"], 0);
    assert_eq!(research["evidence_gate_summary"]["runtime_rule_count"], 0);
    assert!(!required_array(&research, "unresolved_blockers").is_empty());
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn open_source_references_are_pinned_outside_rule_evidence_and_runtime() {
    let research: Value =
        serde_json::from_str(OPEN_SOURCE_REFERENCE_RESEARCH).expect("valid reference JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 2);
    assert_eq!(
        research["status"],
        "reference_inventory_blocked_from_runtime_and_rule_evidence"
    );
    assert!(!required_bool(&research, "runtime_rule_seed_allowed"));
    assert!(!required_bool(&research, "executable_seed_eligible"));
    assert!(!required_bool(&research, "third_party_code_vendored"));
    assert!(!required_bool(&research, "direct_code_copy_allowed"));
    assert!(!required_bool(
        &research,
        "open_source_claim_is_ancient_text_evidence"
    ));
    assert_eq!(required_array(&research, "required_pipeline").len(), 8);
    assert!(!required_array(&research, "global_forbidden_uses").is_empty());

    let projects = required_array(&research, "projects");
    assert_eq!(projects.len(), 14);
    let mut project_ids = BTreeSet::new();
    let mut covered_methods = BTreeSet::new();
    for project in projects {
        let project_id = project["project_id"]
            .as_str()
            .expect("project_id must be a string");
        assert!(
            project_ids.insert(project_id),
            "duplicate reference project id: {project_id}"
        );
        assert!(
            project["url"]
                .as_str()
                .is_some_and(|url| url.starts_with("https://github.com/")),
            "reference project must use an explicit GitHub repository URL"
        );
        assert!(!required_array(project, "allowed_uses").is_empty());
        assert!(!required_array(project, "forbidden_uses").is_empty());
        assert!(!required_array(project, "risk_notes").is_empty());
        for method in required_array(project, "methods") {
            covered_methods.insert(method.as_str().expect("method must be a string"));
        }

        let pin = &project["source_pin"];
        let crosscheck_allowed = pin["crosscheck_execution_allowed"]
            .as_bool()
            .expect("crosscheck flag must be boolean");
        let hashes_recorded = pin["content_hashes_recorded"]
            .as_bool()
            .expect("hash flag must be boolean");
        if hashes_recorded {
            let commit = pin["commit_sha"]
                .as_str()
                .expect("recorded content hashes require a pinned commit");
            assert_lower_hex(commit, 40);
            assert_eq!(
                pin["source_manifest_id"],
                "OPEN-SOURCE-SOURCE-PINS-2026-09-07-v1"
            );
        } else {
            assert!(pin["commit_sha"].is_null());
        }
        if crosscheck_allowed {
            assert!(hashes_recorded);
        }
    }

    assert_eq!(
        covered_methods,
        BTreeSet::from([
            "bazi_and_calendar",
            "chinese_ancient_astrology",
            "liuyao",
            "meihua_yishu",
            "qimen_dunjia",
            "qizheng_siyu",
            "taiyi",
            "yarrow",
            "ziwei_doushu",
        ])
    );

    let priorities = required_array(&research, "priority_groups");
    assert_eq!(priorities.len(), 9);
    for (index, priority) in priorities.iter().enumerate() {
        assert_eq!(priority["rank"].as_u64(), Some((index + 1) as u64));
        for id in required_array(priority, "project_ids") {
            let id = id.as_str().expect("priority project id must be a string");
            assert!(
                project_ids.contains(id),
                "unknown priority project id: {id}"
            );
        }
    }

    for audit_name in ["taiyi", "qizheng_siyu"] {
        let audit = &research["first_pass_audits"][audit_name];
        assert_eq!(audit["implementation_allowed"], false);
        assert!(required_array(audit, "trusted_expected_outputs").is_empty());
        assert!(!required_array(audit, "blockers").is_empty());
    }
    assert_eq!(research["evidence_gate_summary"]["decision"], "blocked");
    assert_eq!(
        research["evidence_gate_summary"]["reference_project_count"],
        14
    );
    assert_eq!(research["evidence_gate_summary"]["pinned_project_count"], 2);
    assert_eq!(
        research["evidence_gate_summary"]["crosscheck_execution_count"],
        0
    );
    assert_eq!(research["evidence_gate_summary"]["implementation_count"], 0);
    assert_eq!(research["evidence_gate_summary"]["runtime_rule_count"], 0);
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn pinned_reference_manifest_is_complete_and_non_executable() {
    let manifest: Value =
        serde_json::from_str(REFERENCE_SOURCE_MANIFEST).expect("valid source manifest JSON");
    let registry: Value =
        serde_json::from_str(OPEN_SOURCE_REFERENCE_RESEARCH).expect("valid reference JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(
        manifest["manifest_id"],
        "OPEN-SOURCE-SOURCE-PINS-2026-09-07-v1"
    );
    for flag in [
        "third_party_source_vendored",
        "archives_retained_in_repository",
        "reference_execution_allowed",
        "runtime_ingestion_allowed",
        "rule_evidence_allowed",
        "direct_code_copy_allowed",
    ] {
        assert!(!required_bool(&manifest, flag));
    }

    let projects = required_array(&manifest, "projects");
    assert_eq!(projects.len(), 2);
    let mut project_ids = BTreeSet::new();
    for project in projects {
        let project_id = project["project_id"]
            .as_str()
            .expect("manifest project id must be a string");
        assert!(project_ids.insert(project_id));
        let commit = project["commit_sha"]
            .as_str()
            .expect("manifest commit must be a string");
        assert_lower_hex(commit, 40);
        assert_lower_hex(
            project["tree_sha"]
                .as_str()
                .expect("manifest tree must be a string"),
            40,
        );
        assert_lower_hex(
            project["archive"]["sha256"]
                .as_str()
                .expect("archive hash must be a string"),
            64,
        );
        assert!(project["archive"]["bytes"].as_u64().is_some_and(|v| v > 0));
        assert_eq!(project["archive"]["retained_in_repository"], false);
        assert_eq!(
            project["execution_gate"]["crosscheck_execution_allowed"],
            false
        );
        assert!(!required_array(&project["execution_gate"], "blockers").is_empty());

        let mut paths = BTreeSet::new();
        for file in required_array(project, "selected_files") {
            let path = file["path"]
                .as_str()
                .expect("selected file path must be a string");
            assert!(paths.insert(path), "duplicate manifest path: {path}");
            assert!(file["bytes"].as_u64().is_some_and(|v| v > 0));
            assert_lower_hex(
                file["sha256"]
                    .as_str()
                    .expect("selected file hash must be a string"),
                64,
            );
        }
        assert!(!paths.is_empty());

        let registry_project = required_array(&registry, "projects")
            .iter()
            .find(|candidate| candidate["project_id"] == project_id)
            .expect("manifest project must exist in registry");
        assert_eq!(registry_project["source_pin"]["commit_sha"], commit);
        assert_eq!(
            registry_project["source_pin"]["content_hashes_recorded"],
            true
        );
        assert_eq!(
            registry_project["source_pin"]["crosscheck_execution_allowed"],
            false
        );
    }
    assert_eq!(project_ids, BTreeSet::from(["kinastro", "kintaiyi"]));

    let kintaiyi = projects
        .iter()
        .find(|project| project["project_id"] == "kintaiyi")
        .expect("kintaiyi pin");
    assert_eq!(kintaiyi["license_audit"]["license_file_present"], true);
    assert_eq!(kintaiyi["execution_gate"]["license_closed"], true);
    assert_eq!(
        kintaiyi["execution_gate"]["jiuheng_independent_implementation_present"],
        true
    );
    let kinastro = projects
        .iter()
        .find(|project| project["project_id"] == "kinastro")
        .expect("kinastro pin");
    assert_eq!(kinastro["license_audit"]["license_file_present"], false);
    assert_eq!(
        kinastro["license_audit"]["redistribution_allowed_by_project_gate"],
        false
    );
    assert_eq!(kinastro["execution_gate"]["license_closed"], false);
    assert_eq!(
        required_array(&kinastro["license_audit"], "missing_paths"),
        &[Value::String("LICENSE".to_owned())]
    );

    assert_eq!(manifest["gate_summary"]["pinned_project_count"], 2);
    assert_eq!(manifest["gate_summary"]["license_closed_project_count"], 1);
    assert_eq!(
        manifest["gate_summary"]["projects_with_partial_jiuheng_independent_implementation"],
        1
    );
    assert_eq!(manifest["gate_summary"]["crosscheck_execution_count"], 0);
    assert_eq!(manifest["gate_summary"]["runtime_rule_count"], 0);
    assert_object_key_absent(&manifest, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn reference_crosscheck_fixtures_are_non_executable_input_contracts() {
    let fixtures: Value = serde_json::from_str(REFERENCE_CROSSCHECK_FIXTURES)
        .expect("valid reference crosscheck fixture JSON");
    let manifest: Value =
        serde_json::from_str(REFERENCE_SOURCE_MANIFEST).expect("valid source manifest JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(fixtures["schema_version"], 1);
    assert_eq!(fixtures["status"], "input_contract_only_not_executed");
    for flag in [
        "reference_execution_allowed",
        "runtime_ingestion_allowed",
        "rule_evidence_allowed",
        "result_signal_generation_allowed",
    ] {
        assert!(!required_bool(&fixtures, flag));
    }
    assert!(!required_array(&fixtures, "forbidden_output_classes").is_empty());

    let cases = required_array(&fixtures, "cases");
    assert_eq!(cases.len(), 6);
    let mut case_ids = BTreeSet::new();
    let mut methods = BTreeSet::new();
    for case in cases {
        let case_id = case["case_id"].as_str().expect("case id must be a string");
        assert!(case_ids.insert(case_id), "duplicate case id: {case_id}");
        let method = case["method"]
            .as_str()
            .expect("case method must be a string");
        methods.insert(method);
        assert!(case["comparison_status"]
            .as_str()
            .is_some_and(|status| status.starts_with("not_run")));
        assert_eq!(case["trusted_expected_output"], Value::Null);
        assert!(!required_array(case, "requested_fields").is_empty());
        assert!(!required_array(case, "blockers").is_empty());
        if method == "taiyi" {
            assert_eq!(case["input"]["enable_game_theory"], false);
        }

        let project_id = case["reference_project"]
            .as_str()
            .expect("case project must be a string");
        let project = required_array(&manifest, "projects")
            .iter()
            .find(|candidate| candidate["project_id"] == project_id)
            .expect("case project must be pinned in manifest");
        assert_eq!(case["reference_commit"], project["commit_sha"]);
        assert_eq!(
            project["execution_gate"]["crosscheck_execution_allowed"],
            false
        );
    }
    assert_eq!(methods, BTreeSet::from(["qizheng_siyu", "taiyi"]));
    assert_eq!(fixtures["gate_summary"]["case_count"], 6);
    assert_eq!(fixtures["gate_summary"]["executed_case_count"], 0);
    assert_eq!(fixtures["gate_summary"]["trusted_expected_output_count"], 0);
    assert_eq!(fixtures["gate_summary"]["runtime_rule_count"], 0);
    assert_eq!(fixtures["gate_summary"]["decision"], "blocked");
    assert_object_key_absent(&fixtures, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn taiyi_year_position_candidate_is_pinned_tested_and_non_executable() {
    let research: Value = serde_json::from_str(TAIYI_YEAR_POSITION_RESEARCH)
        .expect("valid Taiyi year-position research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "taiyi");
    assert_eq!(
        research["status"],
        "mechanical_candidate_from_supplied_accumulated_year_only"
    );
    for flag in [
        "runtime_ingestion_allowed",
        "calendar_date_derivation_allowed",
        "complete_board_generation_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert!(!required_bool(&research, flag));
    }

    let source = &research["source_identity"];
    assert_eq!(source["kanripo_id"], "KR3g0047");
    assert_lower_hex(
        source["kanripo_commit"]
            .as_str()
            .expect("Kanripo commit must be a string"),
        40,
    );
    assert_eq!(source["facsimile_verified"], false);
    assert_eq!(source["transcription_verified_against_facsimile"], false);

    let raw_files = required_array(source, "raw_files");
    assert_eq!(raw_files.len(), 3);
    let mut raw_paths = BTreeSet::new();
    for file in raw_files {
        assert!(raw_paths.insert(
            file["path"]
                .as_str()
                .expect("Taiyi source path must be a string")
        ));
        assert!(file["bytes"].as_u64().is_some_and(|bytes| bytes > 0));
        assert_lower_hex(
            file["sha256"]
                .as_str()
                .expect("Taiyi source hash must be a string"),
            64,
        );
    }
    assert_eq!(
        raw_paths,
        BTreeSet::from(["KR3g0047_001.txt", "KR3g0047_002.txt", "KR3g0047_003.txt"])
    );

    let fragments = required_array(&research, "received_fragments");
    assert_eq!(fragments.len(), 4);
    let mut fragment_ids = BTreeSet::new();
    for fragment in fragments {
        assert!(fragment_ids.insert(
            fragment["fragment_id"]
                .as_str()
                .expect("Taiyi fragment ID must be a string")
        ));
        assert!(!required_array(fragment, "page_markers").is_empty());
        assert!(fragment["transcription"]
            .as_str()
            .or_else(|| fragment["transcription_summary"].as_str())
            .is_some_and(|text| !text.trim().is_empty()));
    }

    let candidate = &research["candidate"];
    assert_eq!(
        candidate["algorithm_version"],
        TAIYI_ACCUMULATED_YEAR_POSITION_ALGORITHM_VERSION
    );
    assert_eq!(candidate["calendar_epoch_selected"], false);
    assert_eq!(candidate["civil_calendar_converter_present"], false);
    assert_eq!(candidate["center_five_visited"], false);
    let palace_order = required_array(candidate, "forward_palace_order")
        .iter()
        .map(|value| value.as_u64().expect("palace number"))
        .collect::<Vec<_>>();
    assert_eq!(palace_order, [1, 2, 3, 4, 6, 7, 8, 9]);

    let anchors = required_array(&research, "machine_test_anchors");
    assert_eq!(anchors.len(), 10);
    for anchor in anchors {
        let count = anchor["count"].as_u64().expect("anchor count");
        let position = taiyi_year_position_from_accumulated_count(count).unwrap();
        assert_eq!(u64::from(position.palace.number()), anchor["palace"]);
        assert_eq!(
            u64::from(position.year_within_palace),
            anchor["year_within_palace"]
        );
        assert!(!position.interpretation_eligible);
        assert!(!position.signal_eligible);
        assert!(!position.rule_eligible);
    }

    assert!(!required_array(&research, "variants_and_blockers").is_empty());
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(
        research["review_state"]["decision"],
        "chart_only_candidate_no_date_derivation_no_rules_no_signals"
    );
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn taiyi_annual_frame_candidate_matches_archived_cycles_and_stays_non_executable() {
    let research: Value = serde_json::from_str(TAIYI_ANNUAL_FRAME_RESEARCH)
        .expect("valid Taiyi annual-frame research JSON");
    let parent: Value = serde_json::from_str(TAIYI_YEAR_POSITION_RESEARCH)
        .expect("valid Taiyi year-position research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "taiyi");
    assert_eq!(
        research["status"],
        "mechanical_annual_frame_from_supplied_accumulated_year_only"
    );
    for flag in [
        "runtime_ingestion_allowed",
        "civil_date_input_allowed",
        "epoch_selection_allowed",
        "yin_yang_dun_selection_allowed",
        "complete_board_generation_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert!(!required_bool(&research, flag));
    }

    let source = &research["source_identity"];
    assert_eq!(source["parent_archive"], parent["archive_id"]);
    assert_eq!(
        source["kanripo_commit"],
        parent["source_identity"]["kanripo_commit"]
    );
    assert_eq!(source["facsimile_verified"], false);
    assert_eq!(source["transcription_verified_against_facsimile"], false);
    let hashes = source["raw_file_hashes"]
        .as_object()
        .expect("Taiyi annual-frame raw hashes must be an object");
    assert_eq!(hashes.len(), 3);
    for hash in hashes.values() {
        assert_lower_hex(
            hash.as_str().expect("Taiyi source hash must be a string"),
            64,
        );
    }

    let fragments = required_array(&research, "received_fragments");
    assert_eq!(fragments.len(), 6);
    let mut fragment_ids = BTreeSet::new();
    for fragment in fragments {
        assert!(fragment_ids.insert(
            fragment["fragment_id"]
                .as_str()
                .expect("Taiyi annual fragment ID must be a string")
        ));
        assert!(!required_array(fragment, "page_markers").is_empty());
        assert!(!required_array(fragment, "implemented_fields").is_empty());
    }

    let candidate = &research["candidate"];
    assert_eq!(
        candidate["algorithm_version"],
        TAIYI_ANNUAL_FRAME_V1_ALGORITHM_VERSION
    );
    assert!(!required_array(candidate, "outputs").is_empty());
    assert!(!required_array(candidate, "forbidden_outputs").is_empty());

    let anchors = &research["machine_test_anchors"];
    let expected_tianmu = required_array(anchors, "first_24_tianmu");
    assert_eq!(expected_tianmu.len(), 24);
    for count in 1..=24_u64 {
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        assert_eq!(
            frame.tianmu.zh_name(),
            expected_tianmu[(count - 1) as usize]
                .as_str()
                .expect("Tianmu anchor must be a string")
        );
    }

    let expected_jishen = required_array(anchors, "first_12_jishen");
    assert_eq!(expected_jishen.len(), 12);
    for count in 1..=12_u64 {
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        assert_eq!(
            frame.jishen_branch.zh_name(),
            expected_jishen[(count - 1) as usize]
                .as_str()
                .expect("Jishen anchor must be a string")
        );
    }

    for anchor in required_array(anchors, "taisui_heshin") {
        let count = anchor["count"].as_u64().expect("Tai Sui anchor count");
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        assert_eq!(
            frame.tai_sui_designation.zh_name(),
            anchor["tai_sui"]
                .as_str()
                .expect("Tai Sui anchor must be a string")
        );
        assert_eq!(
            frame.heshin_branch.zh_name(),
            anchor["heshin"]
                .as_str()
                .expect("Heshin anchor must be a string")
        );
        assert!(!frame.interpretation_eligible);
        assert!(!frame.signal_eligible);
        assert!(!frame.rule_eligible);
    }

    assert!(!required_array(&research, "variants_and_blockers").is_empty());
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(
        research["review_state"]["decision"],
        "chart_only_partial_annual_frame_no_date_no_full_board_no_rules_no_signals"
    );
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn taiyi_eyes_and_counts_match_all_archived_yang_rows_and_stay_non_executable() {
    let research: Value = serde_json::from_str(TAIYI_EYES_COUNTS_RESEARCH)
        .expect("valid Taiyi eyes/counts research JSON");
    let parent: Value = serde_json::from_str(TAIYI_ANNUAL_FRAME_RESEARCH)
        .expect("valid Taiyi annual-frame research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "taiyi");
    assert_eq!(
        research["status"],
        "mechanical_shiji_main_guest_counts_from_supplied_accumulated_year_only"
    );
    for flag in [
        "runtime_ingestion_allowed",
        "civil_date_input_allowed",
        "epoch_selection_allowed",
        "yin_yang_dun_selection_allowed",
        "large_and_assistant_generals_generation_allowed",
        "complete_board_generation_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert!(!required_bool(&research, flag));
    }

    let source = &research["source_identity"];
    assert_eq!(source["parent_archive"], parent["archive_id"]);
    assert_eq!(
        source["kanripo_commit"],
        parent["source_identity"]["kanripo_commit"]
    );
    assert_eq!(source["facsimile_verified"], false);
    assert_eq!(source["transcription_verified_against_facsimile"], false);
    let hashes = source["raw_file_hashes"]
        .as_object()
        .expect("Taiyi eyes/counts raw hashes must be an object");
    assert_eq!(hashes.len(), 3);
    for hash in hashes.values() {
        assert_lower_hex(
            hash.as_str().expect("Taiyi source hash must be a string"),
            64,
        );
    }

    let fragments = required_array(&research, "received_fragments");
    assert_eq!(fragments.len(), 4);
    for fragment in fragments {
        assert!(!required_array(fragment, "page_markers").is_empty());
        assert!(!required_array(fragment, "implemented_fields").is_empty());
    }

    let candidate = &research["candidate"];
    assert_eq!(
        candidate["algorithm_version"],
        TAIYI_ANNUAL_FRAME_V2_ALGORITHM_VERSION
    );
    assert_eq!(candidate["large_and_assistant_generals_generated"], false);
    assert_eq!(candidate["interpretation_eligible"], false);
    assert_eq!(candidate["signal_eligible"], false);
    assert_eq!(candidate["rule_eligible"], false);

    let anchors = &research["machine_test_anchors"];
    let expected_shiji = required_array(anchors, "all_72_shiji");
    let expected_main = required_array(anchors, "all_72_main_counts");
    let expected_guest = required_array(anchors, "all_72_guest_counts");
    assert_eq!(expected_shiji.len(), 72);
    assert_eq!(expected_main.len(), 72);
    assert_eq!(expected_guest.len(), 72);
    for count in 1..=72_u64 {
        let index = (count - 1) as usize;
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        assert_eq!(
            frame.shiji.zh_name(),
            expected_shiji[index]
                .as_str()
                .expect("Shiji anchor must be a string"),
            "count {count}"
        );
        assert_eq!(
            u64::from(frame.main_count),
            expected_main[index]
                .as_u64()
                .expect("main count anchor must be unsigned"),
            "count {count}"
        );
        assert_eq!(
            u64::from(frame.guest_count),
            expected_guest[index]
                .as_u64()
                .expect("guest count anchor must be unsigned"),
            "count {count}"
        );
    }

    assert!(!required_array(&research, "variants_and_blockers").is_empty());
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(
        research["review_state"]["decision"],
        "chart_only_eyes_and_counts_no_date_no_generals_no_interpretation_no_rules"
    );
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn taiyi_general_palaces_match_archived_cross_volume_anchors_and_stay_non_executable() {
    let research: Value =
        serde_json::from_str(TAIYI_GENERALS_RESEARCH).expect("valid Taiyi generals research JSON");
    let parent: Value = serde_json::from_str(TAIYI_EYES_COUNTS_RESEARCH)
        .expect("valid Taiyi eyes/counts research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "taiyi");
    assert_eq!(
        research["status"],
        "mechanical_main_guest_great_and_assistant_general_palaces_from_existing_counts_only"
    );
    for flag in [
        "runtime_ingestion_allowed",
        "civil_date_input_allowed",
        "epoch_selection_allowed",
        "yin_yang_dun_selection_allowed",
        "complete_board_generation_allowed",
        "general_relationship_classification_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert!(!required_bool(&research, flag));
    }

    let source = &research["source_identity"];
    assert_eq!(source["parent_archive"], parent["archive_id"]);
    assert_eq!(
        source["kanripo_commit"],
        parent["source_identity"]["kanripo_commit"]
    );
    assert_eq!(source["facsimile_verified"], false);
    assert_eq!(source["transcription_verified_against_facsimile"], false);
    let hashes = source["raw_file_hashes"]
        .as_object()
        .expect("Taiyi generals raw hashes must be an object");
    assert_eq!(hashes.len(), 3);
    for hash in hashes.values() {
        assert_lower_hex(
            hash.as_str().expect("Taiyi source hash must be a string"),
            64,
        );
    }

    let fragments = required_array(&research, "received_fragments");
    assert_eq!(fragments.len(), 4);
    let mut fragment_ids = BTreeSet::new();
    for fragment in fragments {
        assert!(fragment_ids.insert(
            fragment["fragment_id"]
                .as_str()
                .expect("Taiyi general fragment ID must be a string")
        ));
        assert!(!required_array(fragment, "page_markers").is_empty());
        assert!(!required_array(fragment, "implemented_fields").is_empty());
    }

    let candidate = &research["candidate"];
    assert_eq!(
        candidate["algorithm_version"],
        TAIYI_ANNUAL_FRAME_V3_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["great_general_palace_includes_central_five"],
        true
    );
    assert_eq!(candidate["interpretation_eligible"], false);
    assert_eq!(candidate["signal_eligible"], false);
    assert_eq!(candidate["rule_eligible"], false);

    let reduce = |count: u64| {
        let units = count % 10;
        if units == 0 {
            count / 10
        } else {
            units
        }
    };
    let anchors = &research["machine_test_anchors"];
    let expected = required_array(anchors, "great_general_palaces_for_counts_1_through_40");
    assert_eq!(expected.len(), 40);
    for count in 1..=40_u64 {
        assert_eq!(
            reduce(count),
            expected[(count - 1) as usize]
                .as_u64()
                .expect("general palace anchor must be unsigned"),
            "count {count}"
        );
    }
    for anchor in required_array(anchors, "cross_volume_cases") {
        let count = anchor["count"].as_u64().expect("general source count");
        let great = reduce(count);
        assert_eq!(great, anchor["great"]);
        assert_eq!(reduce(great * 3), anchor["assistant"]);
    }

    let annual = &anchors["annual_frame_first_yang_row"];
    let frame = taiyi_annual_frame_from_accumulated_count(
        annual["accumulated_year_count"]
            .as_u64()
            .expect("annual-frame count"),
    )
    .unwrap();
    assert_eq!(u64::from(frame.main_count), annual["main_count"]);
    assert_eq!(
        u64::from(frame.main_great_general_palace.number()),
        annual["main_great"]
    );
    assert_eq!(
        u64::from(frame.main_assistant_general_palace.number()),
        annual["main_assistant"]
    );
    assert_eq!(u64::from(frame.guest_count), annual["guest_count"]);
    assert_eq!(
        u64::from(frame.guest_great_general_palace.number()),
        annual["guest_great"]
    );
    assert_eq!(
        u64::from(frame.guest_assistant_general_palace.number()),
        annual["guest_assistant"]
    );
    assert!(!frame.yin_yang_dun_selected);
    assert!(!frame.complete_board_generated);
    assert!(!frame.interpretation_eligible);
    assert!(!frame.signal_eligible);
    assert!(!frame.rule_eligible);

    assert!(!required_array(&research, "variants_and_blockers").is_empty());
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(
        research["review_state"]["decision"],
        "chart_only_general_palaces_no_date_no_relations_no_interpretation_no_rules"
    );
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn taiyi_annual_duty_gate_matches_archived_catalog_and_cycles_and_stays_non_executable() {
    let research: Value = serde_json::from_str(TAIYI_EIGHT_GATE_DUTY_RESEARCH)
        .expect("valid Taiyi eight-gate research JSON");
    let parent: Value =
        serde_json::from_str(TAIYI_GENERALS_RESEARCH).expect("valid Taiyi generals research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "taiyi");
    assert_eq!(
        research["status"],
        "mechanical_eight_gate_catalog_and_annual_duty_gate_from_supplied_accumulated_year_only"
    );
    for flag in [
        "runtime_ingestion_allowed",
        "civil_date_input_allowed",
        "epoch_selection_allowed",
        "yin_yang_dun_selection_allowed",
        "rotated_gate_boards_generation_allowed",
        "three_auspicious_gates_classification_allowed",
        "complete_board_generation_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert!(!required_bool(&research, flag));
    }

    let source = &research["source_identity"];
    assert_eq!(source["parent_archive"], parent["archive_id"]);
    assert_eq!(
        source["kanripo_commit"],
        parent["source_identity"]["kanripo_commit"]
    );
    assert_eq!(source["facsimile_verified"], false);
    assert_eq!(source["transcription_verified_against_facsimile"], false);
    let hashes = source["raw_file_hashes"]
        .as_object()
        .expect("Taiyi eight-gate raw hashes must be an object");
    assert_eq!(hashes.len(), 3);
    for hash in hashes.values() {
        assert_lower_hex(
            hash.as_str().expect("Taiyi source hash must be a string"),
            64,
        );
    }

    let fragments = required_array(&research, "received_fragments");
    assert_eq!(fragments.len(), 3);
    let mut fragment_ids = BTreeSet::new();
    for fragment in fragments {
        assert!(fragment_ids.insert(
            fragment["fragment_id"]
                .as_str()
                .expect("Taiyi eight-gate fragment ID must be a string")
        ));
        assert!(!required_array(fragment, "page_markers").is_empty());
        assert!(!required_array(fragment, "implemented_fields").is_empty());
    }

    let candidate = &research["candidate"];
    assert_eq!(
        candidate["algorithm_version"],
        TAIYI_ANNUAL_FRAME_V4_ALGORITHM_VERSION
    );
    assert_eq!(candidate["rotated_gate_boards_generated"], false);
    assert_eq!(candidate["interpretation_eligible"], false);
    assert_eq!(candidate["signal_eligible"], false);
    assert_eq!(candidate["rule_eligible"], false);

    let anchors = &research["machine_test_anchors"];
    let catalog = required_array(anchors, "catalog");
    assert_eq!(catalog.len(), 8);
    for (index, anchor) in catalog.iter().enumerate() {
        let gate = TaiyiGate::RECEIVED_LEFTWARD_ORDER[index];
        assert_eq!(anchor["index"], index as u64);
        assert_eq!(
            gate.zh_name(),
            anchor["gate"]
                .as_str()
                .expect("Taiyi gate name must be a string")
        );
        assert_eq!(
            u64::from(gate.fixed_taiyi_palace().number()),
            anchor["taiyi_palace"]
        );
    }

    for anchor in required_array(anchors, "cycle_boundaries") {
        let count = anchor["count"].as_u64().expect("Taiyi gate count");
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        assert_eq!(
            u64::from(frame.eight_gate_position_in_720),
            anchor["position_in_720"],
            "count {count}"
        );
        assert_eq!(
            u64::from(frame.eight_gate_position_in_240),
            anchor["position_in_240"],
            "count {count}"
        );
        assert_eq!(
            frame.annual_duty_gate.zh_name(),
            anchor["gate"].as_str().expect("Taiyi duty gate name"),
            "count {count}"
        );
        assert_eq!(
            u64::from(frame.year_within_duty_gate),
            anchor["year_within_gate"],
            "count {count}"
        );
        let gate_boards = frame
            .annual_open_gate_boards
            .as_ref()
            .expect("current annual frame includes the later gate-board candidate");
        assert_eq!(
            frame.rotated_gate_boards_generated,
            gate_boards.all_three_boards_generated
        );
        assert!(!frame.interpretation_eligible);
        assert!(!frame.signal_eligible);
        assert!(!frame.rule_eligible);
    }

    assert!(!required_array(&research, "variants_and_blockers").is_empty());
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(
        research["review_state"]["decision"],
        "chart_only_duty_gate_no_date_no_rotated_boards_no_interpretation_no_rules"
    );
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn taiyi_annual_open_gate_boards_match_archived_rotations_and_fail_closed_at_center() {
    let research: Value = serde_json::from_str(TAIYI_ANNUAL_GATE_BOARDS_RESEARCH)
        .expect("valid Taiyi annual gate-board research JSON");
    let parent: Value = serde_json::from_str(TAIYI_SEVENTY_TWO_BOARD_RESEARCH)
        .expect("valid Taiyi seventy-two-board research JSON");
    let dependency: Value = serde_json::from_str(TAIYI_EIGHT_GATE_DUTY_RESEARCH)
        .expect("valid Taiyi eight-gate research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "taiyi");
    assert_eq!(
        research["status"],
        "annual_open_anchored_gate_rotations_with_center_five_failure_only"
    );
    assert!(required_bool(
        &research,
        "annual_open_anchored_mechanical_rotation_allowed"
    ));
    for flag in [
        "runtime_ingestion_allowed",
        "civil_date_input_allowed",
        "epoch_selection_allowed",
        "time_method_duty_gate_rotation_allowed",
        "center_five_projection_allowed",
        "fixed_count_general_board_generation_allowed",
        "three_auspicious_gates_classification_allowed",
        "complete_board_generation_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert!(!required_bool(&research, flag), "flag {flag}");
    }

    let source = &research["source_identity"];
    assert_eq!(source["parent_archive"], parent["archive_id"]);
    assert_eq!(source["depends_on_archive"], dependency["archive_id"]);
    assert_eq!(
        source["kanripo_commit"],
        parent["source_identity"]["kanripo_commit"]
    );
    assert_eq!(source["facsimile_verified"], false);
    assert_eq!(source["transcription_verified_against_facsimile"], false);
    let hashes = source["raw_file_hashes"]
        .as_object()
        .expect("Taiyi annual gate-board hashes must be an object");
    assert_eq!(hashes.len(), 3);
    for hash in hashes.values() {
        assert_lower_hex(hash.as_str().expect("Taiyi raw hash"), 64);
    }

    let fragments = required_array(&research, "received_fragments");
    assert_eq!(fragments.len(), 3);
    assert_eq!(
        fragments[0]["implemented_fields"]
            .as_array()
            .expect("implemented annual gate fields")
            .len(),
        3
    );
    assert!(required_array(&fragments[1], "implemented_fields").is_empty());

    let candidate = &research["candidate"];
    assert_eq!(
        candidate["algorithm_version"],
        TAIYI_ANNUAL_GATE_BOARDS_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["parent_annual_frame_version"],
        TAIYI_ANNUAL_FRAME_V6_ALGORITHM_VERSION
    );
    assert_eq!(candidate["anchor_gate"], TaiyiGate::Open.zh_name());
    assert_eq!(candidate["fixed_count_general_board_generated"], false);
    assert_eq!(candidate["time_method_duty_gate_rotation_generated"], false);
    assert_eq!(candidate["three_auspicious_gates_classified"], false);
    assert_eq!(candidate["interpretation_eligible"], false);
    assert_eq!(candidate["signal_eligible"], false);
    assert_eq!(candidate["rule_eligible"], false);

    let first = taiyi_annual_frame_from_accumulated_count(1).unwrap();
    let boards = first
        .annual_open_gate_boards
        .as_ref()
        .expect("current frame has annual gate boards");
    let archived_anchors = &research["machine_test_anchors"];
    for (key, board) in [
        ("open_at_palace_one", &boards.taiyi_board),
        (
            "open_at_palace_seven",
            boards.main_great_general_board.as_ref().unwrap(),
        ),
        (
            "open_at_palace_three",
            boards.guest_great_general_board.as_ref().unwrap(),
        ),
    ] {
        let archived = required_array(archived_anchors, key);
        assert_eq!(archived.len(), 8);
        for (placement, expected) in board.placements.iter().zip(archived) {
            assert_eq!(
                placement.gate.zh_name(),
                expected["gate"].as_str().expect("gate name")
            );
            assert_eq!(u64::from(placement.palace.number()), expected["palace"]);
        }
    }

    for anchor in required_array(archived_anchors, "annual_frames") {
        let count = anchor["count"].as_u64().expect("annual count");
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        let boards = frame.annual_open_gate_boards.as_ref().unwrap();
        assert_eq!(
            u64::from(frame.taiyi_position.palace.number()),
            anchor["taiyi"]
        );
        assert_eq!(
            u64::from(frame.main_great_general_palace.number()),
            anchor["main_great"]
        );
        assert_eq!(
            u64::from(frame.guest_great_general_palace.number()),
            anchor["guest_great"]
        );
        let expected_complete = anchor["all_three_boards_generated"]
            .as_bool()
            .expect("all-three flag");
        assert_eq!(boards.all_three_boards_generated, expected_complete);
        assert_eq!(frame.rotated_gate_boards_generated, expected_complete);
        match anchor["blocked_board"].as_str() {
            Some("main_great_general") => {
                assert_eq!(frame.main_great_general_palace, TaiyiGeneralPalace::Five);
                assert!(boards.main_great_general_board.is_none());
                assert!(boards.main_great_general_unavailable_reason.is_some());
            }
            Some("guest_great_general") => {
                assert_eq!(frame.guest_great_general_palace, TaiyiGeneralPalace::Five);
                assert!(boards.guest_great_general_board.is_none());
                assert!(boards.guest_great_general_unavailable_reason.is_some());
            }
            None => {
                assert!(boards.main_great_general_board.is_some());
                assert!(boards.guest_great_general_board.is_some());
            }
            other => panic!("unknown blocked board: {other:?}"),
        }
        assert!(!boards.fixed_count_general_board_generated);
        assert!(!boards.time_method_duty_gate_rotation_generated);
        assert!(!boards.three_auspicious_gates_classified);
        assert!(!boards.interpretation_eligible);
        assert!(!boards.signal_eligible);
        assert!(!boards.rule_eligible);
    }

    assert!(!required_array(&research, "variants_and_blockers").is_empty());
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(
        research["review_state"]["decision"],
        "chart_only_annual_open_anchored_gate_rotations_no_time_method_no_center_projection_no_interpretation_no_rules"
    );
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn taiyi_direct_position_marks_match_archived_anchors_and_stay_non_executable() {
    let research: Value = serde_json::from_str(TAIYI_MECHANICAL_RELATIONS_RESEARCH)
        .expect("valid Taiyi mechanical-relations research JSON");
    let parent: Value = serde_json::from_str(TAIYI_EIGHT_GATE_DUTY_RESEARCH)
        .expect("valid Taiyi eight-gate research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "taiyi");
    assert_eq!(
        research["status"],
        "mechanical_cover_imprisonment_and_ge_position_marks_only"
    );
    assert!(required_bool(
        &research,
        "position_label_generation_allowed"
    ));
    for flag in [
        "runtime_ingestion_allowed",
        "civil_date_input_allowed",
        "epoch_selection_allowed",
        "yin_yang_dun_selection_allowed",
        "strike_classification_allowed",
        "pressure_classification_allowed",
        "barrier_classification_allowed",
        "opposition_classification_allowed",
        "compound_pattern_classification_allowed",
        "complete_board_generation_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert!(!required_bool(&research, flag));
    }

    let source = &research["source_identity"];
    assert_eq!(source["parent_archive"], parent["archive_id"]);
    assert_eq!(
        source["kanripo_commit"],
        parent["source_identity"]["kanripo_commit"]
    );
    assert_eq!(source["facsimile_verified"], false);
    assert_eq!(source["transcription_verified_against_facsimile"], false);
    let hashes = source["raw_file_hashes"]
        .as_object()
        .expect("Taiyi relation raw hashes must be an object");
    assert_eq!(hashes.len(), 3);
    for hash in hashes.values() {
        assert_lower_hex(
            hash.as_str().expect("Taiyi source hash must be a string"),
            64,
        );
    }

    let fragments = required_array(&research, "received_fragments");
    assert_eq!(fragments.len(), 4);
    let mut fragment_ids = BTreeSet::new();
    for fragment in fragments {
        assert!(fragment_ids.insert(
            fragment["fragment_id"]
                .as_str()
                .expect("Taiyi relation fragment ID must be a string")
        ));
        assert!(!required_array(fragment, "page_markers").is_empty());
        assert!(!required_array(fragment, "implemented_fields").is_empty());
    }

    let candidate = &research["candidate"];
    assert_eq!(
        candidate["algorithm_version"],
        TAIYI_MECHANICAL_RELATIONS_V1_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["annual_frame_algorithm_version"],
        TAIYI_ANNUAL_FRAME_V5_ALGORITHM_VERSION
    );
    assert_eq!(candidate["strike_classified"], false);
    assert_eq!(candidate["pressure_classified"], false);
    assert_eq!(candidate["barrier_classified"], false);
    assert_eq!(candidate["opposition_classified"], false);
    assert_eq!(candidate["interpretation_eligible"], false);
    assert_eq!(candidate["signal_eligible"], false);
    assert_eq!(candidate["rule_eligible"], false);

    for anchor in required_array(&research, "machine_test_anchors") {
        let count = anchor["count"].as_u64().expect("Taiyi relation count");
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        let serialized = serde_json::to_value(&frame.mechanical_relations)
            .expect("serialize Taiyi mechanical relations");
        for field in required_array(anchor, "expected_true") {
            let field = field
                .as_str()
                .expect("Taiyi relation field must be a string");
            assert_eq!(serialized[field], true, "count {count}, field {field}");
        }
        if let Some(expected) = anchor["compact_marks_zh"].as_str() {
            assert_eq!(
                frame.mechanical_relations.compact_marks_zh(),
                expected,
                "count {count}"
            );
        }
        assert!(frame.mechanical_relations.strike_classified);
        assert!(frame.mechanical_relations.pressure_classified);
        assert!(!frame.mechanical_relations.barrier_classified);
        assert!(frame.mechanical_relations.opposition_classified);
        assert!(!frame.mechanical_relations.interpretation_eligible);
        assert!(!frame.mechanical_relations.signal_eligible);
        assert!(!frame.mechanical_relations.rule_eligible);
    }

    assert_eq!(
        TAIYI_ANNUAL_FRAME_V4_ALGORITHM_VERSION,
        parent["candidate"]["algorithm_version"]
    );
    assert!(!required_array(&research, "variants_and_blockers").is_empty());
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(
        research["review_state"]["decision"],
        "chart_only_position_marks_no_interpretation_no_rules_no_signals"
    );
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn taiyi_explicit_dun_board_matches_archived_table_and_stays_non_executable() {
    let research: Value = serde_json::from_str(TAIYI_SEVENTY_TWO_BOARD_RESEARCH)
        .expect("valid Taiyi seventy-two-board research JSON");
    let parent: Value = serde_json::from_str(TAIYI_MECHANICAL_RELATIONS_RESEARCH)
        .expect("valid Taiyi relations research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "taiyi");
    assert_eq!(
        research["status"],
        "explicit_yang_or_yin_dun_seventy_two_board_mechanics_only"
    );
    for flag in [
        "explicit_dun_input_allowed",
        "positive_board_count_input_allowed",
        "mechanical_board_generation_allowed",
    ] {
        assert!(required_bool(&research, flag));
    }
    for flag in [
        "runtime_ingestion_allowed",
        "civil_time_input_allowed",
        "automatic_dun_selection_allowed",
        "automatic_board_count_derivation_allowed",
        "complete_board_generation_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert!(!required_bool(&research, flag));
    }

    let source = &research["source_identity"];
    assert_eq!(source["parent_archive"], parent["archive_id"]);
    assert_eq!(
        source["kanripo_commit"],
        parent["source_identity"]["kanripo_commit"]
    );
    assert_eq!(source["facsimile_verified"], false);
    assert_eq!(source["transcription_verified_against_facsimile"], false);
    let hashes = source["raw_file_hashes"]
        .as_object()
        .expect("Taiyi board raw hashes must be an object");
    assert_eq!(hashes.len(), 3);
    for hash in hashes.values() {
        assert_lower_hex(
            hash.as_str().expect("Taiyi source hash must be a string"),
            64,
        );
    }

    let fragments = required_array(&research, "received_fragments");
    assert_eq!(fragments.len(), 4);
    let mut fragment_ids = BTreeSet::new();
    for fragment in fragments {
        assert!(fragment_ids.insert(
            fragment["fragment_id"]
                .as_str()
                .expect("Taiyi board fragment ID must be a string")
        ));
        assert!(!required_array(fragment, "page_markers").is_empty());
        assert!(!required_array(fragment, "implemented_fields").is_empty());
    }

    let candidate = &research["candidate"];
    assert_eq!(
        candidate["algorithm_version"],
        TAIYI_SEVENTY_TWO_BOARD_V1_ALGORITHM_VERSION
    );
    assert_eq!(candidate["dun_derived_from_civil_time"], false);
    assert_eq!(candidate["board_count_derived_from_civil_time"], false);
    assert_eq!(candidate["complete_board_generated"], false);
    assert_eq!(candidate["interpretation_eligible"], false);
    assert_eq!(candidate["signal_eligible"], false);
    assert_eq!(candidate["rule_eligible"], false);

    let table = &research["received_yin_table"];
    let palace_order = required_array(table, "taiyi_palace_order");
    let tianmu_cycle = required_array(table, "tianmu_cycle_with_stops");
    let jishen_cycle = required_array(table, "jishen_cycle");
    let shiji_rows = required_array(table, "shiji_by_row");
    let main_rows = required_array(table, "main_counts_by_row");
    let guest_rows = required_array(table, "guest_counts_by_row");
    assert_eq!(palace_order.len(), 8);
    assert_eq!(tianmu_cycle.len(), 18);
    assert_eq!(jishen_cycle.len(), 12);
    assert_eq!(shiji_rows.len(), 72);
    assert_eq!(main_rows.len(), 72);
    assert_eq!(guest_rows.len(), 72);

    let documented = &research["documented_formula_variants"];
    let documented_shiji_rows: BTreeSet<u64> = required_array(documented, "shiji")
        .iter()
        .map(|item| item["row"].as_u64().expect("Taiyi Shiji variant row"))
        .collect();
    let documented_count_rows: BTreeSet<u64> = required_array(documented, "counts")
        .iter()
        .map(|item| item["row"].as_u64().expect("Taiyi count variant row"))
        .collect();
    let mut actual_shiji_variants = BTreeSet::new();
    let mut actual_count_variants = BTreeSet::new();

    for count in 1..=72_u64 {
        let frame = taiyi_seventy_two_board_frame_from_count(count, TaiyiDun::Yin).unwrap();
        let index = (count - 1) as usize;
        assert_eq!(
            u64::from(frame.taiyi_palace.number()),
            palace_order[index / 3 % 8],
            "count {count}"
        );
        assert_eq!(
            frame.tianmu.zh_name(),
            tianmu_cycle[index % 18]
                .as_str()
                .expect("Taiyi Tianmu table name"),
            "count {count}"
        );
        assert_eq!(
            frame.jishen_branch.zh_name(),
            jishen_cycle[index % 12]
                .as_str()
                .expect("Taiyi Jishen table name"),
            "count {count}"
        );
        if frame.shiji.zh_name() != shiji_rows[index].as_str().expect("Taiyi Shiji table name") {
            actual_shiji_variants.insert(count);
        }
        if u64::from(frame.main_count) != main_rows[index]
            || u64::from(frame.guest_count) != guest_rows[index]
        {
            actual_count_variants.insert(count);
        }
        assert!(!frame.dun_derived_from_civil_time);
        assert!(!frame.board_count_derived_from_civil_time);
        assert!(!frame.complete_board_generated);
        assert!(!frame.interpretation_eligible);
        assert!(!frame.signal_eligible);
        assert!(!frame.rule_eligible);
    }
    assert_eq!(actual_shiji_variants, documented_shiji_rows);
    assert_eq!(actual_count_variants, documented_count_rows);

    let first = &research["machine_test_anchors"]["yin_first_row"];
    let frame = taiyi_seventy_two_board_frame_from_count(1, TaiyiDun::Yin).unwrap();
    assert_eq!(frame.tianmu.zh_name(), first["tianmu"]);
    assert_eq!(frame.jishen_branch.zh_name(), first["jishen"]);
    assert_eq!(frame.shiji.zh_name(), first["shiji"]);
    assert_eq!(u64::from(frame.main_count), first["main_count"]);
    assert_eq!(u64::from(frame.guest_count), first["guest_count"]);
    assert_eq!(
        u64::from(frame.main_great_general_palace.number()),
        first["main_great_general"]
    );
    assert_eq!(
        u64::from(frame.main_assistant_general_palace.number()),
        first["main_assistant_general"]
    );
    assert_eq!(
        u64::from(frame.guest_great_general_palace.number()),
        first["guest_great_general"]
    );
    assert_eq!(
        u64::from(frame.guest_assistant_general_palace.number()),
        first["guest_assistant_general"]
    );

    assert!(!required_array(&research, "variants_and_blockers").is_empty());
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(
        research["review_state"]["decision"],
        "chart_only_explicit_dun_and_count_no_civil_time_selection_no_interpretation_no_rules"
    );
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn taiyi_near_ring_geometry_and_opposition_match_archived_boundaries() {
    let research: Value = serde_json::from_str(TAIYI_NEAR_RING_OPPOSITION_RESEARCH)
        .expect("valid Taiyi near-ring/opposition research JSON");
    let parent: Value = serde_json::from_str(TAIYI_MECHANICAL_RELATIONS_RESEARCH)
        .expect("valid Taiyi mechanical-relations research JSON");
    let dependency: Value = serde_json::from_str(TAIYI_ANNUAL_GATE_BOARDS_RESEARCH)
        .expect("valid Taiyi annual gate-board research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "taiyi");
    assert_eq!(
        research["status"],
        "unoriented_near_ring_geometry_and_direct_opposition_position_mark_only"
    );
    assert!(required_bool(
        &research,
        "near_ring_geometry_generation_allowed"
    ));
    assert!(required_bool(
        &research,
        "opposition_position_label_generation_allowed"
    ));
    for flag in [
        "runtime_ingestion_allowed",
        "civil_date_input_allowed",
        "epoch_selection_allowed",
        "yin_yang_dun_selection_allowed",
        "strike_classification_allowed",
        "pressure_classification_allowed",
        "barrier_classification_allowed",
        "compound_pattern_classification_allowed",
        "complete_board_generation_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert!(!required_bool(&research, flag), "flag {flag}");
    }

    let source = &research["source_identity"];
    assert_eq!(source["parent_archive"], parent["archive_id"]);
    assert_eq!(source["depends_on_archive"], dependency["archive_id"]);
    assert_eq!(
        source["kanripo_commit"],
        parent["source_identity"]["kanripo_commit"]
    );
    assert_eq!(source["facsimile_verified"], false);
    assert_eq!(source["transcription_verified_against_facsimile"], false);
    let hashes = source["raw_file_hashes"]
        .as_object()
        .expect("Taiyi near-ring raw hashes must be an object");
    assert_eq!(hashes.len(), 3);
    for hash in hashes.values() {
        assert_lower_hex(hash.as_str().expect("Taiyi raw hash"), 64);
    }

    let fragments = required_array(&research, "received_fragments");
    assert_eq!(fragments.len(), 4);
    let mut fragment_ids = BTreeSet::new();
    for fragment in fragments {
        assert!(fragment_ids.insert(fragment["fragment_id"].as_str().expect("Taiyi fragment ID")));
        assert!(!required_array(fragment, "page_markers").is_empty());
        let implemented = required_array(fragment, "implemented_fields");
        if fragment["heading"] == "推关法" {
            assert!(implemented.is_empty());
            assert!(!fragment["blocked_reason"]
                .as_str()
                .expect("barrier blocker")
                .is_empty());
        } else {
            assert!(!implemented.is_empty());
        }
    }

    let candidate = &research["candidate"];
    assert_eq!(
        candidate["algorithm_version"],
        TAIYI_MECHANICAL_RELATIONS_V2_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["annual_frame_algorithm_version"],
        TAIYI_ANNUAL_FRAME_V7_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["seventy_two_board_algorithm_version"],
        TAIYI_SEVENTY_TWO_BOARD_V2_ALGORITHM_VERSION
    );
    assert_eq!(required_array(candidate, "received_ring_order").len(), 16);
    assert_eq!(
        candidate["near_geometry_values"]
            .as_object()
            .expect("near geometry dictionary")
            .len(),
        4
    );
    assert_eq!(candidate["near_ring_geometry_computed"], true);
    assert_eq!(candidate["strike_classified"], false);
    assert_eq!(candidate["pressure_classified"], false);
    assert_eq!(candidate["barrier_classified"], false);
    assert_eq!(candidate["opposition_classified"], true);
    assert_eq!(candidate["interpretation_eligible"], false);
    assert_eq!(candidate["signal_eligible"], false);
    assert_eq!(candidate["rule_eligible"], false);

    for anchor in required_array(&research, "machine_test_anchors") {
        let count = anchor["count"].as_u64().expect("Taiyi anchor count");
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        let relations =
            serde_json::to_value(&frame.mechanical_relations).expect("serialize Taiyi relation v2");
        for field in [
            "shiji_near_taiyi_geometry",
            "tianmu_near_taiyi_geometry",
            "tianmu_opposes_taiyi",
        ] {
            if anchor.get(field).is_some() {
                assert_eq!(relations[field], anchor[field], "count {count}, {field}");
            }
        }
        assert!(frame.mechanical_relations.near_ring_geometry_computed);
        assert!(frame.mechanical_relations.strike_classified);
        assert!(frame.mechanical_relations.pressure_classified);
        assert!(!frame.mechanical_relations.barrier_classified);
        assert!(frame.mechanical_relations.opposition_classified);
        assert!(!frame.mechanical_relations.interpretation_eligible);
        assert!(!frame.mechanical_relations.signal_eligible);
        assert!(!frame.mechanical_relations.rule_eligible);
    }

    for count in 1..=72 {
        for dun in [TaiyiDun::Yang, TaiyiDun::Yin] {
            let frame = taiyi_seventy_two_board_frame_from_count(count, dun).unwrap();
            assert_eq!(
                frame.mechanical_relations.algorithm_version,
                TAIYI_MECHANICAL_RELATIONS_ALGORITHM_VERSION
            );
            assert!(frame.mechanical_relations.near_ring_geometry_computed);
            assert!(frame.mechanical_relations.opposition_classified);
            assert!(!frame.mechanical_relations.rule_eligible);
        }
    }

    assert!(!required_array(&research, "variants_and_blockers").is_empty());
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(
        research["review_state"]["decision"],
        "chart_only_unoriented_near_geometry_and_opposition_mark_no_strike_pressure_barrier_interpretation_or_rules"
    );
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn taiyi_strike_and_pressure_match_archived_direction_anchors_without_rules() {
    let research: Value = serde_json::from_str(TAIYI_STRIKE_PRESSURE_RESEARCH)
        .expect("valid Taiyi strike/pressure research JSON");
    let parent: Value = serde_json::from_str(TAIYI_NEAR_RING_OPPOSITION_RESEARCH)
        .expect("valid Taiyi near-ring/opposition research JSON");
    let dependency: Value = serde_json::from_str(TAIYI_SEVENTY_TWO_BOARD_RESEARCH)
        .expect("valid Taiyi seventy-two-board research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "taiyi");
    assert_eq!(
        research["status"],
        "directional_strike_and_pressure_position_marks_only"
    );
    assert!(required_bool(
        &research,
        "strike_position_classification_allowed"
    ));
    assert!(required_bool(
        &research,
        "pressure_position_classification_allowed"
    ));
    for flag in [
        "runtime_ingestion_allowed",
        "civil_date_input_allowed",
        "epoch_selection_allowed",
        "yin_yang_dun_selection_allowed",
        "barrier_classification_allowed",
        "compound_pattern_classification_allowed",
        "complete_board_generation_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert!(!required_bool(&research, flag), "flag {flag}");
    }

    let source = &research["source_identity"];
    assert_eq!(source["parent_archive"], parent["archive_id"]);
    assert_eq!(source["depends_on_archive"], dependency["archive_id"]);
    assert_eq!(
        source["kanripo_commit"],
        parent["source_identity"]["kanripo_commit"]
    );
    assert_eq!(source["facsimile_verified"], false);
    assert_eq!(source["transcription_verified_against_facsimile"], false);
    let hashes = source["raw_file_hashes"]
        .as_object()
        .expect("Taiyi strike/pressure raw hashes must be an object");
    assert_eq!(hashes.len(), 3);
    for hash in hashes.values() {
        assert_lower_hex(hash.as_str().expect("Taiyi raw hash"), 64);
    }

    let fragments = required_array(&research, "received_fragments");
    assert_eq!(fragments.len(), 4);
    let mut fragment_ids = BTreeSet::new();
    for fragment in fragments {
        assert!(fragment_ids.insert(
            fragment["fragment_id"]
                .as_str()
                .expect("Taiyi strike/pressure fragment ID")
        ));
        assert!(!required_array(fragment, "page_markers").is_empty());
        let implemented = required_array(fragment, "implemented_fields");
        if fragment["heading"] == "推关法" {
            assert!(implemented.is_empty());
            assert!(!fragment["blocked_reason"]
                .as_str()
                .expect("barrier blocker")
                .is_empty());
        } else {
            assert!(!implemented.is_empty());
        }
    }

    let candidate = &research["candidate"];
    assert_eq!(
        candidate["algorithm_version"],
        TAIYI_MECHANICAL_RELATIONS_V3_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["annual_frame_algorithm_version"],
        TAIYI_ANNUAL_FRAME_V8_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["seventy_two_board_algorithm_version"],
        TAIYI_SEVENTY_TWO_BOARD_V3_ALGORITHM_VERSION
    );
    let direction_map = candidate["direction_map"]
        .as_object()
        .expect("Taiyi direction map");
    assert_eq!(direction_map.len(), 4);
    assert_eq!(
        direction_map["plus_one_interstitial"]["strike"],
        "front_chen"
    );
    assert_eq!(
        direction_map["minus_one_interstitial"]["pressure"],
        "inner_chen"
    );
    assert_eq!(
        direction_map["plus_two_principal_palace"]["pressure"],
        "outer_palace"
    );
    assert_eq!(
        direction_map["minus_two_principal_palace"]["strike"],
        "inner_palace"
    );
    let pressure_subjects: BTreeSet<&str> = required_array(candidate, "pressure_subjects")
        .iter()
        .map(|item| item.as_str().expect("pressure subject"))
        .collect();
    assert_eq!(pressure_subjects.len(), 5);
    assert_eq!(candidate["shiji_pressure_generated"], false);
    assert_eq!(candidate["strike_classified"], true);
    assert_eq!(candidate["pressure_classified"], true);
    assert_eq!(candidate["barrier_classified"], false);
    assert_eq!(candidate["opposition_classified"], true);
    assert_eq!(candidate["interpretation_eligible"], false);
    assert_eq!(candidate["signal_eligible"], false);
    assert_eq!(candidate["rule_eligible"], false);

    for anchor in required_array(&research, "annual_eye_test_anchors") {
        let count = anchor["count"].as_u64().expect("Taiyi eye anchor count");
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        let relations =
            serde_json::to_value(&frame.mechanical_relations).expect("serialize Taiyi relation v3");
        assert_eq!(
            relations["shiji_strike"], anchor["shiji_strike"],
            "count {count}, shiji"
        );
        assert_eq!(
            relations["tianmu_pressure"], anchor["tianmu_pressure"],
            "count {count}, tianmu"
        );
    }
    for anchor in required_array(&research, "annual_general_test_anchors") {
        let count = anchor["count"]
            .as_u64()
            .expect("Taiyi general anchor count");
        let field = anchor["field"].as_str().expect("Taiyi pressure field");
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        let relations =
            serde_json::to_value(&frame.mechanical_relations).expect("serialize Taiyi relation v3");
        assert_eq!(
            relations[field], anchor["expected"],
            "count {count}, {field}"
        );
    }

    for count in 1..=720 {
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        let relations = &frame.mechanical_relations;
        assert!(relations.near_ring_geometry_computed);
        assert!(relations.strike_classified);
        assert!(relations.pressure_classified);
        assert!(!relations.barrier_classified);
        assert!(relations.opposition_classified);
        assert!(!relations.interpretation_eligible);
        assert!(!relations.signal_eligible);
        assert!(!relations.rule_eligible);
    }
    for count in 1..=72 {
        for dun in [TaiyiDun::Yang, TaiyiDun::Yin] {
            let frame = taiyi_seventy_two_board_frame_from_count(count, dun).unwrap();
            assert_eq!(
                frame.mechanical_relations.algorithm_version,
                TAIYI_MECHANICAL_RELATIONS_ALGORITHM_VERSION
            );
            assert!(frame.mechanical_relations.strike_classified);
            assert!(frame.mechanical_relations.pressure_classified);
            assert!(!frame.mechanical_relations.barrier_classified);
            assert!(!frame.mechanical_relations.rule_eligible);
        }
    }

    assert!(!required_array(&research, "variants_and_blockers").is_empty());
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(
        research["review_state"]["decision"],
        "chart_only_directional_strike_and_pressure_marks_no_barrier_interpretation_signals_or_rules"
    );
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}

#[test]
fn taiyi_barrier_raw_observations_are_complete_but_never_classify_barrier() {
    let research: Value = serde_json::from_str(TAIYI_BARRIER_RAW_OBSERVATIONS_RESEARCH)
        .expect("valid Taiyi barrier raw-observations research JSON");
    let parent: Value = serde_json::from_str(TAIYI_STRIKE_PRESSURE_RESEARCH)
        .expect("valid Taiyi strike/pressure research JSON");
    let dependency: Value = serde_json::from_str(TAIYI_SEVENTY_TWO_BOARD_RESEARCH)
        .expect("valid Taiyi seventy-two-board research JSON");
    let runtime: Value = serde_json::from_str(RUNTIME_RULE_SEED).expect("valid rule seed JSON");

    assert_eq!(research["schema_version"], 1);
    assert_eq!(research["method"], "taiyi");
    assert_eq!(
        research["status"],
        "barrier_raw_observations_only_no_barrier_classification"
    );
    assert!(required_bool(
        &research,
        "barrier_raw_observation_generation_allowed"
    ));
    for flag in [
        "runtime_ingestion_allowed",
        "civil_date_input_allowed",
        "epoch_selection_allowed",
        "yin_yang_dun_selection_allowed",
        "barrier_classification_allowed",
        "compound_pattern_classification_allowed",
        "complete_board_generation_allowed",
        "interpretation_allowed",
        "signal_generation_allowed",
        "timing_generation_allowed",
        "advice_generation_allowed",
        "rule_id_generation_allowed",
    ] {
        assert!(!required_bool(&research, flag), "flag {flag}");
    }

    let source = &research["source_identity"];
    assert_eq!(source["parent_archive"], parent["archive_id"]);
    assert_eq!(source["depends_on_archive"], dependency["archive_id"]);
    assert_eq!(
        source["kanripo_commit"],
        parent["source_identity"]["kanripo_commit"]
    );
    assert_eq!(source["facsimile_verified"], false);
    assert_eq!(source["transcription_verified_against_facsimile"], false);
    let hashes = source["raw_file_hashes"]
        .as_object()
        .expect("Taiyi barrier raw-observation hashes must be an object");
    assert_eq!(hashes.len(), 3);
    for hash in hashes.values() {
        assert_lower_hex(hash.as_str().expect("Taiyi raw hash"), 64);
    }

    let fragments = required_array(&research, "received_fragments");
    assert_eq!(fragments.len(), 2);
    let mut fragment_ids = BTreeSet::new();
    for fragment in fragments {
        assert!(fragment_ids.insert(
            fragment["fragment_id"]
                .as_str()
                .expect("Taiyi barrier fragment ID")
        ));
        assert!(!required_array(fragment, "page_markers").is_empty());
        assert_eq!(fragment["barrier_label_implemented"], false);
    }

    let candidate = &research["candidate"];
    assert_eq!(
        candidate["algorithm_version"],
        TAIYI_MECHANICAL_RELATIONS_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["barrier_raw_observations_algorithm_version"],
        TAIYI_BARRIER_RAW_OBSERVATIONS_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["annual_frame_algorithm_version"],
        TAIYI_ANNUAL_FRAME_ALGORITHM_VERSION
    );
    assert_eq!(
        candidate["seventy_two_board_algorithm_version"],
        TAIYI_SEVENTY_TWO_BOARD_ALGORITHM_VERSION
    );
    assert_eq!(
        required_array(candidate, "eye_general_candidate_pairs").len(),
        8
    );
    assert_eq!(
        required_array(candidate, "general_candidate_pairs").len(),
        6
    );
    assert_eq!(candidate["candidate_pairs_exhaustively_enumerated"], true);
    assert_eq!(candidate["received_pairing_resolved"], false);
    assert_eq!(candidate["received_number_equality_resolved"], false);
    assert_eq!(candidate["barrier_classified"], false);
    assert_eq!(candidate["compound_patterns_classified"], false);
    assert_eq!(candidate["interpretation_eligible"], false);
    assert_eq!(candidate["signal_eligible"], false);
    assert_eq!(candidate["rule_eligible"], false);

    for anchor in required_array(&research, "annual_test_anchors") {
        let count = anchor["count"]
            .as_u64()
            .expect("Taiyi barrier anchor count");
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        let relations = &frame.mechanical_relations;
        let raw = relations
            .barrier_raw_observations
            .as_ref()
            .expect("current Taiyi frame has raw barrier observations");
        let serialized = serde_json::to_value(raw).expect("serialize barrier observations");
        assert_eq!(
            serialized["eye_general_same_palace"], anchor["eye_general_same_palace"],
            "count {count}, eye-general pairs"
        );
        assert_eq!(
            serialized["general_same_palace"], anchor["general_same_palace"],
            "count {count}, general pairs"
        );
        assert_eq!(
            serialized["main_guest_counts_equal"], anchor["main_guest_counts_equal"],
            "count {count}, count equality"
        );
        assert_eq!(serialized["barrier_classified"], false);
        assert!(!relations.barrier_classified);
    }

    for count in 1..=720 {
        let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
        let raw = frame
            .mechanical_relations
            .barrier_raw_observations
            .as_ref()
            .expect("current annual frame has raw barrier observations");
        assert!(raw.candidate_pairs_exhaustively_enumerated);
        assert!(!raw.received_pairing_resolved);
        assert!(!raw.received_number_equality_resolved);
        assert!(!raw.barrier_classified);
        assert!(!raw.interpretation_eligible);
        assert!(!raw.signal_eligible);
        assert!(!raw.rule_eligible);
        assert!(!frame.mechanical_relations.barrier_classified);
    }
    for count in 1..=72 {
        for dun in [TaiyiDun::Yang, TaiyiDun::Yin] {
            let frame = taiyi_seventy_two_board_frame_from_count(count, dun).unwrap();
            assert!(frame
                .mechanical_relations
                .barrier_raw_observations
                .is_some());
            assert!(!frame.mechanical_relations.barrier_classified);
            assert!(!frame.mechanical_relations.rule_eligible);
        }
    }

    assert!(!required_array(&research, "variants_and_blockers").is_empty());
    assert_eq!(research["review_state"]["runtime_release"], "blocked");
    assert_eq!(
        research["review_state"]["decision"],
        "chart_only_barrier_raw_observations_no_barrier_classification_interpretation_signals_or_rules"
    );
    assert_object_key_absent(&research, "formal_rule_ids");
    assert!(required_array(&runtime, "rules").is_empty());
}
