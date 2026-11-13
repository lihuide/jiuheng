use divination_core::*;
use divination_storage::{calculate_astronomy_snapshot, AstronomyAuditSnapshot, Storage};
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub const SCOPE: &str = "梅花 / 奇门机械盘试用：按原占问时间和地点展示结构，不提供吉凶或应期。";
pub const QIMEN_LAYOUT: [[u8; 3]; 3] = [[4, 9, 2], [3, 5, 7], [8, 1, 6]];

#[derive(Debug, Clone)]
pub struct MeihuaBoard {
    pub title: String,
    pub calculation: Vec<String>,
    pub primary: String,
    pub changed: String,
    pub primary_bits: u8,
    pub changed_bits: u8,
    pub moving_line: u8,
    pub middle: String,
    pub boundary_safe: bool,
}

#[derive(Debug, Clone)]
pub struct QimenCell {
    pub number: u8,
    pub title: String,
    pub stem: String,
    pub hidden_jia: String,
    pub markers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct QimenBoard {
    pub title: String,
    pub context: Vec<String>,
    pub cells: Vec<QimenCell>,
    pub missing: String,
    pub boundary_safe: bool,
}

#[derive(Debug, Clone)]
pub struct TimeReport {
    pub question_id: Uuid,
    pub question: String,
    pub metadata: Vec<(String, String)>,
    pub meihua: Vec<MeihuaBoard>,
    pub qimen: Vec<QimenBoard>,
    pub meihua_notice: String,
    pub qimen_notice: String,
    pub verification: String,
}

pub fn load(storage: &Storage, id: Uuid) -> Result<TimeReport, String> {
    let detail = storage
        .load_question_detail(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "记录不存在，请刷新永久记录。".to_owned())?;
    let question = &detail.question;
    let mut report = TimeReport {
        question_id: id,
        question: question.text.clone(),
        metadata: vec![
            (
                "原占问时间（北京时间）".into(),
                question.divination_time_bjt.to_rfc3339(),
            ),
            ("占问地点".into(), crate::location_label(&question.location)),
            (
                "地点精度".into(),
                crate::location_precision_notice(&question.location).into(),
            ),
        ],
        meihua: Vec::new(),
        qimen: Vec::new(),
        meihua_notice: "旧记录未保存梅花时间起卦，未补造历史卦象。".into(),
        qimen_notice: "旧记录未保存奇门机械盘，未回填新字段。".into(),
        verification: "已读取原占问；旧记录没有历法快照。".into(),
    };
    let Some(stored) = storage
        .load_astronomy_snapshot(id)
        .map_err(|e| e.to_string())?
    else {
        return Ok(report);
    };
    let current = calculate_astronomy_snapshot(question).map_err(|e| e.to_string())?;
    verify(&stored, &current)?;
    report.metadata.push((
        "真太阳时（同一时区表示）".into(),
        stored
            .true_solar_time_bjt
            .clone()
            .unwrap_or_else(|| "旧记录未保存".into()),
    ));
    report.verification = format!(
        "原始快照内容已校验，并按原占问时间、地点重算核对；显示快照 v{} 保存的梅花与奇门字段。",
        stored.snapshot_version
    );
    populate(&mut report, &stored)?;
    Ok(report)
}

fn verify(stored: &AstronomyAuditSnapshot, current: &AstronomyAuditSnapshot) -> Result<(), String> {
    if !(1..=30).contains(&stored.snapshot_version) {
        return Err("此历法快照版本暂不支持，已停止生成报告。".into());
    }
    let meihua_matches = if stored.snapshot_version >= 14 {
        stored.meihua_time_casting_audit == current.meihua_time_casting_audit
    } else {
        stored.meihua_time_casting_audit.is_none()
    };
    let qimen_matches = match stored.snapshot_version {
        1..=17 => stored.qimen_ju_audit.is_none(),
        version => match (&stored.qimen_ju_audit, &current.qimen_ju_audit) {
            (Some(old), Some(now)) => match version {
                18 => crate::qimen_ju_v1_fields_match(old, now),
                19 => crate::qimen_ju_v2_fields_match(old, now),
                20 => crate::qimen_ju_v3_fields_match(old, now),
                _ => old == now,
            },
            _ => false,
        },
    };
    if stored.question_id != current.question_id
        || (stored.snapshot_version >= 2
            && stored.true_solar_time_bjt != current.true_solar_time_bjt)
        || !meihua_matches
        || !qimen_matches
    {
        return Err("保存的梅花或奇门机械盘与原输入核对不一致，已停止生成报告。".into());
    }
    Ok(())
}

fn populate(report: &mut TimeReport, snapshot: &AstronomyAuditSnapshot) -> Result<(), String> {
    if snapshot.meihua_time_casting_audit.is_none() && snapshot.snapshot_version >= 14 {
        report.meihua_notice = "本记录的农历日期未能形成可用候选，未生成梅花时间卦象。".into();
    }
    if let Some(audit) = &snapshot.meihua_time_casting_audit {
        let lunar = &audit.lunisolar_date_candidate;
        report.metadata.push((
            "梅花取数日期".into(),
            format!(
                "{}年（{}年数 {}）· {}{}月 · {}日；现代农历候选",
                lunar.nominal_lunar_year,
                audit.year_branch.zh_name(),
                audit.year_branch.index() + 1,
                if lunar.is_leap_month { "闰" } else { "" },
                lunar.month_number,
                lunar.day_number
            ),
        ));
        report.meihua_notice = if lunar.is_leap_month {
            "当前为闰月，取数口径尚未确定，未生成卦象。".into()
        } else {
            format!("北京时间与真太阳时两套候选{}，保持并列。中爻结构是从保存卦象派生的观察，尚未选定互卦口径；体用、旺衰、吉凶与应期未开放。", if audit.candidates_identical { "相同" } else { "不同" })
        };
        for (label, candidate) in [
            ("北京时间", &audit.beijing_time_candidate),
            ("真太阳时", &audit.true_solar_time_candidate),
        ] {
            if let Some(candidate) = candidate {
                let cast = &candidate.cast;
                let input = cast.input;
                let primary = hexagram_identity(cast.primary_bits).map_err(|e| e.to_string())?;
                let changed = hexagram_identity(cast.changed_bits).map_err(|e| e.to_string())?;
                report.meihua.push(MeihuaBoard {
                    title: format!("梅花 · {label}"),
                    calculation: vec![
                        format!(
                            "年支 {} + 月 {} + 日 {} = {}；除 8 取数 {} → 上卦{}",
                            input.year_branch_number,
                            input.lunar_month_number,
                            input.lunar_day_number,
                            cast.year_month_day_sum,
                            cast.upper_trigram_number,
                            cast.upper_trigram.zh_name()
                        ),
                        format!(
                            "{} + {}时数 {} = {}；除 8 取数 {} → 下卦{}",
                            cast.year_month_day_sum,
                            candidate.shichen.branch.zh_name(),
                            input.hour_branch_number,
                            cast.year_month_day_hour_sum,
                            cast.lower_trigram_number,
                            cast.lower_trigram.zh_name()
                        ),
                        format!(
                            "{} 除 6 取数 {} → {}动；整除 8 按 8，整除 6 按 6。",
                            cast.year_month_day_hour_sum,
                            cast.moving_line_bottom_up,
                            line_label(cast.moving_line_bottom_up)
                        ),
                    ],
                    primary: primary.compact_zh(),
                    changed: changed.compact_zh(),
                    primary_bits: cast.primary_bits,
                    changed_bits: cast.changed_bits,
                    moving_line: cast.moving_line_bottom_up,
                    middle: cast
                        .mutual_raw_audit()
                        .map_err(|e| e.to_string())?
                        .compact_zh(),
                    boundary_safe: candidate.mechanical_boundaries_safe,
                });
            }
        }
    }
    if let Some(audit) = &snapshot.qimen_ju_audit {
        report.qimen_notice = "仅展示节气三元局数候选、九宫地盘、旬首及本旬值符值使身份。两种时间口径未选定；天盘、逐时星门移动、八神、吉凶与应期尚未完成。".into();
        for (label, day) in [
            ("北京时间", &audit.beijing_midnight_candidate),
            ("真太阳时", &audit.true_solar_midnight_candidate),
        ] {
            let mut board = QimenBoard {
                title: format!("奇门 · {label}零点日界"),
                context: vec![
                    format!(
                        "节气 {} → {} · {} · {} · {}",
                        audit.active_solar_term.zh_name(),
                        audit.next_solar_term.zh_name(),
                        audit.dun.zh_name(),
                        day.yuan.zh_name(),
                        day.bureau_number
                            .map(|n| format!("{n}局"))
                            .unwrap_or_else(|| "局数未确定".into())
                    ),
                    format!(
                        "日干支 {} · 符头 {}",
                        day.day_designation.zh_name(),
                        day.symbol_head.zh_name()
                    ),
                ],
                cells: Vec::new(),
                missing: String::new(),
                boundary_safe: day.mechanical_boundaries_safe,
            };
            let hour = day.hour_xun_candidate.as_ref();
            let identity = hour.and_then(|h| h.duty_identity_candidate.as_ref());
            if let Some(hour) = hour {
                board.context.push(format!(
                    "时干支 {} · 旬首 {} · 所隐六仪 {}",
                    hour.hour_designation.zh_name(),
                    hour.xun_head.zh_name(),
                    hour.hidden_instrument.zh_name()
                ));
                board.boundary_safe &= hour.mechanical_boundaries_safe;
            } else {
                board.context.push("旧记录未保存时旬首，未补算。".into());
            }
            if let Some(identity) = identity {
                board.context.push(format!(
                    "本旬值符：{}；本旬值使：{}；当前逐时落宫尚未计算。",
                    identity.duty_chief_star.zh_name(),
                    identity.duty_envoy_gate.zh_name()
                ));
                if identity.center_hosting_applied {
                    board.context.push(
                        "旬首落中五；本候选寄坤二取值使门，图中分别标出原宫与取门宫。".into(),
                    );
                }
            } else {
                board
                    .context
                    .push("本记录未保存值符值使身份；未生成逐时星门盘。".into());
            }
            if let Some(earth) = &day.earth_plate {
                for cell in &earth.cells_by_palace_number {
                    let mut markers = Vec::new();
                    if hour.is_some_and(|h| h.xun_head_palace == Some(cell.palace)) {
                        markers.push("旬首地盘宫".into());
                    }
                    if let Some(identity) = identity {
                        if identity.xun_head_palace == cell.palace {
                            markers.push(format!(
                                "本旬值符身份：{}",
                                identity.duty_chief_star.zh_name()
                            ));
                        }
                        if identity.duty_envoy_source_palace == cell.palace {
                            markers.push(format!(
                                "本旬值使取门宫：{}",
                                identity.duty_envoy_gate.zh_name()
                            ));
                        }
                    }
                    board.cells.push(QimenCell {
                        number: cell.palace.number(),
                        title: cell.palace.compact_zh(),
                        stem: cell.instrument_or_wonder.zh_name().into(),
                        hidden_jia: cell
                            .hidden_jia
                            .map(|jia| format!("隐{}", jia.zh_name()))
                            .unwrap_or_default(),
                        markers,
                    });
                }
            } else {
                board.missing = if audit.table_entry_evidence_conflict {
                    "当前节气的局数有传本文字冲突，未生成地盘，也不推定旬首落宫或值符值使。".into()
                } else {
                    "旧记录未保存地盘，未回填新九宫。".into()
                };
            }
            report.qimen.push(board);
        }
    }
    Ok(())
}

pub fn line_label(line: u8) -> &'static str {
    match line {
        1 => "初爻",
        2 => "二爻",
        3 => "三爻",
        4 => "四爻",
        5 => "五爻",
        6 => "上爻",
        _ => "未知爻",
    }
}

