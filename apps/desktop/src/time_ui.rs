use super::*;
use time_report::{MeihuaBoard, QimenBoard, TimeReport, QIMEN_LAYOUT};

impl JiuhengApp {
    pub(crate) fn open_time_report(&mut self, id: Uuid) {
        self.time_report_id = Some(id);
        self.time_report = Some(time_report::load(&self.storage, id));
        self.time_report_message = None;
        self.time_report_export_path = None;
        self.page = Page::TimeMechanical;
    }

    pub(crate) fn render_time_report(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "梅花 / 奇门机械盘", time_report::SCOPE, "盘");
        ui.add_space(12.0);
        let Some(result) = self.time_report.clone() else {
            ui.label("先在“新建占问”填写问题、占问时间和地点并锁定，随后可查看梅花和奇门机械盘。也可从永久记录打开原盘。");
            self.time_report_navigation(ui);
            return;
        };
        let report = match result {
            Ok(report) => report,
            Err(error) => {
                warning_box(
                    ui,
                    &format!("机械盘读取失败：{error}"),
                    Color32::from_rgb(167, 54, 54),
                );
                if ui.button("重新读取并核对时间盘").clicked() {
                    if let Some(id) = self.time_report_id {
                        self.open_time_report(id);
                    }
                }
                self.time_report_navigation(ui);
                return;
            }
        };
        ui.heading(&report.question);
        ui.small(format!("记录编号：{}", report.question_id));
        ui.horizontal_wrapped(|ui| {
            if ui.button("复制梅花与奇门全部候选").clicked() {
                ui.ctx().copy_text(report.text());
                self.time_report_message = Some("已复制两项机械盘的全部候选。".into());
            }
            if ui
                .add_enabled(
                    self.storage_path != std::path::Path::new(":memory:"),
                    egui::Button::new("导出梅花与奇门网页"),
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
                        self.time_report_message = Some(format!("已保存：{}", path.display()));
                        self.time_report_export_path = Some(path);
                    }
                    Err(error) => self.time_report_message = Some(format!("导出失败：{error}")),
                }
            }
            if ui.button("重新读取并核对时间盘").clicked() {
                self.open_time_report(report.question_id);
            }
            if ui.button("返回永久记录").clicked() {
                self.select_history(report.question_id);
                self.page = Page::History;
            }
        });
        if let Some(message) = &self.time_report_message {
            ui.label(message);
        }
        if let Some(path) = &self.time_report_export_path {
            if ui.button("打开时间盘所在文件夹").clicked() {
                if let Some(parent) = path.parent() {
                    if let Err(error) = std::process::Command::new("explorer.exe")
                        .arg(parent)
                        .spawn()
                    {
                        self.time_report_message = Some(format!("无法打开文件夹：{error}"));
                    }
                }
            }
        }
        if let Some(warning) = &self.startup_warning {
            warning_box(ui, warning, Color32::from_rgb(150, 92, 34));
        }
        ui.collapsing("原占问资料与时间口径", |ui| {
            for (name, value) in &report.metadata {
                ui.label(format!("{name}：{value}"));
            }
        });
        ui.small(&report.verification);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.time_report_tab, 0, "梅花 · 时间卦象");
            ui.selectable_value(&mut self.time_report_tab, 1, "奇门 · 九宫地盘");
        });
        ui.separator();
        render_time_boards(ui, &report, self.time_report_tab);
    }

    fn time_report_navigation(&mut self, ui: &mut egui::Ui) {
        if ui.button("填写占问资料").clicked() {
            self.page = Page::NewQuestion;
        }
        if ui.button("查看永久记录").clicked() {
            self.page = Page::History;
        }
    }
}

fn render_time_boards(ui: &mut egui::Ui, report: &TimeReport, tab: usize) {
    if tab == 0 {
        ui.label(&report.meihua_notice);
        for (index, board) in report.meihua.iter().enumerate() {
            ui.push_id(("meihua_board", index), |ui| render_meihua(ui, board));
        }
    } else {
        ui.label(&report.qimen_notice);
        for (index, board) in report.qimen.iter().enumerate() {
            ui.push_id(("qimen_board", index), |ui| render_qimen(ui, board));
        }
    }
}

