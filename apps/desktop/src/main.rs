#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use chrono::{DateTime, Datelike, FixedOffset, NaiveDateTime, TimeZone, Timelike, Utc};
use directories::ProjectDirs;
use divination_core::{
    assess_first_rain_scope, build_event_fingerprint, chinese_astrology_structure_audit,
    civil_calendar_audit, classified_question_requires_health_notice, estimate_delta_t,
    hexagram_transformation, historical_datetime_label_zh, jie_month_candidate,
    julian_day_tt_from_ut1, lichun_year_candidate, liuyao_hidden_spirits_audit, liuyao_sanhe_audit,
    liuyao_six_relatives_audit, liuyao_six_spirits_audit, lunation_interval,
    lunisolar_date_candidate_from_structure, lunisolar_month_structure_for_civil_date,
    meihua_time_casting_audit, practical_action_notice_for_classified_question,
    qimen_ju_duty_identity_audit, qimen_midnight_boundary_safe, sexagenary_day_for_fixed_date,
    sexagenary_hour_from_day_stem, sexagenary_year_for_astronomical_year_label,
    shichen_from_true_solar_time, solar_term_class_interval, solar_term_interval,
    solve_solar_term_near_bjt_date, true_solar_time_foundation,
    true_solar_time_from_equation_of_time, validate_question_parameters, AncientQuestionType,
    Applicability, ApplicabilityStatus, BiologicalSex, BirthParameters, CalendarSystem,
    ClassificationError, CoordinatePrecision, EngineRegistry, EntropyEvent, EphemerisProvider,
    EventFingerprint, EventMatchReport, EventRevision, GeoLocation, HistoricalDateTime,
    KeywordClassifier, LineCast, LiuyaoBranchPairRelationsAudit, LiuyaoCalendarMovingAudit,
    LiuyaoCalendarRelationsAudit, LiuyaoCast, LiuyaoFanyinFuyinAudit, LiuyaoHiddenSpiritsAudit,
    LiuyaoNaJiaAudit, LiuyaoProgressRetreatAudit, LiuyaoProgressRetreatIdentity, LiuyaoSanheAudit,
    LiuyaoSixRelativesAudit, LiuyaoSixSpiritsAudit, LiuyaoTranscript, MansionQuadrant,
    PointerSample, QizhengSiyuIdentityAudit, QuestionCategory, QuestionInput, SolarTerm,
    SolarTermClass, SpaSolarEphemeris, YarrowCast, YarrowSession,
    TAIYI_ANNUAL_FRAME_ALGORITHM_VERSION, TAIYI_ANNUAL_FRAME_STATUS,
    TAIYI_ANNUAL_GATE_BOARDS_ALGORITHM_VERSION, TAIYI_ANNUAL_GATE_BOARDS_STATUS,
    TAIYI_BARRIER_RAW_OBSERVATIONS_ALGORITHM_VERSION, TAIYI_BARRIER_RAW_OBSERVATIONS_STATUS,
    TAIYI_MECHANICAL_RELATIONS_ALGORITHM_VERSION, TAIYI_MECHANICAL_RELATIONS_STATUS,
    TAIYI_SEVENTY_TWO_BOARD_ALGORITHM_VERSION, TAIYI_SEVENTY_TWO_BOARD_STATUS,
    YARROW_PROCEDURE_STATUS_ZH,
};
use divination_storage::{
    birth_time_audits_match, calculate_birth_time_audit_snapshot, BirthTimeAuditSnapshot,
    DuplicateEvent, HistoryReplayAudit, QuestionHistoryDetail, QuestionHistorySummary,
    RuleEvidenceDossier, Storage, BIRTH_TIME_AUDIT_STATUS,
};
use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, RichText};
use serde::Deserialize;
use std::time::Instant;
use std::{collections::BTreeSet, path::PathBuf};
use uuid::Uuid;

