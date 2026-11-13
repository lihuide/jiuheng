use super::*;

fn event(sequence: u8) -> EntropyEvent {
    EntropyEvent {
        sequence,
        wall_clock_unix_nanos: 1_788_068_725_000_000_000 + sequence as i64,
        monotonic_nanos: 10_000 + sequence as u64,
        press_duration_nanos: 100_000 + sequence as u64,
        delta_since_previous_nanos: 200_000,
        click_x_micropoints: 100,
        click_y_micropoints: 200,
        pointer_path: Vec::new(),
    }
}

#[test]
fn resuming_six_coins_preserves_a_previously_completed_yarrow_session() {
    let mut storage = Storage::open_in_memory().unwrap();
    let id = reading_report::tests::fixture(&mut storage, false);
    let mut session = YarrowSession::default();
    for _ in 0..18 {
        session.apply_split(session.active_stalks() / 2).unwrap();
    }
    let cast = session.finish().unwrap();
    storage
        .save_yarrow_artifact(id, session.transcript(), &cast)
        .unwrap();
    let mut app = JiuhengApp::with_storage(storage, PathBuf::from(":memory:"), None);
    app.resume_history_casting(id);
    assert!(app.yarrow_restore_error.is_none());
    assert_eq!(app.yarrow_session.transcript(), session.transcript());
    assert_eq!(
        app.yarrow_session.completed_lines(),
        session.completed_lines()
    );
    assert!(app.yarrow_session.is_complete());
    assert_eq!(app.yarrow_cast.as_ref(), Some(&cast));
    assert!(app.liuyao_lines.is_empty());
}

fn rendered_text(app: &mut JiuhengApp, page: Page) -> String {
    fn collect(shape: &egui::epaint::Shape, text: &mut String) {
        match shape {
            egui::epaint::Shape::Text(shape) => {
                text.push_str(shape.galley.text());
                text.push('\n');
            }
            egui::epaint::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, text);
                }
            }
            _ => {}
        }
    }
    let context = egui::Context::default();
    let mut text = String::new();
    for _ in 0..2 {
        let output = context.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1120.0, 760.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| match page {
                    Page::History => app.render_history(ui),
                    Page::Casting => app.render_casting(ui),
                    _ => unreachable!(),
                });
            },
        );
        text.clear();
        for shape in output.shapes {
            collect(&shape.shape, &mut text);
        }
    }
    text
}

#[test]
fn ordinary_ui_exposes_resume_and_hides_new_throw_while_save_is_pending() {
    let mut storage = Storage::open_in_memory().unwrap();
    let id = reading_report::tests::fixture(&mut storage, false);
    let initial = storage.begin_or_resume_liuyao(id).unwrap();
    storage.append_liuyao_event(&initial, &event(0)).unwrap();
    let mut app = JiuhengApp::with_storage(storage, PathBuf::from(":memory:"), None);
    app.select_history(id);
    let history = rendered_text(&mut app, Page::History);
    assert!(history.contains("六爻已保存 1/6 · 可继续"));
    assert!(history.contains("继续六爻起法"));
    app.resume_history_casting(id);
    assert!(rendered_text(&mut app, Page::Casting).contains("按住并松开 · 第 2 次"));
    app.liuyao_pending_event = Some(event(1));
    let pending = rendered_text(&mut app, Page::Casting);
    assert!(pending.contains("重试保存本次操作"));
    assert!(pending.contains("重新读取已保存进度"));
    assert!(!pending.contains("按住并松开"));
}

#[test]
fn desktop_restarts_and_finishes_original_question_without_changing_saved_lines() {
    let root = std::env::temp_dir().join(format!("jiuheng-desktop-resume-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("data.sqlite3");
    let (id, question_json, saved, snapshot) = {
        let mut storage = Storage::open(&path).unwrap();
        let id = reading_report::tests::fixture(&mut storage, false);
        let question_json =
            serde_json::to_string(&storage.load_question_detail(id).unwrap().unwrap().question)
                .unwrap();
        let snapshot =
            serde_json::to_string(&storage.load_astronomy_snapshot(id).unwrap()).unwrap();
        let mut app = JiuhengApp::with_storage(storage, path.clone(), None);
        app.resume_history_casting(id);
        for sequence in 0..3 {
            app.liuyao_pending_event = Some(event(sequence));
            app.save_pending_liuyao_event();
        }
        assert_eq!(app.page, Page::Casting);
        assert_eq!(app.liuyao_lines.len(), 3);
        let report = reading_report::load(&app.storage, id).unwrap();
        assert!(report.changes.contains("3/6"));
        assert!(report.transcript_hash.is_none());
        (id, question_json, app.liuyao_lines.clone(), snapshot)
    };
    {
        let mut app = JiuhengApp::with_storage(Storage::open(&path).unwrap(), path.clone(), None);
        assert!(app.current_question.is_none());
        app.resume_history_casting(id);
        assert_eq!(app.liuyao_lines, saved);
        assert_eq!(
            serde_json::to_string(app.current_question.as_ref().unwrap()).unwrap(),
            question_json
        );
        assert_eq!(
            serde_json::to_string(&app.storage.load_astronomy_snapshot(id).unwrap()).unwrap(),
            snapshot
        );
        assert!(app.liuyao_monotonic_base > event(2).monotonic_nanos);
        for sequence in 3..6 {
            app.liuyao_pending_event = Some(event(sequence));
            app.save_pending_liuyao_event();
        }
        assert!(app.liuyao_pending_event.is_none());
        assert!(app.liuyao_cast.is_some());
        assert_eq!(&app.liuyao_lines[..3], saved);
        app.open_reading(id);
        assert_eq!(
            app.reading
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .lines_top_down
                .len(),
            6
        );
        assert_eq!(app.storage.question_count().unwrap(), 1);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn stale_desktop_retains_failed_operation_until_saved_progress_is_reloaded() {
    let mut storage = Storage::open_in_memory().unwrap();
    let id = reading_report::tests::fixture(&mut storage, false);
    let mut app = JiuhengApp::with_storage(storage, PathBuf::from(":memory:"), None);
    app.resume_history_casting(id);
    let initial = app.liuyao_transcript.clone().unwrap();
    let external = app
        .storage
        .append_liuyao_event(&initial, &event(0))
        .unwrap();
    let mut competing = event(0);
    competing.click_x_micropoints += 1;
    app.liuyao_pending_event = Some(competing.clone());
    app.save_pending_liuyao_event();
    assert_eq!(app.liuyao_pending_event.as_ref(), Some(&competing));
    assert!(app.liuyao_lines.is_empty());
    assert!(app.casting_message.as_ref().unwrap().contains("暂停"));
    app.save_pending_liuyao_event();
    assert_eq!(app.liuyao_pending_event.as_ref(), Some(&competing));
    app.initialize_casting(id);
    assert!(app.liuyao_pending_event.is_none());
    assert_eq!(app.liuyao_lines, external.replay_lines().unwrap());
    app.liuyao_pending_event = Some(event(1));
    app.save_pending_liuyao_event();
    assert_eq!(app.liuyao_lines.len(), 2);
    assert!(app.casting_message.as_ref().unwrap().contains("临时内存"));
}