fn render_meihua(ui: &mut egui::Ui, board: &MeihuaBoard) {
    ui.add_space(16.0);
    ui.heading(&board.title);
    ui.small(mechanical_report::boundary_text(board.boundary_safe));
    for value in &board.calculation {
        ui.label(value);
    }
    ui.label(format!("本卦 {} → 变卦 {}", board.primary, board.changed));
    egui::Grid::new("meihua_lines")
        .num_columns(4)
        .striped(true)
        .spacing([22.0, 10.0])
        .show(ui, |ui| {
            for title in ["爻位", "本卦", "变卦", "变化"] {
                ui.strong(title);
            }
            ui.end_row();
            for line in (1..=6).rev() {
                let moving = line == board.moving_line;
                ui.label(time_report::line_label(line));
                draw_line(ui, board.primary_bits, line, moving);
                draw_line(ui, board.changed_bits, line, moving);
                if moving {
                    ui.colored_label(Color32::from_rgb(152, 88, 57), "动爻 · 阴阳翻转");
                } else {
                    ui.label("静爻");
                }
                ui.end_row();
            }
        });
    ui.small(format!("派生中爻观察：{}", board.middle));
}

fn draw_line(ui: &mut egui::Ui, bits: u8, line: u8, moving: bool) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(94.0, 20.0), egui::Sense::hover());
    let color = if moving {
        Color32::from_rgb(152, 88, 57)
    } else {
        Color32::from_rgb(43, 74, 60)
    };
    let y = rect.center().y;
    let left = rect.left();
    let right = rect.right();
    let stroke = egui::Stroke::new(5.0_f32, color);
    if bits & (1 << (line - 1)) != 0 {
        ui.painter()
            .line_segment([egui::pos2(left, y), egui::pos2(right, y)], stroke);
    } else {
        ui.painter()
            .line_segment([egui::pos2(left, y), egui::pos2(left + 38.0, y)], stroke);
        ui.painter()
            .line_segment([egui::pos2(right - 38.0, y), egui::pos2(right, y)], stroke);
    }
}

fn render_qimen(ui: &mut egui::Ui, board: &QimenBoard) {
    ui.add_space(16.0);
    ui.heading(&board.title);
    for value in &board.context {
        ui.label(value);
    }
    ui.small(mechanical_report::boundary_text(board.boundary_safe));
    if !board.missing.is_empty() {
        ui.label(&board.missing);
    }
    if board.cells.is_empty() {
        return;
    }
    ui.small("九宫示意 · 上南下北；仅地盘，非完整时盘");
    egui::ScrollArea::horizontal().show(ui, |ui| {
        let width = ((ui.available_width() - 32.0) / 3.0).max(170.0);
        egui::Grid::new("qimen_grid")
            .num_columns(3)
            .spacing([6.0, 6.0])
            .show(ui, |ui| {
                for row in QIMEN_LAYOUT {
                    for number in row {
                        if let Some(cell) = board.cells.iter().find(|cell| cell.number == number) {
                            egui::Frame::new()
                                .fill(Color32::from_rgb(237, 242, 237))
                                .inner_margin(8.0)
                                .corner_radius(5.0)
                                .show(ui, |ui| {
                                    ui.set_width(width - 16.0);
                                    ui.set_min_height(150.0);
                                    ui.vertical(|ui| {
                                        ui.strong(&cell.title);
                                        ui.label(RichText::new(&cell.stem).size(25.0).strong());
                                        ui.small(&cell.hidden_jia);
                                        for marker in &cell.markers {
                                            ui.colored_label(
                                                Color32::from_rgb(152, 88, 57),
                                                marker,
                                            );
                                        }
                                    });
                                });
                        }
                    }
                    ui.end_row();
                }
            });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_history_time_page_shows_both_methods_without_a_coin_cast() {
        let mut storage = Storage::open_in_memory().unwrap();
        let id = time_report::tests::fixture(&mut storage, 2026, 11, 12);
        let mut app = JiuhengApp::with_storage(storage, PathBuf::from(":memory:"), None);
        app.open_time_report(id);
        assert_eq!(app.page, Page::TimeMechanical);
        assert!(app.current_question.is_none());
        assert!(!app.developer_mode);
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
        for tab in [0, 1] {
            app.time_report_tab = tab;
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
                        egui::CentralPanel::default().show(ctx, |ui| app.render_time_report(ui));
                    },
                );
                text.clear();
                for shape in output.shapes {
                    collect(&shape.shape, &mut text);
                }
            }
            for required in if tab == 0 {
                vec![
                    "梅花 · 北京时间",
                    "梅花 · 真太阳时",
                    "动爻",
                    "初爻",
                    "上爻",
                    "派生中爻观察",
                ]
            } else {
                vec![
                    "奇门 · 北京时间零点日界",
                    "奇门 · 真太阳时零点日界",
                    "中五宫",
                    "坎1宫",
                    "旬首地盘宫",
                    "本旬值符",
                    "本旬值使",
                ]
            } {
                assert!(text.contains(required), "missing {required}: {text}");
            }
            assert!(text.contains("复制梅花与奇门全部候选"));
        }
        assert_eq!(app.storage.question_count().unwrap(), 1);
        assert!(app.storage.load_liuyao_artifact(id).unwrap().is_none());
    }
}