const APP_NAME: &str = "九衡";
const BJT_SECONDS: i32 = 8 * 60 * 60;
const PROGRESS_CHECKLIST_JSON: &str = include_str!("../../../data/progress.checklist.json");

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("九衡 · 古籍规则推演")
            .with_inner_size([1120.0, 760.0])
            .with_min_inner_size([900.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        "九衡",
        native_options,
        Box::new(|creation_context| Ok(Box::new(JiuhengApp::new(creation_context)))),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    NewQuestion,
    History,
    Casting,
    Result,
    Developer,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CastingTab {
    Liuyao,
    Yarrow,
    Meihua,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YarrowUiPhase {
    Divide,
    HangOne,
    CountLeft,
    CountRight,
    Gather,
}

#[derive(Debug, Clone, Deserialize)]
struct ProjectAcceptanceChecklist {
    schema_version: u16,
    product_version: String,
    last_verified: String,
    measurement_rule: String,
    items: Vec<ProjectAcceptanceItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct ProjectAcceptanceItem {
    id: String,
    title: String,
    status: ProjectAcceptanceStatus,
    evidence: String,
    remaining: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProjectAcceptanceStatus {
    Passed,
    InProgress,
    Blocked,
}

impl ProjectAcceptanceStatus {
    const fn zh_label(self) -> &'static str {
        match self {
            Self::Passed => "已通过",
            Self::InProgress => "进行中",
            Self::Blocked => "未通过/受阻",
        }
    }

    const fn color(self) -> Color32 {
        match self {
            Self::Passed => Color32::from_rgb(45, 112, 82),
            Self::InProgress => Color32::from_rgb(150, 92, 34),
            Self::Blocked => Color32::from_rgb(167, 54, 54),
        }
    }
}

fn load_project_acceptance_checklist() -> Result<ProjectAcceptanceChecklist, String> {
    let checklist: ProjectAcceptanceChecklist = serde_json::from_str(PROGRESS_CHECKLIST_JSON)
        .map_err(|error| format!("项目验收清单无法解析：{error}"))?;
    if checklist.schema_version != 1 {
        return Err(format!(
            "项目验收清单结构版本不受支持：{}",
            checklist.schema_version
        ));
    }
    if checklist.product_version != env!("CARGO_PKG_VERSION") {
        return Err(format!(
            "项目验收清单版本 {} 与程序版本 {} 不一致",
            checklist.product_version,
            env!("CARGO_PKG_VERSION")
        ));
    }
    if checklist.items.len() != 16 {
        return Err(format!(
            "项目验收清单必须逐项覆盖16项一级需求，当前为{}项",
            checklist.items.len()
        ));
    }
    if checklist.measurement_rule.trim().is_empty() || checklist.last_verified.trim().is_empty() {
        return Err("项目验收清单缺少测量规则或核验日期".to_owned());
    }
    let mut ids = BTreeSet::new();
    for (index, item) in checklist.items.iter().enumerate() {
        let expected = format!("REQ-{:02}", index + 1);
        if item.id != expected
            || !ids.insert(item.id.as_str())
            || item.title.trim().is_empty()
            || item.evidence.trim().is_empty()
            || item.remaining.trim().is_empty()
        {
            return Err(format!("项目验收项 {} 不完整或顺序不正确", item.id));
        }
    }
    Ok(checklist)
}

struct JiuhengApp {
    storage: Storage,
    storage_path: PathBuf,
    startup_warning: Option<String>,
    locations: Vec<GeoLocation>,
    registry: EngineRegistry,
    classifier: KeywordClassifier,
    page: Page,
    developer_mode: bool,
    category: QuestionCategory,
    subcategory: String,
    question_text: String,
    subject: String,
    object: String,
    target_event: String,
    target_time_text: String,
    options_text: String,
    location_index: usize,
    location_search: String,
    include_birth_parameters: bool,
    birth_sex: BiologicalSex,
    birth_time_text: String,
    birth_location_index: usize,
    birth_location_search: String,
    use_current_time: bool,
    manual_time_text: String,
    error_message: Option<String>,
    pending_duplicate: Option<DuplicateEvent>,
    pending_question: Option<QuestionInput>,
    pending_classification: Option<divination_core::Classification>,
    pending_fingerprint: Option<EventFingerprint>,
    revision_description: String,
    current_question: Option<QuestionInput>,
    current_classification: Option<divination_core::Classification>,
    current_fingerprint: Option<EventFingerprint>,
    applicability: Vec<Applicability>,
    casting_tab: CastingTab,
    casting_message: Option<String>,
    liuyao_transcript: Option<LiuyaoTranscript>,
    liuyao_cast: Option<LiuyaoCast>,
    liuyao_lines: Vec<LineCast>,
    liuyao_session_started: Option<Instant>,
    liuyao_press_started: Option<Instant>,
    liuyao_previous_event: Option<Instant>,
    liuyao_pointer_path: Vec<PointerSample>,
    yarrow_session: YarrowSession,
    yarrow_cast: Option<YarrowCast>,
    yarrow_left_pile: u8,
    yarrow_phase: YarrowUiPhase,
    yarrow_pending_left: Option<u8>,
    yarrow_last_change: Option<divination_core::YarrowChange>,
    history_items: Vec<QuestionHistorySummary>,
    history_selected: Option<Uuid>,
    history_detail: Option<QuestionHistoryDetail>,
    history_audit: Option<HistoryReplayAudit>,
    history_message: Option<String>,
    facsimile_source_id: String,
    facsimile_file_path: String,
    facsimile_verification_message: Option<(bool, String)>,
    project_acceptance: Result<ProjectAcceptanceChecklist, String>,
}

impl JiuhengApp {
    fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        configure_fonts_and_style(&creation_context.egui_ctx);
        let (storage, storage_path, startup_warning) = open_storage();
        let locations = storage.locations().unwrap_or_default();
        let zone = beijing_zone();
        let manual_time_text = Utc::now()
            .with_timezone(&zone)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let mut app = Self {
            storage,
            storage_path,
            startup_warning,
            locations,
            registry: EngineRegistry::default(),
            classifier: KeywordClassifier::default(),
            page: Page::NewQuestion,
            developer_mode: false,
            category: QuestionCategory::Career,
            subcategory: String::new(),
            question_text: String::new(),
            subject: String::new(),
            object: String::new(),
            target_event: String::new(),
            target_time_text: String::new(),
            options_text: String::new(),
            location_index: 0,
            location_search: String::new(),
            include_birth_parameters: false,
            birth_sex: BiologicalSex::Female,
            birth_time_text: String::new(),
            birth_location_index: 0,
            birth_location_search: String::new(),
            use_current_time: true,
            manual_time_text,
            error_message: None,
            pending_duplicate: None,
            pending_question: None,
            pending_classification: None,
            pending_fingerprint: None,
            revision_description: String::new(),
            current_question: None,
            current_classification: None,
            current_fingerprint: None,
            applicability: Vec::new(),
            casting_tab: CastingTab::Liuyao,
            casting_message: None,
            liuyao_transcript: None,
            liuyao_cast: None,
            liuyao_lines: Vec::new(),
            liuyao_session_started: None,
            liuyao_press_started: None,
            liuyao_previous_event: None,
            liuyao_pointer_path: Vec::new(),
            yarrow_session: YarrowSession::default(),
            yarrow_cast: None,
            yarrow_left_pile: 24,
            yarrow_phase: YarrowUiPhase::Divide,
            yarrow_pending_left: None,
            yarrow_last_change: None,
            history_items: Vec::new(),
            history_selected: None,
            history_detail: None,
            history_audit: None,
            history_message: None,
            facsimile_source_id: String::new(),
            facsimile_file_path: String::new(),
            facsimile_verification_message: None,
            project_acceptance: load_project_acceptance_checklist(),
        };
        app.refresh_history();
        app
    }

    fn render_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar")
            .frame(egui::Frame::new().fill(Color32::from_rgb(249, 247, 242)))
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(18.0);
                    ui.label(
                        RichText::new("九衡")
                            .size(27.0)
                            .strong()
                            .color(Color32::from_rgb(42, 57, 50)),
                    );
                    ui.label(
                        RichText::new("离线 · 无 AI · 古籍规则可追溯")
                            .size(13.0)
                            .color(Color32::from_rgb(104, 109, 98)),
                    );
                    ui.add_space(ui.available_width() - 235.0);
                    ui.toggle_value(&mut self.developer_mode, "开发者模式");
                    ui.add_space(12.0);
                });
                ui.add_space(8.0);
            });
    }

    fn render_side_bar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation")
            .exact_width(188.0)
            .frame(egui::Frame::new().fill(Color32::from_rgb(38, 54, 48)))
            .show(ctx, |ui| {
                ui.add_space(24.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("☰  推演台").color(Color32::from_rgb(217, 204, 168)));
                });
                ui.add_space(24.0);
                nav_button(ui, &mut self.page, Page::NewQuestion, "新建占问");
                nav_button(ui, &mut self.page, Page::History, "永久记录");
                if self.current_question.is_some() {
                    nav_button(ui, &mut self.page, Page::Casting, "独立起法");
                }
                nav_button(ui, &mut self.page, Page::Result, "本次结果");
                if self.developer_mode {
                    nav_button(ui, &mut self.page, Page::Developer, "校验详情");
                }
                nav_button(ui, &mut self.page, Page::About, "范围与声明");
                ui.add_space(ui.available_height() - 110.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(
                    RichText::new("数据仅保存在此设备")
                        .small()
                        .color(Color32::from_rgb(177, 186, 179)),
                );
                ui.label(
                    RichText::new(format!("证据门禁版 {}", env!("CARGO_PKG_VERSION")))
                        .small()
                        .color(Color32::from_rgb(177, 186, 179)),
                );
                ui.add_space(12.0);
            });
    }

    fn render_new_question(&mut self, ui: &mut egui::Ui) {
        section_title(
            ui,
            "锁定一个清晰的问题",
            "问题锁定后不可修改；改题必须新建占问。",
            "一",
        );

        if let Some(warning) = &self.startup_warning {
            warning_box(ui, warning, Color32::from_rgb(150, 92, 34));
        }
        if let Some(error) = &self.error_message {
            warning_box(ui, error, Color32::from_rgb(167, 54, 54));
        }

        ui.add_space(16.0);
        ui.label(RichText::new("选择类别").strong());
        ui.add_space(7.0);
        egui::Grid::new("category_grid")
            .num_columns(7)
            .spacing([7.0, 7.0])
            .show(ui, |ui| {
                for (index, category) in QuestionCategory::ALL.into_iter().enumerate() {
                    if ui
                        .selectable_label(self.category == category, category.zh_name())
                        .clicked()
                    {
                        self.category = category;
                        if category == QuestionCategory::LongTermFate {
                            self.include_birth_parameters = true;
                        }
                    }
                    if (index + 1) % 7 == 0 {
                        ui.end_row();
                    }
                }
            });

        ui.add_space(14.0);
        ui.columns(2, |columns| {
            labeled_text(
                &mut columns[0],
                "子类",
                "例如：项目成败",
                &mut self.subcategory,
            );
            let selected = self
                .locations
                .get(self.location_index)
                .map(location_label)
                .unwrap_or_else(|| "地点库暂不可用".to_owned());
            columns[1].label(RichText::new("地点").strong());
            columns[1].add(
                egui::TextEdit::singleline(&mut self.location_search)
                    .hint_text("输入省 / 市 / 区县 / 街道筛选离线地点"),
            );
            let search = self.location_search.trim();
            let mut matched_total = 0_usize;
            let choices = self
                .locations
                .iter()
                .enumerate()
                .filter_map(|(index, location)| {
                    if search.is_empty() || location_matches_search(location, search) {
                        matched_total += 1;
                        (matched_total <= 200).then(|| (index, location_label(location)))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            egui::ComboBox::from_id_salt("location")
                .selected_text(selected)
                .width(columns[1].available_width())
                .height(280.0)
                .show_ui(&mut columns[1], |ui| {
                    for (index, label) in &choices {
                        ui.selectable_value(&mut self.location_index, *index, label);
                    }
                });
            if matched_total > choices.len() {
                columns[1].small(format!(
                    "匹配 {matched_total} 条，当前显示前 {} 条；继续输入可缩小范围。",
                    choices.len()
                ));
            } else {
                columns[1].small(format!("离线库匹配 {matched_total} 条行政中心坐标。"));
            }
            if let Some(location) = self.locations.get(self.location_index) {
                columns[1].small(location_precision_notice(location));
            }
        });

        ui.add_space(14.0);
        ui.label(RichText::new("问题原文").strong());
        ui.add(
            egui::TextEdit::multiline(&mut self.question_text)
                .hint_text("只问一件事。例如：这个项目在未来三个月能否按计划完成？")
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        );
        let other_classification_preview = (self.category == QuestionCategory::Other
            && !self.question_text.trim().is_empty())
        .then(|| {
            self.classifier
                .classify_detailed(self.category, &self.question_text)
        });
        if let Some(preview) = &other_classification_preview {
            match preview {
                Ok(classification) => {
                    ui.small(format!(
                        "“其他”将映射为：{}",
                        classification.ancient_type.zh_name()
                    ));
                }
                Err(error) => {
                    ui.colored_label(
                        Color32::from_rgb(150, 92, 34),
                        classification_error_zh(error),
                    );
                }
            }
        }

        ui.add_space(12.0);
        ui.columns(3, |columns| {
            labeled_text(
                &mut columns[0],
                "主体",
                "例如：本人 / 项目组",
                &mut self.subject,
            );
            labeled_text(
                &mut columns[1],
                "客体",
                "例如：合作方 / 降雨",
                &mut self.object,
            );
            labeled_text(
                &mut columns[2],
                "目标事件",
                "例如：按期上线",
                &mut self.target_event,
            );
        });

        ui.add_space(12.0);
        ui.columns(2, |columns| {
            labeled_text(
                &mut columns[0],
                "目标时间 / 范围",
                "例如：三个月内",
                &mut self.target_time_text,
            );
            columns[1].label(RichText::new("占问时间（北京时间）").strong());
            columns[1].horizontal(|ui| {
                ui.checkbox(&mut self.use_current_time, "使用电脑当前真实时间");
            });
            if !self.use_current_time {
                columns[1].text_edit_singleline(&mut self.manual_time_text);
                columns[1].small(
                    "格式 YYYY-MM-DD HH:MM:SS；公元前采用天文年，如 -0999 表示公元前 1000 年。",
                );
                columns[1].label(
                    RichText::new("传统占事以实际起占时刻为宜，非特殊用途建议不要修改。")
                        .small()
                        .color(Color32::from_rgb(150, 92, 34)),
                );
            }
        });

        if self.category == QuestionCategory::Weather {
            ui.small(
                "首条六爻降雨流程：请明确填写‘是否下雨’和目标时段（如 1小时后、3小时内、今日内）。参数符合范围不代表已能占断；古籍规则未放行前仍不生成晴雨结论。",
            );
        }

        if self.category == QuestionCategory::LongTermFate {
            ui.add_space(14.0);
            ui.group(|ui| {
                ui.label(RichText::new("长期命理出生参数").strong());
                ui.checkbox(
                    &mut self.include_birth_parameters,
                    "本次提供完整出生参数",
                );
                if self.include_birth_parameters {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("性别：");
                        ui.radio_value(&mut self.birth_sex, BiologicalSex::Female, "女");
                        ui.radio_value(&mut self.birth_sex, BiologicalSex::Male, "男");
                    });
                    ui.columns(2, |columns| {
                        columns[0].label(RichText::new("出生时间（北京时间）").strong());
                        columns[0].add(
                            egui::TextEdit::singleline(&mut self.birth_time_text)
                                .hint_text("例如：1990-05-17 08:30:00"),
                        );
                        columns[0].small(
                            "统一按 UTC+8 输入；公元前用天文年（0000=公元前1年）；真太阳时按经度计算。",
                        );

                        let selected = self
                            .locations
                            .get(self.birth_location_index)
                            .map(location_label)
                            .unwrap_or_else(|| "出生地点库暂不可用".to_owned());
                        columns[1].label(RichText::new("出生地点").strong());
                        columns[1].add(
                            egui::TextEdit::singleline(&mut self.birth_location_search)
                                .hint_text("输入省 / 市 / 区县 / 街道筛选"),
                        );
                        let search = self.birth_location_search.trim();
                        let mut matched_total = 0_usize;
                        let choices = self
                            .locations
                            .iter()
                                .enumerate()
                                .filter_map(|(index, location)| {
                                    if search.is_empty()
                                        || location_matches_search(location, search)
                                    {
                                        matched_total += 1;
                                        (matched_total <= 200)
                                            .then(|| (index, location_label(location)))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        egui::ComboBox::from_id_salt("birth_location")
                            .selected_text(selected)
                            .width(columns[1].available_width())
                            .height(280.0)
                            .show_ui(&mut columns[1], |ui| {
                                for (index, label) in &choices {
                                    ui.selectable_value(
                                        &mut self.birth_location_index,
                                        *index,
                                        label,
                                    );
                                }
                            });
                        columns[1].small(format!("离线库匹配 {matched_total} 条行政中心坐标。"));
                        if let Some(location) = self.locations.get(self.birth_location_index) {
                            columns[1].small(location_precision_notice(location));
                        }
                    });
                } else {
                    ui.add_space(6.0);
                    ui.colored_label(
                        Color32::from_rgb(150, 92, 34),
                        "未提供时仍可锁定问题，但四柱八字、紫微斗数和七政四余会以“参数不足”自动跳过。",
                    );
                }
            });
        }

        ui.add_space(12.0);
        ui.label(RichText::new("多选项（可选，每行一个）").strong());
        ui.add(
            egui::TextEdit::multiline(&mut self.options_text)
                .hint_text("A 方案\nB 方案\nC 方案")
                .desired_rows(2)
                .desired_width(f32::INFINITY),
        );

        if health_notice_required_for_input(self.category, other_classification_preview.as_ref()) {
            ui.add_space(12.0);
            warning_box(
                ui,
                "健康内容仅属传统术数文化推演，不能用于诊断、治疗或生死决策；如有症状请及时咨询合格医疗人员。",
                Color32::from_rgb(150, 92, 34),
            );
        }

        ui.add_space(18.0);
        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [190.0, 42.0],
                    egui::Button::new(RichText::new("静心确认并锁定问题").strong())
                        .fill(Color32::from_rgb(154, 67, 55)),
                )
                .clicked()
            {
                self.try_lock_question();
            }
            ui.label(
                RichText::new("九套引擎都会检查适用性；证据不足者自动退出。")
                    .small()
                    .color(Color32::from_rgb(104, 109, 98)),
            );
        });

        let duplicate_snapshot = self.pending_duplicate.as_ref().map(|duplicate| {
            (
                duplicate.question_text.clone(),
                duplicate.locked_at_bjt.clone(),
                duplicate.exact_digest,
                duplicate.score_percent,
                duplicate.algorithm_version.clone(),
                duplicate.match_reasons.clone(),
            )
        });
        if let Some((
            question_text,
            locked_at_bjt,
            exact_digest,
            score_percent,
            algorithm_version,
            match_reasons,
        )) = duplicate_snapshot
        {
            ui.add_space(18.0);
            ui.group(|ui| {
                ui.label(
                    RichText::new("不二占拦截")
                        .strong()
                        .color(Color32::from_rgb(154, 67, 55)),
                );
                ui.label(format!("系统识别为同一件事：{question_text}"));
                ui.label(format!("上次锁定：{locked_at_bjt}"));
                ui.label(format!(
                    "识别方式：{} · 规则匹配分数：{}%",
                    if exact_digest {
                        "事件指纹完全一致"
                    } else {
                        "多字段纯规则相似匹配"
                    },
                    score_percent
                ));
                ui.label(format!("匹配依据：{}", match_reasons.join("；")));
                if self.developer_mode {
                    ui.label(format!("算法版本：{algorithm_version}"));
                }
                ui.label("同一件事不反复占。只有现实情况发生实质变化时，才可建立 EVENT_REVISION。");
                ui.add(
                    egui::TextEdit::multiline(&mut self.revision_description)
                        .hint_text("填写已经发生的实质变化；不能只写“想再问一次”。")
                        .desired_rows(2)
                        .desired_width(f32::INFINITY),
                );
                if ui.button("确认情况已变化并建立新占问").clicked() {
                    self.commit_revision();
                }
            });
        }
    }

    fn render_history(&mut self, ui: &mut egui::Ui) {
        section_title(
            ui,
            "永久占问记录",
            "记录只读；锁定原文、参数、事件修订和起法材料均不能在此修改。",
            "录",
        );
        ui.add_space(12.0);
        if ui.button("刷新本地记录").clicked() {
            self.refresh_history();
        }
        if let Some(message) = &self.history_message {
            ui.add_space(8.0);
            warning_box(ui, message, Color32::from_rgb(150, 92, 34));
        }
        if self.history_items.is_empty() {
            ui.add_space(24.0);
            ui.label("本机尚无已锁定占问。");
            return;
        }

        let selected = self.history_selected;
        let detail = self.history_detail.clone();
        let audit = self.history_audit.clone();
        let developer_mode = self.developer_mode;
        let mut requested_selection = None;
        let mut run_audit = false;

        ui.add_space(10.0);
        ui.columns(2, |columns| {
            columns[0].label(
                RichText::new(format!("本机记录 · {} 条", self.history_items.len())).strong(),
            );
            columns[0].separator();
            egui::ScrollArea::vertical()
                .id_salt("history_list")
                .max_height(610.0)
                .show(&mut columns[0], |ui| {
                    for summary in &self.history_items {
                        let question = &summary.question;
                        let mut badges = Vec::new();
                        if summary.has_liuyao {
                            badges.push("六爻");
                        }
                        if summary.has_yarrow {
                            badges.push("蓍草");
                        }
                        if summary.is_event_revision {
                            badges.push("情况变化");
                        }
                        let badge_text = if badges.is_empty() {
                            "仅锁题".to_owned()
                        } else {
                            badges.join(" · ")
                        };
                        let label = format!(
                            "{}  {}\n{}\n{}",
                            question.locked_at_bjt.format("%Y-%m-%d %H:%M"),
                            question.category.zh_name(),
                            question.text,
                            badge_text
                        );
                        if ui
                            .selectable_label(selected == Some(question.id), label)
                            .clicked()
                        {
                            requested_selection = Some(question.id);
                        }
                        ui.add_space(4.0);
                    }
                });

            columns[1].label(RichText::new("只读详情").strong());
            columns[1].separator();
            egui::ScrollArea::vertical()
                .id_salt("history_detail")
                .max_height(610.0)
                .show(&mut columns[1], |ui| {
                    let Some(detail) = detail else {
                        ui.label("请选择一条记录。");
                        return;
                    };
                    let question = &detail.question;
                    ui.label(RichText::new(&question.text).size(20.0).strong());
                    ui.add_space(10.0);
                    egui::Grid::new("history_detail_meta")
                        .num_columns(2)
                        .striped(true)
                        .spacing([14.0, 7.0])
                        .show(ui, |ui| {
                            ui.label("类别");
                            ui.label(format!(
                                "{}{}",
                                question.category.zh_name(),
                                if question.subcategory.is_empty() {
                                    String::new()
                                } else {
                                    format!(" · {}", question.subcategory)
                                }
                            ));
                            ui.end_row();
                            ui.label("锁定时间");
                            ui.monospace(question.locked_at_bjt.to_rfc3339());
                            ui.end_row();
                            ui.label("占问时间");
                            ui.monospace(question.divination_time_bjt.to_rfc3339());
                            ui.end_row();
                            ui.label("地点");
                            ui.label(location_label(&question.location));
                            ui.end_row();
                            ui.label("目标时间");
                            ui.label(if question.target_time_text.is_empty() {
                                "未填写"
                            } else {
                                &question.target_time_text
                            });
                            ui.end_row();
                            ui.label("主体 / 客体");
                            ui.label(format!("{} / {}", question.subject, question.object));
                            ui.end_row();
                            ui.label("记录状态");
                            ui.label(history_state_label(&detail.state));
                            ui.end_row();
                        });

                    if let Some(birth) = &question.birth {
                        ui.add_space(10.0);
                        ui.label(RichText::new("出生参数（锁定只读）").strong());
                        ui.label(format!(
                            "性别：{} · 出生时间：{} · 出生地点：{}",
                            biological_sex_label(birth.sex),
                            birth.date_time_bjt.format("%Y-%m-%d %H:%M:%S UTC+8"),
                            location_label(&birth.location)
                        ));
                    } else if question.category == QuestionCategory::LongTermFate {
                        ui.add_space(10.0);
                        ui.colored_label(
                            Color32::from_rgb(150, 92, 34),
                            "本记录未提供出生参数；相关命理术式按参数不足跳过。",
                        );
                    }

                    if !question.options.is_empty() {
                        ui.add_space(10.0);
                        ui.label(RichText::new("比较选项").strong());
                        for (index, option) in question.options.iter().enumerate() {
                            ui.label(format!("{}. {option}", index + 1));
                        }
                    }

                    ui.add_space(12.0);
                    ui.label(RichText::new("结果").strong());
                    ui.label(
                        detail
                            .final_conclusion
                            .as_deref()
                            .unwrap_or("结论尚未生成：没有术式通过古籍断法证据门禁。"),
                    );
                    ui.label(format!(
                        "时间：{}",
                        detail.final_timing.as_deref().unwrap_or("暂无法断定")
                    ));
                    ui.add_space(6.0);
                    ui.label(RichText::new("当下建议").strong());
                    if detail.final_action_advice.items.is_empty() {
                        ui.label(&detail.final_action_advice.notice);
                    } else {
                        for (index, advice) in
                            detail.final_action_advice.items.iter().enumerate()
                        {
                            ui.label(format!(
                                "{}. {}：{}",
                                index + 1,
                                advice.polarity.zh_name(),
                                advice.text
                            ));
                        }
                        if !detail
                            .final_action_advice
                            .suppressed_action_keys
                            .is_empty()
                        {
                            ui.colored_label(
                                Color32::from_rgb(150, 92, 34),
                                format!(
                                    "{} 个行动项因规则方向冲突已隐藏。",
                                    detail.final_action_advice.suppressed_action_keys.len()
                                ),
                            );
                        }
                    }
                    ui.small(format!(
                        "现实行动底线：{}",
                        practical_action_notice_for_classified_question(
                            question.category,
                            Some(&detail.ancient_question_type)
                        )
                    ));
                    if classified_question_requires_health_notice(
                        question.category,
                        Some(&detail.ancient_question_type),
                    ) {
                        ui.colored_label(
                            Color32::from_rgb(150, 92, 34),
                            "仅属传统术数文化推演，不能用于诊断或治疗决策。",
                        );
                    }
                    ui.add_space(6.0);
                    ui.label(format!(
                        "古法依据等级：{}",
                        detail.final_evidence_tier.as_deref().unwrap_or("未评定")
                    ));
                    if developer_mode {
                        if !detail.engine_runs.is_empty() {
                            ui.add_space(10.0);
                            ui.collapsing("术式规则运行审计", |ui| {
                                for run in &detail.engine_runs {
                                    ui.group(|ui| {
                                        ui.monospace(format!(
                                            "{} / {} / {}",
                                            run.method.zh_name(),
                                            run.implementation_version,
                                            status_label(run.status)
                                        ));
                                        ui.monospace(format!(
                                            "started {} / completed {}",
                                            run.started_at,
                                            run.completed_at.as_deref().unwrap_or("—")
                                        ));
                                        if !run.reasons.is_empty() {
                                            ui.label(run.reasons.join("；"));
                                        }
                                        if let Some(chart) = &run.chart {
                                            ui.collapsing("排盘快照", |ui| {
                                                ui.monospace(
                                                    serde_json::to_string_pretty(chart)
                                                        .unwrap_or_else(|_| "<invalid>".to_owned()),
                                                );
                                            });
                                        }
                                        if let Some(signals) = &run.signals {
                                            ui.collapsing("标准化中间信号", |ui| {
                                                ui.monospace(
                                                    serde_json::to_string_pretty(signals)
                                                        .unwrap_or_else(|_| "<invalid>".to_owned()),
                                                );
                                            });
                                        }
                                        for hit in &run.rule_hits {
                                            ui.collapsing(
                                                format!("命中 Rule ID：{}", hit.rule_id),
                                                |ui| {
                                                    ui.monospace(
                                                        serde_json::to_string_pretty(
                                                            &hit.input_snapshot,
                                                        )
                                                        .unwrap_or_else(|_| {
                                                            "<invalid input>".to_owned()
                                                        }),
                                                    );
                                                    ui.monospace(
                                                        serde_json::to_string_pretty(
                                                            &hit.output_snapshot,
                                                        )
                                                        .unwrap_or_else(|_| {
                                                            "<invalid output>".to_owned()
                                                        }),
                                                    );
                                                    for trace in &hit.evaluation_trace {
                                                        ui.monospace(format!("trace: {trace}"));
                                                    }
                                                },
                                            );
                                        }
                                    });
                                    ui.add_space(6.0);
                                }
                            });
                        }
                        match &detail.fusion_decision {
                            Some(decision) => {
                                ui.add_space(10.0);
                                ui.collapsing("综合裁决完整审计", |ui| {
                                    ui.monospace(format!(
                                        "{} / signal schema v{} / {:?}",
                                        decision.algorithm_version,
                                        decision.signal_schema_version,
                                        decision.mode
                                    ));
                                    ui.monospace(format!(
                                        "methods {} / raw {} / decision {} / conflict break {}",
                                        decision.participating_methods,
                                        decision.raw_equal_weight_score,
                                        decision.decision_score,
                                        decision.conflict_break_used
                                    ));
                                    if let Some(option) = &decision.selected_option {
                                        ui.monospace(format!("selected option: {option}"));
                                    }
                                    ui.monospace(format!(
                                        "action advice schema v{} / status {:?} / items {} / suppressed {}",
                                        decision.action_advice.schema_version,
                                        decision.action_advice.status,
                                        decision.action_advice.items.len(),
                                        decision.action_advice.suppressed_action_keys.len()
                                    ));
                                    for advice in &decision.action_advice.items {
                                        ui.monospace(format!(
                                            "action {} / {:?} / tier {:?} / methods {} / rules {} / {}",
                                            advice.action_key,
                                            advice.polarity,
                                            advice.evidence_tier,
                                            advice
                                                .supporting_methods
                                                .iter()
                                                .map(|method| method.key())
                                                .collect::<Vec<_>>()
                                                .join(","),
                                            advice.supporting_rule_ids.join(","),
                                            advice.text
                                        ));
                                    }
                                    for trace in &decision.action_advice.trace {
                                        ui.monospace(format!("action trace: {trace}"));
                                    }
                                    for contribution in &decision.method_contributions {
                                        ui.monospace(format!(
                                            "{} / weight {} / composite {:?} / tier {:?} / rules {}",
                                            contribution.method.zh_name(),
                                            contribution.weight_basis,
                                            contribution.composite_score,
                                            contribution.evidence_tier,
                                            contribution.hit_rule_ids.join(",")
                                        ));
                                    }
                                    for trace in &decision.trace {
                                        ui.monospace(trace);
                                    }
                                });
                            }
                            None => {
                                ui.monospace(format!(
                                    "Fusion: {} / 无通过门禁的术式结果，未生成快照",
                                    divination_core::FUSION_ALGORITHM_VERSION
                                ));
                            }
                        }
                    }

                    if let Some(revision) = &detail.revision {
                        ui.add_space(12.0);
                        warning_box(
                            ui,
                            &format!(
                                "EVENT_REVISION：{}\n前一记录：{}\n建立时间：{}",
                                revision.change_description,
                                revision.prior_question_id,
                                revision.created_at_bjt
                            ),
                            Color32::from_rgb(150, 92, 34),
                        );
                        if let Some(score) = revision.match_score_percent {
                            ui.label(format!(
                                "建立修订前的不二占识别：{} · {}%",
                                if revision.match_exact_digest == Some(true) {
                                    "事件指纹完全一致"
                                } else {
                                    "多字段纯规则相似匹配"
                                },
                                score
                            ));
                            if !revision.match_reasons.is_empty() {
                                ui.label(format!(
                                    "识别依据：{}",
                                    revision.match_reasons.join("；")
                                ));
                            }
                            if self.developer_mode {
                                ui.label(format!(
                                    "识别算法：{}",
                                    revision
                                        .match_algorithm_version
                                        .as_deref()
                                        .unwrap_or("旧记录未保存")
                                ));
                            }
                        } else {
                            ui.label("旧版 EVENT_REVISION 未保存相似匹配审计字段。");
                        }
                        if ui.button("查看前一事件版本").clicked() {
                            requested_selection = Some(revision.prior_question_id);
                        }
                    }
                    if !detail.later_revision_ids.is_empty() {
                        ui.add_space(8.0);
                        ui.label(RichText::new("后续情况变化版本").strong());
                        for later_id in &detail.later_revision_ids {
                            if ui.button(format!("查看 {later_id}")).clicked() {
                                requested_selection = Some(*later_id);
                            }
                        }
                    }

                    ui.add_space(14.0);
                    ui.label(RichText::new("起法材料重放").strong());
                    if ui.button("从 SQLite 重新计算并校验").clicked() {
                        run_audit = true;
                    }
                    match audit {
                        Some(audit) => {
                            render_replay_proof(ui, "六爻", audit.liuyao.as_ref(), developer_mode);
                            render_replay_proof(ui, "蓍草", audit.yarrow.as_ref(), developer_mode);
                            if audit.liuyao.is_none() && audit.yarrow.is_none() {
                                ui.label("此记录没有已完成的随机起法材料。");
                            }
                        }
                        None => {
                            ui.label("尚未执行本次读取后重放。");
                        }
                    }

                    if developer_mode {
                        ui.add_space(14.0);
                        ui.collapsing("开发者审计字段", |ui| {
                            ui.monospace(format!("Question ID: {}", question.id));
                            ui.monospace(format!(
                                "Ancient type: {}",
                                ancient_type_label(&detail.ancient_question_type)
                            ));
                            ui.monospace(format!(
                                "Fingerprint v{}: {}",
                                detail.event_fingerprint_version, detail.event_fingerprint
                            ));
                            ui.monospace(format!(
                                "Normalized terms: {}",
                                detail
                                    .normalized_terms
                                    .iter()
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join(" | ")
                            ));
                            ui.monospace(format!("Created at: {}", detail.created_at));
                            if let Some(snapshot) = &detail.astronomy_snapshot {
                                ui.separator();
                                ui.monospace(format!(
                                    "Astronomy snapshot v{}: {}",
                                    snapshot.snapshot_version, snapshot.solar_algorithm_version
                                ));
                                ui.monospace(format!(
                                    "JD UT1≈ {:.9} / ΔT {:.3}±{:.3}s / JD TT {:.9}",
                                    snapshot.jd_ut1_approx,
                                    snapshot.delta_t_seconds,
                                    snapshot.delta_t_uncertainty_seconds,
                                    snapshot.jd_tt
                                ));
                                ui.monospace(format!(
                                    "Sun λ {:.9}° / next {} {:.9}±{:.2}s",
                                    snapshot.solar_longitude_deg,
                                    snapshot.next_solar_term.zh_name(),
                                    snapshot.next_solar_term_jd_tt,
                                    snapshot.next_solar_term_uncertainty_seconds
                                ));
                                if let (
                                    Some(previous),
                                    Some(previous_jd),
                                    Some(previous_uncertainty),
                                    Some(seconds_since),
                                    Some(seconds_until),
                                    Some(boundary_safe),
                                ) = (
                                    snapshot.previous_solar_term,
                                    snapshot.previous_solar_term_jd_tt,
                                    snapshot.previous_solar_term_uncertainty_seconds,
                                    snapshot.seconds_since_previous_solar_term,
                                    snapshot.seconds_until_next_solar_term,
                                    snapshot.solar_term_boundary_safe,
                                ) {
                                    ui.monospace(format!(
                                        "Term span: {} {:.9}±{:.2}s → {} / elapsed {:.6}d / remaining {:.6}d / safe {}",
                                        previous.zh_name(),
                                        previous_jd,
                                        previous_uncertainty,
                                        snapshot.next_solar_term.zh_name(),
                                        seconds_since / 86_400.0,
                                        seconds_until / 86_400.0,
                                        boundary_safe
                                    ));
                                }
                                if let Some(interval) = &snapshot.jie_interval {
                                    ui.monospace(format!(
                                        "Jie span: {} → {} / elapsed {:.6}d / remaining {:.6}d / safe {}",
                                        interval.previous.term.zh_name(),
                                        interval.next.term.zh_name(),
                                        interval.seconds_since_previous / 86_400.0,
                                        interval.seconds_until_next / 86_400.0,
                                        interval.boundary_safe
                                    ));
                                }
                                if let Some(interval) = &snapshot.zhongqi_interval {
                                    ui.monospace(format!(
                                        "Zhongqi span: {} → {} / elapsed {:.6}d / remaining {:.6}d / safe {}",
                                        interval.previous.term.zh_name(),
                                        interval.next.term.zh_name(),
                                        interval.seconds_since_previous / 86_400.0,
                                        interval.seconds_until_next / 86_400.0,
                                        interval.boundary_safe
                                    ));
                                }
                                if let Some(interval) = &snapshot.lunation_interval {
                                    ui.monospace(format!(
                                        "Lunation: {} {:.9}±{:.0}s → {} {:.9}±{:.0}s → {} {:.9}±{:.0}s / elapsed {:.6}d / remaining {:.6}d / safe {}",
                                        interval.previous_new_moon.phase.zh_name(),
                                        interval.previous_new_moon.jd_tt,
                                        interval.previous_new_moon.estimated_formula_uncertainty_seconds,
                                        interval.full_moon.phase.zh_name(),
                                        interval.full_moon.jd_tt,
                                        interval.full_moon.estimated_formula_uncertainty_seconds,
                                        interval.next_new_moon.phase.zh_name(),
                                        interval.next_new_moon.jd_tt,
                                        interval.next_new_moon.estimated_formula_uncertainty_seconds,
                                        interval.seconds_since_previous_new_moon / 86_400.0,
                                        interval.seconds_until_next_new_moon / 86_400.0,
                                        interval.new_moon_boundary_safe
                                    ));
                                    ui.monospace(format!(
                                        "Lunation status: {} / {}",
                                        interval.status, interval.algorithm_version
                                    ));
                                }
                                if let Some(structure) = &snapshot.lunisolar_month_structure {
                                    ui.monospace(format!(
                                        "Lunisolar structure: winter-solstice year {} / {} months / leap {} / all safe {}",
                                        structure.winter_solstice_year,
                                        structure.month_count,
                                        structure
                                            .leap_month_number
                                            .map_or_else(|| "none".to_owned(), |month| month.to_string()),
                                        structure.all_boundaries_safe
                                    ));
                                    for month in &structure.months {
                                        let start = month.start_bjt.local_date_time;
                                        ui.monospace(format!(
                                            "  #{:02} {}{}月 {:04}-{:02}-{:02} / Zhongqi {} / safe {}",
                                            month.sequence_index,
                                            if month.is_leap { "leap " } else { "" },
                                            month.month_number,
                                            start.year,
                                            start.month,
                                            start.day,
                                            month
                                                .major_solar_term
                                                .as_ref()
                                                .map_or("none", |term| term.term.zh_name()),
                                            month.term_classification_safe
                                        ));
                                    }
                                    ui.monospace(format!(
                                        "Lunisolar status: {} / {}",
                                        structure.status, structure.algorithm_version
                                    ));
                                } else if let Some(error) =
                                    &snapshot.lunisolar_month_structure_error
                                {
                                    ui.monospace(format!("Lunisolar structure blocked: {error}"));
                                }
                                if let Some(candidate) = &snapshot.lunisolar_date_candidate {
                                    ui.monospace(format!(
                                        "Lunisolar date candidate: nominal year {} / {}{}月 {}日 / {} days / assignment safe {}",
                                        candidate.nominal_lunar_year,
                                        if candidate.is_leap_month { "leap " } else { "" },
                                        candidate.month_number,
                                        candidate.day_number,
                                        candidate.month_length_days,
                                        candidate.date_assignment_safe
                                    ));
                                    ui.monospace(format!(
                                        "Lunisolar date status: {} / {}",
                                        candidate.status, candidate.algorithm_version
                                    ));
                                } else if let Some(error) =
                                    &snapshot.lunisolar_date_candidate_error
                                {
                                    ui.monospace(format!("Lunisolar date blocked: {error}"));
                                }
                                if let Some(audit) = &snapshot.meihua_time_casting_audit {
                                    ui.monospace(format!(
                                        "Meihua time audit v{}: year {}={} / month {}{} / day {} / candidates identical {} / selected none / rule eligible {}",
                                        audit.audit_version,
                                        audit.year_branch.zh_name(),
                                        audit.year_branch.index() + 1,
                                        if audit.lunisolar_date_candidate.is_leap_month {
                                            "leap "
                                        } else {
                                            ""
                                        },
                                        audit.lunisolar_date_candidate.month_number,
                                        audit.lunisolar_date_candidate.day_number,
                                        audit.candidates_identical,
                                        audit.rule_eligible
                                    ));
                                    if let Some(candidate) = &audit.beijing_time_candidate {
                                        ui.monospace(format!(
                                            "  Beijing-hour candidate: {} / safe {}",
                                            candidate.cast.compact_zh(),
                                            candidate.mechanical_boundaries_safe
                                        ));
                                        match candidate.cast.mutual_raw_audit() {
                                            Ok(raw) => ui.monospace(format!(
                                                "    Raw inner trigrams: {} / {}",
                                                raw.compact_zh(),
                                                raw.algorithm_version
                                            )),
                                            Err(error) => ui.monospace(format!(
                                                "    Raw inner trigrams blocked: {error}"
                                            )),
                                        };
                                    }
                                    if let Some(candidate) = &audit.true_solar_time_candidate {
                                        ui.monospace(format!(
                                            "  True-solar-hour candidate: {} / safe {}",
                                            candidate.cast.compact_zh(),
                                            candidate.mechanical_boundaries_safe
                                        ));
                                        match candidate.cast.mutual_raw_audit() {
                                            Ok(raw) => ui.monospace(format!(
                                                "    Raw inner trigrams: {} / {}",
                                                raw.compact_zh(),
                                                raw.algorithm_version
                                            )),
                                            Err(error) => ui.monospace(format!(
                                                "    Raw inner trigrams blocked: {error}"
                                            )),
                                        };
                                    }
                                    ui.monospace(format!(
                                        "  Meihua blocked: {}",
                                        audit.blocked_reasons.join("; ")
                                    ));
                                }
                                if let Some(audit) =
                                    &snapshot.chinese_astrology_structure_audit
                                {
                                    ui.monospace(format!(
                                        "Chinese astrology structure v{} / {} / enclosure boundary selected {} / mansion boundary selected {} / positions calculated {} / rule eligible {}",
                                        audit.audit_version,
                                        audit.algorithm_version,
                                        audit.enclosure_boundary_model_selected,
                                        audit.angular_boundary_model_selected,
                                        audit.celestial_positions_calculated,
                                        audit.rule_eligible
                                    ));
                                    ui.monospace(format!(
                                        "  Three enclosures: {}",
                                        three_enclosure_catalog_zh(audit)
                                    ));
                                    for quadrant in MansionQuadrant::ALL_IN_RECEIVED_ORDER {
                                        ui.monospace(format!(
                                            "  {}{}：{}",
                                            quadrant.zh_name(),
                                            quadrant.emblem().received_zh_name(),
                                            mansion_sequence_for_quadrant_zh(audit, quadrant)
                                        ));
                                    }
                                    ui.monospace(format!(
                                        "  Seven luminaries: {}",
                                        seven_luminary_catalog_zh(audit)
                                    ));
                                    ui.monospace(format!(
                                        "  Chinese astrology blocked: {}",
                                        audit.blocked_reasons.join("; ")
                                    ));
                                }
                                if let Some(audit) = &snapshot.qimen_ju_audit {
                                    ui.monospace(format!(
                                        "Qimen ju audit v{}: {}→{} / {} / table {:?} / candidates identical {} / selected none / rule eligible {}",
                                        audit.audit_version,
                                        audit.active_solar_term.zh_name(),
                                        audit.next_solar_term.zh_name(),
                                        audit.dun.zh_name(),
                                        audit.received_bureau_numbers,
                                        audit.candidates_identical,
                                        audit.rule_eligible
                                    ));
                                    ui.monospace(format!(
                                        "  Beijing-midnight candidate: {} / safe {}",
                                        audit
                                            .beijing_midnight_candidate
                                            .compact_zh(audit.dun),
                                        audit
                                            .beijing_midnight_candidate
                                            .mechanical_boundaries_safe
                                    ));
                                    if let Some(plate) =
                                        &audit.beijing_midnight_candidate.earth_plate
                                    {
                                        ui.monospace(format!(
                                            "    Earth plate: {}",
                                            plate.compact_zh()
                                        ));
                                    }
                                    if let Some(hour) =
                                        &audit.beijing_midnight_candidate.hour_xun_candidate
                                    {
                                        ui.monospace(format!(
                                            "    Hour xun: {} / safe {}",
                                            hour.compact_zh(),
                                            hour.mechanical_boundaries_safe
                                        ));
                                        if let Some(identity) = &hour.duty_identity_candidate {
                                            ui.monospace(format!(
                                                "      Duty identities: {} / safe {}",
                                                identity.compact_zh(),
                                                identity.mechanical_boundaries_safe
                                            ));
                                        }
                                    }
                                    ui.monospace(format!(
                                        "  True-solar-midnight candidate: {} / safe {}",
                                        audit
                                            .true_solar_midnight_candidate
                                            .compact_zh(audit.dun),
                                        audit
                                            .true_solar_midnight_candidate
                                            .mechanical_boundaries_safe
                                    ));
                                    if let Some(plate) =
                                        &audit.true_solar_midnight_candidate.earth_plate
                                    {
                                        ui.monospace(format!(
                                            "    Earth plate: {}",
                                            plate.compact_zh()
                                        ));
                                    }
                                    if let Some(hour) =
                                        &audit.true_solar_midnight_candidate.hour_xun_candidate
                                    {
                                        ui.monospace(format!(
                                            "    Hour xun: {} / safe {}",
                                            hour.compact_zh(),
                                            hour.mechanical_boundaries_safe
                                        ));
                                        if let Some(identity) = &hour.duty_identity_candidate {
                                            ui.monospace(format!(
                                                "      Duty identities: {} / safe {}",
                                                identity.compact_zh(),
                                                identity.mechanical_boundaries_safe
                                            ));
                                        }
                                    }
                                    ui.monospace(format!(
                                        "  Qimen blocked: {}",
                                        audit.blocked_reasons.join("; ")
                                    ));
                                }
                                if let Some(label) = snapshot.astronomical_year_label {
                                    ui.monospace(format!(
                                        "Civil-year label: {} (not a boundary convention)",
                                        label.zh_name()
                                    ));
                                }
                                if let Some(candidate) = &snapshot.lichun_year_candidate {
                                    ui.monospace(format!(
                                        "Lichun year candidate: {} / boundary {:.9} / offset {:+.3}s / safe {}",
                                        candidate.designation.zh_name(),
                                        candidate.boundary_jd_tt,
                                        candidate.seconds_from_boundary,
                                        candidate.boundary_safe
                                    ));
                                }
                                if let Some(candidate) = &snapshot.jie_month_candidate {
                                    ui.monospace(format!(
                                        "Jie month candidate: {} / {} → {} / safe {} / audit only",
                                        candidate.designation.zh_name(),
                                        candidate.previous_jie.zh_name(),
                                        candidate.next_jie.zh_name(),
                                        candidate.boundary_safe
                                    ));
                                }
                                if let Some(calendar) = &snapshot.civil_calendar_audit {
                                    ui.monospace(format!(
                                        "Civil calendar {}: Gregorian {} / Julian {} / historical {} ({}) / JD {:.9}",
                                        calendar.algorithm_version,
                                        historical_datetime_label_zh(calendar.proleptic_gregorian),
                                        historical_datetime_label_zh(calendar.julian),
                                        historical_datetime_label_zh(calendar.historical_cutover),
                                        calendar_system_label(
                                            calendar.historical_cutover_resolved_system
                                        ),
                                        calendar.julian_day
                                    ));
                                }
                                if let Some(birth_audit) = &snapshot.birth_time_audit {
                                    ui.monospace(format!(
                                        "Birth audit v{}: {} → true solar {} / correction {:+.6}m / day {} / hour {} / safe {} / {}",
                                        birth_audit.audit_version,
                                        birth_audit.input_time_bjt,
                                        birth_audit.true_solar_time_bjt,
                                        birth_audit.true_solar_total_correction_minutes,
                                        birth_audit.sexagenary_day_true_solar_midnight.zh_name(),
                                        birth_audit.sexagenary_hour_true_solar_midnight.zh_name(),
                                        birth_audit.true_solar_boundary_safe,
                                        birth_audit.status
                                    ));
                                    if let Some(calendar) = &birth_audit.civil_calendar_audit {
                                        ui.monospace(format!(
                                            "Birth calendar: Gregorian {} / Julian {} / historical {} ({})",
                                            historical_datetime_label_zh(
                                                calendar.proleptic_gregorian
                                            ),
                                            historical_datetime_label_zh(calendar.julian),
                                            historical_datetime_label_zh(
                                                calendar.historical_cutover
                                            ),
                                            calendar_system_label(
                                                calendar.historical_cutover_resolved_system
                                            )
                                        ));
                                    }
                                    if let Some(chart) =
                                        &birth_audit.four_pillars_chart_candidate
                                    {
                                        ui.monospace(format!(
                                            "Four-pillars chart candidate: {} / boundaries safe {} / late-Zi ambiguous {} / rule eligible {}",
                                            chart.compact_zh(),
                                            chart.mechanical_boundaries_safe,
                                            chart.late_zi_hour_day_ambiguous,
                                            chart.rule_eligible
                                        ));
                                        ui.monospace(format!(
                                            "Four-pillars status: {} / {}",
                                            chart.status, chart.algorithm_version
                                        ));
                                    }
                                    if let Some(metadata) =
                                        &birth_audit.four_pillars_structural_metadata
                                    {
                                        ui.monospace(format!(
                                            "Stem/branch structure: {} / {}",
                                            metadata.compact_zh(), metadata.status
                                        ));
                                    }
                                    if let Some(ten_gods) =
                                        &birth_audit.four_pillars_visible_stem_ten_gods
                                    {
                                        ui.monospace(format!(
                                            "Visible-stem ten-god identities (midnight candidate): {} / hidden stems false / strength false / rule eligible false / {}",
                                            ten_gods.compact_zh(), ten_gods.algorithm_version
                                        ));
                                    }
                                    if let Some(zi_start) =
                                        &birth_audit.four_pillars_visible_stem_ten_gods_zi_start
                                    {
                                        ui.monospace(format!(
                                            "Visible-stem ten-god identities (Zi-start candidate): {} / school selection none / rule eligible false",
                                            zi_start.compact_zh()
                                        ));
                                    }
                                    if let Some(dual) = &birth_audit.late_zi_dual_candidates {
                                        ui.monospace(format!(
                                            "Late-Zi parallel candidates: midnight {} {} / Zi-start {} {} / selected none / {}",
                                            dual.midnight_boundary.day_pillar.zh_name(),
                                            dual.midnight_boundary.hour_pillar.zh_name(),
                                            dual.zi_start_boundary.day_pillar.zh_name(),
                                            dual.zi_start_boundary.hour_pillar.zh_name(),
                                            dual.status
                                        ));
                                    }
                                    if let Some(lunar) = &birth_audit.lunisolar_date_candidate {
                                        ui.monospace(format!(
                                            "Birth lunisolar candidate: nominal {} / {}{}月{}日 / safe {} / modern audit only",
                                            lunar.nominal_lunar_year,
                                            if lunar.is_leap_month { "闰" } else { "" },
                                            lunar.month_number,
                                            lunar.day_number,
                                            lunar.date_assignment_safe
                                        ));
                                    }
                                    if let Some(ziwei) = &birth_audit.ziwei_life_body_audit {
                                        ui.monospace(format!(
                                            "Ziwei life/body: BJT {} / true solar {} / identical {} / selected none / rule eligible false",
                                            ziwei.beijing_time_candidate.chart.compact_zh(),
                                            ziwei.true_solar_time_candidate.chart.compact_zh(),
                                            ziwei.candidates_identical
                                        ));
                                        ui.monospace(format!(
                                            "Ziwei status: {} / {}",
                                            ziwei.status, ziwei.algorithm_version
                                        ));
                                        for year in &ziwei.year_stem_convention_candidates {
                                            ui.monospace(format!(
                                                "Ziwei bureau [{} {} / safe {}]: BJT {} / true solar {} / selected year none",
                                                year.year_convention,
                                                year.year_designation.zh_name(),
                                                year.year_boundary_safe,
                                                year.beijing_time_candidate.compact_zh(),
                                                year.true_solar_time_candidate.compact_zh()
                                            ));
                                        }
                                        for candidate in &ziwei.star_anchor_candidates {
                                            ui.monospace(format!(
                                                "Ziwei star anchor [{} / {}]: {} / rule eligible false",
                                                candidate.year_convention,
                                                candidate.hour_convention,
                                                candidate.anchor.compact_zh()
                                            ));
                                            if let Some(stars) = &candidate.major_stars {
                                                ui.monospace(format!(
                                                    "  Ziwei group: {}",
                                                    stars.group_compact_zh("ziwei_reverse")
                                                ));
                                                ui.monospace(format!(
                                                    "  Tianfu group: {} / rule eligible false",
                                                    stars.group_compact_zh("tianfu_forward")
                                                ));
                                            }
                                            if let Some(stars) = &candidate.auxiliary_stars {
                                                ui.monospace(format!(
                                                    "  Auxiliary stars: {} / rule eligible false",
                                                    stars.compact_zh()
                                                ));
                                            }
                                        }
                                    } else if let Some(error) =
                                        &birth_audit.ziwei_life_body_audit_error
                                    {
                                        ui.monospace(format!("Ziwei life/body skipped: {error}"));
                                    }
                                    if let Some(qizheng) =
                                        &birth_audit.qizheng_siyu_identity_audit
                                    {
                                        ui.monospace(format!(
                                            "Qizheng Siyu identities: {}",
                                            qizheng_siyu_identity_compact_zh(qizheng)
                                        ));
                                        ui.monospace(format!(
                                            "Qizheng Siyu status: {} / complete positions false / natal chart false / rule eligible false",
                                            qizheng.status
                                        ));
                                        for line in qizheng_planetary_research_lines(birth_audit) {
                                            ui.monospace(line);
                                        }
                                    } else {
                                        ui.monospace(
                                            "Qizheng Siyu identities: legacy record / unavailable",
                                        );
                                    }
                                }
                                ui.monospace(format!("UT1 basis: {}", snapshot.ut1_basis));
                            } else {
                                ui.monospace("Astronomy snapshot: legacy record / unavailable");
                            }
                        });
                    }
                });
        });

        if let Some(question_id) = requested_selection {
            self.select_history(question_id);
        } else if run_audit {
            self.audit_selected_history();
        }
    }

    fn render_casting(&mut self, ui: &mut egui::Ui) {
        section_title(
            ui,
            "三种独立起法",
            "只生成并保存可复现的卦爻结构；未经复核的断法仍不执行。",
            "筮",
        );
        let Some(question_id) = self.current_question.as_ref().map(|question| question.id) else {
            ui.add_space(18.0);
            ui.label("请先锁定一个占问。");
            return;
        };

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.casting_tab, CastingTab::Liuyao, "六爻 · 三枚铜钱");
            ui.selectable_value(&mut self.casting_tab, CastingTab::Yarrow, "蓍草 · 研究重建");
            ui.selectable_value(&mut self.casting_tab, CastingTab::Meihua, "梅花 · 时间起卦");
        });
        if let Some(message) = &self.casting_message {
            ui.add_space(10.0);
            warning_box(ui, message, Color32::from_rgb(150, 92, 34));
        }
        ui.add_space(14.0);

        match self.casting_tab {
            CastingTab::Liuyao => self.render_liuyao_casting(ui, question_id),
            CastingTab::Yarrow => self.render_yarrow_casting(ui, question_id),
            CastingTab::Meihua => self.render_meihua_casting(ui, question_id),
        }
    }

    fn render_liuyao_casting(&mut self, ui: &mut egui::Ui, question_id: Uuid) {
        ui.heading("三枚铜钱，连续六次");
        ui.label("每次按住按钮片刻再松开。高精度时间、按压时长、操作间隔、指针位置与按压期间轨迹会组成该爻的版本化随机材料。六爻按初爻到上爻保存。");
        ui.add_space(12.0);

        egui::Grid::new("liuyao_lines")
            .num_columns(4)
            .striped(true)
            .spacing([18.0, 7.0])
            .show(ui, |ui| {
                ui.strong("次序");
                ui.strong("三币");
                ui.strong("爻值");
                ui.strong("结构");
                ui.end_row();
                for line in &self.liuyao_lines {
                    ui.label(format!("第 {} 爻", line.line_index_bottom_up + 1));
                    ui.label(
                        line.coins
                            .iter()
                            .map(|coin| coin.zh_name())
                            .collect::<Vec<_>>()
                            .join(" · "),
                    );
                    ui.monospace(format!("{} · {}", line.value, line.line_type.zh_name()));
                    ui.monospace(line_symbol(line.line_type));
                    ui.end_row();
                }
            });

        ui.add_space(14.0);
        if self.liuyao_cast.is_none() {
            let next = self.liuyao_lines.len() + 1;
            let response = ui.add_sized(
                [270.0, 48.0],
                egui::Button::new(
                    RichText::new(format!("按住并松开 · 第 {next} 次"))
                        .strong()
                        .size(17.0),
                )
                .fill(Color32::from_rgb(154, 67, 55)),
            );

            let now = Instant::now();
            if response.is_pointer_button_down_on() {
                let press_started = *self.liuyao_press_started.get_or_insert(now);
                let position = ui
                    .ctx()
                    .pointer_latest_pos()
                    .unwrap_or(response.rect.center());
                let sample = PointerSample {
                    elapsed_nanos: duration_nanos(now.duration_since(press_started)),
                    x_micropoints: point_to_micro(position.x),
                    y_micropoints: point_to_micro(position.y),
                };
                let should_store = self.liuyao_pointer_path.last().is_none_or(|previous| {
                    previous.x_micropoints != sample.x_micropoints
                        || previous.y_micropoints != sample.y_micropoints
                        || sample.elapsed_nanos.saturating_sub(previous.elapsed_nanos) >= 1_000_000
                });
                if should_store && self.liuyao_pointer_path.len() < 4_096 {
                    self.liuyao_pointer_path.push(sample);
                }
                ui.ctx().request_repaint();
            }

            if response.clicked() {
                let press_started = self.liuyao_press_started.take().unwrap_or(now);
                let session_started = *self.liuyao_session_started.get_or_insert(now);
                let position = response
                    .interact_pointer_pos()
                    .or_else(|| ui.ctx().pointer_latest_pos())
                    .unwrap_or(response.rect.center());
                let event = EntropyEvent {
                    sequence: self.liuyao_lines.len() as u8,
                    wall_clock_unix_nanos: Utc::now()
                        .timestamp_nanos_opt()
                        .unwrap_or_else(|| Utc::now().timestamp_micros() * 1_000),
                    monotonic_nanos: duration_nanos(now.duration_since(session_started)),
                    press_duration_nanos: duration_nanos(now.duration_since(press_started)),
                    delta_since_previous_nanos: self
                        .liuyao_previous_event
                        .map(|previous| duration_nanos(now.duration_since(previous)))
                        .unwrap_or(0),
                    click_x_micropoints: point_to_micro(position.x),
                    click_y_micropoints: point_to_micro(position.y),
                    pointer_path: std::mem::take(&mut self.liuyao_pointer_path),
                };
                self.liuyao_previous_event = Some(now);

                let line_result = self
                    .liuyao_transcript
                    .as_mut()
                    .ok_or_else(|| "六爻起法会话尚未初始化".to_owned())
                    .and_then(|transcript| {
                        transcript
                            .push_event(event)
                            .map_err(|error| error.to_string())
                    });
                match line_result {
                    Ok(line) => {
                        self.liuyao_lines.push(line);
                        self.casting_message = None;
                    }
                    Err(error) => self.casting_message = Some(error),
                }

                let completed = self
                    .liuyao_transcript
                    .as_ref()
                    .is_some_and(LiuyaoTranscript::is_complete);
                if completed {
                    let transcript = self.liuyao_transcript.as_ref().expect("checked").clone();
                    match transcript.finish() {
                        Ok(cast) => {
                            match self
                                .storage
                                .save_liuyao_artifact(question_id, &transcript, &cast)
                            {
                                Ok(()) => {
                                    self.casting_message = Some(
                                        "六次三币起法已锁定、写入本地数据库并通过重放校验。"
                                            .to_owned(),
                                    );
                                    self.liuyao_cast = Some(cast);
                                    self.refresh_history();
                                }
                                Err(error) => self.casting_message = Some(error.to_string()),
                            }
                        }
                        Err(error) => self.casting_message = Some(error.to_string()),
                    }
                }
            }
        } else if let Some(cast) = &self.liuyao_cast {
            ui.group(|ui| {
                ui.label(RichText::new("六爻起法已完成并锁定").strong());
                render_hexagram_pair(
                    ui,
                    cast.primary_bits,
                    cast.changed_bits,
                    self.developer_mode,
                );
                match load_liuyao_six_spirits_audit(&self.storage, question_id, cast.primary_bits) {
                    Ok(Some(audit)) => {
                        render_liuyao_six_spirits_audit(ui, &audit, self.developer_mode)
                    }
                    Ok(None) => {
                        if let Ok(audit) = liuyao_six_relatives_audit(cast.primary_bits) {
                            render_liuyao_six_relatives_audit(ui, &audit, self.developer_mode);
                        }
                    }
                    Err(error) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error);
                    }
                }
                if let Ok(audit) = liuyao_hidden_spirits_audit(cast.primary_bits) {
                    render_liuyao_hidden_spirits_audit(ui, &audit, self.developer_mode);
                }
                match load_liuyao_sanhe_audit(&self.storage, question_id, cast) {
                    Ok(Some(audit)) => {
                        let fanyin_fuyin = &audit.fanyin_fuyin;
                        render_liuyao_calendar_moving_audit(
                            ui,
                            &fanyin_fuyin
                                .progress_retreat
                                .branch_pair_relations
                                .calendar_relations
                                .calendar_moving,
                            self.developer_mode,
                        );
                        render_liuyao_calendar_relations_audit(
                            ui,
                            &fanyin_fuyin
                                .progress_retreat
                                .branch_pair_relations
                                .calendar_relations,
                            self.developer_mode,
                        );
                        render_liuyao_branch_pair_relations_audit(
                            ui,
                            &fanyin_fuyin.progress_retreat.branch_pair_relations,
                            self.developer_mode,
                        );
                        render_liuyao_progress_retreat_audit(
                            ui,
                            &fanyin_fuyin.progress_retreat,
                            self.developer_mode,
                        );
                        render_liuyao_fanyin_fuyin_audit(ui, fanyin_fuyin, self.developer_mode);
                        render_liuyao_sanhe_audit(ui, &audit, self.developer_mode);
                    }
                    Ok(None) => {
                        ui.small("旧记录缺少可核验月建或日辰快照；不以当前时间补算。");
                    }
                    Err(error) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error);
                    }
                };
                ui.monospace(format!("转录哈希：{}", cast.transcript_hash));
                ui.monospace(format!("主种子：{}", cast.master_seed_hex));
            });
        }
        ui.add_space(10.0);
        ui.label(
            RichText::new("操作一旦开始不可重摇；这是同一锁定占问的唯一六爻随机材料。")
                .small()
                .color(Color32::from_rgb(150, 92, 34)),
        );
    }

    fn render_yarrow_casting(&mut self, ui: &mut egui::Ui, question_id: Uuid) {
        ui.heading("大衍五十，其用四十九；三变成一爻，十八变成一卦");
        warning_box(
            ui,
            "本页是唐代《周易正义》程序解释的可复现研究重建，不是已通过独立复核的正式古法规则；只保存卦爻结构，绝不参与断语、应期或综合裁决。",
            Color32::from_rgb(150, 92, 34),
        );
        ui.label(format!("程序状态：{YARROW_PROCEDURE_STATUS_ZH}。每一变依次参与五个操作，十八次分堆数量及程序约定会共同固化并用于重放。"));
        ui.add_space(12.0);
        let displayed_change = if self.yarrow_session.is_complete() {
            18
        } else {
            self.yarrow_session.completed_changes() + 1
        };
        ui.label(format!(
            "进度：第 {} / 18 变 · 已成 {} / 6 爻 · 当前参与运算 {} 策",
            displayed_change,
            self.yarrow_session.completed_lines().len(),
            self.yarrow_session.active_stalks()
        ));

        ui.add_space(10.0);
        egui::Grid::new("yarrow_lines")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                ui.strong("次序");
                ui.strong("三变余策");
                ui.strong("爻");
                ui.end_row();
                for line in self.yarrow_session.completed_lines() {
                    ui.label(format!("第 {} 爻", line.line_index_bottom_up + 1));
                    ui.monospace(format!("{} 策 / {}", line.remaining_stalks, line.value));
                    ui.monospace(format!(
                        "{}  {}",
                        line_symbol(line.line_type),
                        line.line_type.zh_name()
                    ));
                    ui.end_row();
                }
            });

        ui.add_space(12.0);
        if self.yarrow_cast.is_none() {
            let minimum_left = self.yarrow_session.minimum_left_pile();
            let maximum_left = self.yarrow_session.maximum_left_pile();
            self.yarrow_left_pile = self.yarrow_left_pile.clamp(minimum_left, maximum_left);
            match self.yarrow_phase {
                YarrowUiPhase::Divide => {
                    ui.label(
                        "一、分二：移动分界，确定左堆策数。左堆至少保留两策，以便从代表天的左堆挂一后仍可揲四。",
                    );
                    ui.add(
                        egui::Slider::new(&mut self.yarrow_left_pile, minimum_left..=maximum_left)
                            .text("左堆策数"),
                    );
                    ui.label(format!(
                        "左 {} 策 · 右 {} 策",
                        self.yarrow_left_pile,
                        self.yarrow_session.active_stalks() - self.yarrow_left_pile
                    ));
                    if ui.button("确认分二").clicked() {
                        self.yarrow_pending_left = Some(self.yarrow_left_pile);
                        self.yarrow_phase = YarrowUiPhase::HangOne;
                    }
                }
                YarrowUiPhase::HangOne => {
                    ui.label("二、挂一：按唐疏‘天于左手，于天数之中分挂其一’，从左堆取一策另置。");
                    if ui.button("挂一").clicked() {
                        self.yarrow_phase = YarrowUiPhase::CountLeft;
                    }
                }
                YarrowUiPhase::CountLeft => {
                    ui.label("三、揲四：先以四策为组数挂一后的左堆，余数为零时按四策计。");
                    if ui.button("左堆揲四").clicked() {
                        self.yarrow_phase = YarrowUiPhase::CountRight;
                    }
                }
                YarrowUiPhase::CountRight => {
                    ui.label("四、揲四：再以四策为组数右堆，余数为零时按四策计。");
                    if ui.button("右堆揲四").clicked() {
                        self.yarrow_phase = YarrowUiPhase::Gather;
                    }
                }
                YarrowUiPhase::Gather => {
                    ui.label("五、归奇：将挂一、左余、右余合并移出，完成本变。");
                    if ui.button("归奇并锁定本变").clicked() {
                        let left = self
                            .yarrow_pending_left
                            .expect("divide phase records split");
                        match self.yarrow_session.apply_split(left) {
                            Ok(step) => {
                                self.yarrow_last_change = Some(step.change);
                                self.yarrow_phase = YarrowUiPhase::Divide;
                                self.yarrow_pending_left = None;
                                self.yarrow_left_pile = self.yarrow_session.active_stalks() / 2;
                                self.casting_message = None;
                            }
                            Err(error) => self.casting_message = Some(error.to_string()),
                        }

                        if self.yarrow_session.is_complete() {
                            let transcript = self.yarrow_session.transcript().clone();
                            match self.yarrow_session.finish() {
                                Ok(cast) => {
                                    match self.storage.save_yarrow_artifact(
                                        question_id,
                                        &transcript,
                                        &cast,
                                    ) {
                                        Ok(()) => {
                                            self.casting_message = Some(
                                                "蓍草十八变已锁定、写入本地数据库并通过重放校验。"
                                                    .to_owned(),
                                            );
                                            self.yarrow_cast = Some(cast);
                                            self.refresh_history();
                                        }
                                        Err(error) => {
                                            self.casting_message = Some(error.to_string())
                                        }
                                    }
                                }
                                Err(error) => self.casting_message = Some(error.to_string()),
                            }
                        }
                    }
                }
            }
        }

        if let Some(change) = &self.yarrow_last_change {
            ui.add_space(12.0);
            ui.group(|ui| {
                ui.label(RichText::new("最近一变的数量轨迹").strong());
                ui.monospace(format!(
                    "{} → 分二 {}|{} → {}挂一 1 → 左余 {} + 右余 {} → 归奇移出 {} → {}",
                    change.stalks_before,
                    change.left_pile,
                    change.right_pile,
                    change.hanging_side.zh_name(),
                    change.left_remainder,
                    change.right_remainder,
                    change.removed_stalks,
                    change.stalks_after
                ));
            });
        }
        if let Some(cast) = &self.yarrow_cast {
            ui.add_space(12.0);
            ui.group(|ui| {
                ui.label(RichText::new("蓍草十八变已完成并锁定").strong());
                render_hexagram_pair(
                    ui,
                    cast.primary_bits,
                    cast.changed_bits,
                    self.developer_mode,
                );
                ui.monospace(format!("算法版本：{}", cast.algorithm_version));
                ui.monospace(format!(
                    "程序约定：{}",
                    self.yarrow_session.transcript().procedure_convention_id
                ));
                ui.monospace(format!("十八变转录哈希：{}", cast.transcript_hash));
            });
        }
    }

    fn render_meihua_casting(&mut self, ui: &mut egui::Ui, question_id: Uuid) {
        ui.heading("年月日时机械起卦候选");
        warning_box(
            ui,
            "只执行《梅花易数》传本文字中的取数、除八和除六公式；开发者模式另显示本卦与变卦的原始中爻经卦。闰月编号、时辰基准、乾坤无互例句、底本题署与断法尚未闭合，因此不选择互卦或时辰解释口径，也不输出体用、吉凶或应期。",
            Color32::from_rgb(150, 92, 34),
        );
        ui.add_space(12.0);
        match self.storage.load_astronomy_snapshot(question_id) {
            Ok(Some(snapshot)) => match snapshot.meihua_time_casting_audit {
                Some(audit) => {
                    let lunar = &audit.lunisolar_date_candidate;
                    ui.label(format!(
                        "取数日期：名义年 {}（{}年数 {}）· {}{}月 · {}日",
                        lunar.nominal_lunar_year,
                        audit.year_branch.zh_name(),
                        audit.year_branch.index() + 1,
                        if lunar.is_leap_month { "闰" } else { "" },
                        lunar.month_number,
                        lunar.day_number
                    ));
                    ui.label(format!(
                        "两种时辰结果：{}；选择状态：未选择",
                        if audit.candidates_identical {
                            "当前相同"
                        } else {
                            "当前不同"
                        }
                    ));
                    ui.add_space(10.0);
                    egui::Grid::new("meihua_time_candidates")
                        .num_columns(3)
                        .striped(true)
                        .spacing([18.0, 8.0])
                        .show(ui, |ui| {
                            ui.strong("时辰口径");
                            ui.strong("取数");
                            ui.strong("机械卦象");
                            ui.end_row();
                            for (label, candidate) in [
                                ("北京时间", &audit.beijing_time_candidate),
                                ("真太阳时", &audit.true_solar_time_candidate),
                            ] {
                                ui.label(label);
                                match candidate {
                                    Some(candidate) => {
                                        ui.monospace(format!(
                                            "{}时={} / {}",
                                            candidate.shichen.branch.zh_name(),
                                            candidate.cast.input.hour_branch_number,
                                            if candidate.mechanical_boundaries_safe {
                                                "边界安全"
                                            } else {
                                                "边界不安全"
                                            }
                                        ));
                                        ui.vertical(|ui| {
                                            ui.monospace(candidate.cast.compact_zh());
                                            render_hexagram_pair(
                                                ui,
                                                candidate.cast.primary_bits,
                                                candidate.cast.changed_bits,
                                                self.developer_mode,
                                            );
                                            if self.developer_mode {
                                                match candidate.cast.mutual_raw_audit() {
                                                    Ok(raw) => ui.monospace(raw.compact_zh()),
                                                    Err(error) => ui.monospace(format!(
                                                        "中爻原始结构不可重算：{error}"
                                                    )),
                                                };
                                            }
                                        });
                                    }
                                    None => {
                                        ui.monospace("—");
                                        ui.monospace("闰月口径未定义，失败关闭");
                                    }
                                }
                                ui.end_row();
                            }
                        });
                    ui.add_space(10.0);
                    ui.monospace(format!(
                        "算法：{} / 状态：{} / 规则准入：否",
                        audit.algorithm_version, audit.status
                    ));
                    ui.monospace(format!("阻断原因：{}", audit.blocked_reasons.join("；")));
                }
                None => {
                    ui.label("当前历法基础未形成可保存的梅花时间起卦审计。");
                }
            },
            Ok(None) => {
                ui.label("当前占问没有可读取的不可变历法快照。");
            }
            Err(error) => {
                ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string());
            }
        }
    }

    fn render_result(&mut self, ui: &mut egui::Ui) {
        section_title(
            ui,
            "本次结果",
            "结果页只呈现可由已验证古籍规则支持的内容。",
            "果",
        );
        ui.add_space(24.0);
        if let Some(question) = &self.current_question {
            ui.label(RichText::new(&question.text).size(19.0));
            ui.add_space(18.0);
            egui::Frame::new()
                .fill(Color32::from_rgb(247, 244, 236))
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("结论：当前规则库尚未达到可推演条件。")
                            .size(24.0)
                            .strong()
                            .color(Color32::from_rgb(43, 61, 53)),
                    );
                    ui.add_space(12.0);
                    ui.label(RichText::new("时间：暂无法断定").size(18.0));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("当下建议：暂无可靠古籍行动建议。").size(18.0),
                    );
                    ui.label("规则尚未通过证据门禁时，系统不会根据吉凶分数或常见套话编造行动建议。");
                    ui.add_space(8.0);
                    let ancient_type = self
                        .current_classification
                        .as_ref()
                        .map(|classification| &classification.ancient_type);
                    ui.label(RichText::new(format!(
                        "现实行动底线：{}",
                        practical_action_notice_for_classified_question(
                            question.category,
                            ancient_type,
                        )
                    )).size(16.0));
                    if classified_question_requires_health_notice(question.category, ancient_type) {
                        ui.colored_label(
                            Color32::from_rgb(150, 92, 34),
                            "仅属传统术数文化推演，不能用于诊断或治疗决策。",
                        );
                    }
                    ui.label(RichText::new("古法依据等级：未评定").size(18.0));
                    ui.add_space(16.0);
                    ui.label("九术接口已完成适用性检查，但古籍规则尚未完成版本核对与独立复核，因此没有术式被允许进入裁决。这是证据门禁的预期行为。");
                });
            ui.add_space(16.0);
            if ui.button("进入三种独立起法").clicked() {
                self.page = Page::Casting;
            }
            if self.developer_mode && ui.button("查看九术校验详情").clicked() {
                self.page = Page::Developer;
            }
        } else {
            ui.label("尚未锁定占问。");
            if ui.button("返回新建占问").clicked() {
                self.page = Page::NewQuestion;
            }
        }
    }

    fn render_developer(&mut self, ui: &mut egui::Ui) {
        section_title(
            ui,
            "开发者校验模式",
            "所有中间数据用于复现与审计，不改变普通结果。",
            "验",
        );
        ui.add_space(16.0);

        ui.label(RichText::new("古籍规则发布门禁").size(18.0).strong());
        match self.storage.rule_evidence_summary() {
            Ok(summary) => {
                egui::Grid::new("rule_evidence_gate_summary")
                    .num_columns(4)
                    .striped(true)
                    .spacing([18.0, 8.0])
                    .show(ui, |ui| {
                        ui.strong("正式规则");
                        ui.monospace(summary.total_rules.to_string());
                        ui.strong("允许执行");
                        ui.monospace(summary.executable_rules.to_string());
                        ui.end_row();
                        ui.label("版本书目");
                        ui.monospace(summary.source_editions.to_string());
                        ui.label("影印字节验证");
                        ui.monospace(summary.facsimile_verifications.to_string());
                        ui.end_row();
                        ui.label("原文片段");
                        ui.monospace(summary.source_passages.to_string());
                        ui.label("独立复核记录");
                        ui.monospace(summary.reviews.to_string());
                        ui.end_row();
                        ui.label("启用测试用例");
                        ui.monospace(summary.enabled_test_cases.to_string());
                        ui.end_row();
                        ui.label("完整测试运行");
                        ui.monospace(summary.test_runs.to_string());
                        ui.label("被门禁阻止");
                        ui.monospace(
                            summary
                                .total_rules
                                .saturating_sub(summary.executable_rules)
                                .to_string(),
                        );
                        ui.end_row();
                    });
                ui.add_space(10.0);
                if summary.executable_rules == 0 {
                    warning_box(
                        ui,
                        "当前没有任何规则同时通过影印文件字节验证、主版本原文、四类复核和当前测试套件；裁决引擎保持关闭。",
                        Color32::from_rgb(150, 92, 34),
                    );
                }
            }
            Err(error) => warning_box(
                ui,
                &format!("规则门禁状态读取失败：{error}"),
                Color32::from_rgb(167, 54, 54),
            ),
        }

        ui.add_space(12.0);
        ui.group(|ui| {
            ui.strong("本地影印文件字节验证");
            ui.small("只读取本地文件，流式计算 SHA-256；文件内容不会写入数据库。验证成功后保存文件名、字节数、校验和、算法版本和时间。\n登记校验和不匹配、空文件或未知 source_id 均不会生成验证记录。");
            ui.add_space(6.0);
            egui::Grid::new("facsimile_file_verification_input")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("source_id");
                    ui.text_edit_singleline(&mut self.facsimile_source_id);
                    ui.end_row();
                    ui.label("本地文件路径");
                    ui.text_edit_singleline(&mut self.facsimile_file_path);
                    ui.end_row();
                });
            if ui.button("读取文件并核验登记 SHA-256").clicked() {
                let source_id = self.facsimile_source_id.trim();
                let path = self.facsimile_file_path.trim();
                if source_id.is_empty() || path.is_empty() {
                    self.facsimile_verification_message =
                        Some((false, "source_id 和本地文件路径均不能为空。".to_owned()));
                } else {
                    self.facsimile_verification_message = Some(
                        match self
                            .storage
                            .verify_source_facsimile_file(source_id, &PathBuf::from(path))
                        {
                            Ok(record) => (
                                true,
                                format!(
                                    "验证成功：{} 字节；{}；{}",
                                    record.byte_length,
                                    record.digital_checksum,
                                    record.verifier_version
                                ),
                            ),
                            Err(error) => (false, format!("验证失败：{error}")),
                        },
                    );
                }
            }
            if let Some((success, message)) = &self.facsimile_verification_message {
                ui.colored_label(
                    if *success {
                        Color32::from_rgb(45, 112, 82)
                    } else {
                        Color32::from_rgb(167, 54, 54)
                    },
                    message,
                );
            }
        });

        match self.storage.list_rule_gate_reports() {
            Ok(items) if items.is_empty() => {
                ui.label("正式规则清单为空；测试夹具只存在于内存测试，不会导入产品数据库。");
            }
            Ok(items) => {
                ui.collapsing(
                    format!("逐规则门禁详情（{} 条）", items.len()),
                    |ui| {
                        for item in items {
                            let state = if item.report.may_execute() {
                                "允许执行"
                            } else {
                                "已阻止"
                            };
                            let dossier = self.storage.rule_evidence_dossier(&item.rule_id);
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.strong(item.method.zh_name());
                                    ui.monospace(&item.rule_id);
                                    ui.label(state);
                                });
                                ui.label(format!("《{}》 {}", item.book_title, item.chapter));
                                match (
                                    item.action_key.as_deref(),
                                    item.action_polarity,
                                    item.action_expression.as_deref(),
                                ) {
                                    (Some(key), Some(polarity), Some(expression)) => {
                                        ui.label(format!(
                                            "行动建议：{} / {} / {}",
                                            key,
                                            polarity.zh_name(),
                                            expression
                                        ));
                                    }
                                    _ => {
                                        ui.small("行动建议：本规则未提供");
                                    }
                                }
                                ui.monospace(format!(
                                    "rule_hash: {}",
                                    item.report.rule_content_hash
                                ));
                                if !item.report.blocking_reasons.is_empty() {
                                    ui.label(item.report.blocking_reasons.join("；"));
                                }
                                let passed_checks = item
                                    .report
                                    .checks
                                    .iter()
                                    .filter(|check| check.passed)
                                    .count();
                                ui.collapsing(
                                    format!(
                                        "逐项证据检查（{passed_checks}/{}）",
                                        item.report.checks.len()
                                    ),
                                    |ui| {
                                        egui::Grid::new(format!(
                                            "rule_gate_checks_{}",
                                            item.rule_id
                                        ))
                                        .num_columns(3)
                                        .striped(true)
                                        .spacing([12.0, 6.0])
                                        .show(ui, |ui| {
                                            for check in &item.report.checks {
                                                ui.colored_label(
                                                    if check.passed {
                                                        Color32::from_rgb(45, 112, 82)
                                                    } else {
                                                        Color32::from_rgb(167, 54, 54)
                                                    },
                                                    if check.passed { "通过" } else { "缺失" },
                                                );
                                                ui.label(&check.label);
                                                ui.label(&check.detail);
                                                ui.end_row();
                                            }
                                        });
                                    },
                                );
                                ui.collapsing("完整证据档案（只读）", |ui| match &dossier
                                {
                                    Ok(Some(dossier)) => render_rule_evidence_dossier(ui, dossier),
                                    Ok(None) => {
                                        ui.colored_label(
                                            Color32::from_rgb(167, 54, 54),
                                            "规则已从证据档案查询中消失。",
                                        );
                                    }
                                    Err(error) => {
                                        ui.colored_label(
                                            Color32::from_rgb(167, 54, 54),
                                            format!("证据档案读取失败：{error}"),
                                        );
                                    }
                                });
                            });
                            ui.add_space(6.0);
                        }
                    },
                );
            }
            Err(error) => warning_box(
                ui,
                &format!("逐规则门禁详情读取失败：{error}"),
                Color32::from_rgb(167, 54, 54),
            ),
        }

        ui.add_space(18.0);
        ui.label(RichText::new("确定性规则程序门禁").size(18.0).strong());
        egui::Grid::new("rule_program_gate_summary")
            .num_columns(2)
            .striped(true)
            .spacing([18.0, 8.0])
            .show(ui, |ui| {
                ui.label("条件程序");
                ui.monospace(format!(
                    "JSON v{} / JSON Pointer / all-any-not-eq-in-整数比较",
                    divination_core::RULE_CONDITION_SCHEMA_VERSION
                ));
                ui.end_row();
                ui.label("结果程序");
                ui.monospace(format!(
                    "JSON v{} / 仅标准化中间信号 -100..100",
                    divination_core::RULE_RESULT_SCHEMA_VERSION
                ));
                ui.end_row();
                ui.label("应期程序");
                ui.monospace(format!(
                    "JSON v{} / 仅固定 TimingWindow 枚举",
                    divination_core::RULE_TIMING_SCHEMA_VERSION
                ));
                ui.end_row();
                ui.label("运行输入");
                ui.monospace(format!(
                    "JSON v{} / 锁定问题 + 古代类型 + 术式 + 排盘快照",
                    divination_core::RULE_RUNTIME_INPUT_SCHEMA_VERSION
                ));
                ui.end_row();
                ui.label("术内合并");
                ui.monospace(divination_core::RULE_METHOD_AGGREGATION_VERSION);
                ui.end_row();
                ui.label("执行边界");
                ui.monospace("不执行脚本 / 不解析自然语言 / 冲突字段不平均 / 类型错误即失败关闭");
                ui.end_row();
            });
        warning_box(
            ui,
            "自由文本条件和结果不能进入执行层；规则程序即使语法有效，也仍须通过影印原文、四类复核和当前测试套件。融合前会重放每个输入、输出和轨迹，保存信号不能脱离 Rule ID 单独生效。",
            Color32::from_rgb(150, 92, 34),
        );

        ui.add_space(18.0);
        ui.label(RichText::new("中间信号与综合裁决门禁").size(18.0).strong());
        egui::Grid::new("fusion_gate_summary")
            .num_columns(2)
            .striped(true)
            .spacing([18.0, 8.0])
            .show(ui, |ui| {
                ui.label("融合算法");
                ui.monospace(divination_core::FUSION_ALGORITHM_VERSION);
                ui.end_row();
                ui.label("信号结构版本");
                ui.monospace(divination_core::INTERMEDIATE_SIGNAL_SCHEMA_VERSION.to_string());
                ui.end_row();
                ui.label("术式权重");
                ui.monospace("全部适用且放行的术式恒为 1/N");
                ui.end_row();
                ui.label("古籍等级用途");
                ui.monospace("只在同权僵局时破局；平时不加权");
                ui.end_row();
                ui.label("行动建议策略");
                ui.monospace("仅取命中 Rule ID；同一行动方向冲突时整项隐藏");
                ui.end_row();
            });
        warning_box(
            ui,
            "融合器不会直接读取测试夹具。当前正式规则包为空，因此产品运行时仍拒绝生成结论与古法行动建议。",
            Color32::from_rgb(150, 92, 34),
        );

        ui.add_space(18.0);
        ui.label(RichText::new("本次占问审计").size(18.0).strong());
        let Some(question) = &self.current_question else {
            ui.label("尚未锁定占问；规则门禁总览仍可独立审计。");
            return;
        };
        let parameter_validation = validate_question_parameters(question);
        let first_rain_scope = assess_first_rain_scope(question);
        let chinese_astrology_structure = chinese_astrology_structure_audit();
        egui::Grid::new("developer_meta")
            .num_columns(2)
            .striped(true)
            .spacing([18.0, 8.0])
            .show(ui, |ui| {
                ui.label("Question ID");
                ui.monospace(question.id.to_string());
                ui.end_row();
                ui.label("事件指纹");
                ui.monospace(
                    self.current_fingerprint
                        .as_ref()
                        .map(|fp| fp.digest_hex.as_str())
                        .unwrap_or("—"),
                );
                ui.end_row();
                ui.label("古代问题类型");
                ui.monospace(
                    self.current_classification
                        .as_ref()
                        .map(|item| ancient_type_label(&item.ancient_type))
                        .unwrap_or("—"),
                );
                ui.end_row();
                ui.label("参数校验");
                ui.monospace(format!(
                    "{} / {} / 警告 {} 项",
                    parameter_validation.validator_version,
                    if parameter_validation.can_lock {
                        "通过"
                    } else {
                        "失败"
                    },
                    parameter_validation.warnings.len()
                ));
                ui.end_row();
                if question.category == QuestionCategory::Weather {
                    ui.label("范围算法版本");
                    ui.monospace(&first_rain_scope.algorithm_version);
                    ui.end_row();
                    ui.label("首条降雨范围");
                    ui.monospace(if first_rain_scope.in_research_scope {
                        "参数符合研究范围；正式规则未放行"
                    } else {
                        "参数不符合首条降雨范围"
                    });
                    ui.end_row();
                    ui.label("目标时段换算");
                    ui.monospace(match &first_rain_scope.target_window {
                        Some(window) => format!(
                            "{} → {}（北京时间）",
                            window.start_bjt.to_rfc3339(),
                            window.end_bjt.to_rfc3339()
                        ),
                        None => "未能保守换算".to_owned(),
                    });
                    ui.end_row();
                }
                ui.label("出生参数");
                match &question.birth {
                    Some(birth) => ui.monospace(format!(
                        "{} / {} / {}",
                        biological_sex_label(birth.sex),
                        birth.date_time_bjt.to_rfc3339(),
                        location_label(&birth.location)
                    )),
                    None => ui.monospace("未提供；需要出生参数的术式自动跳过"),
                };
                ui.end_row();
                ui.label("数据库");
                ui.monospace(self.storage_path.display().to_string());
                ui.end_row();
                ui.label("占问记录数");
                ui.monospace(self.storage.question_count().unwrap_or(0).to_string());
                ui.end_row();
            });

        if question.category == QuestionCategory::Weather {
            for reason in &first_rain_scope.reasons {
                ui.colored_label(Color32::from_rgb(150, 92, 34), reason);
            }
        }

        ui.add_space(18.0);
        ui.label(RichText::new("中国古代星占结构目录").size(18.0).strong());
        egui::Grid::new("chinese_astrology_structure")
            .num_columns(2)
            .striped(true)
            .spacing([18.0, 8.0])
            .show(ui, |ui| {
                ui.label("三垣（传本名目）");
                ui.monospace(three_enclosure_catalog_zh(&chinese_astrology_structure));
                ui.end_row();
                for quadrant in MansionQuadrant::ALL_IN_RECEIVED_ORDER {
                    ui.label(format!(
                        "{} · {}",
                        quadrant.zh_name(),
                        quadrant.emblem().received_zh_name()
                    ));
                    ui.monospace(mansion_sequence_for_quadrant_zh(
                        &chinese_astrology_structure,
                        quadrant,
                    ));
                    ui.end_row();
                }
                ui.label("七政（日月五星身份）");
                ui.monospace(seven_luminary_catalog_zh(&chinese_astrology_structure));
                ui.end_row();
                ui.label("结构状态");
                ui.monospace(format!(
                    "{} / 三垣与宿度边界未选 / 天体位置未算 / 规则准入否",
                    chinese_astrology_structure.algorithm_version
                ));
                ui.end_row();
            });
        warning_box(
            ui,
            "这里只保存三垣、二十八宿和日月五星的传本身份；不生成垣墙/星官成员，不采用西方黄道十二宫，不计算宿度或行星位置，也不输出任何星占吉凶。",
            Color32::from_rgb(150, 92, 34),
        );

        ui.add_space(18.0);
        ui.label(
            RichText::new("出生时间、四柱与紫微机械排盘候选")
                .size(18.0)
                .strong(),
        );
        if let Some(birth) = &question.birth {
            let birth_audit = calculate_birth_time_audit_snapshot(birth);
            match birth_audit {
                Ok(audit) => {
                    egui::Grid::new("birth_time_audit")
                        .num_columns(2)
                        .striped(true)
                        .spacing([18.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("出生输入");
                            ui.monospace(format!(
                                "{} / {} / {}",
                                biological_sex_label(birth.sex),
                                birth.date_time_bjt.to_rfc3339(),
                                location_label(&birth.location)
                            ));
                            ui.end_row();
                            ui.label("JD（UTC≈UT1 / TT）");
                            ui.monospace(format!(
                                "{:.9} / {:.9}",
                                audit.jd_ut1_approx, audit.jd_tt
                            ));
                            ui.end_row();
                            ui.label("ΔT");
                            ui.monospace(format!(
                                "{:.3} ±{:.3} 秒 / {}",
                                audit.delta_t_seconds,
                                audit.delta_t_uncertainty_seconds,
                                audit.delta_t_model_version
                            ));
                            ui.end_row();
                            ui.label("出生真太阳时");
                            ui.monospace(format!(
                                "{} / 总校正 {:+.6} 分钟 / {}",
                                audit.true_solar_time_bjt,
                                audit.true_solar_total_correction_minutes,
                                if audit.true_solar_boundary_safe {
                                    "时界安全"
                                } else {
                                    "时界不安全"
                                }
                            ));
                            ui.end_row();
                            ui.label("出生历法同瞬间表示");
                            match &audit.civil_calendar_audit {
                                Some(calendar) => ui.monospace(format!(
                                    "公历 {} / 儒略历 {} / 改革历 {}（{}）",
                                    historical_datetime_label_zh(calendar.proleptic_gregorian),
                                    historical_datetime_label_zh(calendar.julian),
                                    historical_datetime_label_zh(calendar.historical_cutover),
                                    calendar_system_label(
                                        calendar.historical_cutover_resolved_system
                                    )
                                )),
                                None => ui.monospace("旧记录未保存"),
                            };
                            ui.end_row();
                            ui.label("条件日/时干支");
                            ui.monospace(format!(
                                "{} / {}时 / {}（仅机械候选）",
                                audit.sexagenary_day_true_solar_midnight.zh_name(),
                                audit.true_solar_shichen.branch.zh_name(),
                                audit.sexagenary_hour_true_solar_midnight.zh_name()
                            ));
                            ui.end_row();
                            ui.label("四柱机械候选");
                            match &audit.four_pillars_chart_candidate {
                                Some(chart) => ui.monospace(format!(
                                    "{} / {}",
                                    chart.compact_zh(),
                                    if chart.mechanical_boundaries_safe {
                                        "机械边界安全"
                                    } else {
                                        "存在不安全或未决边界"
                                    }
                                )),
                                None => ui.monospace("旧记录未保存"),
                            };
                            ui.end_row();
                            ui.label("四柱口径与门禁");
                            match &audit.four_pillars_chart_candidate {
                                Some(chart) => ui.monospace(format!(
                                    "年：立春 / 月：节 / 日：真太阳时零点 / 时：真太阳时辰 / 晚子时争议{} / 规则准入{}",
                                    if chart.late_zi_hour_day_ambiguous {
                                        "未决"
                                    } else {
                                        "当前未触发"
                                    },
                                    if chart.rule_eligible { "是" } else { "否" }
                                )),
                                None => ui.monospace("—"),
                            };
                            ui.end_row();
                            ui.label("干支阴阳五行结构");
                            match &audit.four_pillars_structural_metadata {
                                Some(metadata) => ui.monospace(metadata.compact_zh()),
                                None => ui.monospace("旧记录未保存"),
                            };
                            ui.end_row();
                            ui.label("明干十神机械身份（零点换日候选）");
                            match &audit.four_pillars_visible_stem_ten_gods {
                                Some(ten_gods) => ui.monospace(ten_gods.compact_zh()),
                                None => ui.monospace("旧记录未保存"),
                            };
                            ui.end_row();
                            ui.label("明干十神机械身份（子初换日候选）");
                            match &audit.four_pillars_visible_stem_ten_gods_zi_start {
                                Some(ten_gods) => ui.monospace(format!(
                                    "{} / 晚子时口径未选择",
                                    ten_gods.compact_zh()
                                )),
                                None if audit.audit_version < 10 => ui.monospace("旧记录未保存"),
                                None => ui.monospace("当前出生时刻未触发晚子时双候选"),
                            };
                            ui.end_row();
                            ui.label("十神范围与门禁");
                            match &audit.four_pillars_visible_stem_ten_gods {
                                Some(ten_gods) => ui.monospace(format!(
                                    "仅年/月/日/时四个表面天干；藏干{} / 旺衰{} / 格局{} / 规则准入{}",
                                    if ten_gods.hidden_stems_included { "已纳入" } else { "未纳入" },
                                    if ten_gods.strength_assessed { "已判断" } else { "未判断" },
                                    if ten_gods.pattern_selected { "已选择" } else { "未选择" },
                                    if ten_gods.rule_eligible { "是" } else { "否" }
                                )),
                                None => ui.monospace("—"),
                            };
                            ui.end_row();
                            ui.label("晚子时双候选");
                            match &audit.late_zi_dual_candidates {
                                Some(dual) => ui.monospace(format!(
                                    "零点换日：{} {} / 子初换日：{} {} / 未选择流派口径",
                                    dual.midnight_boundary.day_pillar.zh_name(),
                                    dual.midnight_boundary.hour_pillar.zh_name(),
                                    dual.zi_start_boundary.day_pillar.zh_name(),
                                    dual.zi_start_boundary.hour_pillar.zh_name()
                                )),
                                None => ui.monospace("当前出生时刻未触发晚子时双候选"),
                            };
                            ui.end_row();
                            ui.label("出生现代农历候选");
                            match &audit.lunisolar_date_candidate {
                                Some(lunar) => ui.monospace(format!(
                                    "名义年 {} / {}{}月 {}日 / {}（仅现代口径审计）",
                                    lunar.nominal_lunar_year,
                                    if lunar.is_leap_month { "闰" } else { "" },
                                    lunar.month_number,
                                    lunar.day_number,
                                    if lunar.date_assignment_safe {
                                        "日期归属安全"
                                    } else {
                                        "日期边界不安全"
                                    }
                                )),
                                None => ui.monospace(
                                    audit
                                        .lunisolar_date_candidate_error
                                        .as_deref()
                                        .unwrap_or("未形成"),
                                ),
                            };
                            ui.end_row();
                            ui.label("紫微命身宫双候选");
                            match &audit.ziwei_life_body_audit {
                                Some(ziwei) => {
                                    ui.vertical(|ui| {
                                        ui.monospace(format!(
                                            "北京时间：{} / {}时 / {}",
                                            ziwei.beijing_time_candidate.chart.compact_zh(),
                                            ziwei.beijing_time_candidate.shichen.branch.zh_name(),
                                            if ziwei
                                                .beijing_time_candidate
                                                .mechanical_boundaries_safe
                                            {
                                                "边界安全"
                                            } else {
                                                "边界不安全"
                                            }
                                        ));
                                        ui.monospace(format!(
                                            "真太阳时：{} / {}时 / {}",
                                            ziwei.true_solar_time_candidate.chart.compact_zh(),
                                            ziwei
                                                .true_solar_time_candidate
                                                .shichen
                                                .branch
                                                .zh_name(),
                                            if ziwei
                                                .true_solar_time_candidate
                                                .mechanical_boundaries_safe
                                            {
                                                "边界安全"
                                            } else {
                                                "边界不安全"
                                            }
                                        ));
                                        ui.monospace(format!(
                                            "两候选{} / 时辰口径未选择 / 规则准入否",
                                            if ziwei.candidates_identical {
                                                "相同"
                                            } else {
                                                "不同"
                                            }
                                        ));
                                    });
                                }
                                None => {
                                    ui.monospace(
                                        audit
                                            .ziwei_life_body_audit_error
                                            .as_deref()
                                            .unwrap_or("未形成"),
                                    );
                                }
                            };
                            ui.end_row();
                            ui.label("紫微十二宫逆布");
                            match &audit.ziwei_life_body_audit {
                                Some(ziwei) => {
                                    ui.vertical(|ui| {
                                        ui.monospace(format!(
                                            "北京时间：{}",
                                            ziwei_palace_sequence_zh(
                                                &ziwei.beijing_time_candidate.chart
                                            )
                                        ));
                                        ui.monospace(format!(
                                            "真太阳时：{}",
                                            ziwei_palace_sequence_zh(
                                                &ziwei.true_solar_time_candidate.chart
                                            )
                                        ));
                                    });
                                }
                                None => {
                                    ui.monospace("—");
                                }
                            };
                            ui.end_row();
                            ui.label("紫微宫干与五行局");
                            match &audit.ziwei_life_body_audit {
                                Some(ziwei) if !ziwei.year_stem_convention_candidates.is_empty() => {
                                    ui.vertical(|ui| {
                                        for year in &ziwei.year_stem_convention_candidates {
                                            ui.monospace(format!(
                                                "{}：{}（年界{}）",
                                                ziwei_year_convention_label(
                                                    &year.year_convention
                                                ),
                                                year.year_designation.zh_name(),
                                                if year.year_boundary_safe {
                                                    "安全"
                                                } else {
                                                    "不安全"
                                                }
                                            ));
                                            ui.monospace(format!(
                                                "宫干：{}",
                                                ziwei_branch_stem_sequence_zh(
                                                    &year.beijing_time_candidate
                                                )
                                            ));
                                            ui.monospace(format!(
                                                "北京时间：{} / 真太阳时：{}",
                                                year.beijing_time_candidate.compact_zh(),
                                                year.true_solar_time_candidate.compact_zh()
                                            ));
                                        }
                                        for candidate in &ziwei.star_anchor_candidates {
                                            ui.monospace(format!(
                                                "紫微星锚 [{} / {}]：{} / 边界{}",
                                                ziwei_year_convention_label(&candidate.year_convention),
                                                if candidate.hour_convention.starts_with("beijing") {
                                                    "北京时间"
                                                } else {
                                                    "真太阳时"
                                                },
                                                candidate.anchor.compact_zh(),
                                                if candidate.anchor.mechanical_boundaries_safe {
                                                    "安全"
                                                } else {
                                                    "不安全"
                                                }
                                            ));
                                            if let Some(stars) = &candidate.major_stars {
                                                ui.monospace(format!(
                                                    "  紫微系：{}",
                                                    stars.group_compact_zh("ziwei_reverse")
                                                ));
                                                ui.monospace(format!(
                                                    "  天府系：{} / 仅机械排盘，规则准入否",
                                                    stars.group_compact_zh("tianfu_forward")
                                                ));
                                            }
                                            if let Some(stars) = &candidate.auxiliary_stars {
                                                ui.monospace(format!(
                                                    "  辅星候选：{} / 仅机械排盘，规则准入否",
                                                    stars.compact_zh()
                                                ));
                                            }
                                        }
                                        ui.monospace(format!(
                                            "两年干口径{} / 年干与时辰口径均未选择 / 规则准入否",
                                            if ziwei.year_stem_conventions_identical {
                                                "结果相同"
                                            } else {
                                                "结果不同"
                                            }
                                        ));
                                    });
                                }
                                Some(_) => {
                                    ui.monospace("旧记录未保存宫干与五行局候选");
                                }
                                None => {
                                    ui.monospace("—");
                                }
                            };
                            ui.end_row();
                            ui.label("七政四余身份目录");
                            match &audit.qizheng_siyu_identity_audit {
                                Some(qizheng) => {
                                    ui.vertical(|ui| {
                                        ui.monospace(qizheng_siyu_identity_compact_zh(qizheng));
                                        ui.monospace(
                                            "五星仅有 JPL 近似研究向量；日月与四余位置未完成 / 命盘未生成 / 规则准入否",
                                        );
                                        for line in qizheng_planetary_research_lines(&audit) {
                                            ui.monospace(line);
                                        }
                                    });
                                }
                                None => {
                                    ui.monospace("旧记录未保存");
                                }
                            };
                            ui.end_row();
                            ui.label("审计状态");
                            ui.monospace(&audit.status);
                            ui.end_row();
                        });
                    ui.colored_label(
                        Color32::from_rgb(167, 103, 32),
                        "四柱和紫微仍仅有机械候选；七政四余增加五星 JPL 近似几何位置研究向量，但不是视地位置，更不能直接判宿度。日月、四余、宫制、古籍断法和完整命盘仍未完成，不生成吉凶结论。",
                    );
                }
                Err(error) => {
                    ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string());
                }
            }
        } else {
            ui.label("未提供出生参数；需要该参数的术式已在适用性检查中自动跳过。");
        }

        ui.add_space(18.0);
        ui.label(RichText::new("历法与天文基础").size(18.0).strong());
        let civil_calendar_views = civil_calendar_audit(
            historical_datetime_from_bjt(question.divination_time_bjt),
            CalendarSystem::Gregorian,
        )
        .map_err(|error| error.to_string());
        let (foundation_true_solar, foundation_correction) = true_solar_time_foundation(
            question.divination_time_bjt,
            question.location.longitude_deg,
            5.0,
        );
                                                                           
                                                                                 
        let jd_ut1_approx =
            question.divination_time_bjt.timestamp_millis() as f64 / 86_400_000.0 + 2_440_587.5;
        let delta_t = estimate_delta_t(
            question.divination_time_bjt.year(),
            question.divination_time_bjt.month() as u8,
        );
        let jd_tt = delta_t
            .as_ref()
            .ok()
            .and_then(|estimate| julian_day_tt_from_ut1(jd_ut1_approx, estimate.seconds).ok());
        let solar = jd_tt.map(|jd| SpaSolarEphemeris.sun_geocentric_ecliptic(jd));
        let spa_coordinates = jd_tt.map(|jd| SpaSolarEphemeris.calculate(jd));
        let spa_true_solar = spa_coordinates
            .as_ref()
            .map(|coordinates| match coordinates {
                Ok(sun) => true_solar_time_from_equation_of_time(
                    question.divination_time_bjt,
                    question.location.longitude_deg,
                    sun.equation_of_time_minutes,
                    5.0,
                )
                .map_err(|error| error.to_string()),
                Err(error) => Err(error.to_string()),
            });
        let sexagenary_day_bjt =
            sexagenary_day_for_fixed_date(question.divination_time_bjt, CalendarSystem::Gregorian)
                .map_err(|error| error.to_string());
        let beijing_shichen = shichen_from_true_solar_time(question.divination_time_bjt, 5.0)
            .map_err(|error| error.to_string());
        let true_solar_sexagenary = spa_true_solar.as_ref().map(|result| match result {
            Ok((time, _)) => {
                let day = sexagenary_day_for_fixed_date(*time, CalendarSystem::Gregorian)
                    .map_err(|error| error.to_string())?;
                let shichen =
                    shichen_from_true_solar_time(*time, 5.0).map_err(|error| error.to_string())?;
                let hour = sexagenary_hour_from_day_stem(day.stem, shichen.branch);
                Ok((day, shichen, hour))
            }
            Err(error) => Err(error.to_string()),
        });
        let term_interval = jd_tt.map(|jd| solar_term_interval(&SpaSolarEphemeris, jd, 300.0));
        let jie_interval = jd_tt.map(|jd| {
            solar_term_class_interval(&SpaSolarEphemeris, jd, SolarTermClass::Jie, 300.0)
        });
        let zhongqi_interval = jd_tt.map(|jd| {
            solar_term_class_interval(&SpaSolarEphemeris, jd, SolarTermClass::Zhongqi, 300.0)
        });
        let lunar_interval = jd_tt.map(|jd| lunation_interval(jd, 300.0));
        let lunisolar_structure = lunisolar_month_structure_for_civil_date(
            &SpaSolarEphemeris,
            historical_datetime_from_bjt(question.divination_time_bjt),
            300.0,
        );
        let lunisolar_date = lunisolar_structure.as_ref().ok().map(|structure| {
            lunisolar_date_candidate_from_structure(
                structure,
                historical_datetime_from_bjt(question.divination_time_bjt),
            )
        });
        let meihua_time_audit = match (&lunisolar_date, &beijing_shichen, &true_solar_sexagenary) {
            (Some(Ok(date)), Ok(beijing_shichen), Some(Ok((_, true_solar_shichen, _)))) => {
                meihua_time_casting_audit(date, *beijing_shichen, *true_solar_shichen).ok()
            }
            _ => None,
        };
        let qimen_ju_candidate = match (
            &term_interval,
            &sexagenary_day_bjt,
            &beijing_shichen,
            &true_solar_sexagenary,
            &spa_true_solar,
        ) {
            (
                Some(Ok(interval)),
                Ok(beijing_day),
                Ok(beijing_shichen),
                Some(Ok((true_solar_day, true_solar_shichen, true_solar_hour))),
                Some(Ok((true_solar_time, _))),
            ) => match (
                qimen_midnight_boundary_safe(question.divination_time_bjt, 5.0),
                qimen_midnight_boundary_safe(*true_solar_time, 5.0),
            ) {
                (Ok(beijing_safe), Ok(true_solar_safe)) => {
                    let beijing_hour =
                        sexagenary_hour_from_day_stem(beijing_day.stem, beijing_shichen.branch);
                    Some(qimen_ju_duty_identity_audit(
                        interval,
                        *beijing_day,
                        beijing_safe,
                        beijing_hour,
                        beijing_shichen.boundary_safe,
                        *true_solar_day,
                        true_solar_safe,
                        *true_solar_hour,
                        true_solar_shichen.boundary_safe,
                    ))
                }
                _ => None,
            },
            _ => None,
        };
        let civil_year = question.divination_time_bjt.year();
        let pillar_candidates = jd_tt.map(|jd| {
            let lichun = solve_solar_term_near_bjt_date(
                &SpaSolarEphemeris,
                SolarTerm::Lichun,
                civil_year,
                2,
                4,
                7.0,
            )
            .map_err(|error| error.to_string())?;
            let year = lichun_year_candidate(civil_year, jd, &lichun, 300.0)
                .map_err(|error| error.to_string())?;
            let jie = match &jie_interval {
                Some(Ok(interval)) => interval,
                Some(Err(error)) => return Err(error.to_string()),
                None => return Err("缺少节区间".to_owned()),
            };
            let month = jie_month_candidate(year.designation.stem, jie)
                .map_err(|error| error.to_string())?;
            Ok((
                sexagenary_year_for_astronomical_year_label(civil_year),
                year,
                month,
            ))
        });
        let stored_astronomy = self.storage.load_astronomy_snapshot(question.id);
        egui::Grid::new("calendar_debug")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("北京时间");
                ui.monospace(question.divination_time_bjt.to_rfc3339());
                ui.end_row();
                ui.label("统一输入历法");
                ui.monospace("前推格里高利历 ISO / 天文年编号");
                ui.end_row();
                ui.label("同瞬间 · 儒略历");
                match &civil_calendar_views {
                    Ok(calendar) => ui.monospace(historical_datetime_label_zh(calendar.julian)),
                    Err(error) => ui.colored_label(Color32::from_rgb(167, 54, 54), error),
                };
                ui.end_row();
                ui.label("同瞬间 · 1582 改革历");
                match &civil_calendar_views {
                    Ok(calendar) => ui.monospace(format!(
                        "{} / 采用{}",
                        historical_datetime_label_zh(calendar.historical_cutover),
                        calendar_system_label(calendar.historical_cutover_resolved_system)
                    )),
                    Err(error) => ui.colored_label(Color32::from_rgb(167, 54, 54), error),
                };
                ui.end_row();
                ui.label("历法换算算法 / 日数");
                match &civil_calendar_views {
                    Ok(calendar) => ui.monospace(format!(
                        "{} / JD {:.9}",
                        calendar.algorithm_version, calendar.julian_day
                    )),
                    Err(_) => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("真太阳时（SPA）");
                match &spa_true_solar {
                    Some(Ok((time, _))) => ui.monospace(time.to_rfc3339()),
                    Some(Err(error)) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string())
                    }
                    None => ui.label("—"),
                };
                ui.end_row();
                ui.label("均时差 / 经度 / 总校正（SPA）");
                match &spa_true_solar {
                    Some(Ok((_, correction))) => ui.monospace(format!(
                        "{:+.6} / {:+.3} / {:+.6} 分钟",
                        correction.equation_of_time_minutes,
                        correction.longitude_correction_minutes,
                        correction.total_correction_minutes
                    )),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("均时差算法");
                ui.monospace(divination_core::SPA_EQUATION_OF_TIME_VERSION);
                ui.end_row();
                ui.label("时界安全（5 分钟边距）");
                match &spa_true_solar {
                    Some(Ok((_, correction))) => ui.monospace(if correction.boundary_safe {
                        "是"
                    } else {
                        "否，禁止驱动规则"
                    }),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("干支日（北京时间零点换日）");
                match &sexagenary_day_bjt {
                    Ok(day) => ui.monospace(format!(
                        "{} / 六十循环序号 {}",
                        day.zh_name(),
                        day.cycle_index + 1
                    )),
                    Err(error) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string())
                    }
                };
                ui.end_row();
                ui.label("干支日（真太阳时零点换日）");
                match &true_solar_sexagenary {
                    Some(Ok((day, _, _))) => ui.monospace(format!(
                        "{} / 六十循环序号 {}",
                        day.zh_name(),
                        day.cycle_index + 1
                    )),
                    Some(Err(error)) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string())
                    }
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("真太阳时辰");
                match &true_solar_sexagenary {
                    Some(Ok((_, shichen, hour))) => ui.monospace(format!(
                        "{}时 / 距时界 {:.3} 分钟 / {} / 条件时柱 {}",
                        shichen.branch.zh_name(),
                        shichen.minutes_to_boundary,
                        if shichen.boundary_safe {
                            "边界安全"
                        } else {
                            "边界不安全"
                        },
                        hour.zh_name()
                    )),
                    Some(Err(error)) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string())
                    }
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("日/时算法版本");
                ui.monospace(format!(
                    "{} / {}",
                    divination_core::SEXAGENARY_DAY_ALGORITHM_VERSION,
                    divination_core::SHICHEN_ALGORITHM_VERSION
                ));
                ui.end_row();
                ui.label("子时换日口径");
                ui.colored_label(
                    Color32::from_rgb(167, 103, 32),
                    "传统差异未裁定；条件时柱不得驱动正式规则",
                );
                ui.end_row();
                ui.label("基础近似对照");
                ui.monospace(format!(
                    "{} / 总校正 {:+.3} 分钟",
                    foundation_true_solar.to_rfc3339(),
                    foundation_correction.total_correction_minutes
                ));
                ui.end_row();
                ui.label("JD（UTC≈UT1）");
                ui.monospace(format!("{jd_ut1_approx:.9}"));
                ui.end_row();
                ui.label("ΔT（TT−UT1）");
                match &delta_t {
                    Ok(estimate) => {
                        ui.monospace(format!(
                            "{:.3} 秒 / 模型不确定度约 ±{:.3} 秒",
                            estimate.seconds, estimate.estimated_uncertainty_seconds
                        ));
                    }
                    Err(error) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string());
                    }
                }
                ui.end_row();
                ui.label("JD（TT）");
                ui.monospace(
                    jd_tt
                        .map(|value| format!("{value:.9}"))
                        .unwrap_or_else(|| "—".to_owned()),
                );
                ui.end_row();
                ui.label("太阳视黄经（SPA 重实现）");
                match &solar {
                    Some(Ok(sun)) => {
                        ui.monospace(format!(
                            "{:.9}° / 黄纬 {:+.9}° / R {:.9} AU / 保守误差 {:.1}′",
                            sun.longitude_deg,
                            sun.latitude_deg,
                            sun.distance_au,
                            sun.estimated_error_arcmin
                        ));
                    }
                    Some(Err(error)) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string());
                    }
                    None => {
                        ui.label("—");
                    }
                }
                ui.end_row();
                ui.label("太阳算法版本");
                match &solar {
                    Some(Ok(sun)) => ui.monospace(&sun.algorithm_version),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("上一节气");
                match &term_interval {
                    Some(Ok(interval)) => {
                        ui.monospace(format!(
                            "{} / JD(TT) {:.9} / 合成不确定度 ±{:.2} 秒 / {} 次迭代",
                            interval.previous.term.zh_name(),
                            interval.previous.jd_tt,
                            interval.previous.combined_uncertainty_seconds,
                            interval.previous.iterations
                        ));
                    }
                    Some(Err(error)) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string());
                    }
                    None => {
                        ui.label("—");
                    }
                }
                ui.end_row();
                ui.label("下一节气");
                match &term_interval {
                    Some(Ok(interval)) => {
                        ui.monospace(format!(
                            "{} / JD(TT) {:.9} / 合成不确定度 ±{:.2} 秒 / {} 次迭代",
                            interval.next.term.zh_name(),
                            interval.next.jd_tt,
                            interval.next.combined_uncertainty_seconds,
                            interval.next.iterations
                        ));
                    }
                    Some(Err(error)) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string());
                    }
                    None => {
                        ui.label("—");
                    }
                }
                ui.end_row();
                ui.label("当前节气区间");
                match &term_interval {
                    Some(Ok(interval)) => ui.monospace(format!(
                        "{} → {} / 已过 {:.6} 天 / 剩余 {:.6} 天",
                        interval.previous.term.zh_name(),
                        interval.next.term.zh_name(),
                        interval.seconds_since_previous / 86_400.0,
                        interval.seconds_until_next / 86_400.0
                    )),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("节气边界安全（额外 5 分钟边距）");
                match &term_interval {
                    Some(Ok(interval)) => ui.monospace(if interval.boundary_safe {
                        "是"
                    } else {
                        "否，禁止驱动节令边界规则"
                    }),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("节气区间算法");
                match &term_interval {
                    Some(Ok(interval)) => ui.monospace(&interval.algorithm_version),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("奇门阴阳遁 / 三元局数机械候选");
                match &qimen_ju_candidate {
                    Some(audit) => ui.monospace(format!(
                        "{} / {} / 表 {:?} / 处暑异文冲突 {}",
                        audit.active_solar_term.zh_name(),
                        audit.dun.zh_name(),
                        audit.received_bureau_numbers,
                        audit.table_entry_evidence_conflict
                    )),
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("奇门候选 · 北京零点换日");
                match &qimen_ju_candidate {
                    Some(audit) => {
                        ui.monospace(audit.beijing_midnight_candidate.compact_zh(audit.dun))
                    }
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("奇门地盘 · 北京零点换日");
                match &qimen_ju_candidate {
                    Some(audit) => match &audit.beijing_midnight_candidate.earth_plate {
                        Some(plate) => ui.monospace(plate.compact_zh()),
                        None => ui.monospace("未生成（局数证据停用）"),
                    },
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("奇门时旬首 · 北京时间");
                match &qimen_ju_candidate {
                    Some(audit) => match &audit.beijing_midnight_candidate.hour_xun_candidate {
                        Some(candidate) => ui.monospace(candidate.compact_zh()),
                        None => ui.monospace("历史快照未包含"),
                    },
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("奇门值符/值使身份 · 北京时间");
                match &qimen_ju_candidate {
                    Some(audit) => match audit
                        .beijing_midnight_candidate
                        .hour_xun_candidate
                        .as_ref()
                        .and_then(|hour| hour.duty_identity_candidate.as_ref())
                    {
                        Some(identity) => ui.monospace(identity.compact_zh()),
                        None => ui.monospace("未生成（旧快照或局数证据停用）"),
                    },
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("奇门候选 · 真太阳时零点换日");
                match &qimen_ju_candidate {
                    Some(audit) => {
                        ui.monospace(audit.true_solar_midnight_candidate.compact_zh(audit.dun))
                    }
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("奇门地盘 · 真太阳时零点换日");
                match &qimen_ju_candidate {
                    Some(audit) => match &audit.true_solar_midnight_candidate.earth_plate {
                        Some(plate) => ui.monospace(plate.compact_zh()),
                        None => ui.monospace("未生成（局数证据停用）"),
                    },
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("奇门时旬首 · 真太阳时");
                match &qimen_ju_candidate {
                    Some(audit) => match &audit
                        .true_solar_midnight_candidate
                        .hour_xun_candidate
                    {
                        Some(candidate) => ui.monospace(candidate.compact_zh()),
                        None => ui.monospace("历史快照未包含"),
                    },
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("奇门值符/值使身份 · 真太阳时");
                match &qimen_ju_candidate {
                    Some(audit) => match audit
                        .true_solar_midnight_candidate
                        .hour_xun_candidate
                        .as_ref()
                        .and_then(|hour| hour.duty_identity_candidate.as_ref())
                    {
                        Some(identity) => ui.monospace(identity.compact_zh()),
                        None => ui.monospace("未生成（旧快照或局数证据停用）"),
                    },
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("奇门候选状态");
                match &qimen_ju_candidate {
                    Some(audit) => ui.colored_label(
                        Color32::from_rgb(167, 103, 32),
                        format!(
                            "{}；不选日界/时制/超神接气口径，不做值符值使逐时飞转，不排天盘/神盘，不参与规则",
                            audit.status
                        ),
                    ),
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("节区间（每隔一节气）");
                match &jie_interval {
                    Some(Ok(interval)) => ui.monospace(format!(
                        "{} → {} / 已过 {:.6} 天 / 剩余 {:.6} 天 / {}",
                        interval.previous.term.zh_name(),
                        interval.next.term.zh_name(),
                        interval.seconds_since_previous / 86_400.0,
                        interval.seconds_until_next / 86_400.0,
                        if interval.boundary_safe {
                            "边界安全"
                        } else {
                            "边界不安全"
                        }
                    )),
                    Some(Err(error)) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string())
                    }
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("中气区间（每隔一节气）");
                match &zhongqi_interval {
                    Some(Ok(interval)) => ui.monospace(format!(
                        "{} → {} / 已过 {:.6} 天 / 剩余 {:.6} 天 / {}",
                        interval.previous.term.zh_name(),
                        interval.next.term.zh_name(),
                        interval.seconds_since_previous / 86_400.0,
                        interval.seconds_until_next / 86_400.0,
                        if interval.boundary_safe {
                            "边界安全"
                        } else {
                            "边界不安全"
                        }
                    )),
                    Some(Err(error)) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string())
                    }
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("节/中气区间算法");
                match &jie_interval {
                    Some(Ok(interval)) => ui.monospace(&interval.algorithm_version),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("当前朔望月天文区间");
                match &lunar_interval {
                    Some(Ok(interval)) => ui.monospace(format!(
                        "{} JD(TT) {:.9} → {} {:.9} → {} {:.9} / 已过 {:.6} 天 / 剩余 {:.6} 天",
                        interval.previous_new_moon.phase.zh_name(),
                        interval.previous_new_moon.jd_tt,
                        interval.full_moon.phase.zh_name(),
                        interval.full_moon.jd_tt,
                        interval.next_new_moon.phase.zh_name(),
                        interval.next_new_moon.jd_tt,
                        interval.seconds_since_previous_new_moon / 86_400.0,
                        interval.seconds_until_next_new_moon / 86_400.0
                    )),
                    Some(Err(error)) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string())
                    }
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("朔界安全（公式误差 + 额外 5 分钟）");
                match &lunar_interval {
                    Some(Ok(interval)) => ui.monospace(if interval.new_moon_boundary_safe {
                        "是"
                    } else {
                        "否，禁止驱动月界规则"
                    }),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("朔望基础状态");
                match &lunar_interval {
                    Some(Ok(interval)) => ui.monospace(format!(
                        "{} / {} / 仅天文相位，不是农历日期或置闰结果",
                        interval.algorithm_version, interval.status
                    )),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("冬至—中气月结构候选");
                match &lunisolar_structure {
                    Ok(structure) => ui.monospace(format!(
                        "冬至年 {} / {} 个月 / 闰月 {} / 全部日界{}",
                        structure.winter_solstice_year,
                        structure.month_count,
                        structure
                            .leap_month_number
                            .map_or_else(|| "无".to_owned(), |month| format!("{month} 月")),
                        if structure.all_boundaries_safe {
                            "安全"
                        } else {
                            "存在不安全项"
                        }
                    )),
                    Err(error) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string())
                    }
                };
                ui.end_row();
                ui.label("月序、中气与起始日");
                match &lunisolar_structure {
                    Ok(structure) => {
                        let labels = structure
                            .months
                            .iter()
                            .map(|month| {
                                let start = month.start_bjt.local_date_time;
                                format!(
                                    "{}{}月 {:04}-{:02}-{:02} [{}; {}]",
                                    if month.is_leap { "闰" } else { "" },
                                    month.month_number,
                                    start.year,
                                    start.month,
                                    start.day,
                                    month
                                        .major_solar_term
                                        .as_ref()
                                        .map_or("无中气", |term| term.term.zh_name()),
                                    if month.term_classification_safe {
                                        "安全"
                                    } else {
                                        "日界不安全"
                                    }
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" | ");
                        ui.monospace(labels)
                    }
                    Err(_) => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("月结构状态");
                match &lunisolar_structure {
                    Ok(structure) => ui.monospace(format!(
                        "{} / {} / 候选审计，不是完整农历日期",
                        structure.algorithm_version, structure.status
                    )),
                    Err(_) => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("现代农历日期候选");
                match &lunisolar_date {
                    Some(Ok(candidate)) => ui.monospace(format!(
                        "名义年 {} / {}{}月 {}日 / 本月 {} 天 / {}",
                        candidate.nominal_lunar_year,
                        if candidate.is_leap_month { "闰" } else { "" },
                        candidate.month_number,
                        candidate.day_number,
                        candidate.month_length_days,
                        if candidate.date_assignment_safe {
                            "日期归属安全"
                        } else {
                            "边界存在不安全项，禁止驱动规则"
                        }
                    )),
                    Some(Err(error)) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string())
                    }
                    None => ui.monospace("—（月结构未形成）"),
                };
                ui.end_row();
                ui.label("日期候选状态");
                match &lunisolar_date {
                    Some(Ok(candidate)) => ui.monospace(format!(
                        "{} / {} / 现代官方口径审计，不等于历代统一历法",
                        candidate.algorithm_version, candidate.status
                    )),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("梅花时间起卦候选");
                match &meihua_time_audit {
                    Some(audit) => {
                        let beijing = audit.beijing_time_candidate.as_ref().map_or_else(
                            || "闰月失败关闭".to_owned(),
                            |candidate| candidate.cast.compact_zh(),
                        );
                        let true_solar = audit.true_solar_time_candidate.as_ref().map_or_else(
                            || "闰月失败关闭".to_owned(),
                            |candidate| candidate.cast.compact_zh(),
                        );
                        ui.monospace(format!(
                            "北京时间：{} / 真太阳时：{} / 选择：无 / 规则准入：否",
                            beijing, true_solar
                        ))
                    }
                    None => ui.monospace("—（历法或时辰基础未形成）"),
                };
                ui.end_row();
                ui.label("梅花公式状态");
                match &meihua_time_audit {
                    Some(audit) => ui.monospace(format!(
                        "{} / {} / {}",
                        audit.algorithm_version,
                        audit.status,
                        audit.blocked_reasons.join("；")
                    )),
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("公历年干支标签");
                match &pillar_candidates {
                    Some(Ok((label, _, _))) => ui.monospace(format!(
                        "{}（仅对应公历年编号，不代表换年口径）",
                        label.zh_name()
                    )),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("立春换年候选");
                match &pillar_candidates {
                    Some(Ok((_, year, _))) => ui.monospace(format!(
                        "{} / 边界 JD(TT) {:.9} / 距边界 {:+.3} 秒 / {}",
                        year.designation.zh_name(),
                        year.boundary_jd_tt,
                        year.seconds_from_boundary,
                        if year.boundary_safe {
                            "边界安全"
                        } else {
                            "边界不安全"
                        }
                    )),
                    Some(Err(error)) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string())
                    }
                    None => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("节令月柱候选");
                match &pillar_candidates {
                    Some(Ok((_, _, month))) => ui.monospace(format!(
                        "{} / {} → {} / {}",
                        month.designation.zh_name(),
                        month.previous_jie.zh_name(),
                        month.next_jie.zh_name(),
                        if month.boundary_safe {
                            "边界安全"
                        } else {
                            "边界不安全"
                        }
                    )),
                    _ => ui.monospace("—"),
                };
                ui.end_row();
                ui.label("年/月柱候选状态");
                ui.colored_label(
                    Color32::from_rgb(167, 103, 32),
                    "仅供审计；尚无术式规则选定，不参与排盘或裁决",
                );
                ui.end_row();
                ui.label("固化天文快照");
                match &stored_astronomy {
                    Ok(Some(snapshot)) => {
                        ui.monospace(format!(
                            "v{} / {} / {} / {}",
                            snapshot.snapshot_version,
                            snapshot.solar_algorithm_version,
                            snapshot
                                .equation_of_time_algorithm_version
                                .as_deref()
                                .unwrap_or("旧版未固化均时差"),
                            snapshot.ut1_basis
                        ));
                    }
                    Ok(None) => {
                        ui.colored_label(Color32::from_rgb(167, 103, 32), "旧记录无快照");
                    }
                    Err(error) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string());
                    }
                }
                ui.end_row();
                ui.label("快照与当前算法重算");
                let recompute_matches = match (
                    &stored_astronomy,
                    jd_tt,
                    &solar,
                    &spa_true_solar,
                    &term_interval,
                    &lunar_interval,
                ) {
                    (
                        Ok(Some(snapshot)),
                        Some(jd),
                        Some(Ok(sun)),
                        Some(Ok((true_solar, correction))),
                        Some(Ok(interval)),
                        Some(Ok(current_lunation)),
                    ) => {
                        let true_solar_matches = if snapshot.snapshot_version >= 2 {
                            let current_true_solar = true_solar.to_rfc3339();
                            snapshot.equation_of_time_minutes.is_some_and(|value| {
                                (value - correction.equation_of_time_minutes).abs() < 1.0e-12
                            }) && snapshot.longitude_correction_minutes.is_some_and(|value| {
                                (value - correction.longitude_correction_minutes).abs() < 1.0e-12
                            }) && snapshot.true_solar_total_correction_minutes.is_some_and(
                                |value| {
                                    (value - correction.total_correction_minutes).abs() < 1.0e-12
                                },
                            ) && snapshot.true_solar_time_bjt.as_deref()
                                == Some(current_true_solar.as_str())
                                && snapshot.true_solar_boundary_safe
                                    == Some(correction.boundary_safe)
                        } else {
                            true
                        };
                        let sexagenary_matches = if snapshot.snapshot_version >= 3 {
                            match (&sexagenary_day_bjt, &true_solar_sexagenary) {
                                (Ok(bjt_day), Some(Ok((true_solar_day, shichen, hour)))) => {
                                    snapshot.sexagenary_calendar_system.as_deref()
                                        == Some("proleptic_gregorian_iso_input.v1")
                                        && snapshot.sexagenary_day_algorithm_version.as_deref()
                                            == Some(
                                                divination_core::SEXAGENARY_DAY_ALGORITHM_VERSION,
                                            )
                                        && snapshot.sexagenary_day_bjt_midnight == Some(*bjt_day)
                                        && snapshot.sexagenary_day_true_solar_midnight
                                            == Some(*true_solar_day)
                                        && snapshot.shichen_algorithm_version.as_deref()
                                            == Some(divination_core::SHICHEN_ALGORITHM_VERSION)
                                        && snapshot.true_solar_shichen == Some(*shichen)
                                        && snapshot.sexagenary_hour_true_solar_midnight
                                            == Some(*hour)
                                        && snapshot.zi_hour_day_boundary_status.as_deref()
                                            == Some(divination_core::ZI_HOUR_DAY_BOUNDARY_STATUS)
                                }
                                _ => false,
                            }
                        } else {
                            true
                        };
                        let solar_term_interval_matches = if snapshot.snapshot_version >= 4 {
                            snapshot.solar_term_interval_algorithm_version.as_deref()
                                == Some(interval.algorithm_version.as_str())
                                && snapshot.previous_solar_term == Some(interval.previous.term)
                                && snapshot.previous_solar_term_jd_tt.is_some_and(|value| {
                                    (value - interval.previous.jd_tt).abs() < 1.0e-12
                                })
                                && snapshot
                                    .previous_solar_term_uncertainty_seconds
                                    .is_some_and(|value| {
                                        (value - interval.previous.combined_uncertainty_seconds)
                                            .abs()
                                            < 1.0e-12
                                    })
                                && snapshot.previous_solar_term_iterations
                                    == Some(interval.previous.iterations)
                                && snapshot.solar_term_boundary_margin_seconds
                                    == Some(interval.boundary_margin_seconds)
                                && snapshot
                                    .seconds_since_previous_solar_term
                                    .is_some_and(|value| {
                                        (value - interval.seconds_since_previous).abs() < 1.0e-12
                                    })
                                && snapshot.seconds_until_next_solar_term.is_some_and(|value| {
                                    (value - interval.seconds_until_next).abs() < 1.0e-12
                                })
                                && snapshot.solar_term_boundary_safe == Some(interval.boundary_safe)
                        } else {
                            true
                        };
                        let solar_term_class_matches = if snapshot.snapshot_version >= 5 {
                            match (&jie_interval, &zhongqi_interval) {
                                (Some(Ok(jie)), Some(Ok(zhongqi))) => {
                                    snapshot.jie_interval.as_ref() == Some(jie)
                                        && snapshot.zhongqi_interval.as_ref() == Some(zhongqi)
                                }
                                _ => false,
                            }
                        } else {
                            true
                        };
                        let pillar_candidates_match = if snapshot.snapshot_version >= 6 {
                            match &pillar_candidates {
                                Some(Ok((label, year, month))) => {
                                    snapshot.astronomical_year_label == Some(*label)
                                        && snapshot.lichun_year_candidate.as_ref() == Some(year)
                                        && snapshot.jie_month_candidate.as_ref() == Some(month)
                                        && snapshot.pillar_candidate_status.as_deref()
                                            == Some(divination_core::PILLAR_CANDIDATE_STATUS)
                                }
                                _ => false,
                            }
                        } else {
                            true
                        };
                        let birth_time_matches = if snapshot.snapshot_version >= 7 {
                            match (&question.birth, &snapshot.birth_time_audit) {
                                (None, None) => true,
                                (Some(birth), Some(stored_birth)) => {
                                    stored_birth.status == BIRTH_TIME_AUDIT_STATUS
                                        && calculate_birth_time_audit_snapshot(birth).is_ok_and(
                                            |current| {
                                                birth_time_audits_match(&current, stored_birth)
                                            },
                                        )
                                }
                                _ => false,
                            }
                        } else {
                            true
                        };
                        let civil_calendar_matches = if snapshot.snapshot_version >= 8 {
                            match &civil_calendar_views {
                                Ok(current) => {
                                    snapshot.civil_calendar_audit.as_ref() == Some(current)
                                }
                                Err(_) => false,
                            }
                        } else {
                            true
                        };
                        let lunation_matches = if snapshot.snapshot_version >= 9 {
                            snapshot.lunation_interval.as_ref() == Some(current_lunation)
                        } else {
                            true
                        };
                        let lunisolar_matches = if snapshot.snapshot_version >= 10 {
                            match (
                                &lunisolar_structure,
                                &snapshot.lunisolar_month_structure,
                                &snapshot.lunisolar_month_structure_error,
                            ) {
                                (Ok(current), Some(stored), None) => current == stored,
                                (Err(error), None, Some(stored_error)) => {
                                    error.to_string() == *stored_error
                                }
                                _ => false,
                            }
                        } else {
                            true
                        };
                        let lunisolar_date_matches = if snapshot.snapshot_version >= 11 {
                            match (
                                &lunisolar_date,
                                &snapshot.lunisolar_date_candidate,
                                &snapshot.lunisolar_date_candidate_error,
                            ) {
                                (Some(Ok(current)), Some(stored), None) => current == stored,
                                (Some(Err(error)), None, Some(stored_error)) => {
                                    error.to_string() == *stored_error
                                }
                                (None, None, None) => true,
                                _ => false,
                            }
                        } else {
                            true
                        };
                        let meihua_time_matches = if snapshot.snapshot_version >= 14 {
                            snapshot.meihua_time_casting_audit.as_ref()
                                == meihua_time_audit.as_ref()
                        } else {
                            true
                        };
                        let mansion_catalog_matches = if snapshot.snapshot_version >= 22 {
                            snapshot.chinese_astrology_structure_audit.as_ref()
                                == Some(&chinese_astrology_structure)
                        } else if snapshot.snapshot_version >= 17 {
                            snapshot
                                .chinese_astrology_structure_audit
                                .as_ref()
                                .is_some_and(|stored| {
                                    chinese_astrology_v1_fields_match(
                                        stored,
                                        &chinese_astrology_structure,
                                    )
                                })
                        } else {
                            true
                        };
                        let qimen_ju_matches = if snapshot.snapshot_version >= 21 {
                            snapshot.qimen_ju_audit.as_ref() == qimen_ju_candidate.as_ref()
                        } else if snapshot.snapshot_version == 20 {
                            match (
                                snapshot.qimen_ju_audit.as_ref(),
                                qimen_ju_candidate.as_ref(),
                            ) {
                                (Some(stored), Some(current)) => {
                                    qimen_ju_v3_fields_match(stored, current)
                                }
                                (None, None) => true,
                                _ => false,
                            }
                        } else if snapshot.snapshot_version == 19 {
                            match (
                                snapshot.qimen_ju_audit.as_ref(),
                                qimen_ju_candidate.as_ref(),
                            ) {
                                (Some(stored), Some(current)) => {
                                    qimen_ju_v2_fields_match(stored, current)
                                }
                                (None, None) => true,
                                _ => false,
                            }
                        } else if snapshot.snapshot_version == 18 {
                            match (
                                snapshot.qimen_ju_audit.as_ref(),
                                qimen_ju_candidate.as_ref(),
                            ) {
                                (Some(stored), Some(current)) => {
                                    qimen_ju_v1_fields_match(stored, current)
                                }
                                (None, None) => true,
                                _ => false,
                            }
                        } else {
                            true
                        };
                        (snapshot.jd_tt - jd).abs() < 1.0e-12
                            && (snapshot.solar_longitude_deg - sun.longitude_deg).abs() < 1.0e-12
                            && (snapshot.next_solar_term_jd_tt - interval.next.jd_tt).abs()
                                < 1.0e-12
                            && snapshot.next_solar_term == interval.next.term
                            && (snapshot.next_solar_term_uncertainty_seconds
                                - interval.next.combined_uncertainty_seconds)
                                .abs()
                                < 1.0e-12
                            && snapshot.next_solar_term_iterations == interval.next.iterations
                            && snapshot.solar_algorithm_version == sun.algorithm_version
                            && true_solar_matches
                            && sexagenary_matches
                            && solar_term_interval_matches
                            && solar_term_class_matches
                            && pillar_candidates_match
                            && birth_time_matches
                            && civil_calendar_matches
                            && lunation_matches
                            && lunisolar_matches
                            && lunisolar_date_matches
                            && meihua_time_matches
                            && mansion_catalog_matches
                            && qimen_ju_matches
                    }
                    _ => false,
                };
                ui.monospace(if recompute_matches {
                    "一致"
                } else {
                    "不一致或缺少快照（禁止静默覆盖历史值）"
                });
                ui.end_row();
            });

        ui.add_space(18.0);
        ui.label(
            RichText::new("随机起法持久化与读取后重放")
                .size(18.0)
                .strong(),
        );
        egui::Grid::new("casting_replay_debug")
            .num_columns(3)
            .striped(true)
            .spacing([18.0, 8.0])
            .show(ui, |ui| {
                ui.strong("术式");
                ui.strong("材料");
                ui.strong("数据库读取校验");
                ui.end_row();

                ui.label("六爻");
                match self.storage.load_liuyao_artifact(question.id) {
                    Ok(Some(artifact)) => {
                        ui.vertical(|ui| {
                            ui.monospace(format!(
                                "{} 次 / {}",
                                artifact.transcript.events.len(),
                                artifact.cast.transcript_hash
                            ));
                            render_hexagram_pair(
                                ui,
                                artifact.cast.primary_bits,
                                artifact.cast.changed_bits,
                                true,
                            );
                            match load_liuyao_six_spirits_audit(
                                &self.storage,
                                question.id,
                                artifact.cast.primary_bits,
                            ) {
                                Ok(Some(audit)) => {
                                    render_liuyao_six_spirits_audit(ui, &audit, true)
                                }
                                Ok(None) => {
                                    if let Ok(audit) =
                                        liuyao_six_relatives_audit(artifact.cast.primary_bits)
                                    {
                                        render_liuyao_six_relatives_audit(ui, &audit, true);
                                    }
                                }
                                Err(error) => {
                                    ui.colored_label(Color32::from_rgb(167, 54, 54), error);
                                }
                            }
                            if let Ok(audit) =
                                liuyao_hidden_spirits_audit(artifact.cast.primary_bits)
                            {
                                render_liuyao_hidden_spirits_audit(ui, &audit, true);
                            }
                            match load_liuyao_sanhe_audit(
                                &self.storage,
                                question.id,
                                &artifact.cast,
                            ) {
                                Ok(Some(audit)) => {
                                    let fanyin_fuyin = &audit.fanyin_fuyin;
                                    render_liuyao_calendar_moving_audit(
                                        ui,
                                        &fanyin_fuyin
                                            .progress_retreat
                                            .branch_pair_relations
                                            .calendar_relations
                                            .calendar_moving,
                                        true,
                                    );
                                    render_liuyao_calendar_relations_audit(
                                        ui,
                                        &fanyin_fuyin
                                            .progress_retreat
                                            .branch_pair_relations
                                            .calendar_relations,
                                        true,
                                    );
                                    render_liuyao_branch_pair_relations_audit(
                                        ui,
                                        &fanyin_fuyin.progress_retreat.branch_pair_relations,
                                        true,
                                    );
                                    render_liuyao_progress_retreat_audit(
                                        ui,
                                        &fanyin_fuyin.progress_retreat,
                                        true,
                                    );
                                    render_liuyao_fanyin_fuyin_audit(ui, fanyin_fuyin, true);
                                    render_liuyao_sanhe_audit(ui, &audit, true);
                                }
                                Ok(None) => {
                                    ui.small("旧记录缺少可核验月建或日辰快照；不以当前时间补算。");
                                }
                                Err(error) => {
                                    ui.colored_label(Color32::from_rgb(167, 54, 54), error);
                                }
                            };
                        });
                        ui.colored_label(Color32::from_rgb(47, 120, 78), "重放一致");
                    }
                    Ok(None) => {
                        ui.label("尚未完成六次");
                        ui.label("—");
                    }
                    Err(error) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string());
                        ui.colored_label(Color32::from_rgb(167, 54, 54), "失败");
                    }
                }
                ui.end_row();

                ui.label("蓍草");
                match self.storage.load_yarrow_artifact(question.id) {
                    Ok(Some(artifact)) => {
                        ui.vertical(|ui| {
                            ui.monospace(format!(
                                "{} 变 / {} / {}",
                                artifact.transcript.left_splits.len(),
                                artifact.transcript.algorithm_version,
                                artifact.cast.transcript_hash
                            ));
                            render_hexagram_pair(
                                ui,
                                artifact.cast.primary_bits,
                                artifact.cast.changed_bits,
                                true,
                            );
                        });
                        ui.vertical(|ui| {
                            ui.colored_label(Color32::from_rgb(47, 120, 78), "重放一致");
                            ui.small(&artifact.transcript.procedure_convention_id);
                        });
                    }
                    Ok(None) => {
                        ui.label("尚未完成十八变");
                        ui.label("—");
                    }
                    Err(error) => {
                        ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string());
                        ui.colored_label(Color32::from_rgb(167, 54, 54), "失败");
                    }
                }
                ui.end_row();
            });

        ui.add_space(18.0);
        ui.collapsing("太乙年计、岁计八门与阴阳七十二局机械候选", |ui| {
            ui.label("年计输入边界：只接受外部已经给定的正整数上元积年；本软件尚不从现代日期推算积年。");
            ui.monospace(TAIYI_ANNUAL_FRAME_ALGORITHM_VERSION);
            ui.monospace(TAIYI_ANNUAL_FRAME_STATUS);
            ui.label("已闭合：太岁、太乙、天目、计神、合神、始击、主客算、主客大小将宫位、八门目录、岁计直门，以及不含解释的掩/囚/格/对/击/迫位置标签。");
            ui.monospace(TAIYI_MECHANICAL_RELATIONS_ALGORITHM_VERSION);
            ui.monospace(TAIYI_MECHANICAL_RELATIONS_STATUS);
            ui.label("击迫方向以卷三立成表交叉固定：固定环序 +1/+2 对应前/外，-1/-2 对应后/内。始击生成前击、后击、外宫击或内宫击；文昌和主客四将只生成各自适用的迫位置标记，不包含相邻吉凶解释。");
            ui.monospace(TAIYI_BARRIER_RAW_OBSERVATIONS_ALGORITHM_VERSION);
            ui.monospace(TAIYI_BARRIER_RAW_OBSERVATIONS_STATUS);
            ui.label("关法只保存原始观察：穷举两目与四将 8 组、四将彼此 6 组同宫配对，并记录主客算值是否相等。立成表证明单个目将同宫不足以直接判关；主体配对与‘数齐’释义未决，因此不会显示关标签。");
            ui.monospace(TAIYI_ANNUAL_GATE_BOARDS_ALGORITHM_VERSION);
            ui.monospace(TAIYI_ANNUAL_GATE_BOARDS_STATUS);
            ui.label("岁计门盘严格采用本段‘开门加太乙、主大将、客大将’：三个锚点分别独立旋转。若主大将或客大将在中五宫，只停用对应子盘，不投影到外宫；定计大将尚未生成。");
            ui.separator();
            ui.label("七十二局输入边界：只接受正整数局序，并要求调用方明确选择阳遁或阴遁；不会从当前日期、节气或时刻自动选遁求局。");
            ui.monospace(TAIYI_SEVENTY_TWO_BOARD_ALGORITHM_VERSION);
            ui.monospace(TAIYI_SEVENTY_TWO_BOARD_STATUS);
            ui.label("阴局卷三表有 3 个客目和 9 个主客算单元与通用公式冲突，已逐项留档；实现保留公式结果，不把冲突表值静默覆盖进去。");
            ui.label("仍阻止：两套历元选择、日期求局、自动选遁、時計直门旋转盘、定计大将门盘、中五投影、关判定与复合格局、三门五将结论、断法、信号、应期与行动建议。");
        });
        ui.add_space(18.0);
        ui.label(RichText::new("九术适用性检查").size(18.0).strong());
        egui::Grid::new("applicability_grid")
            .num_columns(3)
            .striped(true)
            .spacing([20.0, 9.0])
            .show(ui, |ui| {
                ui.strong("术式");
                ui.strong("状态");
                ui.strong("原因");
                ui.end_row();
                for item in &self.applicability {
                    ui.label(item.method.zh_name());
                    ui.colored_label(status_color(item.status), status_label(item.status));
                    ui.label(item.reasons.join("；"));
                    ui.end_row();
                }
            });
        ui.add_space(14.0);
        warning_box(
            ui,
            "融合引擎未收到任何通过证据门禁的中间信号，因此拒绝生成总分、应期、结论或古法行动建议。",
            Color32::from_rgb(150, 92, 34),
        );
    }

    fn render_about(&mut self, ui: &mut egui::Ui) {
        section_title(
            ui,
            "范围与声明",
            "软件能力边界会在每次版本中明确标注。",
            "界",
        );
        ui.add_space(18.0);
        ui.heading("完全离线");
        ui.label("不登录、不注册、不连接服务器、不调用 API，也不包含生成式 AI。所有占问与校验数据只写入本机 SQLite 数据库。");
        ui.add_space(12.0);
        ui.heading("证据先于功能");
        ui.label("争议大、版本残缺、规则条件不闭合或未经复核的材料直接停用。现代常见说法与高等级早期文献冲突时，以经过版本核对的早期权威经典为准。");
        ui.add_space(12.0);
        ui.heading("项目验收状态");
        match &self.project_acceptance {
            Ok(checklist) => {
                let passed = checklist
                    .items
                    .iter()
                    .filter(|item| item.status == ProjectAcceptanceStatus::Passed)
                    .count();
                let in_progress = checklist
                    .items
                    .iter()
                    .filter(|item| item.status == ProjectAcceptanceStatus::InProgress)
                    .count();
                let blocked = checklist.items.len() - passed - in_progress;
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(
                        ProjectAcceptanceStatus::Passed.color(),
                        format!("已通过 {passed} 项"),
                    );
                    ui.separator();
                    ui.colored_label(
                        ProjectAcceptanceStatus::InProgress.color(),
                        format!("进行中 {in_progress} 项"),
                    );
                    ui.separator();
                    ui.colored_label(
                        ProjectAcceptanceStatus::Blocked.color(),
                        format!("未通过/受阻 {blocked} 项"),
                    );
                });
                ui.small(format!(
                    "版本 {} · 核验日期 {}。{}",
                    checklist.product_version, checklist.last_verified, checklist.measurement_rule
                ));
                ui.collapsing("查看16项逐项证据和剩余工作", |ui| {
                    for item in &checklist.items {
                        egui::CollapsingHeader::new(format!(
                            "{} · {} · {}",
                            item.id,
                            item.title,
                            item.status.zh_label()
                        ))
                        .show(ui, |ui| {
                            ui.label(format!("已有证据：{}", item.evidence));
                            ui.colored_label(
                                item.status.color(),
                                format!("剩余验收：{}", item.remaining),
                            );
                        });
                    }
                });
            }
            Err(error) => warning_box(ui, error, Color32::from_rgb(167, 54, 54)),
        }
        ui.add_space(12.0);
        ui.heading("当前工程边界");
        ui.label("本版已经完成工程骨架和审计门禁，但九术古籍规则库仍在研究阶段，因此不会输出占断。地点库含 3290 条 GeoNames 行政中心，以及 34535 条经唯一匹配区县中心坐标的乡镇街道名称。街道地点仍采用区县中心近似坐标；其余约 4188 条缺可信坐标的乡镇街道暂不可选，绝不冒充街道实测点。");
        ui.add_space(12.0);
        ui.heading("健康安全");
        ui.label("健康、疾病和生死相关内容只能作为传统术数文化参考，不能用于医学诊断、治疗选择或延误就医。古法依据等级也不代表现实概率。");
        ui.add_space(12.0);
        ui.heading("数据来源与许可");
        ui.label("街道名称来自国家地名信息库的开源整理快照；坐标沿用 GeoNames 区县中心。两种来源及其许可证均保留在本机交付物中。");
        ui.collapsing("查看完整第三方说明", |ui| {
            ui.monospace(include_str!("../../../docs/THIRD_PARTY_NOTICES.md"));
        });
    }

    fn try_lock_question(&mut self) {
        self.error_message = None;
        self.pending_duplicate = None;
        if self.question_text.trim().is_empty() {
            self.error_message = Some("请先填写明确的问题原文。".to_owned());
            return;
        }
        let Some(location) = self.locations.get(self.location_index).cloned() else {
            self.error_message = Some("离线地点库没有可用记录。".to_owned());
            return;
        };
        let classification = match self
            .classifier
            .classify_detailed(self.category, &self.question_text)
        {
            Ok(classification) => classification,
            Err(error) => {
                self.error_message = Some(classification_error_zh(&error));
                return;
            }
        };
        let divination_time = match self.selected_time() {
            Ok(time) => time,
            Err(message) => {
                self.error_message = Some(message);
                return;
            }
        };
        let birth = match self.selected_birth_parameters() {
            Ok(birth) => birth,
            Err(message) => {
                self.error_message = Some(message);
                return;
            }
        };
        let options = self
            .options_text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if options.len() == 1 {
            self.error_message = Some("多选项比较至少需要两个选项。".to_owned());
            return;
        }
        let now = Utc::now().with_timezone(&beijing_zone());
        let question = QuestionInput {
            id: Uuid::new_v4(),
            category: self.category,
            subcategory: self.subcategory.trim().to_owned(),
            text: self.question_text.trim().to_owned(),
            subject: self.subject.trim().to_owned(),
            object: self.object.trim().to_owned(),
            target_event: if self.target_event.trim().is_empty() {
                self.question_text.trim().to_owned()
            } else {
                self.target_event.trim().to_owned()
            },
            result_terms: classification.matched_terms.clone(),
            options,
            location,
            divination_time_bjt: divination_time,
            target_time_text: self.target_time_text.trim().to_owned(),
            birth,
            locked_at_bjt: now,
        };
        let validation = validate_question_parameters(&question);
        if !validation.can_lock {
            self.error_message = Some(format!("参数校验未通过：{}", validation.errors.join("；")));
            return;
        }
        let fingerprint = build_event_fingerprint(&question, &classification);
        match self.storage.find_duplicate_event(&fingerprint) {
            Ok(Some(duplicate)) => {
                self.pending_duplicate = Some(duplicate);
                self.pending_question = Some(question);
                self.pending_classification = Some(classification);
                self.pending_fingerprint = Some(fingerprint);
            }
            Ok(None) => self.commit_new_question(question, classification, fingerprint, None),
            Err(error) => self.error_message = Some(error.to_string()),
        }
    }

    fn commit_revision(&mut self) {
        if self.revision_description.trim().len() < 4 {
            self.error_message = Some("请具体说明现实情况发生了什么实质变化。".to_owned());
            return;
        }
        let (Some(duplicate), Some(question), Some(classification), Some(fingerprint)) = (
            self.pending_duplicate.take(),
            self.pending_question.take(),
            self.pending_classification.take(),
            self.pending_fingerprint.take(),
        ) else {
            self.error_message = Some("没有可建立修订的占问。".to_owned());
            return;
        };
        let revision = EventRevision {
            id: Uuid::new_v4(),
            prior_question_id: duplicate.question_id,
            change_description: self.revision_description.trim().to_owned(),
            created_at_bjt: Utc::now().with_timezone(&beijing_zone()),
            match_report: Some(EventMatchReport {
                algorithm_version: duplicate.algorithm_version,
                same_event: true,
                exact_digest: duplicate.exact_digest,
                score_percent: duplicate.score_percent,
                reasons: duplicate.match_reasons,
            }),
        };
        self.commit_new_question(question, classification, fingerprint, Some(revision));
    }

    fn commit_new_question(
        &mut self,
        question: QuestionInput,
        classification: divination_core::Classification,
        fingerprint: EventFingerprint,
        revision: Option<EventRevision>,
    ) {
        match self.storage.insert_locked_question(
            &question,
            &classification,
            &fingerprint,
            revision.as_ref(),
        ) {
            Ok(()) => {
                let question_id = question.id;
                self.applicability = self.registry.assess_all(&question);
                self.current_question = Some(question);
                self.current_classification = Some(classification);
                self.current_fingerprint = Some(fingerprint);
                self.pending_duplicate = None;
                self.revision_description.clear();
                self.initialize_casting(question_id);
                self.refresh_history();
                self.page = Page::Casting;
            }
            Err(error) => self.error_message = Some(error.to_string()),
        }
    }

    fn selected_time(&self) -> Result<DateTime<FixedOffset>, String> {
        let zone = beijing_zone();
        if self.use_current_time {
            return Ok(Utc::now().with_timezone(&zone));
        }
        let parsed = parse_proleptic_gregorian_datetime(self.manual_time_text.trim())
            .map_err(|_| calendar_input_format_error("指定时间"))?;
        zone.from_local_datetime(&parsed)
            .single()
            .ok_or_else(|| "指定时间无法按北京时间解释。".to_owned())
    }

    fn selected_birth_parameters(&self) -> Result<Option<BirthParameters>, String> {
        if self.category != QuestionCategory::LongTermFate || !self.include_birth_parameters {
            return Ok(None);
        }
        if self.birth_time_text.trim().is_empty() {
            return Err("请填写出生时间，或取消“本次提供完整出生参数”。".to_owned());
        }
        let naive = parse_proleptic_gregorian_datetime(self.birth_time_text.trim())
            .map_err(|_| calendar_input_format_error("出生时间"))?;
        let date_time_bjt = beijing_zone()
            .from_local_datetime(&naive)
            .single()
            .ok_or_else(|| "出生时间无法按北京时间解析。".to_owned())?;
        let location = self
            .locations
            .get(self.birth_location_index)
            .cloned()
            .ok_or_else(|| "离线地点库没有可用的出生地点记录。".to_owned())?;
        Ok(Some(BirthParameters {
            sex: self.birth_sex,
            date_time_bjt,
            location,
        }))
    }

    fn initialize_casting(&mut self, question_id: Uuid) {
        self.casting_tab = CastingTab::Liuyao;
        self.casting_message = None;
        self.liuyao_transcript = Some(LiuyaoTranscript::with_random_nonce(question_id));
        self.liuyao_cast = None;
        self.liuyao_lines.clear();
        self.liuyao_session_started = None;
        self.liuyao_press_started = None;
        self.liuyao_previous_event = None;
        self.liuyao_pointer_path.clear();
        self.yarrow_session = YarrowSession::default();
        self.yarrow_cast = None;
        self.yarrow_left_pile = 24;
        self.yarrow_phase = YarrowUiPhase::Divide;
        self.yarrow_pending_left = None;
        self.yarrow_last_change = None;
    }

    fn refresh_history(&mut self) {
        match self.storage.recent_questions(200) {
            Ok(items) => {
                let selected = self
                    .history_selected
                    .filter(|selected| items.iter().any(|summary| summary.question.id == *selected))
                    .or_else(|| items.first().map(|summary| summary.question.id));
                self.history_items = items;
                self.history_selected = selected;
                self.history_audit = None;
                self.history_message = None;
                self.history_detail = selected.and_then(|question_id| {
                    match self.storage.load_question_detail(question_id) {
                        Ok(detail) => detail,
                        Err(error) => {
                            self.history_message = Some(error.to_string());
                            None
                        }
                    }
                });
            }
            Err(error) => {
                self.history_items.clear();
                self.history_detail = None;
                self.history_audit = None;
                self.history_message = Some(error.to_string());
            }
        }
    }

    fn select_history(&mut self, question_id: Uuid) {
        match self.storage.load_question_detail(question_id) {
            Ok(Some(detail)) => {
                self.history_selected = Some(question_id);
                self.history_detail = Some(detail);
                self.history_audit = None;
                self.history_message = None;
            }
            Ok(None) => {
                self.history_message = Some("所选历史记录已不存在。".to_owned());
            }
            Err(error) => self.history_message = Some(error.to_string()),
        }
    }

    fn audit_selected_history(&mut self) {
        let Some(question_id) = self.history_selected else {
            self.history_message = Some("请先选择一条历史记录。".to_owned());
            return;
        };
        match self.storage.audit_history_replay(question_id) {
            Ok(audit) => {
                self.history_audit = Some(audit);
                self.history_message = Some(
                    "已从 SQLite 重新读取材料并重新计算；现有起法结果与保存值一致。".to_owned(),
                );
            }
            Err(error) => {
                self.history_audit = None;
                self.history_message = Some(format!("重放校验失败：{error}"));
            }
        }
    }
}

impl eframe::App for JiuhengApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_top_bar(ctx);
        self.render_side_bar(ctx);
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(253, 252, 248))
                    .inner_margin(egui::Margin::symmetric(34, 24)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| match self.page {
                    Page::NewQuestion => self.render_new_question(ui),
                    Page::History => self.render_history(ui),
                    Page::Casting => self.render_casting(ui),
                    Page::Result => self.render_result(ui),
                    Page::Developer => self.render_developer(ui),
                    Page::About => self.render_about(ui),
                });
            });
    }
}

