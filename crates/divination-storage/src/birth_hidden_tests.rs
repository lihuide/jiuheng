use super::*;
use chrono::{FixedOffset, TimeZone};

fn birth(hour: u32, minute: u32) -> BirthParameters {
    BirthParameters {
        sex: BiologicalSex::Female,
        date_time_bjt: FixedOffset::east_opt(8 * 3600)
            .unwrap()
            .with_ymd_and_hms(1990, 5, 17, hour, minute, 0)
            .unwrap(),
        location: crate::tests::sample_question().location,
    }
}

#[test]
fn hidden_stems_persist_and_any_changed_identity_fails_replay() {
    let mut storage = Storage::open_in_memory().unwrap();
    let mut question = crate::tests::sample_question();
    question.category = divination_core::QuestionCategory::LongTermFate;
    question.text = "未来长期趋势如何？".into();
    question.target_event = "长期趋势".into();
    question.birth = Some(birth(8, 30));
    let classification = divination_core::KeywordClassifier::default()
        .classify(question.category, &question.text)
        .unwrap();
    storage
        .insert_locked_question(
            &question,
            &classification,
            &build_event_fingerprint(&question, &classification),
            None,
        )
        .unwrap();
    let snapshot = storage
        .load_astronomy_snapshot(question.id)
        .unwrap()
        .unwrap();
    assert_eq!(snapshot.snapshot_version, 30);
    let saved = snapshot.birth_time_audit.unwrap();
    assert_eq!(saved.audit_version, 14);
    assert_eq!(saved.four_pillars_hidden_stems.as_ref().unwrap().len(), 1);
    let current = calculate_birth_time_audit_snapshot(question.birth.as_ref().unwrap()).unwrap();
    assert!(birth_time_audits_match(&current, &saved));
    let mut damaged = saved;
    damaged.four_pillars_hidden_stems.as_mut().unwrap()[0].pillars[0].stems[0].ten_god =
        divination_core::TenGodIdentity::Peer;
    assert!(!birth_time_audits_match(&current, &damaged));
}

#[test]
fn late_zi_hidden_stems_follow_both_actual_day_and_hour_candidates() {
    let audit = calculate_birth_time_audit_snapshot(&birth(23, 50)).unwrap();
    let candidates = audit.four_pillars_hidden_stems.unwrap();
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].day_master, divination_core::HeavenlyStem::Ren);
    assert_eq!(candidates[1].day_master, divination_core::HeavenlyStem::Gui);
    assert_eq!(
        candidates[0].pillars[2].branch,
        divination_core::EarthlyBranch::Wu
    );
    assert_eq!(
        candidates[1].pillars[2].branch,
        divination_core::EarthlyBranch::Wei
    );
    assert_eq!(
        candidates[0].pillars[3].stems[0].ten_god,
        divination_core::TenGodIdentity::RobWealth
    );
    assert_eq!(
        candidates[1].pillars[3].stems[0].ten_god,
        divination_core::TenGodIdentity::Peer
    );
    assert!(candidates.iter().all(|item| !item.weights_assigned
        && !item.strength_assessed
        && !item.rule_eligible
        && !item.interpretation_eligible));
    assert!(audit
        .late_zi_dual_candidates
        .unwrap()
        .selected_convention
        .is_none());
}

#[test]
fn version_13_birth_snapshot_stays_readable_without_backfilling_hidden_stems() {
    let current = calculate_birth_time_audit_snapshot(&birth(8, 30)).unwrap();
    let mut value = serde_json::to_value(&current).unwrap();
    value["audit_version"] = 13.into();
    value
        .as_object_mut()
        .unwrap()
        .remove("four_pillars_hidden_stems");
    let old: BirthTimeAuditSnapshot = serde_json::from_value(value).unwrap();
    assert!(old.four_pillars_hidden_stems.is_none());
    assert!(birth_time_audits_match(&current, &old));
    let mut current_missing = current.clone();
    current_missing.four_pillars_hidden_stems = None;
    assert!(!birth_time_audits_match(&current, &current_missing));
}
