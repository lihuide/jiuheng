use super::*;
use crate::{taiyi_ui::TaiyiState, JiuhengApp, Page};
use divination_storage::TaiyiRequest;

#[test]
fn input_form_requires_explicit_epoch_and_dun_and_rejects_bad_values() {
    let mut form = TaiyiState::default();
    assert!(form.request().is_err());
    for value in ["0", "-1", "1.5", "2026-11-13", "18446744073709551616"] {
        form.value = value.into();
        assert!(form.request().is_err());
    }
    form.mode = 1;
    form.value = "2026".into();
    assert!(form.request().is_err());
    form.epoch_acknowledged = true;
    assert!(form.request().is_ok());
    form.value = "3001".into();
    assert!(form.request().is_err());
    form.mode = 2;
    form.value = " 72 ".into();
    assert!(form.request().is_err());
    form.dun = Some(TaiyiDun::Yin);
    assert_eq!(
        form.request().unwrap(),
        TaiyiRequest::ExplicitBoard {
            count: 72,
            dun: TaiyiDun::Yin
        }
    );
}

#[test]
fn saved_reports_reopen_export_safely_and_keep_taiyi_orientation_and_blocked_gate() {
    let root = std::env::temp_dir().join(format!("jiuheng-taiyi-report-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("data.sqlite3");
    let (id, original) = {
        let mut storage = Storage::open(&path).unwrap();
        let id = crate::time_report::tests::fixture(&mut storage, 2026, 11, 13);
        let blocked_count = (1..=72)
            .find(|count| {
                taiyi_annual_frame_from_accumulated_count(*count)
                    .unwrap()
                    .main_great_general_palace
                    .number()
                    == 5
            })
            .unwrap();
        storage
            .save_taiyi_candidate(
                id,
                TaiyiRequest::AccumulatedYear {
                    count: blocked_count,
                },
            )
            .unwrap();
        storage
            .save_taiyi_candidate(
                id,
                TaiyiRequest::ExplicitBoard {
                    count: 3,
                    dun: TaiyiDun::Yin,
                },
            )
            .unwrap();
        storage
            .save_taiyi_candidate(
                id,
                TaiyiRequest::TraditionalYear {
                    year: 2026,
                    profile: TaiyiYearEpochProfile::ReceivedKaiyuan12Year724Count1937281,
                },
            )
            .unwrap();
        let original = load(&storage, id)
            .unwrap()
            .iter()
            .map(|r| r.text())
            .collect::<Vec<_>>();
        (id, original)
    };
    {
        let storage = Storage::open(&path).unwrap();
        let reports = load(&storage, id).unwrap();
        assert_eq!(
            reports.iter().map(|r| r.text()).collect::<Vec<_>>(),
            original
        );
        let report = &reports[0];
        assert!(report.cells[4].items.iter().any(|s| s == "主大将"));
        assert_eq!(report.gates.len(), 3);
        assert!(report.gates[1].cells.is_empty());
        assert!(report.gates[1].missing.contains("中五"));
        assert!(!report.gates[0].cells.is_empty());
        assert_eq!(report.spirits.len(), 16);
        assert_eq!(
            report
                .spirits
                .iter()
                .filter(|(_, _, m)| m.contains("天目"))
                .count(),
            1
        );
        let html = report.html();
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("default-src 'none'"));
        let labels = [
            "九宫 · 巽 / 东南",
            "二宫 · 离 / 南",
            "七宫 · 坤 / 西南",
            "四宫 · 震 / 东",
            "五宫 · 中",
            "六宫 · 兑 / 西",
            "三宫 · 艮 / 东北",
            "八宫 · 坎 / 北",
            "一宫 · 乾 / 西北",
        ];
        for pair in labels.windows(2) {
            assert!(html.find(pair[0]).unwrap() < html.find(pair[1]).unwrap());
        }
        assert!(reports[1].gates.is_empty());
        assert!(reports[1].text().contains(DISPUTES));
        assert!(reports[2].text().contains("1938583"));
        let first = report.export_html(&root).unwrap();
        let second = report.export_html(&root).unwrap();
        assert_ne!(first, second);
        assert_eq!(std::fs::read_to_string(first).unwrap(), html);
        if let Some(directory) = std::env::var_os("JIUHENG_TAIYI_PREVIEW_DIR") {
            let directory = PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(directory.join("index.html"), reports[2].html()).unwrap();
            std::fs::write(directory.join("blocked.html"), html).unwrap();
            std::fs::write(directory.join("yin.html"), reports[1].html()).unwrap();
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn ordinary_ui_saves_selected_history_question_without_changing_current_question() {
    let mut storage = Storage::open_in_memory().unwrap();
    let id = crate::time_report::tests::fixture(&mut storage, 2026, 11, 13);
    let current = crate::time_report::tests::fixture(&mut storage, 2026, 11, 12);
    let mut app = JiuhengApp::with_storage(storage, PathBuf::from(":memory:"), None);
    app.current_question = app
        .storage
        .load_question_detail(current)
        .unwrap()
        .map(|d| d.question);
    app.open_taiyi(id);
    assert_eq!(app.page, Page::Taiyi);
    assert!(app.taiyi.value.is_empty() && app.taiyi.dun.is_none() && !app.taiyi.epoch_acknowledged);
    app.taiyi.value = "1938583".into();
    app.save_taiyi_form();
    app.save_taiyi_form();
    assert_eq!(app.storage.load_taiyi_candidates(id).unwrap().len(), 1);
    assert!(app
        .storage
        .load_taiyi_candidates(current)
        .unwrap()
        .is_empty());
    assert_eq!(app.current_question.as_ref().unwrap().id, current);
    assert!(app.taiyi.message.as_ref().unwrap().contains("临时会话"));
    fn collect(shape: &eframe::egui::epaint::Shape, out: &mut String) {
        match shape {
            eframe::egui::epaint::Shape::Text(text) => {
                out.push_str(text.galley.text());
                out.push('\n');
            }
            eframe::egui::epaint::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, out);
                }
            }
            _ => {}
        }
    }
    let context = eframe::egui::Context::default();
    let mut text = String::new();
    for _ in 0..2 {
        let output = context.run(
            eframe::egui::RawInput {
                screen_rect: Some(eframe::egui::Rect::from_min_size(
                    eframe::egui::Pos2::ZERO,
                    eframe::egui::vec2(1120.0, 2600.0),
                )),
                ..Default::default()
            },
            |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| app.render_taiyi(ui));
            },
        );
        text.clear();
        for shape in output.shapes {
            collect(&shape.shape, &mut text);
        }
    }
    for expected in [
        "太乙机械盘",
        "复制当前太乙候选",
        "导出太乙网页",
        "太乙与主客将位",
        "年计八门",
        "主大将",
        "重新读取并核对太乙记录",
    ] {
        assert!(text.contains(expected), "missing {expected}");
    }
    app.open_taiyi(id);
    assert_eq!(
        app.taiyi.reports.as_ref().unwrap().as_ref().unwrap().len(),
        1
    );
}