fn open_storage() -> (Storage, PathBuf, Option<String>) {
    let preferred = std::env::var_os("JIUHENG_DATA_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|directory| directory.join("jiuheng.sqlite3"))
        .or_else(|| {
            ProjectDirs::from("cn", "Jiuheng", APP_NAME)
                .map(|dirs| dirs.data_local_dir().join("jiuheng.sqlite3"))
        })
        .unwrap_or_else(|| PathBuf::from("jiuheng.sqlite3"));
    let parent_result = preferred.parent().map(std::fs::create_dir_all).transpose();
    match parent_result.and_then(|_| Storage::open(&preferred).map_err(std::io::Error::other)) {
        Ok(storage) => (storage, preferred, None),
        Err(error) => {
            let fallback = Storage::open_in_memory().expect("embedded SQLite schema must be valid");
            (
                fallback,
                PathBuf::from(":memory:"),
                Some(format!(
                    "本地数据库暂不可写，当前会话使用临时内存库：{error}"
                )),
            )
        }
    }
}

fn configure_fonts_and_style(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    for path in [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert("jiuheng_cjk".to_owned(), FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "jiuheng_cjk".to_owned());
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "jiuheng_cjk".to_owned());
            break;
        }
    }
    ctx.set_fonts(fonts);
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::light();
    style.visuals.panel_fill = Color32::from_rgb(253, 252, 248);
    style.visuals.selection.bg_fill = Color32::from_rgb(178, 78, 63);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(241, 238, 229);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(229, 222, 205);
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    ctx.set_style(style);
}

