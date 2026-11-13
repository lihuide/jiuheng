use super::*;
use mechanical_report::{boundary_text, MechanicalReport, ZiweiBoard, PALACE_LAYOUT};

impl JiuhengApp {
    pub(crate) fn open_mechanical(&mut self, id: Uuid) {
        self.mechanical_id = Some(id);
        self.mechanical_report = Some(mechanical_report::load(&self.storage, id));
        self.mechanical_message = None;
        self.mechanical_export_path = None;
        self.page = Page::Mechanical;
    }

    pub(crate) fn render_mechanical(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "四柱 / 紫微机械盘", mechanical_report::SCOPE, "盘");
        ui.add_space(12.0);
        if self.mechanical_report.is_none() {
            ui.label("新建占问时选择“长期命运”，填写出生资料并锁定；也可从永久记录打开旧机械盘。");
            self.mechanical_navigation(ui);
            return;
        }
        let report = match self.mechanical_report.clone().expect("checked") {
            Ok(report) => report,
            Err(error) => {
                warning_box(
                    ui,
                    &format!("机械盘读取失败：{error}"),
                    Color32::from_rgb(167, 54, 54),
                );
                if ui.button("重新读取并核对机械盘").clicked() {
                    if let Some(id) = self.mechanical_id {
                        self.open_mechanical(id);
                    }
                }
                self.mechanical_navigation(ui);
                return;
            }
        };
        ui.heading(&report.question);
        ui.small(format!("记录编号：{}", report.question_id));
        ui.horizontal_wrapped(|ui| {
            if ui.button("复制全部机械盘").clicked() {
                ui.ctx().copy_text(report.text());
                self.mechanical_message = Some("已复制四柱与紫微全部候选。".into());
            }
            if ui
                .add_enabled(
                    self.storage_path != std::path::Path::new(":memory:"),
                    egui::Button::new("导出机械盘网页"),
                )
                .clicked()
            {
                let folder = self
                    .storage_path
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("exports");
                match report.export_html(&folder) {
                    Ok(path) => {
                        self.mechanical_message = Some(format!("已保存：{}", path.display()));
                        self.mechanical_export_path = Some(path);
                    }
                    Err(error) => self.mechanical_message = Some(format!("导出失败：{error}")),
                }
            }
            if ui.button("重新读取并核对机械盘").clicked() {
                self.open_mechanical(report.question_id);
            }
            if ui.button("返回永久记录").clicked() {
                self.select_history(report.question_id);
                self.page = Page::History;
            }
        });
        if let Some(message) = &self.mechanical_message {
            ui.label(message);
        }
        if let Some(path) = &self.mechanical_export_path {
            if ui.button("打开机械盘所在文件夹").clicked() {
                if let Some(parent) = path.parent() {
                    if let Err(error) = std::process::Command::new("explorer.exe")
                        .arg(parent)
                        .spawn()
                    {
                        self.mechanical_message = Some(format!("无法打开文件夹：{error}"));
                    }
                }
            }
        }
        if let Some(warning) = &self.startup_warning {
            warning_box(ui, warning, Color32::from_rgb(150, 92, 34));
        }
        ui.collapsing("出生资料与时间口径", |ui| {
            for (name, value) in &report.metadata {
                ui.label(format!("{name}：{value}"));
            }
        });
        ui.small(&report.verification);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.mechanical_tab, 0, "四柱 · 藏干与十神");
            ui.selectable_value(&mut self.mechanical_tab, 1, "紫微 · 十二宫");
        });
        ui.separator();
        if self.mechanical_tab == 0 {
            render_pillars(ui, &report);
        } else {
            ui.label(&report.ziwei_notice);
            for (index, board) in report.ziwei.iter().enumerate() {
                ui.push_id(("ziwei_board", index), |ui| render_ziwei(ui, board));
            }
        }
        if report.pillars.is_empty() && report.ziwei.is_empty() {
            self.mechanical_navigation(ui);
        }
    }

    fn mechanical_navigation(&mut self, ui: &mut egui::Ui) {
        if ui.button("填写出生资料").clicked() {
            self.category = QuestionCategory::LongTermFate;
            self.include_birth_parameters = true;
            self.page = Page::NewQuestion;
        }
        if ui.button("查看永久记录").clicked() {
            self.page = Page::History;
        }
    }
}

