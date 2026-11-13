use super::*;
use divination_core::{TaiyiDun, TaiyiYearEpochProfile};
use divination_storage::TaiyiRequest;
use taiyi_report::{Cell, TaiyiReport};

#[derive(Default)]
pub(crate) struct TaiyiState {
    pub question_id: Option<Uuid>,
    pub question: String,
    pub reports: Option<Result<Vec<TaiyiReport>, String>>,
    pub selected: Option<Uuid>,
    pub mode: usize,
    pub value: String,
    pub dun: Option<TaiyiDun>,
    pub epoch_acknowledged: bool,
    pub message: Option<String>,
    pub export_path: Option<PathBuf>,
}

impl TaiyiState {
    pub fn request(&self) -> Result<TaiyiRequest, String> {
        let value = self.value.trim();
        let request = match self.mode {
            0 => TaiyiRequest::AccumulatedYear {
                count: value.parse().map_err(|_| "请填写大于零的积年整数。")?,
            },
            1 if self.epoch_acknowledged => TaiyiRequest::TraditionalYear {
                year: value
                    .parse()
                    .map_err(|_| "请填写 -999 至 3000 的传统年标签整数。")?,
                profile: TaiyiYearEpochProfile::ReceivedKaiyuan12Year724Count1937281,
            },
            1 => return Err("请明确选择页面列出的传本积年假设。".into()),
            2 => TaiyiRequest::ExplicitBoard {
                count: value.parse().map_err(|_| "请填写大于零的局序整数。")?,
                dun: self.dun.ok_or("请明确选择阴遁或阳遁。")?,
            },
            _ => return Err("请选择排盘方式。".into()),
        };
        request.calculate().map_err(|e| e.to_string())?;
        Ok(request)
    }
}

impl JiuhengApp {
    pub(crate) fn open_taiyi(&mut self, id: Uuid) {
        let question = self
            .storage
            .load_question_detail(id)
            .ok()
            .flatten()
            .map(|d| d.question.text)
            .unwrap_or_default();
        let reports = taiyi_report::load(&self.storage, id);
        let selected = reports
            .as_ref()
            .ok()
            .and_then(|r| r.last())
            .map(|r| r.record.id);
        self.taiyi = TaiyiState {
            question_id: Some(id),
            question,
            reports: Some(reports),
            selected,
            ..Default::default()
        };
        self.page = Page::Taiyi;
    }

    pub(crate) fn save_taiyi_form(&mut self) {
        self.taiyi.export_path = None;
        let result = (|| {
            let id = self.taiyi.question_id.ok_or("请先锁定占问记录。")?;
            let request = self.taiyi.request()?;
            let record = self
                .storage
                .save_taiyi_candidate(id, request)
                .map_err(|e| e.to_string())?;
            self.taiyi.reports = Some(taiyi_report::load(&self.storage, id));
            if let Some(Err(error)) = &self.taiyi.reports {
                return Err(format!("候选已保存，但重新读取校验失败：{error}"));
            }
            self.taiyi.selected = Some(record.id);
            Ok::<(), String>(())
        })();
        self.taiyi.message = Some(match result {
            Ok(()) if self.storage_path == std::path::Path::new(":memory:") => {
                "候选仅保存在本次临时会话，关闭后丢失。".into()
            }
            Ok(()) => "已保存并按原参数复算；同一问题的相同参数只保留一份。".into(),
            Err(error) => format!("操作未完成：{error}"),
        });
    }