fn nav_button(ui: &mut egui::Ui, page: &mut Page, target: Page, label: &str) {
    let selected = *page == target;
    let text = if selected {
        RichText::new(label).strong().color(Color32::WHITE)
    } else {
        RichText::new(label).color(Color32::from_rgb(212, 219, 213))
    };
    if ui
        .add_sized(
            [ui.available_width(), 38.0],
            egui::Button::new(text).fill(if selected {
                Color32::from_rgb(71, 91, 80)
            } else {
                Color32::TRANSPARENT
            }),
        )
        .clicked()
    {
        *page = target;
    }
}

fn section_title(ui: &mut egui::Ui, title: &str, subtitle: &str, seal: &str) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(Color32::from_rgb(154, 67, 55))
            .inner_margin(egui::Margin::symmetric(9, 6))
            .show(ui, |ui| {
                ui.label(RichText::new(seal).color(Color32::WHITE).strong());
            });
        ui.vertical(|ui| {
            ui.label(
                RichText::new(title)
                    .size(25.0)
                    .strong()
                    .color(Color32::from_rgb(43, 61, 53)),
            );
            ui.label(
                RichText::new(subtitle)
                    .size(13.0)
                    .color(Color32::from_rgb(104, 109, 98)),
            );
        });
    });
}

fn labeled_text(ui: &mut egui::Ui, label: &str, hint: &str, value: &mut String) {
    ui.label(RichText::new(label).strong());
    ui.add(egui::TextEdit::singleline(value).hint_text(hint));
}