fn render_pillars(ui: &mut egui::Ui, report: &MechanicalReport) {
    ui.label(&report.pillar_notice);
    for (index, table) in report.pillars.iter().enumerate() {
        ui.add_space(14.0);
        ui.heading(&table.title);
        ui.small(boundary_text(table.boundary_safe));
        egui::ScrollArea::horizontal()
            .id_salt(("pillar_scroll", index))
            .show(ui, |ui| {
                egui::Grid::new(("pillar_table", index))
                    .num_columns(5)
                    .striped(true)
                    .spacing([18.0, 12.0])
                    .show(ui, |ui| {
                        for (row_index, row) in table.rows.iter().enumerate() {
                            for value in row {
                                if row_index <= 1 {
                                    ui.strong(value);
                                } else {
                                    ui.label(value);
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
    }
}

fn render_ziwei(ui: &mut egui::Ui, board: &ZiweiBoard) {
    ui.add_space(18.0);
    ui.heading(&board.title);
    ui.label(&board.subtitle);
    ui.small(boundary_text(board.boundary_safe));
    if !board.missing.is_empty() {
        ui.label(&board.missing);
    }
    egui::ScrollArea::horizontal().show(ui, |ui| {
        let width = ((ui.available_width() - 48.0) / 4.0).max(144.0);
        egui::Grid::new("palace_grid")
            .num_columns(4)
            .spacing([6.0, 6.0])
            .show(ui, |ui| {
                for row in PALACE_LAYOUT {
                    for branch in row {
                        ui.allocate_ui_with_layout(
                            egui::vec2(width, 130.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_max_width(width);
                                if let Some(cell) = branch.and_then(|branch| {
                                    board.cells.iter().find(|cell| cell.branch_index == branch)
                                }) {
                                    egui::Frame::new()
                                        .fill(Color32::from_rgb(237, 242, 237))
                                        .inner_margin(8.0)
                                        .corner_radius(5.0)
                                        .show(ui, |ui| {
                                            ui.set_min_size(egui::vec2(width - 16.0, 114.0));
                                            ui.strong(&cell.title);
                                            if cell.body {
                                                ui.colored_label(
                                                    Color32::from_rgb(152, 88, 57),
                                                    "身宫",
                                                );
                                            }
                                            ui.label(cell.major.join(" · "));
                                            ui.small(cell.auxiliary.join(" · "));
                                        });
                                } else {
                                    ui.centered_and_justified(|ui| {
                                        ui.small("机械结构\n无占断结论");
                                    });
                                }
                            },
                        );
                    }
                    ui.end_row();
                }
            });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(app: &mut JiuhengApp) -> String {
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
                        egui::vec2(900.0, 4000.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.render_mechanical(ui));
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
    fn ordinary_history_mechanical_page_renders_four_pillars_and_all_four_ziwei_candidates() {
        let mut storage = Storage::open_in_memory().unwrap();
        let id = mechanical_report::tests::fixture(&mut storage, false);
        let mut app = JiuhengApp::with_storage(storage, PathBuf::from(":memory:"), None);
        app.open_mechanical(id);
        assert!(!app.developer_mode);
        assert!(app.current_question.is_none());
        let pillars = text(&mut app);
        for required in [
            "年柱",
            "月柱",
            "日柱",
            "时柱",
            "藏干 / 十神",
            "庚午",
            "复制全部机械盘",
            "导出机械盘网页",
            "旧记录",
        ] {
            if required != "旧记录" {
                assert!(pillars.contains(required), "missing {required}");
            }
        }
        assert!(!pillars.contains("旧记录未保存"));
        app.mechanical_tab = 1;
        let ziwei = text(&mut app);
        for required in [
            "农历年口径 / 北京时间",
            "农历年口径 / 真太阳时",
            "立春年口径 / 北京时间",
            "立春年口径 / 真太阳时",
            "紫微",
            "天府",
            "文昌",
            "左辅",
            "身宫",
        ] {
            assert!(ziwei.contains(required), "missing {required}");
        }
        assert_eq!(app.storage.question_count().unwrap(), 1);
    }

    #[test]
    fn locking_birth_parameters_opens_mechanical_chart_without_six_coin_steps() {
        let mut storage = Storage::open_in_memory().unwrap();
        let id = mechanical_report::tests::fixture(&mut storage, false);
        let mut question = storage.load_question_detail(id).unwrap().unwrap().question;
        question.id = Uuid::new_v4();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        let new_id = question.id;
        let mut app = JiuhengApp::with_storage(storage, PathBuf::from(":memory:"), None);
        app.commit_new_question(question, classification, fingerprint, None);
        assert_eq!(app.page, Page::Mechanical);
        assert_eq!(app.mechanical_id, Some(new_id));
        assert!(app.mechanical_report.as_ref().unwrap().is_ok());
        assert!(app.storage.load_liuyao_artifact(new_id).unwrap().is_none());
    }
}
