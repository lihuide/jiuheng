                                                                              
                                                                            

use divination_core::{taiyi_seventy_two_board_frame_from_count, TaiyiDun};
use serde_json::{json, Value};

pub const FIXTURE: &str =
    include_str!("../../../../data/research/kintaiyi-counts-static-reference.json");

pub fn fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("valid static research fixture")
}

pub fn compare(reference: &Value) -> Value {
    let mut differences = Vec::new();
    let mut matching_boards = 0;
    let mut matching_cells = 0;
    for (name, dun) in [("yang", TaiyiDun::Yang), ("yin", TaiyiDun::Yin)] {
        let rows = reference[name].as_array().expect("reference table");
        assert_eq!(rows.len(), 72);
        for (index, row) in rows.iter().enumerate() {
            let counts = row.as_array().expect("reference row");
            assert_eq!(counts.len(), 2);
            let board = index as u64 + 1;
            let frame = taiyi_seventy_two_board_frame_from_count(board, dun).unwrap();
            assert!(!frame.dun_derived_from_civil_time);
            assert!(!frame.board_count_derived_from_civil_time);
            assert!(!frame.complete_board_generated);
            assert!(!frame.interpretation_eligible);
            assert!(!frame.signal_eligible);
            assert!(!frame.rule_eligible);
            let mut board_matches = true;
            for (column, field, actual) in [
                (0, "main_count", frame.main_count),
                (1, "guest_count", frame.guest_count),
            ] {
                let expected = counts[column].as_u64().expect("unsigned reference count");
                assert!((1..=40).contains(&expected));
                if u64::from(actual) == expected {
                    matching_cells += 1;
                } else {
                    board_matches = false;
                    differences.push(json!({
                        "dun": name, "board": board, "field": field,
                        "jiuheng": actual, "reference": expected,
                    }));
                }
            }
            matching_boards += usize::from(board_matches);
        }
    }
    json!({
        "audit_id": reference["audit_id"],
        "recorded_on": reference["recorded_on"],
        "core_package_version": env!("CARGO_PKG_VERSION"),
        "status": "executed_jiuheng_against_pinned_static_literal_tables_with_documented_differences",
        "reference_commit": reference["commit_sha"],
        "reference_source_sha256": reference["source"]["sha256"],
        "comparison_scope": "explicit_dun_and_board_number_main_guest_counts_only",
        "third_party_code_executed": false,
        "complete_time_to_board_crosscheck": false,
        "ancient_evidence_verified": false,
        "runtime_rule_count": 0,
        "summary": {
            "boards": 144, "cells": 288,
            "matching_boards": matching_boards, "matching_cells": matching_cells,
            "different_boards": 144 - matching_boards,
            "different_cells": differences.len(),
        },
        "differences": differences,
    })
}