pub fn line_glyph(bits: u8, line: u8) -> &'static str {
    if bits & (1 << (line - 1)) != 0 {
        "━━━━━━"
    } else {
        "━━　━━"
    }
}

fn html_line(bits: u8, line: u8) -> &'static str {
    if bits & (1 << (line - 1)) != 0 {
        "<span class=yao role=img aria-label=阳爻><i></i></span>"
    } else {
        "<span class=yao role=img aria-label=阴爻><i></i><i></i></span>"
    }
}

impl TimeReport {
    pub fn text(&self) -> String {
        let mut out = format!("九衡 · 梅花 / 奇门机械盘\n{SCOPE}\n{}\n", self.question);
        for (name, value) in &self.metadata {
            out.push_str(&format!("{name}：{value}\n"));
        }
        out.push_str(&format!("\n梅花时间盘\n{}\n", self.meihua_notice));
        for board in &self.meihua {
            out.push_str(&format!(
                "\n{}\n{}\n{}\n本卦 {} → 变卦 {}\n",
                board.title,
                crate::mechanical_report::boundary_text(board.boundary_safe),
                board.calculation.join("\n"),
                board.primary,
                board.changed
            ));
            for line in (1..=6).rev() {
                out.push_str(&format!(
                    "{}：{} → {} {}\n",
                    line_label(line),
                    line_glyph(board.primary_bits, line),
                    line_glyph(board.changed_bits, line),
                    if line == board.moving_line {
                        "动爻"
                    } else {
                        "静爻"
                    }
                ));
            }
            out.push_str(&format!("派生中爻观察：{}\n", board.middle));
        }
        out.push_str(&format!("\n奇门地盘\n{}\n", self.qimen_notice));
        for board in &self.qimen {
            out.push_str(&format!(
                "\n{}\n{}\n{}\n{}\n",
                board.title,
                board.context.join("\n"),
                crate::mechanical_report::boundary_text(board.boundary_safe),
                board.missing
            ));
            for cell in &board.cells {
                out.push_str(&format!(
                    "{}：地盘 {} · {} · {}\n",
                    cell.title,
                    cell.stem,
                    cell.hidden_jia,
                    cell.markers.join("；")
                ));
            }
        }
        out.push_str(&format!(
            "\n{}\n记录编号 {} · 程序 v{}\n",
            self.verification,
            self.question_id,
            env!("CARGO_PKG_VERSION")
        ));
        out
    }