fn warning_box(ui: &mut egui::Ui, message: &str, color: Color32) {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            18,
        ))
        .stroke(egui::Stroke::new(1.0_f32, color))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.colored_label(color, message);
        });
}

fn render_rule_evidence_dossier(ui: &mut egui::Ui, dossier: &RuleEvidenceDossier) {
    ui.small(
        "本档案直接读取 SQLite 当前记录并重新计算三类哈希；只用于复核，不提供放行或修改按钮。",
    );
    egui::Grid::new(format!("rule_dossier_integrity_{}", dossier.rule.rule_id))
        .num_columns(3)
        .striped(true)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            for (label, stored, calculated, matches) in [
                (
                    "规则正文哈希",
                    &dossier.rule_content_hash,
                    &dossier.calculated_rule_content_hash,
                    dossier.rule_content_hash_matches,
                ),
                (
                    "文献证据包哈希",
                    &dossier.evidence_bundle_hash,
                    &dossier.calculated_evidence_bundle_hash,
                    dossier.evidence_bundle_hash_matches,
                ),
                (
                    "测试套件哈希",
                    &dossier.test_suite_hash,
                    &dossier.calculated_test_suite_hash,
                    dossier.test_suite_hash_matches,
                ),
            ] {
                ui.colored_label(
                    if matches {
                        Color32::from_rgb(45, 112, 82)
                    } else {
                        Color32::from_rgb(167, 54, 54)
                    },
                    if matches { "一致" } else { "不一致" },
                );
                ui.label(label);
                ui.vertical(|ui| {
                    ui.monospace(format!("存储：{stored}"));
                    ui.monospace(format!("重算：{calculated}"));
                });
                ui.end_row();
            }
        });

    let question_types = dossier
        .rule
        .applicable_question_types
        .iter()
        .map(AncientQuestionType::zh_name)
        .collect::<Vec<_>>()
        .join("、");
    ui.collapsing("规则正文与确定性程序", |ui| {
        egui::Grid::new(format!("rule_dossier_fields_{}", dossier.rule.rule_id))
            .num_columns(2)
            .striped(true)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label("Rule ID");
                ui.monospace(&dossier.rule.rule_id);
                ui.end_row();
                ui.label("术式 / 适用问题");
                ui.label(format!(
                    "{} / {}",
                    dossier.rule.method.zh_name(),
                    question_types
                ));
                ui.end_row();
                ui.label("证据等级 / 稳定性 / reviewed");
                ui.label(format!(
                    "{} / {} / {}",
                    evidence_tier_label(dossier.rule.evidence_tier),
                    rule_stability_label(dossier.rule.stability),
                    dossier.rule.reviewed
                ));
                ui.end_row();
                ui.label("时代 / 书名");
                ui.label(format!(
                    "{} / 《{}》",
                    dossier.rule.historical_period, dossier.rule.book_title
                ));
                ui.end_row();
                ui.label("卷 / 篇");
                ui.label(format!(
                    "{} / {}",
                    dossier.rule.volume, dossier.rule.chapter
                ));
                ui.end_row();
                ui.label("版本 / 来源定位");
                ui.label(format!(
                    "{} / {}",
                    dossier.rule.edition, dossier.rule.source_reference
                ));
                ui.end_row();
                ui.label("实现版本 / 优先级");
                ui.label(format!(
                    "{} / {}",
                    dossier.rule.implementation_version, dossier.rule.priority
                ));
                ui.end_row();
            });
        ui.label("规则原文");
        ui.label(&dossier.rule.original_text);
        ui.label("异文记录");
        ui.label(&dossier.rule.textual_variants);
        ui.label("条件程序");
        ui.monospace(&dossier.rule.condition_expression);
        ui.label("结果程序");
        ui.monospace(&dossier.rule.result_expression);
        ui.label("应期程序");
        ui.monospace(dossier.rule.timing_expression.as_deref().unwrap_or("无"));
    });

    ui.collapsing(
        format!("来源与原文片段（{}）", dossier.sources.len()),
        |ui| {
            if dossier.sources.is_empty() {
                ui.small("没有关联来源。影印、转录和卷页门禁不会通过。");
            }
            for source in &dossier.sources {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(source.relation.zh_name());
                        ui.monospace(&source.edition.source_id);
                        ui.label(source.edition.edition_type.zh_name());
                        ui.label(source.passage.transcription_status.zh_name());
                    });
                    ui.label(format!(
                        "《{}》·{}，{}，{}，目录号 {}",
                        source.edition.canonical_title,
                        source.edition.edition_title,
                        source.edition.holding_institution,
                        source.edition.edition_year,
                        source.edition.catalog_identifier
                    ));
                    ui.label(format!(
                        "{} / {} / {}；影印定位：{}",
                        source.passage.volume,
                        source.passage.chapter,
                        source.passage.folio_or_page,
                        source.edition.facsimile_reference
                    ));
                    ui.label(format!("关联说明：{}", source.link_notes));
                    ui.label("原文：");
                    ui.label(&source.passage.original_text);
                    ui.label(format!("异文：{}", source.passage.textual_variants));
                    ui.monospace(format!("登记 SHA-256：{}", source.edition.digital_checksum));
                    match &source.facsimile_verification {
                        Some(verification) => {
                            ui.monospace(format!(
                                "实际字节核验：{} / {} 字节 / {} / {} / {}",
                                verification.digital_checksum,
                                verification.byte_length,
                                verification.file_name,
                                verification.verifier_version,
                                verification.verified_at
                            ));
                        }
                        None => {
                            ui.colored_label(
                                Color32::from_rgb(167, 54, 54),
                                "没有实际影印文件字节核验记录。",
                            );
                        }
                    }
                });
            }
        },
    );

    ui.collapsing(
        format!("四类复核记录（{}）", dossier.reviews.len()),
        |ui| {
            if dossier.reviews.is_empty() {
                ui.small("没有复核记录。");
            }
            for review in &dossier.reviews {
                let current = review.rule_content_hash == dossier.rule_content_hash
                    && review.evidence_bundle_hash == dossier.evidence_bundle_hash;
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(review.kind.zh_name());
                        ui.colored_label(
                            if review.decision == divination_storage::ReviewDecision::Approved {
                                Color32::from_rgb(45, 112, 82)
                            } else {
                                Color32::from_rgb(167, 54, 54)
                            },
                            review.decision.zh_name(),
                        );
                        ui.label(if current {
                            "绑定当前哈希"
                        } else {
                            "历史/已失效"
                        });
                    });
                    ui.label(format!(
                        "复核人：{}；时间：{}；记录：{}",
                        review.reviewer, review.reviewed_at, review.review_id
                    ));
                    ui.label(&review.evidence_notes);
                    ui.monospace(format!("rule_hash: {}", review.rule_content_hash));
                    ui.monospace(format!("evidence_hash: {}", review.evidence_bundle_hash));
                });
            }
        },
    );

    ui.collapsing(
        format!("规则测试用例（{}）", dossier.test_cases.len()),
        |ui| {
            if dossier.test_cases.is_empty() {
                ui.small("没有测试用例。");
            }
            for test_case in &dossier.test_cases {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.monospace(&test_case.test_case_id);
                        ui.strong(&test_case.name);
                        ui.label(if test_case.enabled {
                            "启用"
                        } else {
                            "停用"
                        });
                        ui.label(format!("边界：{}", test_case.boundary_class));
                    });
                    ui.label(format!("依据：{}", test_case.source_basis));
                    ui.monospace(format!(
                        "输入：{}",
                        serde_json::to_string(&test_case.input)
                            .unwrap_or_else(|error| format!("<序列化失败：{error}>"))
                    ));
                    ui.monospace(format!(
                        "预期：{}",
                        serde_json::to_string(&test_case.expected_output)
                            .unwrap_or_else(|error| format!("<序列化失败：{error}>"))
                    ));
                });
            }
        },
    );

    ui.collapsing(
        format!("测试运行记录（{}）", dossier.test_runs.len()),
        |ui| {
            if dossier.test_runs.is_empty() {
                ui.small("没有测试运行记录。");
            }
            for run in &dossier.test_runs {
                let current = run.rule_content_hash == dossier.rule_content_hash
                    && run.test_suite_hash == dossier.test_suite_hash;
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.monospace(&run.run_id);
                        ui.label(if current {
                            "绑定当前套件"
                        } else {
                            "历史/已失效"
                        });
                        ui.label(format!(
                            "通过 {} / 失败 {} / 总计 {}",
                            run.passed_count, run.failed_count, run.total_count
                        ));
                    });
                    ui.label(format!(
                        "实现：{}；时间：{}",
                        run.implementation_version, run.executed_at
                    ));
                    ui.monospace(format!("rule_hash: {}", run.rule_content_hash));
                    ui.monospace(format!("suite_hash: {}", run.test_suite_hash));
                    ui.monospace(format!(
                        "环境：{}",
                        serde_json::to_string(&run.environment)
                            .unwrap_or_else(|error| format!("<序列化失败：{error}>"))
                    ));
                });
            }
        },
    );

    ui.small(format!(
        "最终门禁：字段 {} / 主来源 {} / 复核 {} / 测试 {} / 可执行 {}",
        dossier.gate.rule_fields_ready,
        dossier.gate.primary_source_ready,
        dossier.gate.reviews_ready,
        dossier.gate.tests_ready,
        dossier.gate.may_execute()
    ));
}

