use super::*;

fn locked(storage: &mut Storage) -> Uuid {
    let question = crate::tests::sample_question();
    let classification = KeywordClassifier::default()
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
    question.id
}

fn profile() -> TaiyiYearEpochProfile {
    TaiyiYearEpochProfile::ReceivedKaiyuan12Year724Count1937281
}

#[test]
fn rejects_invalid_inputs_missing_dun_profile_and_unlocked_question() {
    for request in [
        TaiyiRequest::AccumulatedYear { count: 0 },
        TaiyiRequest::ExplicitBoard {
            count: 0,
            dun: TaiyiDun::Yin,
        },
        TaiyiRequest::TraditionalYear {
            year: -1000,
            profile: profile(),
        },
        TaiyiRequest::TraditionalYear {
            year: 3001,
            profile: profile(),
        },
    ] {
        assert!(request.calculate().is_err());
    }
    for json in [
        r#"{"mode":"explicit_board","count":1}"#,
        r#"{"mode":"traditional_year","year":2026}"#,
        r#"{"mode":"accumulated_year","count":1,"civil_date":"2026-01-01"}"#,
    ] {
        assert!(serde_json::from_str::<TaiyiRequest>(json).is_err());
    }
    let mut storage = Storage::open_in_memory().unwrap();
    assert!(storage
        .save_taiyi_candidate(Uuid::new_v4(), TaiyiRequest::AccumulatedYear { count: 1 })
        .is_err());
    assert_eq!(storage.question_count().unwrap(), 0);
}

