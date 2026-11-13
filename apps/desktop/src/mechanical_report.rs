use divination_core::*;
use divination_storage::{
    birth_time_audits_match, calculate_birth_time_audit_snapshot, BirthTimeAuditSnapshot, Storage,
};
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub const SCOPE: &str = "四柱 / 紫微机械盘试用：显示排布与结构，暂不提供吉凶、旺衰、大运或应期。";
pub const PALACE_LAYOUT: [[Option<u8>; 4]; 4] = [
    [Some(5), Some(6), Some(7), Some(8)],
    [Some(4), None, None, Some(9)],
    [Some(3), None, None, Some(10)],
    [Some(2), Some(1), Some(0), Some(11)],
];

#[derive(Debug, Clone)]
pub struct PillarTable {
    pub title: String,
    pub rows: Vec<Vec<String>>,
    pub boundary_safe: bool,
}

#[derive(Debug, Clone)]
pub struct PalaceCell {
    pub branch_index: u8,
    pub title: String,
    pub body: bool,
    pub major: Vec<String>,
    pub auxiliary: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ZiweiBoard {
    pub title: String,
    pub subtitle: String,
    pub cells: Vec<PalaceCell>,
    pub boundary_safe: bool,
    pub missing: String,
}

#[derive(Debug, Clone)]
pub struct MechanicalReport {
    pub question_id: Uuid,
    pub question: String,
    pub metadata: Vec<(String, String)>,
    pub pillars: Vec<PillarTable>,
    pub ziwei: Vec<ZiweiBoard>,
    pub pillar_notice: String,
    pub ziwei_notice: String,
    pub verification: String,
}

pub fn load(storage: &Storage, question_id: Uuid) -> Result<MechanicalReport, String> {
    let detail = storage
        .load_question_detail(question_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "记录不存在，请刷新永久记录。".to_owned())?;
    let mut report = MechanicalReport {
        question_id,
        question: detail.question.text.clone(),
        metadata: Vec::new(),
        pillars: Vec::new(),
        ziwei: Vec::new(),
        pillar_notice:
            "本记录未提供出生资料。新建占问时选择“长期命运”，填写出生时间和出生地点后锁定。".into(),
        ziwei_notice: "本记录未提供出生资料，不能用问事时间代替出生时间排盘。".into(),
        verification: "原问题已读取；本记录尚无出生机械盘。".into(),
    };
    let Some(birth) = &detail.question.birth else {
        return Ok(report);
    };
    let Some(snapshot) = storage
        .load_astronomy_snapshot(question_id)
        .map_err(|error| error.to_string())?
    else {
        report.pillar_notice = "旧记录未保存历法快照，未回填新排盘。".into();
        report.ziwei_notice = report.pillar_notice.clone();
        return Ok(report);
    };
    let Some(audit) = snapshot.birth_time_audit else {
        report.pillar_notice = "旧记录未保存出生审计，未回填新排盘。".into();
        report.ziwei_notice = report.pillar_notice.clone();
        return Ok(report);
    };
    let current = calculate_birth_time_audit_snapshot(birth).map_err(|error| error.to_string())?;
    if !birth_time_audits_match(&current, &audit) {
        return Err("保存的出生机械盘与原输入核对不一致，已停止生成报告。".into());
    }
    report.metadata = vec![
        (
            "出生时间（北京时间）".into(),
            birth.date_time_bjt.to_rfc3339(),
        ),
        (
            "真太阳时（同一时区表示）".into(),
            audit.true_solar_time_bjt.clone(),
        ),
        ("出生地点".into(), crate::location_label(&birth.location)),
        (
            "地点精度".into(),
            crate::location_precision_notice(&birth.location).to_owned(),
        ),
        (
            "性别记录".into(),
            crate::biological_sex_label(birth.sex).into(),
        ),
    ];
    report.verification = format!(
        "已校验原始快照内容，并按原出生资料重新核对；显示保存的出生审计 v{}。",
        audit.audit_version
    );
    populate(&mut report, &audit);
    Ok(report)
}

pub fn boundary_text(safe: bool) -> &'static str {
    if safe {
        "当前候选未触及已检查的时间边界；仍属机械盘。"
    } else {
        "存在临界时间或口径未决，请同时核对各候选。"
    }
}