const fn evidence_tier_label(tier: divination_core::EvidenceTier) -> &'static str {
    match tier {
        divination_core::EvidenceTier::EarlyCoreClassic => "先秦—汉唐核心经典",
        divination_core::EvidenceTier::EarlyAuthoritativeCommentary => "权威早期注本",
        divination_core::EvidenceTier::LaterAuthoritativeCommentary => "宋元明清权威注本",
        divination_core::EvidenceTier::Unverified => "未核验",
    }
}

const fn rule_stability_label(stability: divination_core::RuleStability) -> &'static str {
    match stability {
        divination_core::RuleStability::Stable => "稳定",
        divination_core::RuleStability::Disputed => "有争议",
        divination_core::RuleStability::Fragmentary => "残缺",
        divination_core::RuleStability::Unreviewed => "未复核",
    }
}

fn location_label(location: &GeoLocation) -> String {
    let mut parts = vec![location.province.as_str()];
    if location.city != location.province {
        parts.push(location.city.as_str());
    }
    if let Some(district) = location.district.as_deref() {
        if !parts.contains(&district) {
            parts.push(district);
        }
    }
    if let Some(street) = location.street.as_deref() {
        if !parts.contains(&street) {
            parts.push(street);
        }
    }
    parts.join(" · ")
}