#[test]
fn persists_distinct_modes_reopens_idempotently_and_keeps_question_unchanged() {
    let root = std::env::temp_dir().join(format!("jiuheng-taiyi-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("records.sqlite3");
    let requests = [
        TaiyiRequest::AccumulatedYear { count: 1_938_583 },
        TaiyiRequest::TraditionalYear {
            year: 2026,
            profile: profile(),
        },
        TaiyiRequest::ExplicitBoard {
            count: 72,
            dun: TaiyiDun::Yang,
        },
        TaiyiRequest::ExplicitBoard {
            count: 72,
            dun: TaiyiDun::Yin,
        },
    ];
    let (id, records, snapshot) = {
        let mut storage = Storage::open(&path).unwrap();
        let id = locked(&mut storage);
        let snapshot = storage.load_astronomy_snapshot(id).unwrap();
        let records = requests
            .iter()
            .map(|r| storage.save_taiyi_candidate(id, r.clone()).unwrap())
            .collect::<Vec<_>>();
        (id, records, snapshot)
    };
    {
        let mut storage = Storage::open(&path).unwrap();
        for record in &records {
            assert_eq!(
                storage
                    .save_taiyi_candidate(id, record.request.clone())
                    .unwrap(),
                *record
            );
        }
        assert_eq!(storage.load_taiyi_candidates(id).unwrap().len(), 4);
        assert_eq!(storage.load_astronomy_snapshot(id).unwrap(), snapshot);
        assert_eq!(storage.question_count().unwrap(), 1);
        assert!(storage.load_liuyao_artifact(id).unwrap().is_none());
        assert!(storage
            .connection
            .execute("DELETE FROM taiyi_candidates", [])
            .is_err());
        assert!(storage
            .connection
            .execute("UPDATE taiyi_candidates SET request_hash = 'changed'", [])
            .is_err());
        let other = locked(&mut storage);
        assert!(storage.load_taiyi_candidates(other).unwrap().is_empty());
        assert_ne!(
            storage
                .save_taiyi_candidate(other, requests[0].clone())
                .unwrap()
                .id,
            records[0].id
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn corruption_rehashed_wrong_frame_and_record_swaps_stop_read_and_append() {
    let mut storage = Storage::open_in_memory().unwrap();
    let id = locked(&mut storage);
    let request = TaiyiRequest::AccumulatedYear { count: 1 };
    let original = storage.save_taiyi_candidate(id, request.clone()).unwrap();
    storage
        .connection
        .execute_batch("DROP TRIGGER prevent_taiyi_candidate_update;")
        .unwrap();
    storage
        .connection
        .execute("UPDATE taiyi_candidates SET content_hash = 'bad'", [])
        .unwrap();
    assert!(storage.load_taiyi_candidates(id).is_err());
    assert!(storage.save_taiyi_candidate(id, request).is_err());
    let mut damaged = original.clone();
    damaged.calculation = TaiyiRequest::AccumulatedYear { count: 2 }
        .calculate()
        .unwrap();
    let json = serde_json::to_string(&damaged).unwrap();
    storage
        .connection
        .execute(
            "UPDATE taiyi_candidates SET snapshot_json = ?1, content_hash = ?2",
            params![json, blake3::hash(json.as_bytes()).to_hex().to_string()],
        )
        .unwrap();
    assert!(storage.load_taiyi_candidates(id).is_err());
    let mut swapped = original.clone();
    swapped.question_id = Uuid::new_v4();
    let json = serde_json::to_string(&swapped).unwrap();
    storage
        .connection
        .execute(
            "UPDATE taiyi_candidates SET snapshot_json = ?1, content_hash = ?2",
            params![json, blake3::hash(json.as_bytes()).to_hex().to_string()],
        )
        .unwrap();
    assert!(storage.load_taiyi_candidates(id).is_err());
    let json = serde_json::to_string(&original).unwrap();
    storage.connection.execute("UPDATE taiyi_candidates SET snapshot_json = ?1, content_hash = ?2, created_at = 'different'", params![json, blake3::hash(json.as_bytes()).to_hex().to_string()]).unwrap();
    assert!(storage.load_taiyi_candidates(id).is_err());
}

#[test]
fn schema_13_upgrade_preserves_existing_snapshot_and_does_not_backfill_taiyi() {
    let root = std::env::temp_dir().join(format!("jiuheng-taiyi-upgrade-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("records.sqlite3");
    let (id, snapshot) = {
        let mut storage = Storage::open(&path).unwrap();
        let id = locked(&mut storage);
        let snapshot = storage.load_astronomy_snapshot(id).unwrap();
        storage
            .connection
            .execute_batch(
                "DROP TABLE taiyi_candidates; DELETE FROM schema_migrations WHERE version = 14;",
            )
            .unwrap();
        (id, snapshot)
    };
    {
        let mut storage = Storage::open(&path).unwrap();
        assert_eq!(storage.load_astronomy_snapshot(id).unwrap(), snapshot);
        assert!(storage.load_taiyi_candidates(id).unwrap().is_empty());
        assert_eq!(
            storage
                .connection
                .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| r
                    .get::<_, u32>(0))
                .unwrap(),
            14
        );
        storage
            .save_taiyi_candidate(id, TaiyiRequest::AccumulatedYear { count: 1 })
            .unwrap();
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn maximum_count_and_bce_year_survive_json_storage_without_calendar_authorization() {
    let mut storage = Storage::open_in_memory().unwrap();
    let id = locked(&mut storage);
    for request in [
        TaiyiRequest::AccumulatedYear { count: u64::MAX },
        TaiyiRequest::ExplicitBoard {
            count: u64::MAX,
            dun: TaiyiDun::Yin,
        },
        TaiyiRequest::TraditionalYear {
            year: 0,
            profile: profile(),
        },
    ] {
        let record = storage.save_taiyi_candidate(id, request).unwrap();
        match record.calculation {
            TaiyiCalculation::Annual(f) => assert!(
                !f.calendar_epoch_selected
                    && !f.derived_from_civil_date
                    && !f.interpretation_eligible
                    && !f.rule_eligible
            ),
            TaiyiCalculation::Board(f) => assert!(
                !f.dun_derived_from_civil_time
                    && !f.board_count_derived_from_civil_time
                    && !f.interpretation_eligible
            ),
            TaiyiCalculation::Traditional(f) => assert!(
                !f.civil_date_converted
                    && !f.traditional_year_boundary_selected
                    && !f.runtime_epoch_selection_authorized
                    && !f.rule_eligible
            ),
        }
    }
    assert_eq!(storage.load_taiyi_candidates(id).unwrap().len(), 3);
}