fn day_label(value: &str) -> String {
    match value {
        "true_solar_midnight_changes_day.v1"
        | "true_solar_midnight_with_parallel_bjt_midnight_audit.v1" => "真太阳时 · 零点换日".into(),
        "true_solar_zi_start_changes_day.v1" => "真太阳时 · 子初换日（晚子时平行候选）".into(),
        other => format!("日界口径：{other}"),
    }
}

fn populate(report: &mut MechanicalReport, audit: &BirthTimeAuditSnapshot) {
    report.pillar_notice = "四柱采用立春年界、节月界和真太阳时时辰；晚子时并列展示日界候选。藏干按歌诀列举，不表示主次、权重或强弱。".into();
    if let Some(chart) = &audit.four_pillars_chart_candidate {
        let mut candidates = vec![(
            chart.day_convention.as_str(),
            chart.day_pillar_true_solar_midnight,
            chart.hour_pillar_true_solar_midnight,
            audit.four_pillars_visible_stem_ten_gods.as_ref(),
        )];
        if let Some(dual) = &audit.late_zi_dual_candidates {
            candidates.push((
                &dual.zi_start_boundary.convention,
                dual.zi_start_boundary.day_pillar,
                dual.zi_start_boundary.hour_pillar,
                audit.four_pillars_visible_stem_ten_gods_zi_start.as_ref(),
            ));
        }
        for (convention, day, hour, visible) in candidates {
            let pillars = [
                chart.year_candidate.designation,
                chart.month_candidate.designation,
                day,
                hour,
            ];
            let hidden = audit
                .four_pillars_hidden_stems
                .as_ref()
                .and_then(|items| items.iter().find(|item| item.day_convention == convention));
            let mut rows = vec![vec![
                "项目".into(),
                "年柱".into(),
                "月柱".into(),
                "日柱".into(),
                "时柱".into(),
            ]];
            let mut designation = vec!["干支".into()];
            let mut elements = vec!["阴阳五行".into()];
            let mut ten_gods = vec!["明干十神".into()];
            let mut hidden_labels = vec!["藏干 / 十神".into()];
            for (index, pillar) in pillars.iter().enumerate() {
                designation.push(pillar.zh_name());
                elements.push(format!(
                    "{}{} / {}{}",
                    pillar.stem.yin_yang().zh_name(),
                    pillar.stem.five_element().zh_name(),
                    pillar.branch.yin_yang().zh_name(),
                    pillar.branch.five_element().zh_name()
                ));
                ten_gods.push(
                    visible
                        .and_then(|item| item.observations.get(index))
                        .map(|item| {
                            item.ten_god_identity
                                .map(|value| value.zh_name())
                                .unwrap_or("日主")
                        })
                        .unwrap_or("旧记录未保存")
                        .into(),
                );
                hidden_labels.push(
                    hidden
                        .and_then(|item| item.pillars.get(index))
                        .map(|pillar| {
                            pillar
                                .stems
                                .iter()
                                .map(|item| {
                                    format!("{} · {}", item.stem.zh_name(), item.ten_god.zh_name())
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or_else(|| "旧记录未保存".into()),
                );
            }
            rows.extend([designation, elements, ten_gods, hidden_labels]);
            report.pillars.push(PillarTable {
                title: day_label(convention),
                rows,
                boundary_safe: chart.mechanical_boundaries_safe,
            });
        }
    } else {
        report.pillar_notice = "旧记录未保存四柱机械盘。".into();
    }
    report.ziwei_notice = "年界和时辰口径并列展示，程序未选定一套命盘。当前包含十二宫、宫干、命身宫、五行局、十四主星及文昌、文曲、左辅、右弼；四化、其他辅煞和大限尚未完成。".into();
    let Some(ziwei) = &audit.ziwei_life_body_audit else {
        report.ziwei_notice = audit
            .ziwei_life_body_audit_error
            .clone()
            .unwrap_or_else(|| "旧记录未保存紫微机械盘。".into());
        return;
    };
    let lunar = &ziwei.lunisolar_date_candidate;
    report.metadata.push((
        "出生农历候选".into(),
        format!(
            "{}年 {}{}月 {}日（现代农历口径）",
            lunar.nominal_lunar_year,
            if lunar.is_leap_month { "闰" } else { "" },
            lunar.month_number,
            lunar.day_number
        ),
    ));
    let years: Vec<Option<&ZiweiYearStemConventionCandidate>> =
        if ziwei.year_stem_convention_candidates.is_empty() {
            vec![None]
        } else {
            ziwei
                .year_stem_convention_candidates
                .iter()
                .map(Some)
                .collect()
        };
    for year in years {
        for (hour_label, hour, bureau) in [
            (
                "北京时间",
                &ziwei.beijing_time_candidate,
                year.map(|item| &item.beijing_time_candidate),
            ),
            (
                "真太阳时",
                &ziwei.true_solar_time_candidate,
                year.map(|item| &item.true_solar_time_candidate),
            ),
        ] {
            let year_label = match year.map(|item| item.year_convention.as_str()) {
                Some("modern_nominal_lunar_year_label.v1") => "农历年口径",
                Some("astronomical_lichun_year_candidate.v1") => "立春年口径",
                Some(other) => other,
                None => "旧记录未保存年干口径",
            };
            let anchor = year.and_then(|year| {
                ziwei.star_anchor_candidates.iter().find(|item| {
                    item.year_convention == year.year_convention
                        && item.hour_convention == hour.hour_convention
                })
            });
            let major = anchor.and_then(|item| item.major_stars.as_ref());
            let auxiliary = anchor.and_then(|item| item.auxiliary_stars.as_ref());
            let cells = hour
                .chart
                .palaces
                .iter()
                .map(|palace| {
                    let stem = bureau
                        .and_then(|item| {
                            item.branch_stems
                                .iter()
                                .find(|item| item.branch == palace.branch)
                        })
                        .map(|item| item.designation.zh_name())
                        .unwrap_or_else(|| palace.branch.zh_name().into());
                    PalaceCell {
                        branch_index: palace.branch.index(),
                        title: format!("{} · {}", stem, palace.ui_zh_name),
                        body: palace.branch == hour.chart.body_palace_branch,
                        major: major
                            .map(|chart| {
                                chart
                                    .placements
                                    .iter()
                                    .filter(|item| item.branch == palace.branch)
                                    .map(|item| item.kind.zh_name().into())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        auxiliary: auxiliary
                            .map(|chart| {
                                chart
                                    .placements
                                    .iter()
                                    .filter(|item| item.branch == palace.branch)
                                    .map(|item| item.kind.zh_name().into())
                                    .collect()
                            })
                            .unwrap_or_default(),
                    }
                })
                .collect();
            let mut missing = Vec::new();
            if major.is_none() {
                missing.push("十四主星未保存");
            }
            if auxiliary.is_none() {
                missing.push("四辅星未保存");
            }
            report.ziwei.push(ZiweiBoard {
                title: format!("{year_label} / {hour_label}"),
                subtitle: format!(
                    "{}；{}{}",
                    hour.chart.compact_zh(),
                    bureau
                        .map(|item| item.five_element_bureau.zh_name())
                        .unwrap_or_else(|| "五行局未保存".into()),
                    if hour.chart.leap_month_treated_as_next_month {
                        "；闰月暂按下月"
                    } else {
                        ""
                    }
                ),
                cells,
                missing: missing.join("；"),
                boundary_safe: hour.mechanical_boundaries_safe
                    && bureau.is_some_and(|item| item.mechanical_boundaries_safe),
            });
        }
    }
}

impl MechanicalReport {
    pub fn text(&self) -> String {
        let mut lines = vec![
            "九衡 · 四柱 / 紫微机械盘".into(),
            SCOPE.into(),
            self.question.clone(),
            format!("记录编号：{}", self.question_id),
        ];
        lines.extend(
            self.metadata
                .iter()
                .map(|(name, value)| format!("{name}：{value}")),
        );
        lines.push(self.pillar_notice.clone());
        for table in &self.pillars {
            lines.extend([
                table.title.clone(),
                boundary_text(table.boundary_safe).into(),
            ]);
            lines.extend(table.rows.iter().map(|row| {
                row.iter()
                    .map(|item| item.replace('\n', "；"))
                    .collect::<Vec<_>>()
                    .join(" | ")
            }));
        }
        lines.push(self.ziwei_notice.clone());
        for board in &self.ziwei {
            lines.extend([
                board.title.clone(),
                board.subtitle.clone(),
                boundary_text(board.boundary_safe).into(),
                board.missing.clone(),
            ]);
            lines.extend(board.cells.iter().map(|cell| {
                format!(
                    "{}{}：主星 [{}]；辅星 [{}]",
                    cell.title,
                    if cell.body { " · 身宫" } else { "" },
                    cell.major.join("、"),
                    cell.auxiliary.join("、")
                )
            }));
        }
        lines.push(self.verification.clone());
        lines.join("\n")
    }

    pub fn html(&self) -> String {
        use crate::reading_report::escape_html as e;
        let mut body = format!(
            "<h1>九衡 · 四柱 / 紫微机械盘</h1><p class=notice>{}</p><h2>{}</h2>",
            e(SCOPE),
            e(&self.question)
        );
        for (name, value) in &self.metadata {
            body.push_str(&format!(
                "<p><strong>{}</strong>：{}</p>",
                e(name),
                e(value)
            ));
        }
        body.push_str(&format!(
            "<h2>四柱机械盘</h2><p>{}</p>",
            e(&self.pillar_notice)
        ));
        for table in &self.pillars {
            body.push_str(&format!(
                "<section><h3>{}</h3><p>{}</p><div class=scroll><table>",
                e(&table.title),
                e(boundary_text(table.boundary_safe))
            ));
            for (index, row) in table.rows.iter().enumerate() {
                let tag = if index == 0 { "th" } else { "td" };
                body.push_str("<tr>");
                for value in row {
                    body.push_str(&format!("<{tag}>{}</{tag}>", e(value)));
                }
                body.push_str("</tr>");
            }
            body.push_str("</table></div></section>");
        }
        body.push_str(&format!(
            "<h2>紫微机械盘</h2><p>{}</p>",
            e(&self.ziwei_notice)
        ));
        for board in &self.ziwei {
            body.push_str(&format!(
                "<section><h3>{}</h3><p>{}</p><p>{} {}</p><div class=scroll><div class=palaces>",
                e(&board.title),
                e(&board.subtitle),
                e(boundary_text(board.boundary_safe)),
                e(&board.missing)
            ));
            for branch in PALACE_LAYOUT.iter().flatten() {
                if let Some(cell) = branch
                    .and_then(|branch| board.cells.iter().find(|cell| cell.branch_index == branch))
                {
                    body.push_str(&format!("<div class=palace><strong>{}</strong><p class=body>{}</p><p>{}</p><small>{}</small></div>",e(&cell.title), if cell.body {"身宫"} else {""},e(&cell.major.join(" · ")),e(&cell.auxiliary.join(" · "))));
                } else {
                    body.push_str("<div class=center>机械结构<br>无占断结论</div>");
                }
            }
            body.push_str("</div></div></section>");
        }
        body.push_str(&format!("<footer><p>{}</p>记录编号 {} · 程序 v{}<br>包含出生时间和地点；离线可读，可通过浏览器打印。</footer>",e(&self.verification),self.question_id,env!("CARGO_PKG_VERSION")));
        format!(
            r#"<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'"><title>九衡机械盘</title><style>body{{margin:0;background:#f3f1eb;color:#273b31;font:15px/1.6 'Microsoft YaHei',sans-serif}}main{{max-width:1000px;margin:24px auto;padding:32px;background:#fffefb;border-top:6px solid #304f40}}h1,h2,h3{{overflow-wrap:anywhere}}section{{margin:20px 0;padding:16px;border:1px solid #d5ddd4;border-radius:8px;break-inside:avoid}}.notice{{background:#eee7d7;padding:14px}}table{{border-collapse:collapse;min-width:680px;width:100%;table-layout:fixed}}td,th{{padding:12px;vertical-align:top;border:1px solid #d5ddd4;white-space:pre-wrap;overflow-wrap:anywhere}}th{{background:#304f40;color:white}}.scroll{{overflow-x:auto}}.palaces{{display:grid;grid-template-columns:repeat(4,1fr);min-width:680px;gap:6px}}.palace{{min-height:125px;padding:12px;background:#edf2ed;border:1px solid #cbd5cb;border-radius:5px}}.palace p{{margin:5px 0}}.body{{color:#985839;font-weight:bold}}.center{{display:flex;align-items:center;justify-content:center;text-align:center;color:#8a948c}}footer{{border-top:1px solid #ddd;padding-top:16px;color:#657066}}@media(max-width:700px){{main{{margin:0;padding:14px}}}}@media print{{main{{margin:0;padding:0;border:0}}.scroll{{overflow:visible}}.palaces,table{{min-width:0}}section{{padding:8px}}.palace{{padding:6px;min-height:95px;font-size:12px}}}}</style></head><body><main>{body}</main></body></html>"#
        )
    }

    pub fn export_html(&self, directory: &Path) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(directory)?;
        let path = directory.join(format!(
            "Jiuheng-Mechanical-{}-{}.html",
            self.question_id,
            Uuid::new_v4()
        ));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(self.html().as_bytes())?;
        file.sync_all()?;
        Ok(path)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};

    pub(crate) fn fixture(storage: &mut Storage, late: bool) -> Uuid {
        let location = GeoLocation {
            country_code: "CN".into(),
            province: "广东省".into(),
            city: "深圳市".into(),
            district: Some("南山区".into()),
            street: None,
            longitude_deg: 113.9304,
            latitude_deg: 22.5333,
            coordinate_precision: CoordinatePrecision::DistrictCenter,
        };
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        let now = zone.with_ymd_and_hms(2026, 11, 12, 12, 0, 0).unwrap();
        let question = QuestionInput {
            id: Uuid::new_v4(),
            category: QuestionCategory::LongTermFate,
            subcategory: "长期趋势".into(),
            text: "未来长期趋势如何？ <script>alert('test')</script> & 记录".into(),
            subject: "测试本人".into(),
            object: "长期趋势".into(),
            target_event: "长期趋势".into(),
            result_terms: vec![],
            options: vec![],
            location: location.clone(),
            divination_time_bjt: now,
            target_time_text: "长期".into(),
            locked_at_bjt: now,
            birth: Some(BirthParameters {
                sex: BiologicalSex::Female,
                date_time_bjt: zone
                    .with_ymd_and_hms(
                        1990,
                        5,
                        17,
                        if late { 23 } else { 8 },
                        if late { 50 } else { 30 },
                        0,
                    )
                    .unwrap(),
                location,
            }),
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
    fn saved_mechanical_boards_reopen_and_export_all_parallel_candidates() {
        let root = std::env::temp_dir().join(format!("jiuheng-mechanical-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("data.sqlite3");
        let (id, original) = {
            let mut storage = Storage::open(&path).unwrap();
            let id = fixture(&mut storage, false);
            let report = load(&storage, id).unwrap();
            assert_eq!(report.pillars.len(), 1);
            assert_eq!(
                report.pillars[0].rows[1],
                ["干支", "庚午", "辛巳", "壬午", "甲辰"]
            );
            assert_eq!(
                report.pillars[0].rows[4][4],
                "乙 · 伤官\n戊 · 七杀\n癸 · 劫财"
            );
            assert_eq!(report.ziwei.len(), 4);
            let layout = PALACE_LAYOUT
                .iter()
                .flatten()
                .flatten()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(layout.len(), 12);
            for board in &report.ziwei {
                assert_eq!(board.cells.len(), 12);
                assert_eq!(board.cells.iter().filter(|cell| cell.body).count(), 1);
                assert_eq!(
                    board
                        .cells
                        .iter()
                        .map(|cell| cell.major.len())
                        .sum::<usize>(),
                    14
                );
                assert_eq!(
                    board
                        .cells
                        .iter()
                        .map(|cell| cell.auxiliary.len())
                        .sum::<usize>(),
                    4
                );
                assert_eq!(
                    board
                        .cells
                        .iter()
                        .map(|cell| cell.branch_index)
                        .collect::<std::collections::BTreeSet<_>>(),
                    layout
                );
                assert!(board.missing.is_empty());
            }
            (id, report.text())
        };
        {
            let storage = Storage::open(&path).unwrap();
            let report = load(&storage, id).unwrap();
            assert_eq!(report.text(), original);
            let html = report.html();
            assert!(html.contains("&lt;script&gt;"));
            assert!(!html.contains("<script>"));
            assert!(html.contains("default-src 'none'"));
            assert_eq!(html.matches("<div class=palace>").count(), 48);
            let first = report.export_html(&root).unwrap();
            let second = report.export_html(&root).unwrap();
            assert_ne!(first, second);
            assert_eq!(std::fs::read_to_string(first).unwrap(), html);
            if let Ok(preview) = std::env::var("JIUHENG_MECHANICAL_PREVIEW") {
                std::fs::write(preview, html).unwrap();
            }
            assert_eq!(storage.question_count().unwrap(), 1);
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn late_zi_report_keeps_two_days_and_legacy_report_does_not_invent_hidden_stems() {
        let mut storage = Storage::open_in_memory().unwrap();
        let id = fixture(&mut storage, true);
        let report = load(&storage, id).unwrap();
        assert_eq!(report.pillars.len(), 2);
        assert_eq!(report.pillars[0].rows[1][3], "壬午");
        assert_eq!(report.pillars[1].rows[1][3], "癸未");
        assert!(report.pillars.iter().all(|table| !table.boundary_safe));
        assert!(report.pillars[1].title.contains("子初"));
        let mut old = storage
            .load_astronomy_snapshot(id)
            .unwrap()
            .unwrap()
            .birth_time_audit
            .unwrap();
        old.audit_version = 13;
        old.four_pillars_hidden_stems = None;
        let mut old_report = report;
        old_report.pillars.clear();
        old_report.ziwei.clear();
        populate(&mut old_report, &old);
        assert!(old_report.pillars.iter().all(|table| table.rows[4][1..]
            .iter()
            .all(|value| value == "旧记录未保存")));
    }

    #[test]
    fn missing_birth_or_unknown_record_cannot_generate_a_birth_chart() {
        let mut storage = Storage::open_in_memory().unwrap();
        let id = crate::reading_report::tests::fixture(&mut storage, false);
        let report = load(&storage, id).unwrap();
        assert!(report.pillars.is_empty() && report.ziwei.is_empty());
        assert!(report.pillar_notice.contains("未提供出生资料"));
        assert!(!report.verification.contains("已校验"));
        assert!(load(&storage, Uuid::new_v4()).is_err());
    }
}