    pub(crate) fn render_taiyi(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "太乙机械盘", taiyi_report::SCOPE, "盘");
        ui.add_space(12.0);
        let Some(question_id) = self.taiyi.question_id else {
            ui.label("先新建并锁定占问，或在永久记录选择一条问题，打开太乙机械盘。输入已知积年、传统年标签或明确阴阳遁的局序后生成候选。");
            if ui.button("填写占问资料").clicked() {
                self.page = Page::NewQuestion;
            }
            if ui.button("查看永久记录").clicked() {
                self.page = Page::History;
            }
            return;
        };
        ui.heading(&self.taiyi.question);
        ui.small(format!("所属占问：{question_id}"));
        if let Some(warning) = &self.startup_warning {
            warning_box(ui, warning, Color32::from_rgb(150, 92, 34));
        }
        ui.label(taiyi_report::LIMITS);
        ui.group(|ui| {
            ui.strong("新建参数候选");
            let old_mode = self.taiyi.mode;
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.taiyi.mode, 0, "已知积年");
                ui.selectable_value(&mut self.taiyi.mode, 1, "传统年标签");
                ui.selectable_value(&mut self.taiyi.mode, 2, "阴阳遁局序");
            });
            if old_mode != self.taiyi.mode {
                self.taiyi.value.clear();
                self.taiyi.dun = None;
                self.taiyi.epoch_acknowledged = false;
            }
            match self.taiyi.mode {
                1 => {
                    ui.label("范围 -999 至 3000；0 表示公元前 1 年。请输入已明确的传统年标签，不直接等同占问公历年份。");
                    ui.checkbox(&mut self.taiyi.epoch_acknowledged, "明确采用传本假设：724 年积年 1,937,281，每年加减一；原页仍待复核");
                }
                2 => {
                    ui.horizontal(|ui| {
                        ui.label("明确选择：");
                        ui.selectable_value(&mut self.taiyi.dun, Some(TaiyiDun::Yang), "阳遁");
                        ui.selectable_value(&mut self.taiyi.dun, Some(TaiyiDun::Yin), "阴遁");
                    });
                    ui.small("输入从 1 开始的局序；超过 72 按周期归位，72 的整数倍保留为第 72 局。");
                }
                _ => { ui.label("输入已知的正整数积年。年计不自动选择阴阳遁。 "); }
            }
            ui.horizontal(|ui| {
                ui.label(match self.taiyi.mode { 1 => "传统年标签", 2 => "局序", _ => "积年" });
                ui.add(egui::TextEdit::singleline(&mut self.taiyi.value).desired_width(240.0).hint_text("请输入整数"));
            });
            let request = self.taiyi.request();
            let valid = request.is_ok() && matches!(&self.taiyi.reports, Some(Ok(_)));
            let caption = if self.storage_path == std::path::Path::new(":memory:") { "计算候选（临时会话）" } else { "计算并保存候选" };
            if ui.add_enabled(valid, egui::Button::new(caption)).clicked() { self.save_taiyi_form(); }
            if let Err(message) = request { ui.small(message); }
            ui.small("修改参数会新增一份候选，不替换已保存盘；不改变原占问或六爻随机起法。");
        });
        if let Some(message) = &self.taiyi.message {
            ui.label(message);
        }
        ui.horizontal_wrapped(|ui| {
            if ui.button("重新读取并核对太乙记录").clicked() {
                self.open_taiyi(question_id);
            }
            if ui.button("返回永久记录").clicked() {
                self.select_history(question_id);
                self.page = Page::History;
            }
        });
        let reports = match self.taiyi.reports.clone() {
            Some(Ok(reports)) => reports,
            Some(Err(error)) => {
                warning_box(ui, &error, Color32::from_rgb(167, 54, 54));
                return;
            }
            None => return,
        };
        if reports.is_empty() {
            ui.label("本问题尚无已保存的太乙候选。");
            return;
        }
        ui.add_space(12.0);
        egui::ComboBox::from_id_salt("taiyi_saved_candidates")
            .width(480.0)
            .selected_text(
                reports
                    .iter()
                    .find(|r| Some(r.record.id) == self.taiyi.selected)
                    .map(|r| r.record.request.label())
                    .unwrap_or_else(|| "选择已保存候选".into()),
            )
            .show_ui(ui, |ui| {
                for report in &reports {
                    if ui
                        .selectable_value(
                            &mut self.taiyi.selected,
                            Some(report.record.id),
                            report.record.request.label(),
                        )
                        .changed()
                    {
                        self.taiyi.export_path = None;
                        self.taiyi.message = None;
                    }
                }
            });
        let Some(report) = reports
            .iter()
            .find(|r| Some(r.record.id) == self.taiyi.selected)
        else {
            return;
        };
        ui.horizontal_wrapped(|ui| {
            if ui.button("复制当前太乙候选").clicked() {
                ui.ctx().copy_text(report.text());
                self.taiyi.message = Some("已复制当前候选及其输入口径。".into());
            }
            if ui
                .add_enabled(
                    self.storage_path != std::path::Path::new(":memory:"),
                    egui::Button::new("导出太乙网页"),
                )
                .clicked()
            {
                let directory = self
                    .storage_path
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("exports");
                match report.export_html(&directory) {
                    Ok(path) => {
                        self.taiyi.message = Some(format!("已导出：{}", path.display()));
                        self.taiyi.export_path = Some(path);
                    }
                    Err(error) => self.taiyi.message = Some(format!("导出失败：{error}")),
                }
            }
            if let Some(path) = &self.taiyi.export_path {
                if ui.button("打开太乙报告文件夹").clicked() {
                    if let Some(parent) = path.parent() {
                        if let Err(error) = std::process::Command::new("explorer.exe")
                            .arg(parent)
                            .spawn()
                        {
                            self.taiyi.message = Some(format!("无法打开文件夹：{error}"));
                        }
                    }
                }
            }
        });
        ui.small(format!(
            "已核对保存内容并按原参数复算 · 候选 {} · 保存于 {}",
            report.record.id, report.record.created_at
        ));
        for value in &report.context {
            ui.label(value);
        }
        ui.add_space(12.0);
        ui.heading("太乙与主客将位");
        ui.small(taiyi_report::ORIENTATION);
        render_grid(ui, "taiyi_generals", &report.cells);
        ui.collapsing("十六神定位 · 查看天目与始击", |ui| {
            ui.label("十六位独立排列，未挤入九宫。");
            egui::Grid::new("taiyi_spirits")
                .num_columns(3)
                .striped(true)
                .show(ui, |ui| {
                    for (position, spirit, marker) in &report.spirits {
                        ui.label(position);
                        ui.label(spirit);
                        ui.strong(marker);
                        ui.end_row();
                    }
                });
        });
        ui.label(format!("位置标记（不作吉凶解释）：{}", report.marks));
        for board in &report.gates {
            ui.add_space(12.0);
            ui.heading(format!("年计八门 · {}", board.title));
            ui.small("三张独立门盘；不表示时计值事门旋转。");
            if board.missing.is_empty() {
                render_grid(ui, &board.title, &board.cells);
            } else {
                ui.label(&board.missing);
            }
        }
    }
}

fn render_grid(ui: &mut egui::Ui, id: &str, cells: &[Cell]) {
    egui::Grid::new(id)
        .num_columns(3)
        .spacing([8.0, 8.0])
        .show(ui, |ui| {
            for row in taiyi_report::LAYOUT {
                for number in row {
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(237, 242, 237))
                        .show(ui, |ui| {
                            ui.set_min_size(egui::vec2(175.0, 70.0));
                            ui.set_max_width(205.0);
                            ui.strong(taiyi_report::palace_name(number));
                            let items = cells
                                .iter()
                                .find(|c| c.number == number)
                                .map(|c| c.items.join("、"))
                                .filter(|s| !s.is_empty())
                                .unwrap_or_else(|| "—".into());
                            ui.label(
                                RichText::new(items)
                                    .size(18.0)
                                    .color(Color32::from_rgb(146, 80, 43)),
                            );
                        });
                }
                ui.end_row();
            }
        });
}