fn location_matches_search(location: &GeoLocation, search: &str) -> bool {
    location.province.contains(search)
        || location.city.contains(search)
        || location
            .district
            .as_deref()
            .is_some_and(|district| district.contains(search))
        || location
            .street
            .as_deref()
            .is_some_and(|street| street.contains(search))
}

fn location_precision_notice(location: &GeoLocation) -> &'static str {
    match (location.coordinate_precision, location.street.is_some()) {
        (CoordinatePrecision::StreetCenter, _) => "使用街道中心近似坐标，非门牌或 GPS 实测点。",
        (CoordinatePrecision::DistrictCenter, true) => {
            "已选街道；天文计算仍使用区县中心近似坐标，非街道实测点。"
        }
        (CoordinatePrecision::DistrictCenter, false) => "天文计算使用区县中心近似坐标。",
        (CoordinatePrecision::CityCenter, _) => "天文计算使用城市中心近似坐标。",
    }
}

const fn biological_sex_label(sex: BiologicalSex) -> &'static str {
    match sex {
        BiologicalSex::Female => "女",
        BiologicalSex::Male => "男",
    }
}

fn beijing_zone() -> FixedOffset {
    FixedOffset::east_opt(BJT_SECONDS).expect("UTC+8 is valid")
}

fn parse_proleptic_gregorian_datetime(value: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
}

fn calendar_input_format_error(label: &str) -> String {
    format!(
        "{label}格式应为 YYYY-MM-DD HH:MM:SS；公元前使用天文年编号，例如 -0999 表示公元前 1000 年。"
    )
}

fn historical_datetime_from_bjt(value: DateTime<FixedOffset>) -> HistoricalDateTime {
    HistoricalDateTime {
        year: value.year(),
        month: value.month() as u8,
        day: value.day() as u8,
        hour: value.hour() as u8,
        minute: value.minute() as u8,
        second: value.second() as u8,
    }
}

const fn calendar_system_label(system: CalendarSystem) -> &'static str {
    match system {
        CalendarSystem::Gregorian => "格里高利历",
        CalendarSystem::Julian => "儒略历",
        CalendarSystem::HistoricalCutover => "1582 改革历",
    }
}

fn status_label(status: ApplicabilityStatus) -> &'static str {
    match status {
        ApplicabilityStatus::Applicable => "可参与",
        ApplicabilityStatus::InsufficientEvidence => "证据不足",
        ApplicabilityStatus::MissingParameters => "参数不足",
        ApplicabilityStatus::UnsupportedQuestionType => "不适用",
        ApplicabilityStatus::NotImplemented => "未实现",
    }
}

fn status_color(status: ApplicabilityStatus) -> Color32 {
    match status {
        ApplicabilityStatus::Applicable => Color32::from_rgb(47, 120, 78),
        ApplicabilityStatus::InsufficientEvidence => Color32::from_rgb(150, 92, 34),
        ApplicabilityStatus::MissingParameters => Color32::from_rgb(150, 92, 34),
        ApplicabilityStatus::UnsupportedQuestionType => Color32::from_rgb(104, 109, 98),
        ApplicabilityStatus::NotImplemented => Color32::from_rgb(167, 54, 54),
    }
}

fn ancient_type_label(kind: &AncientQuestionType) -> &'static str {
    match kind {
        AncientQuestionType::RelationshipOutcome => "relationship_outcome",
        AncientQuestionType::ProjectOutcome => "project_outcome",
        AncientQuestionType::ProfitAndLoss => "profit_and_loss",
        AncientQuestionType::WeatherRain => "weather_rain",
        AncientQuestionType::TravelSafety => "travel_safety",
        AncientQuestionType::MissingPersonOrObject => "missing_person_or_object",
        AncientQuestionType::CompetitionOutcome => "competition_outcome",
        AncientQuestionType::ExaminationOutcome => "examination_outcome",
        AncientQuestionType::TransactionOutcome => "transaction_outcome",
        AncientQuestionType::LitigationOutcome => "litigation_outcome",
        AncientQuestionType::DwellingFortune => "dwelling_fortune",
        AncientQuestionType::NatalLongTerm => "natal_long_term",
        AncientQuestionType::HealthTrendReference => "health_trend_reference",
    }
}

fn classification_error_zh(error: &ClassificationError) -> String {
    match error {
        ClassificationError::NoMatchingType => {
            "“其他”问题未命中已知的古代问题类型；请选固定类别或明确问题主题。".to_owned()
        }
        ClassificationError::AmbiguousTypes(types) => format!(
            "“其他”同时命中{}；请选固定类别，或把问题收窄为一件事。",
            types
                .iter()
                .map(AncientQuestionType::zh_name)
                .collect::<Vec<_>>()
                .join("、")
        ),
    }
}

fn health_notice_required_for_input(
    category: QuestionCategory,
    other_preview: Option<&Result<divination_core::Classification, ClassificationError>>,
) -> bool {
    category == QuestionCategory::HealthReference
        || matches!(
            other_preview,
            Some(Ok(classification))
                if classification.ancient_type == AncientQuestionType::HealthTrendReference
        )
        || matches!(
            other_preview,
            Some(Err(ClassificationError::AmbiguousTypes(types)))
                if types.contains(&AncientQuestionType::HealthTrendReference)
        )
}

fn duration_nanos(duration: std::time::Duration) -> u64 {
    duration.as_nanos().min(u64::MAX as u128) as u64
}

fn point_to_micro(value: f32) -> i32 {
    (value as f64 * 1_000_000.0)
        .round()
        .clamp(i32::MIN as f64, i32::MAX as f64) as i32
}

fn line_symbol(line_type: divination_core::LineType) -> &'static str {
    match (line_type.primary_is_yang(), line_type.is_moving()) {
        (true, true) => "━━━━━━  ○",
        (true, false) => "━━━━━━",
        (false, true) => "━━  ━━  ×",
        (false, false) => "━━  ━━",
    }
}

fn history_state_label(state: &str) -> &'static str {
    match state {
        "locked" => "已锁题",
        "completed" => "已完成",
        "not_executable" => "证据门禁未开放",
        "failed" => "执行失败",
        _ => "未知状态",
    }
}

fn ziwei_palace_sequence_zh(chart: &divination_core::ZiweiLifeBodyChart) -> String {
    chart
        .palaces
        .iter()
        .map(|palace| format!("{}{}", palace.branch.zh_name(), palace.ui_zh_name))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn ziwei_branch_stem_sequence_zh(candidate: &divination_core::ZiweiBureauCandidate) -> String {
    candidate
        .branch_stems
        .iter()
        .map(|palace| palace.designation.zh_name())
        .collect::<Vec<_>>()
        .join(" · ")
}

fn ziwei_year_convention_label(convention: &str) -> &'static str {
    match convention {
        "modern_nominal_lunar_year_label.v1" => "现代农历名义年",
        "astronomical_lichun_year_candidate.v1" => "立春换年候选",
        _ => "未知年干口径",
    }
}

fn qizheng_siyu_identity_compact_zh(audit: &QizhengSiyuIdentityAudit) -> String {
    let seven = audit
        .seven_governors
        .iter()
        .map(|item| item.received_zh_name.as_str())
        .collect::<Vec<_>>()
        .join(" · ");
    let four = audit
        .four_residuals
        .iter()
        .map(|item| {
            format!(
                "{}（{}余）",
                item.normalized_zh_name,
                item.five_phase.zh_name()
            )
        })
        .collect::<Vec<_>>()
        .join(" · ");
    format!("七政：{seven}；四余：{four}")
}

fn qizheng_planetary_research_lines(audit: &BirthTimeAuditSnapshot) -> Vec<String> {
    match &audit.qizheng_five_planet_geometric_observations {
        Some(observations) if !observations.is_empty() => {
            let mut lines = vec![format!(
                "五星近似研究向量：J2000 黄道 / 行星相对地月质心 / TT≈TDB / {}",
                observations[0].algorithm_version
            )];
            lines.extend(observations.iter().map(|observation| {
                format!(
                    "{}：λ {:.4}°，β {:+.4}°，距离 {:.6} AU；非视地位置，不判宿度或吉凶",
                    observation.planet.received_zh_name(),
                    observation.emb_relative_longitude_deg,
                    observation.emb_relative_latitude_deg,
                    observation.emb_relative_distance_au
                )
            }));
            lines
        }
        _ => vec![
            match &audit.qizheng_five_planet_geometric_observation_error {
                Some(error) => format!("五星近似研究向量不可计算：{error}"),
                None => "旧记录未保存五星近似研究向量；不会用新版公式回填".to_owned(),
            },
        ],
    }
}

fn mansion_sequence_for_quadrant_zh(
    audit: &divination_core::ChineseAstrologyStructureAudit,
    quadrant: MansionQuadrant,
) -> String {
    audit
        .mansions
        .iter()
        .filter(|item| item.quadrant == quadrant)
        .map(|item| item.short_zh_name.as_str())
        .collect::<Vec<_>>()
        .join(" · ")
}

fn chinese_astrology_v1_fields_match(
    stored: &divination_core::ChineseAstrologyStructureAudit,
    current: &divination_core::ChineseAstrologyStructureAudit,
) -> bool {
    stored.audit_version == 1
        && stored.algorithm_version
            == divination_core::TWENTY_EIGHT_MANSION_CATALOG_ALGORITHM_VERSION
        && stored.status
            == "mechanical_received_identity_catalog_no_sky_boundary_or_interpretation.v1"
        && stored.three_enclosures.is_empty()
        && stored.mansions == current.mansions
        && stored.seven_luminaries.is_empty()
        && !stored.enclosure_boundary_model_selected
        && !stored.angular_boundary_model_selected
        && !stored.celestial_positions_calculated
        && !stored.interpretation_eligible
        && !stored.rule_eligible
        && stored.blocked_reasons
            == [
                "historical_determinative_star_and_angular_boundary_model_not_selected",
                "sun_moon_five_planets_guest_star_positions_not_calculated",
                "omen_text_edition_passage_conditions_and_counterexamples_not_reviewed",
            ]
}

fn three_enclosure_catalog_zh(audit: &divination_core::ChineseAstrologyStructureAudit) -> String {
    if audit.three_enclosures.is_empty() {
        return "旧快照未包含".to_owned();
    }
    audit
        .three_enclosures
        .iter()
        .map(|item| {
            format!(
                "{}{}（{}）",
                item.yuan_level.zh_name(),
                item.received_palace_name,
                item.zh_name
            )
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

fn seven_luminary_catalog_zh(audit: &divination_core::ChineseAstrologyStructureAudit) -> String {
    if audit.seven_luminaries.is_empty() {
        return "旧快照未包含".to_owned();
    }
    audit
        .seven_luminaries
        .iter()
        .map(|item| match item.five_phase {
            Some(phase) => format!(
                "{}（{}·{}）",
                item.received_zh_name,
                phase.zh_name(),
                item.modern_zh_name
            ),
            None => format!("{}（{}）", item.received_zh_name, item.modern_zh_name),
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

fn qimen_day_v1_fields_match(
    stored: &divination_core::QimenDayConventionCandidate,
    current: &divination_core::QimenDayConventionCandidate,
) -> bool {
    stored.day_convention == current.day_convention
        && stored.day_designation == current.day_designation
        && stored.symbol_head == current.symbol_head
        && stored.yuan == current.yuan
        && stored.bureau_number == current.bureau_number
        && stored.day_boundary_safe == current.day_boundary_safe
        && stored.mechanical_boundaries_safe == current.mechanical_boundaries_safe
}

fn qimen_ju_v1_fields_match(
    stored: &divination_core::QimenJuAudit,
    current: &divination_core::QimenJuAudit,
) -> bool {
    stored.algorithm_version == current.algorithm_version
        && stored.active_solar_term == current.active_solar_term
        && stored.next_solar_term == current.next_solar_term
        && stored.solar_term_boundary_safe == current.solar_term_boundary_safe
        && stored.dun == current.dun
        && stored.received_bureau_numbers == current.received_bureau_numbers
        && stored.table_entry_evidence_conflict == current.table_entry_evidence_conflict
        && qimen_day_v1_fields_match(
            &stored.beijing_midnight_candidate,
            &current.beijing_midnight_candidate,
        )
        && qimen_day_v1_fields_match(
            &stored.true_solar_midnight_candidate,
            &current.true_solar_midnight_candidate,
        )
        && stored.candidates_identical == qimen_old_day_candidates_identical(current)
        && stored.selected_day_convention == current.selected_day_convention
        && stored.transmission_calendar_adjustment_selected
            == current.transmission_calendar_adjustment_selected
        && stored.rule_eligible == current.rule_eligible
}

fn qimen_old_day_candidates_identical(audit: &divination_core::QimenJuAudit) -> bool {
    let beijing = &audit.beijing_midnight_candidate;
    let true_solar = &audit.true_solar_midnight_candidate;
    beijing.day_designation == true_solar.day_designation
        && beijing.symbol_head == true_solar.symbol_head
        && beijing.yuan == true_solar.yuan
        && beijing.bureau_number == true_solar.bureau_number
        && beijing.day_boundary_safe == true_solar.day_boundary_safe
}

fn qimen_ju_v2_fields_match(
    stored: &divination_core::QimenJuAudit,
    current: &divination_core::QimenJuAudit,
) -> bool {
    qimen_ju_v1_fields_match(stored, current)
        && stored.beijing_midnight_candidate.earth_plate
            == current.beijing_midnight_candidate.earth_plate
        && stored.true_solar_midnight_candidate.earth_plate
            == current.true_solar_midnight_candidate.earth_plate
}

fn qimen_hour_v1_fields_match(
    stored: &divination_core::QimenHourXunCandidate,
    current: &divination_core::QimenHourXunCandidate,
) -> bool {
    stored.algorithm_version == current.algorithm_version
        && stored.time_convention == current.time_convention
        && stored.hour_designation == current.hour_designation
        && stored.xun_head == current.xun_head
        && stored.hidden_instrument == current.hidden_instrument
        && stored.xun_head_palace == current.xun_head_palace
        && stored.hour_boundary_safe == current.hour_boundary_safe
        && stored.mechanical_boundaries_safe == current.mechanical_boundaries_safe
        && stored.interpretation_eligible == current.interpretation_eligible
        && stored.rule_eligible == current.rule_eligible
}

fn qimen_day_v3_fields_match(
    stored: &divination_core::QimenDayConventionCandidate,
    current: &divination_core::QimenDayConventionCandidate,
) -> bool {
    qimen_day_v1_fields_match(stored, current)
        && stored.earth_plate == current.earth_plate
        && match (
            stored.hour_xun_candidate.as_ref(),
            current.hour_xun_candidate.as_ref(),
        ) {
            (Some(stored), Some(current)) => qimen_hour_v1_fields_match(stored, current),
            (None, None) => true,
            _ => false,
        }
}

fn qimen_ju_v3_fields_match(
    stored: &divination_core::QimenJuAudit,
    current: &divination_core::QimenJuAudit,
) -> bool {
    stored.algorithm_version == current.algorithm_version
        && stored.active_solar_term == current.active_solar_term
        && stored.next_solar_term == current.next_solar_term
        && stored.solar_term_boundary_safe == current.solar_term_boundary_safe
        && stored.dun == current.dun
        && stored.received_bureau_numbers == current.received_bureau_numbers
        && stored.table_entry_evidence_conflict == current.table_entry_evidence_conflict
        && qimen_day_v3_fields_match(
            &stored.beijing_midnight_candidate,
            &current.beijing_midnight_candidate,
        )
        && qimen_day_v3_fields_match(
            &stored.true_solar_midnight_candidate,
            &current.true_solar_midnight_candidate,
        )
        && stored.candidates_identical == current.candidates_identical
        && stored.selected_day_convention == current.selected_day_convention
        && stored.selected_hour_convention == current.selected_hour_convention
        && stored.transmission_calendar_adjustment_selected
            == current.transmission_calendar_adjustment_selected
        && stored.rule_eligible == current.rule_eligible
}

fn render_replay_proof(
    ui: &mut egui::Ui,
    label: &str,
    proof: Option<&divination_storage::ArtifactReplayProof>,
    developer_mode: bool,
) {
    match proof {
        Some(proof) => {
            ui.colored_label(
                Color32::from_rgb(47, 120, 78),
                format!("{label}：{} 次操作，读取后重放一致", proof.operation_count),
            );
            ui.monospace(format!("本卦：{}", proof.primary_hexagram.compact_zh()));
            ui.monospace(format!("变卦：{}", proof.changed_hexagram.compact_zh()));
            if let Some(audit) = &proof.liuyao_six_spirits_audit {
                render_liuyao_six_spirits_audit(ui, audit, developer_mode);
            } else if let Some(audit) = &proof.liuyao_six_relatives_audit {
                render_liuyao_six_relatives_audit(ui, audit, developer_mode);
            } else if let Some(audit) = &proof.liuyao_najia_audit {
                render_liuyao_najia_audit(ui, audit, developer_mode);
            } else if let Some(audit) = &proof.liuyao_eight_palace_audit {
                ui.monospace(format!(
                    "八宫世应：{}（{}）",
                    audit.compact_zh(),
                    audit.palace_five_phase.zh_name()
                ));
                if developer_mode {
                    ui.small(format!(
                        "{} / {} / 规则准入：否",
                        audit.algorithm_version, audit.status
                    ));
                }
            }
            if let Some(audit) = &proof.liuyao_hidden_spirits_audit {
                render_liuyao_hidden_spirits_audit(ui, audit, developer_mode);
            }
            if let Some(audit) = &proof.liuyao_calendar_moving_audit {
                render_liuyao_calendar_moving_audit(ui, audit, developer_mode);
            }
            if let Some(audit) = &proof.liuyao_calendar_relations_audit {
                render_liuyao_calendar_relations_audit(ui, audit, developer_mode);
            }
            if let Some(audit) = &proof.liuyao_branch_pair_relations_audit {
                render_liuyao_branch_pair_relations_audit(ui, audit, developer_mode);
            }
            if let Some(audit) = &proof.liuyao_progress_retreat_audit {
                render_liuyao_progress_retreat_audit(ui, audit, developer_mode);
            }
            if let Some(audit) = &proof.liuyao_fanyin_fuyin_audit {
                render_liuyao_fanyin_fuyin_audit(ui, audit, developer_mode);
            }
            if let Some(audit) = &proof.liuyao_sanhe_audit {
                render_liuyao_sanhe_audit(ui, audit, developer_mode);
            }
            if developer_mode {
                ui.monospace(format!(
                    "{} / {} / {}",
                    proof.algorithm_version,
                    proof.transcript_hash,
                    proof.primary_hexagram.algorithm_version
                ));
            }
        }
        None => {
            ui.label(format!("{label}：未保存完整起法材料"));
        }
    }
}

fn render_liuyao_najia_audit(ui: &mut egui::Ui, audit: &LiuyaoNaJiaAudit, developer_mode: bool) {
    ui.monospace(format!(
        "八宫世应：{}（{}）",
        audit.eight_palace.compact_zh(),
        audit.eight_palace.palace_five_phase.zh_name()
    ));
    if developer_mode {
        egui::Grid::new((
            "liuyao_najia_lines",
            audit.eight_palace.hexagram.bits_bottom_up,
        ))
        .num_columns(4)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("爻位");
            ui.strong("上下卦");
            ui.strong("纳甲干支");
            ui.strong("世应");
            ui.end_row();
            for line in audit.lines_bottom_up.iter().rev() {
                ui.label(format!("第{}爻", line.line_number_bottom_up));
                ui.label(format!(
                    "{}{}",
                    line.trigram_position.zh_name(),
                    line.trigram.zh_name()
                ));
                ui.monospace(line.designation_zh());
                ui.label(if line.is_host {
                    "世"
                } else if line.is_response {
                    "应"
                } else {
                    "—"
                });
                ui.end_row();
            }
        });
        ui.small(format!(
            "{} / {} / 规则准入：否",
            audit.algorithm_version, audit.status
        ));
        ui.small("此表仅装纳甲天干、地支及支五行；六亲、六神、伏神与动变由后续独立机械审计显示，用神、旺衰、应期和断法未准入");
    } else {
        ui.small("纳甲干支机械表已生成；开发者模式可展开。六亲、六神和断法尚未准入。");
    }
}

fn render_liuyao_six_relatives_audit(
    ui: &mut egui::Ui,
    audit: &LiuyaoSixRelativesAudit,
    developer_mode: bool,
) {
    ui.monospace(format!(
        "八宫世应：{}（{}）",
        audit.najia.eight_palace.compact_zh(),
        audit.palace_element.zh_name()
    ));
    if developer_mode {
        egui::Grid::new((
            "liuyao_six_relatives_lines",
            audit.najia.eight_palace.hexagram.bits_bottom_up,
        ))
        .num_columns(5)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("爻位");
            ui.strong("上下卦");
            ui.strong("纳甲干支");
            ui.strong("六亲");
            ui.strong("世应");
            ui.end_row();
            for line in audit.lines_bottom_up.iter().rev() {
                ui.label(format!("第{}爻", line.najia.line_number_bottom_up));
                ui.label(format!(
                    "{}{}",
                    line.najia.trigram_position.zh_name(),
                    line.najia.trigram.zh_name()
                ));
                ui.monospace(line.najia.designation_zh());
                ui.label(line.relationship.zh_name());
                ui.label(if line.najia.is_host {
                    "世"
                } else if line.najia.is_response {
                    "应"
                } else {
                    "—"
                });
                ui.end_row();
            }
        });
        ui.small(format!(
            "{} / {} / 规则准入：否",
            audit.algorithm_version, audit.status
        ));
        ui.small("此表的六亲仅按宫五行与爻支五行机械定名；六神、伏神与动变由独立机械审计显示，用神、旺衰、应期和断法未准入");
    } else {
        ui.small("纳甲与六亲身份机械表已生成；开发者模式可展开。用神、旺衰和断法尚未准入。");
    }
}

fn load_liuyao_six_spirits_audit(
    storage: &Storage,
    question_id: Uuid,
    bits_bottom_up: u8,
) -> Result<Option<LiuyaoSixSpiritsAudit>, String> {
    let snapshot = storage
        .load_astronomy_snapshot(question_id)
        .map_err(|error| error.to_string())?;
    let Some(day_stem) = snapshot
        .and_then(|snapshot| snapshot.sexagenary_day_bjt_midnight)
        .map(|day| day.stem)
    else {
        return Ok(None);
    };
    liuyao_six_spirits_audit(bits_bottom_up, day_stem)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn render_liuyao_six_spirits_audit(
    ui: &mut egui::Ui,
    audit: &LiuyaoSixSpiritsAudit,
    developer_mode: bool,
) {
    ui.monospace(format!(
        "八宫世应：{}（{}）",
        audit.six_relatives.najia.eight_palace.compact_zh(),
        audit.six_relatives.palace_element.zh_name()
    ));
    ui.monospace(format!(
        "六神起例：{}日 · 初爻{}（北京时间午夜换日候选）",
        audit.day_stem.zh_name(),
        audit.first_line_spirit.zh_name()
    ));
    if developer_mode {
        egui::Grid::new((
            "liuyao_six_spirits_lines",
            audit
                .six_relatives
                .najia
                .eight_palace
                .hexagram
                .bits_bottom_up,
            audit.day_stem.index(),
        ))
        .num_columns(6)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("爻位");
            ui.strong("上下卦");
            ui.strong("纳甲干支");
            ui.strong("六亲");
            ui.strong("六神");
            ui.strong("世应");
            ui.end_row();
            for line in audit.lines_bottom_up.iter().rev() {
                let najia = &line.relative.najia;
                ui.label(format!("第{}爻", najia.line_number_bottom_up));
                ui.label(format!(
                    "{}{}",
                    najia.trigram_position.zh_name(),
                    najia.trigram.zh_name()
                ));
                ui.monospace(najia.designation_zh());
                ui.label(line.relative.relationship.zh_name());
                ui.label(line.spirit.zh_name());
                ui.label(if najia.is_host {
                    "世"
                } else if najia.is_response {
                    "应"
                } else {
                    "—"
                });
                ui.end_row();
            }
        });
        ui.small(format!(
            "{} / {} / 日界：{} / 规则准入：否",
            audit.algorithm_version, audit.status, audit.day_convention
        ));
        ui.small("六神只按存档日干机械轮排；伏神与动变由独立机械审计显示，用神、旺衰、应期、六神解释和断法未准入");
    } else {
        ui.small("纳甲、六亲与六神身份机械表已生成；开发者模式可展开。用神、旺衰和断法尚未准入。");
    }
}

fn render_liuyao_hidden_spirits_audit(
    ui: &mut egui::Ui,
    audit: &LiuyaoHiddenSpiritsAudit,
    developer_mode: bool,
) {
    let missing = audit
        .missing_relatives
        .iter()
        .map(|relationship| relationship.zh_name())
        .collect::<Vec<_>>()
        .join("、");
    if audit.hidden_lines_bottom_up.is_empty() {
        ui.small("伏神机械检查：本卦六亲齐全，无缺失六亲候选。");
    } else {
        ui.small(format!(
            "伏神机械检查：缺{}；从{}宫本宫卦取得 {} 个同位候选。",
            missing,
            audit.current.najia.eight_palace.palace.zh_name(),
            audit.hidden_lines_bottom_up.len()
        ));
    }
    if developer_mode && !audit.hidden_lines_bottom_up.is_empty() {
        egui::Grid::new((
            "liuyao_hidden_spirits_lines",
            audit.current.najia.eight_palace.hexagram.bits_bottom_up,
        ))
        .num_columns(4)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("爻位");
            ui.strong("伏神候选");
            ui.strong("伏神六亲");
            ui.strong("本卦飞神");
            ui.end_row();
            for line in audit.hidden_lines_bottom_up.iter().rev() {
                ui.label(format!("第{}爻下", line.line_number_bottom_up));
                ui.monospace(line.hidden_line.najia.designation_zh());
                ui.label(line.hidden_line.relationship.zh_name());
                ui.label(format!(
                    "{} {}",
                    line.flying_line.najia.designation_zh(),
                    line.flying_line.relationship.zh_name()
                ));
                ui.end_row();
            }
        });
    }
    if developer_mode {
        ui.small(format!(
            "{} / {} / 飞伏作用判断：否 / 规则准入：否",
            audit.algorithm_version, audit.status
        ));
    }
}

