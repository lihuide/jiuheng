use super::*;
use chrono::{FixedOffset, TimeZone};

pub(crate) fn fixture(storage: &mut Storage, year: i32, month: u32, day: u32) -> Uuid {
    let now = FixedOffset::east_opt(8 * 3600)
        .unwrap()
        .with_ymd_and_hms(year, month, day, 13, 5, 0)
        .unwrap();
    let question = QuestionInput {
        id: Uuid::new_v4(),
        category: QuestionCategory::Weather,
        subcategory: "降雨".into(),
        text: "一小时后会下雨吗？ <script>alert('test')</script> & 记录".into(),
        subject: "深圳市南山区".into(),
        object: "降雨".into(),
        target_event: "下雨".into(),
        result_terms: vec!["雨".into()],
        options: vec![],
        location: GeoLocation {
            country_code: "CN".into(),
            province: "广东省".into(),
            city: "深圳市".into(),
            district: Some("南山区".into()),
            street: None,
            longitude_deg: 113.9304,
            latitude_deg: 22.5333,
            coordinate_precision: CoordinatePrecision::DistrictCenter,
        },
        divination_time_bjt: now,
        target_time_text: "一小时后".into(),
        locked_at_bjt: now,
        birth: None,
    };
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

#[test]
fn saved_time_boards_reopen_export_and_keep_distinct_hours_without_casting() {
    let root = std::env::temp_dir().join(format!("jiuheng-time-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("data.sqlite3");
    let (id, original) = {
        let mut storage = Storage::open(&path).unwrap();
        let id = fixture(&mut storage, 2026, 11, 12);
        let report = load(&storage, id).unwrap();
        assert_eq!(report.meihua.len(), 2);
        assert_eq!(report.qimen.len(), 2);
        assert_ne!(report.meihua[0].primary_bits, report.meihua[1].primary_bits);
        assert!(report.meihua_notice.contains("不同"));
        for board in &report.meihua {
            assert_eq!(
                board.primary_bits ^ board.changed_bits,
                1 << (board.moving_line - 1)
            );
            assert_eq!(board.calculation.len(), 3);
            assert!(board.middle.contains("未选择互卦口径"));
        }
        let layout = QIMEN_LAYOUT
            .into_iter()
            .flatten()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(layout, (1..=9).collect());
        for board in &report.qimen {
            assert_eq!(board.cells.len(), 9);
            assert_eq!(
                board
                    .cells
                    .iter()
                    .map(|c| c.number)
                    .collect::<std::collections::BTreeSet<_>>(),
                layout
            );
            assert_eq!(
                board
                    .cells
                    .iter()
                    .map(|c| &c.stem)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                9
            );
            assert_eq!(
                board
                    .cells
                    .iter()
                    .filter(|c| c.markers.contains(&"旬首地盘宫".into()))
                    .count(),
                1
            );
            assert!(board.context.iter().any(|s| s.contains("逐时落宫尚未计算")));
            assert!(board.missing.is_empty());
        }
        assert!(storage.load_liuyao_artifact(id).unwrap().is_none());
        (id, report.text())
    };
    {
        let storage = Storage::open(&path).unwrap();
        let snapshot = storage.load_astronomy_snapshot(id).unwrap();
        let report = load(&storage, id).unwrap();
        assert_eq!(report.text(), original);
        assert!(report.text().contains("2026-11-12T13:05:00+08:00"));
        let html = report.html();
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("default-src 'none'"));
        assert_eq!(html.matches("<div class=palace>").count(), 18);
        assert_eq!(html.matches("<tr class=moving>").count(), 2);
        assert_eq!(html.matches("<span class=yao role=img").count(), 24);
        assert!(html.find("上爻").unwrap() < html.find("初爻</td>").unwrap());
        let a = report.export_html(&root).unwrap();
        let b = report.export_html(&root).unwrap();
        assert_ne!(a, b);
        assert_eq!(std::fs::read_to_string(a).unwrap(), html);
        if let Ok(preview) = std::env::var("JIUHENG_TIME_PREVIEW") {
            std::fs::write(preview, &html).unwrap();
        }
        assert_eq!(storage.question_count().unwrap(), 1);
        assert_eq!(storage.load_astronomy_snapshot(id).unwrap(), snapshot);
        assert!(load(&storage, Uuid::new_v4()).is_err());
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn time_report_rejects_changed_cast_earth_plate_missing_fields_and_future_versions() {
    let mut storage = Storage::open_in_memory().unwrap();
    let id = fixture(&mut storage, 2026, 11, 12);
    let current = storage.load_astronomy_snapshot(id).unwrap().unwrap();
    let mut changed = current.clone();
    changed
        .meihua_time_casting_audit
        .as_mut()
        .unwrap()
        .beijing_time_candidate
        .as_mut()
        .unwrap()
        .cast
        .primary_bits ^= 1;
    assert!(verify(&changed, &current).is_err());
    changed = current.clone();
    changed
        .qimen_ju_audit
        .as_mut()
        .unwrap()
        .true_solar_midnight_candidate
        .earth_plate
        .as_mut()
        .unwrap()
        .cells_by_palace_number[0]
        .instrument_or_wonder = HeavenlyStem::Jia;
    assert!(verify(&changed, &current).is_err());
    changed = current.clone();
    changed.meihua_time_casting_audit = None;
    assert!(verify(&changed, &current).is_err());
    changed = current.clone();
    changed.qimen_ju_audit = None;
    assert!(verify(&changed, &current).is_err());
    changed = current.clone();
    changed.snapshot_version = 31;
    assert!(verify(&changed, &current).is_err());
}

#[test]
fn legacy_reports_keep_missing_fields_and_replay_supported_qimen_versions() {
    let mut storage = Storage::open_in_memory().unwrap();
    let id = fixture(&mut storage, 2026, 11, 12);
    let current = storage.load_astronomy_snapshot(id).unwrap().unwrap();
    for version in [13, 18, 19, 20] {
        let mut old = current.clone();
        old.snapshot_version = version;
        if version == 13 {
            old.meihua_time_casting_audit = None;
            old.qimen_ju_audit = None;
        } else {
            let audit = old.qimen_ju_audit.as_mut().unwrap();
            audit.audit_version = version - 17;
            if version < 20 {
                audit.candidates_identical = crate::qimen_old_day_candidates_identical(audit);
            }
            for day in [
                &mut audit.beijing_midnight_candidate,
                &mut audit.true_solar_midnight_candidate,
            ] {
                if version == 18 {
                    day.earth_plate = None;
                }
                if version < 20 {
                    day.hour_xun_candidate = None;
                } else {
                    day.hour_xun_candidate
                        .as_mut()
                        .unwrap()
                        .duty_identity_candidate = None;
                }
            }
        }
        assert!(verify(&old, &current).is_ok(), "version {version}");
        let mut report = load(&storage, id).unwrap();
        report.meihua.clear();
        report.qimen.clear();
        populate(&mut report, &old).unwrap();
        if version == 13 {
            assert!(report.meihua.is_empty());
            assert!(report.qimen.is_empty());
        } else {
            for board in &report.qimen {
                assert_eq!(board.cells.len(), if version == 18 { 0 } else { 9 });
                assert!(!board.context.iter().any(|s| s.starts_with("本旬值符：")));
            }
        }
    }
}

#[test]
fn leap_month_and_disputed_solar_term_show_reasons_without_inventing_boards() {
    let mut storage = Storage::open_in_memory().unwrap();
    let leap_id = fixture(&mut storage, 2025, 8, 1);
    let leap = load(&storage, leap_id).unwrap();
    assert!(leap.meihua.is_empty());
    assert!(leap.meihua_notice.contains("闰月"));
    assert!(!leap.qimen.is_empty());
    let dispute_id = fixture(&mut storage, 2026, 8, 30);
    let dispute = load(&storage, dispute_id).unwrap();
    assert_eq!(dispute.qimen.len(), 2);
    for board in &dispute.qimen {
        assert!(board.cells.is_empty());
        assert!(board.missing.contains("冲突"));
        assert!(!board.boundary_safe);
        assert!(!board.context.iter().any(|s| s.starts_with("本旬值符：")));
    }
    assert!(!dispute.html().contains("<div class=palace>"));
}