    pub fn html(&self) -> String {
        use crate::reading_report::escape_html as e;
        let mut out = format!("<!doctype html><html lang=zh-CN><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><meta http-equiv=Content-Security-Policy content=\"default-src 'none'; style-src 'unsafe-inline'\"><title>九衡梅花与奇门机械盘</title><style>body{{margin:0;background:#f3f1eb;color:#243d36;font:16px/1.7 'Microsoft YaHei',sans-serif}}main{{max-width:1020px;margin:24px auto;padding:30px;background:#fffefa;border-top:6px solid #345447}}h1{{font-size:28px}}h2{{margin-top:30px}}h3{{margin:0 0 12px}}.notice{{padding:14px;background:#eee8d8}}section{{border:1px solid #d8ded7;border-radius:6px;padding:18px;margin:20px 0}}.scroll{{overflow-x:auto}}table{{border-collapse:collapse;width:100%;min-width:550px}}th,td{{padding:10px;border:1px solid #d8ded7;text-align:left}}th{{background:#345447;color:white}}.moving{{background:#faf0de;color:#904d2d}}.glyph{{white-space:pre}}.yao{{display:inline-flex;width:96px;gap:16px;vertical-align:middle}}.yao i{{display:block;flex:1;height:5px;background:currentColor}}.palaces{{display:grid;grid-template-columns:repeat(3,1fr);gap:6px;min-width:600px}}.palace{{padding:14px;background:#edf2ed;min-height:135px}}.stem{{font-size:26px;font-weight:bold}}.marker{{color:#8b502c}}.muted,footer{{font-size:14px;color:#687770}}@media print{{body,main{{margin:0;background:white}}main{{padding:10px}}section{{break-inside:avoid}}.scroll{{overflow:visible}}}}</style><main><h1>九衡 · 梅花 / 奇门机械盘</h1><p class=notice>{}</p><h2>{}</h2>", e(SCOPE), e(&self.question));
        for (name, value) in &self.metadata {
            out.push_str(&format!("<p><b>{}</b>：{}</p>", e(name), e(value)));
        }
        out.push_str(&format!(
            "<h2>梅花时间盘</h2><p>{}</p>",
            e(&self.meihua_notice)
        ));
        for board in &self.meihua {
            out.push_str(&format!(
                "<section><h3>{}</h3><p class=muted>{}</p>",
                e(&board.title),
                e(crate::mechanical_report::boundary_text(board.boundary_safe))
            ));
            for value in &board.calculation {
                out.push_str(&format!("<p>{}</p>", e(value)));
            }
            out.push_str(&format!("<p>本卦 <b>{}</b> → 变卦 <b>{}</b></p><div class=scroll><table><tr><th>爻位</th><th>本卦</th><th>变卦</th><th>变化</th></tr>", e(&board.primary), e(&board.changed)));
            for line in (1..=6).rev() {
                let moving = line == board.moving_line;
                out.push_str(&format!("<tr{}><td>{}</td><td class=glyph>{}</td><td class=glyph>{}</td><td>{}</td></tr>", if moving { " class=moving" } else { "" }, line_label(line), html_line(board.primary_bits, line), html_line(board.changed_bits, line), if moving { "动爻 · 阴阳翻转" } else { "静爻" }));
            }
            out.push_str(&format!(
                "</table></div><p class=muted>派生中爻观察：{}</p></section>",
                e(&board.middle)
            ));
        }
        out.push_str(&format!(
            "<h2>奇门地盘</h2><p>{}</p>",
            e(&self.qimen_notice)
        ));
        for board in &self.qimen {
            out.push_str(&format!("<section><h3>{}</h3>", e(&board.title)));
            for value in &board.context {
                out.push_str(&format!("<p>{}</p>", e(value)));
            }
            out.push_str(&format!(
                "<p class=muted>{}</p>",
                e(crate::mechanical_report::boundary_text(board.boundary_safe))
            ));
            if !board.missing.is_empty() {
                out.push_str(&format!("<p class=notice>{}</p>", e(&board.missing)));
            }
            if !board.cells.is_empty() {
                out.push_str("<p class=muted>九宫示意 · 上南下北；仅地盘，非完整时盘</p><div class=scroll><div class=palaces>");
                for number in QIMEN_LAYOUT.into_iter().flatten() {
                    if let Some(cell) = board.cells.iter().find(|cell| cell.number == number) {
                        out.push_str(&format!("<div class=palace><b>{}</b><div class=stem>{}</div><div>{}</div><div class=marker>{}</div></div>", e(&cell.title), e(&cell.stem), e(&cell.hidden_jia), cell.markers.iter().map(|m| e(m)).collect::<Vec<_>>().join("<br>")));
                    }
                }
                out.push_str("</div></div>");
            }
            out.push_str("</section>");
        }
        out.push_str(&format!("<footer><p>{}</p><p>记录编号 {} · 程序 v{}</p><p>包含占问内容、时间和地点；离线可读，可通过浏览器打印。</p></footer></main></html>", e(&self.verification), self.question_id, env!("CARGO_PKG_VERSION")));
        out
    }

    pub fn export_html(&self, directory: &Path) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(directory)?;
        let path = directory.join(format!(
            "Jiuheng-Meihua-Qimen-{}-{}.html",
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
#[path = "time_report_tests.rs"]
pub(crate) mod tests;