fn load_liuyao_sanhe_audit(
    storage: &Storage,
    question_id: Uuid,
    cast: &LiuyaoCast,
) -> Result<Option<LiuyaoSanheAudit>, String> {
    let snapshot = storage
        .load_astronomy_snapshot(question_id)
        .map_err(|error| error.to_string())?;
    let Some(snapshot) = snapshot else {
        return Ok(None);
    };
    let Some((month, day)) = snapshot
        .jie_month_candidate
        .as_ref()
        .zip(snapshot.sexagenary_day_bjt_midnight)
    else {
        return Ok(None);
    };
    liuyao_sanhe_audit(cast, month, day)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn render_liuyao_calendar_moving_audit(
    ui: &mut egui::Ui,
    audit: &LiuyaoCalendarMovingAudit,
    developer_mode: bool,
) {
    let moving = if audit.moving_line_numbers_bottom_up.is_empty() {
        "无动爻".to_owned()
    } else {
        format!(
            "动爻：{}",
            audit
                .moving_line_numbers_bottom_up
                .iter()
                .map(|line| format!("第{line}爻"))
                .collect::<Vec<_>>()
                .join("、")
        )
    };
    ui.small(format!(
        "月建日辰机械候选：{}月（{}建）· {}日；{}。",
        audit.month_designation.zh_name(),
        audit.month_designation.branch.zh_name(),
        audit.day_designation.zh_name(),
        moving
    ));
    if !audit.month_boundary_safe {
        ui.colored_label(
            Color32::from_rgb(167, 54, 54),
            "占问时刻接近节令边界，月建候选不安全；禁止进入规则。",
        );
    }
    if developer_mode && !audit.moving_lines_bottom_up.is_empty() {
        egui::Grid::new((
            "liuyao_calendar_moving_lines",
            audit.primary_hexagram.bits_bottom_up,
            audit.changed_hexagram.bits_bottom_up,
            audit.moving_line_mask_bottom_up,
        ))
        .num_columns(5)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("爻位");
            ui.strong("爻值");
            ui.strong("本爻");
            ui.strong("变爻候选");
            ui.strong("变爻六亲");
            ui.end_row();
            for line in audit.moving_lines_bottom_up.iter().rev() {
                ui.label(format!("第{}爻", line.line_number_bottom_up));
                ui.label(format!(
                    "{} · {}→{}",
                    line.received_value,
                    if line.primary_is_yang { "阳" } else { "阴" },
                    if line.changed_is_yang { "阳" } else { "阴" }
                ));
                ui.label(format!(
                    "{} {}",
                    line.primary_line.najia.designation_zh(),
                    line.primary_line.relationship.zh_name()
                ));
                ui.monospace(format!(
                    "{}{}{}",
                    line.transformed_stem.zh_name(),
                    line.transformed_branch.zh_name(),
                    line.transformed_branch_five_element.zh_name()
                ));
                ui.label(line.transformed_relationship_under_primary_palace.zh_name());
                ui.end_row();
            }
        });
    }
    if developer_mode {
        ui.small(format!(
            "{} / {} / 月界安全：{} / 月日作用判断：否 / 规则准入：否",
            audit.algorithm_version,
            audit.status,
            if audit.month_boundary_safe {
                "是"
            } else {
                "否"
            }
        ));
        ui.small(format!(
            "月界：{}—{}；日界：北京时间午夜候选。只重建六七八九阴阳翻转和动爻变纳甲；旺衰、冲合、空破、用神、应期与断法均未准入。",
            audit.month_previous_jie.zh_name(),
            audit.month_next_jie.zh_name()
        ));
    }
}

fn render_liuyao_calendar_relations_audit(
    ui: &mut egui::Ui,
    audit: &LiuyaoCalendarRelationsAudit,
    developer_mode: bool,
) {
    ui.small(format!(
        "旬支与五行机械候选：{}旬 · 旬中未配{}、{}；月建{}{}（对支{}），日辰{}{}（对支{}）。",
        audit.xun_start.zh_name(),
        audit.xun_void_branches[0].zh_name(),
        audit.xun_void_branches[1].zh_name(),
        audit.month_build_branch.zh_name(),
        audit.month_build_five_element.zh_name(),
        audit.month_opposite_branch.zh_name(),
        audit.day_branch.zh_name(),
        audit.day_branch_five_element.zh_name(),
        audit.day_opposite_branch.zh_name()
    ));

    if developer_mode {
        let flags = |observation: &divination_core::LiuyaoCalendarBranchObservation| {
            let mut labels = Vec::new();
            if observation.equals_month_build {
                labels.push("同月建");
            }
            if observation.opposed_by_month_build {
                labels.push("与月建对支");
            }
            if observation.equals_day_branch {
                labels.push("同日辰");
            }
            if observation.opposed_by_day_branch {
                labels.push("与日辰对支");
            }
            if observation.is_xun_void {
                labels.push("旬中未配支");
            }
            if labels.is_empty() {
                "—".to_owned()
            } else {
                labels.join("、")
            }
        };
        egui::Grid::new((
            "liuyao_calendar_relation_lines",
            audit.calendar_moving.primary_hexagram.bits_bottom_up,
            audit.calendar_moving.changed_hexagram.bits_bottom_up,
            audit.calendar_moving.day_designation.cycle_index,
        ))
        .num_columns(6)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("层");
            ui.strong("爻位");
            ui.strong("爻支");
            ui.strong("原始关系");
            ui.strong("月建五行→爻");
            ui.strong("日辰五行→爻");
            ui.end_row();
            for observation in audit
                .primary_observations_bottom_up
                .iter()
                .chain(audit.transformed_moving_observations_bottom_up.iter())
                .rev()
            {
                ui.label(observation.layer.zh_name());
                ui.label(format!("第{}爻", observation.line_number_bottom_up));
                ui.label(format!(
                    "{}{}",
                    observation.branch.zh_name(),
                    observation.branch_five_element.zh_name()
                ));
                ui.label(flags(observation));
                ui.label(format!(
                    "{}→{}：{}",
                    audit.month_build_five_element.zh_name(),
                    observation.branch_five_element.zh_name(),
                    observation.month_build_five_element_relation.zh_name()
                ));
                ui.label(format!(
                    "{}→{}：{}",
                    audit.day_branch_five_element.zh_name(),
                    observation.branch_five_element.zh_name(),
                    observation.day_branch_five_element_relation.zh_name()
                ));
                ui.end_row();
            }
        });
        ui.small(format!(
            "{} / {} / 原始支位与五行方向：已记录 / 关系效力判断：否 / 规则准入：否",
            audit.algorithm_version, audit.status
        ));
        ui.small("“同支、对支、旬中未配支”和“谁生谁、谁克谁”只描述结构方向；不等于旺衰、月破有效、冲实、填实、得生、受克、吉凶或应期。");
        egui::CollapsingHeader::new("六爻降雨候选结构检查（研究）")
            .default_open(false)
            .show(ui, |ui| {
                match divination_core::liuyao_weather_rain_candidate_audit(audit) {
                    Ok(candidate) => {
                        let line_list = |lines: &[u8]| {
                            if lines.is_empty() {
                                "无".to_owned()
                            } else {
                                lines
                                    .iter()
                                    .map(|line| format!("第{line}爻"))
                                    .collect::<Vec<_>>()
                                    .join("、")
                            }
                        };
                        ui.label(format!(
                            "可见父母爻：{}；父母明动：{}；可见爻受日冲：{}。",
                            line_list(&candidate.visible_parent_lines_bottom_up),
                            line_list(&candidate.moving_parent_lines_bottom_up),
                            line_list(&candidate.day_opposed_visible_lines_bottom_up)
                        ));
                        ui.label(format!(
                            "官鬼明动：{}；变出父母：{}。",
                            line_list(&candidate.moving_officer_ghost_lines_bottom_up),
                            line_list(&candidate.transformed_parent_lines_bottom_up)
                        ));
                        ui.label(format!(
                            "可见爻与月建对支：{}；旬中未配支：{}。",
                            line_list(&candidate.month_opposed_visible_lines_bottom_up),
                            line_list(&candidate.xun_void_visible_lines_bottom_up)
                        ));
                        let visible_parent_pairs = candidate
                            .parent_combined_visible_pairs_bottom_up
                            .iter()
                            .map(|pair| format!("{}-{}爻", pair[0], pair[1]))
                            .collect::<Vec<_>>();
                        ui.label(format!(
                            "父母爻原始六合配对：本卦其他明爻 {}；月建 {}；日辰 {}。",
                            if visible_parent_pairs.is_empty() {
                                "无".to_owned()
                            } else {
                                visible_parent_pairs.join("、")
                            },
                            line_list(&candidate.parent_combined_month_lines_bottom_up),
                            line_list(&candidate.parent_combined_day_lines_bottom_up)
                        ));
                        ui.label(if candidate.narrow_visible_line_pattern_present {
                            "可见爻的狭窄候选条件：符合。"
                        } else {
                            "可见爻的狭窄候选条件：不符合。"
                        });
                        ui.small("这只核对《增刪卜易》天时章一句的部分结构。月对支、旬中未配支和六合配对只是原始标记；狭窄条件排除父母爻与本卦明爻、月建、日辰的原始六合配对。同章另有父母合住、鬼动冲开后有雨的条件，其效力、优先序及应期未审。正式规则未放行，不能据此判断是否下雨。");
                    }
                    Err(error) => {
                        ui.label(format!("候选结构检查未完成：{error}"));
                    }
                }
            });
    }
}

fn render_liuyao_branch_pair_relations_audit(
    ui: &mut egui::Ui,
    audit: &LiuyaoBranchPairRelationsAudit,
    developer_mode: bool,
) {
    let observations = audit
        .primary_visible_pair_observations
        .iter()
        .chain(audit.moving_transformation_observations.iter())
        .chain(audit.flying_hidden_observations.iter())
        .collect::<Vec<_>>();
    let combination_count = observations
        .iter()
        .filter(|observation| observation.is_six_combination_pair)
        .count();
    let opposition_count = observations
        .iter()
        .filter(|observation| observation.is_six_opposition_pair)
        .count();
    ui.small(format!(
        "地支成对机械检查：本卦明爻 {} 组、动爻本变同位 {} 组、飞伏同位 {} 组；六合对 {} 组、六冲对 {} 组。",
        audit.primary_visible_pair_observations.len(),
        audit.moving_transformation_observations.len(),
        audit.flying_hidden_observations.len(),
        combination_count,
        opposition_count
    ));

    if developer_mode {
        let matched = observations
            .into_iter()
            .filter(|observation| {
                observation.is_six_combination_pair || observation.is_six_opposition_pair
            })
            .collect::<Vec<_>>();
        if matched.is_empty() {
            ui.small("当前三个明确比较范围内没有六合对或六冲对。");
        } else {
            egui::Grid::new((
                "liuyao_branch_pair_relations",
                audit
                    .calendar_relations
                    .calendar_moving
                    .primary_hexagram
                    .bits_bottom_up,
                audit
                    .calendar_relations
                    .calendar_moving
                    .changed_hexagram
                    .bits_bottom_up,
                audit
                    .calendar_relations
                    .calendar_moving
                    .moving_line_mask_bottom_up,
            ))
            .num_columns(4)
            .striped(true)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.strong("比较范围");
                ui.strong("第一支");
                ui.strong("第二支");
                ui.strong("原始关系");
                ui.end_row();
                for observation in matched {
                    ui.label(observation.scope.zh_name());
                    ui.label(format!(
                        "{}第{}爻·{}{}",
                        observation.first.layer.zh_name(),
                        observation.first.line_number_bottom_up,
                        observation.first.branch.zh_name(),
                        observation.first.branch_five_element.zh_name()
                    ));
                    ui.label(format!(
                        "{}第{}爻·{}{}",
                        observation.second.layer.zh_name(),
                        observation.second.line_number_bottom_up,
                        observation.second.branch.zh_name(),
                        observation.second.branch_five_element.zh_name()
                    ));
                    ui.label(if observation.is_six_combination_pair {
                        "六合支对"
                    } else {
                        "六冲支对"
                    });
                    ui.end_row();
                }
            });
        }
        ui.small(format!(
            "{} / {} / 整卦六合分类：否 / 整卦六冲分类：否 / 关系效力判断：否 / 规则准入：否",
            audit.algorithm_version, audit.status
        ));
        ui.small("这里只比较本卦明爻两两、动爻与同位变爻、飞神与同位伏神；支对成立不等于合起、合住、冲破、暗动、吉凶或应期。相同月建/日辰关系仍由上一层独立显示。");
    }
}

fn render_liuyao_progress_retreat_audit(
    ui: &mut egui::Ui,
    audit: &LiuyaoProgressRetreatAudit,
    developer_mode: bool,
) {
    let progress_count = audit
        .moving_observations_bottom_up
        .iter()
        .filter(|observation| observation.identity == LiuyaoProgressRetreatIdentity::Progress)
        .count();
    let retreat_count = audit
        .moving_observations_bottom_up
        .iter()
        .filter(|observation| observation.identity == LiuyaoProgressRetreatIdentity::Retreat)
        .count();
    let unclassified_count =
        audit.moving_observations_bottom_up.len() - progress_count - retreat_count;
    ui.small(format!(
        "动变进退身份机械检查：实际动爻 {} 条；明确列出的进神对 {} 条、退神对 {} 条、未分类 {} 条。",
        audit.moving_observations_bottom_up.len(),
        progress_count,
        retreat_count,
        unclassified_count
    ));

    if developer_mode {
        if audit.moving_observations_bottom_up.is_empty() {
            ui.small("本卦无实际动爻，因此不生成进神、退神身份记录。");
        } else {
            egui::Grid::new((
                "liuyao_progress_retreat",
                audit
                    .branch_pair_relations
                    .calendar_relations
                    .calendar_moving
                    .primary_hexagram
                    .bits_bottom_up,
                audit
                    .branch_pair_relations
                    .calendar_relations
                    .calendar_moving
                    .changed_hexagram
                    .bits_bottom_up,
                audit
                    .branch_pair_relations
                    .calendar_relations
                    .calendar_moving
                    .moving_line_mask_bottom_up,
            ))
            .num_columns(4)
            .striped(true)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.strong("爻位");
                ui.strong("本支");
                ui.strong("变支");
                ui.strong("原始身份");
                ui.end_row();
                for observation in audit.moving_observations_bottom_up.iter().rev() {
                    ui.label(format!("第{}爻", observation.line_number_bottom_up));
                    ui.label(format!(
                        "{}{}",
                        observation.primary_branch.zh_name(),
                        observation.primary_branch_five_element.zh_name()
                    ));
                    ui.label(format!(
                        "{}{}",
                        observation.transformed_branch.zh_name(),
                        observation.transformed_branch_five_element.zh_name()
                    ));
                    ui.label(observation.identity.zh_name());
                    ui.end_row();
                }
            });
        }
        ui.small(format!(
            "{} / {} / 明列进神对：{} / 明列退神对：{} / 进退效力判断：否 / 规则准入：否",
            audit.algorithm_version,
            audit.status,
            audit.explicit_progress_pair_count,
            audit.explicit_retreat_pair_count
        ));
        ui.small("本层只认当前已核电子转录明确列出的七进、七退；戌→丑、丑→戌仍标为未分类，等待影印本与版本异文复核。进退有无作用、旺衰、用神、应期、吉凶和行动建议均未准入。");
    }
}

fn render_liuyao_fanyin_fuyin_audit(
    ui: &mut egui::Ui,
    audit: &LiuyaoFanyinFuyinAudit,
    developer_mode: bool,
) {
    let scopes = [&audit.inner_scope, &audit.outer_scope];
    let trigram_fanyin_count = scopes
        .iter()
        .filter(|scope| scope.raw_trigram_fanyin_candidate)
        .count();
    let line_fanyin_scope_count = scopes
        .iter()
        .filter(|scope| scope.raw_line_fanyin_candidate)
        .count();
    let fuyin_scope_count = scopes
        .iter()
        .filter(|scope| scope.raw_fuyin_candidate)
        .count();
    let moving_line_fanyin_count = audit
        .moving_line_observations_bottom_up
        .iter()
        .filter(|line| line.raw_line_fanyin_candidate)
        .count();
    let moving_line_fuyin_count = audit
        .moving_line_observations_bottom_up
        .iter()
        .filter(|line| line.raw_line_fuyin_candidate)
        .count();
    ui.small(format!(
        "反伏结构机械检查：卦向对冲范围 {} 个、三支同位全冲范围 {} 个、三支同位复现范围 {} 个；动爻同支 {} 条、动爻对支 {} 条。",
        trigram_fanyin_count,
        line_fanyin_scope_count,
        fuyin_scope_count,
        moving_line_fuyin_count,
        moving_line_fanyin_count
    ));

    if developer_mode {
        egui::Grid::new((
            "liuyao_fanyin_fuyin_scopes",
            audit
                .progress_retreat
                .branch_pair_relations
                .calendar_relations
                .calendar_moving
                .primary_hexagram
                .bits_bottom_up,
            audit
                .progress_retreat
                .branch_pair_relations
                .calendar_relations
                .calendar_moving
                .changed_hexagram
                .bits_bottom_up,
        ))
        .num_columns(5)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("范围");
            ui.strong("本卦→变卦");
            ui.strong("动爻");
            ui.strong("本支→变支");
            ui.strong("原始候选");
            ui.end_row();
            for scope in scopes {
                let mut labels = Vec::new();
                if scope.raw_trigram_fanyin_candidate {
                    labels.push("卦向对冲");
                }
                if scope.raw_line_fanyin_candidate {
                    labels.push("三支同位全冲");
                }
                if scope.raw_fuyin_candidate {
                    labels.push("三支同位复现");
                }
                ui.label(scope.scope.zh_name());
                ui.label(format!(
                    "{}→{}",
                    scope.primary_trigram.zh_name(),
                    scope.transformed_trigram.zh_name()
                ));
                ui.label(if scope.moving_line_numbers_bottom_up.is_empty() {
                    "无".to_owned()
                } else {
                    scope
                        .moving_line_numbers_bottom_up
                        .iter()
                        .map(|line| line.to_string())
                        .collect::<Vec<_>>()
                        .join("、")
                });
                ui.monospace(
                    scope
                        .primary_branches_bottom_up
                        .iter()
                        .zip(scope.transformed_branches_bottom_up.iter())
                        .map(|(primary, transformed)| {
                            format!("{}→{}", primary.zh_name(), transformed.zh_name())
                        })
                        .collect::<Vec<_>>()
                        .join(" · "),
                );
                ui.label(if labels.is_empty() {
                    "—".to_owned()
                } else {
                    labels.join("、")
                });
                ui.end_row();
            }
        });

        let matched_lines = audit
            .moving_line_observations_bottom_up
            .iter()
            .filter(|line| line.raw_line_fanyin_candidate || line.raw_line_fuyin_candidate)
            .collect::<Vec<_>>();
        if !matched_lines.is_empty() {
            egui::Grid::new((
                "liuyao_fanyin_fuyin_moving_lines",
                audit
                    .progress_retreat
                    .branch_pair_relations
                    .calendar_relations
                    .calendar_moving
                    .moving_line_mask_bottom_up,
            ))
            .num_columns(4)
            .striped(true)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.strong("爻位");
                ui.strong("本支");
                ui.strong("变支");
                ui.strong("原始候选");
                ui.end_row();
                for line in matched_lines.into_iter().rev() {
                    ui.label(format!("第{}爻", line.line_number_bottom_up));
                    ui.label(line.primary_branch.zh_name());
                    ui.label(line.transformed_branch.zh_name());
                    ui.label(if line.raw_line_fuyin_candidate {
                        "同支复现候选"
                    } else {
                        "对支反吟候选"
                    });
                    ui.end_row();
                }
            });
        }
        ui.small(format!(
            "{} / {} / 整卦反吟分类：否 / 整卦伏吟分类：否 / 效力判断：否 / 规则准入：否",
            audit.algorithm_version, audit.status
        ));
        ui.small("这里只保存内外卦方向对冲、三支同位全冲/复现和实际动爻同支/对支的必要结构。不同传本的卦反吟、爻反吟、伏吟组合口径及其效力尚未放行，不生成反复、停滞、吉凶、应期或行动建议。");
    }
}

fn render_liuyao_sanhe_audit(ui: &mut egui::Ui, audit: &LiuyaoSanheAudit, developer_mode: bool) {
    let raw_union_complete_count = audit
        .groups
        .iter()
        .filter(|group| group.raw_union_contains_all_three_members)
        .count();
    ui.small(format!(
        "三合成员机械清点：固定四组生、旺、墓目录；{} 组在本卦明爻、实际变爻、锁定月建和日辰的并集内曾见三支（仅清点，不代表成局）。",
        raw_union_complete_count
    ));

    if developer_mode {
        let presence_label = |members: [divination_core::EarthlyBranch; 3], presence: [bool; 3]| {
            members
                .into_iter()
                .zip(presence)
                .zip(divination_core::LiuyaoSanheMemberRole::ALL)
                .filter(|((_, seen), _)| *seen)
                .map(|((branch, _), role)| format!("{}({})", branch.zh_name(), role.zh_name()))
                .collect::<Vec<_>>()
                .join("、")
        };
        let calendar = &audit
            .fanyin_fuyin
            .progress_retreat
            .branch_pair_relations
            .calendar_relations;
        egui::Grid::new((
            "liuyao_sanhe_raw_occurrences",
            calendar.calendar_moving.primary_hexagram.bits_bottom_up,
            calendar.calendar_moving.changed_hexagram.bits_bottom_up,
            calendar.calendar_moving.moving_line_mask_bottom_up,
        ))
        .num_columns(6)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("局类目录");
            ui.strong("生·旺·墓");
            ui.strong("本卦明爻");
            ui.strong("实际变爻");
            ui.strong("月建/日辰");
            ui.strong("原始并集");
            ui.end_row();
            for group in &audit.groups {
                let members = group.members_birth_prosperity_tomb;
                let label_or_dash = |value: String| {
                    if value.is_empty() {
                        "—".to_owned()
                    } else {
                        value
                    }
                };
                ui.label(format!("{}局目录", group.result_element.zh_name()));
                ui.monospace(
                    members
                        .into_iter()
                        .zip(divination_core::LiuyaoSanheMemberRole::ALL)
                        .map(|(branch, role)| format!("{}({})", branch.zh_name(), role.zh_name()))
                        .collect::<Vec<_>>()
                        .join(" · "),
                );
                ui.label(label_or_dash(presence_label(
                    members,
                    group.primary_visible_presence,
                )));
                ui.label(label_or_dash(presence_label(
                    members,
                    group.moving_transformed_presence,
                )));
                let month = label_or_dash(presence_label(members, group.locked_month_presence));
                let day = label_or_dash(presence_label(members, group.locked_day_presence));
                ui.label(format!("月：{}；日：{}", month, day));
                ui.label(if group.raw_union_contains_all_three_members {
                    "三支均曾出现（非成局）"
                } else {
                    "未见齐"
                });
                ui.end_row();
            }
        });
        ui.small(format!(
            "{} / {} / 来源激活条件分类：否 / 三合成局分类：否 / 效力判断：否 / 规则准入：否",
            audit.algorithm_version, audit.status
        ));
        ui.small("本层把本卦明爻、实际动爻变支、锁定月建和日辰严格分栏，只保存申子辰、亥卯未、寅午戌、巳酉丑成员出现事实。不同传本关于动静、日月、变爻、全局/半局、真假与化局的条件尚未放行；即使并集三支齐全，也不显示已经成局，不生成吉凶、应期或行动建议。");
    }
}

fn render_hexagram_pair(
    ui: &mut egui::Ui,
    primary_bits: u8,
    changed_bits: u8,
    developer_mode: bool,
) {
    match hexagram_transformation(primary_bits, changed_bits) {
        Ok(transformation) => {
            ui.monospace(format!(
                "本卦：{} · {:06b}",
                transformation.primary.compact_zh(),
                primary_bits
            ));
            ui.monospace(format!(
                "变卦：{} · {:06b}",
                transformation.changed.compact_zh(),
                changed_bits
            ));
            if developer_mode {
                ui.small(format!(
                    "结构目录：{} / 不含卦辞、爻辞或吉凶解释",
                    transformation.algorithm_version
                ));
            }
        }
        Err(error) => {
            ui.colored_label(Color32::from_rgb(167, 54, 54), error.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bce_proleptic_gregorian_input_with_astronomical_years() {
        let bce_1000 = parse_proleptic_gregorian_datetime("-0999-06-01 12:34:56").unwrap();
        assert_eq!(bce_1000.year(), -999);
        assert_eq!(bce_1000.month(), 6);
        assert_eq!(bce_1000.day(), 1);

        let bce_1 = parse_proleptic_gregorian_datetime("0000-01-01 00:00:00").unwrap();
        assert_eq!(bce_1.year(), 0);
    }

    #[test]
    fn ambiguous_other_error_names_the_conflicting_topics() {
        let error = ClassificationError::AmbiguousTypes(vec![
            AncientQuestionType::ProjectOutcome,
            AncientQuestionType::HealthTrendReference,
        ]);
        let message = classification_error_zh(&error);
        assert!(message.contains("项目成败"));
        assert!(message.contains("健康参考"));
    }

    #[test]
    fn health_warning_survives_other_mapping_and_ambiguity() {
        let classifier = KeywordClassifier::default();
        let mapped = classifier.classify_detailed(QuestionCategory::Other, "健康情况会如何？");
        assert!(health_notice_required_for_input(
            QuestionCategory::Other,
            Some(&mapped)
        ));
        let ambiguous =
            classifier.classify_detailed(QuestionCategory::Other, "健康 APP 上线后怎么办？");
        assert!(health_notice_required_for_input(
            QuestionCategory::Other,
            Some(&ambiguous)
        ));
        let project = classifier.classify_detailed(QuestionCategory::Other, "APP 上线能成功吗？");
        assert!(!health_notice_required_for_input(
            QuestionCategory::Other,
            Some(&project)
        ));
    }

    #[test]
    fn project_acceptance_checklist_covers_all_original_requirements_without_percentages() {
        fn rejects_obsolete_percentage_fields(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(object) => object.iter().all(|(key, value)| {
                    let normalized = key.to_ascii_lowercase();
                    !normalized.contains("percent")
                        && !normalized.contains("percentage")
                        && rejects_obsolete_percentage_fields(value)
                }),
                serde_json::Value::Array(values) => {
                    values.iter().all(rejects_obsolete_percentage_fields)
                }
                serde_json::Value::Number(number) => number
                    .as_u64()
                    .is_none_or(|value| value != 69 && value != 87),
                serde_json::Value::String(text) => !matches!(text.trim(), "69" | "87"),
                _ => true,
            }
        }

        let checklist = load_project_acceptance_checklist().unwrap();
        assert_eq!(checklist.items.len(), 16);
        assert_eq!(
            checklist
                .items
                .iter()
                .filter(|item| item.status == ProjectAcceptanceStatus::Passed)
                .count(),
            5
        );
        assert_eq!(
            checklist
                .items
                .iter()
                .filter(|item| item.status == ProjectAcceptanceStatus::InProgress)
                .count(),
            9
        );
        assert_eq!(
            checklist
                .items
                .iter()
                .filter(|item| item.status == ProjectAcceptanceStatus::Blocked)
                .count(),
            2
        );
        assert!(!PROGRESS_CHECKLIST_JSON.contains('%'));
        let raw: serde_json::Value = serde_json::from_str(PROGRESS_CHECKLIST_JSON).unwrap();
        assert!(rejects_obsolete_percentage_fields(&raw));
    }
}
