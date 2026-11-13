use divination_core::{
    hexagram_transformation, liuyao_six_relatives_audit, liuyao_six_spirits_audit, LineType,
};
use divination_storage::Storage;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const SCOPE: &str = "排盘记录试用版：展示卦爻结构与变化，自动占断尚未开放。";

#[derive(Debug, Clone)]
pub struct ReadingLine {
    pub position: String,
    pub kind: LineType,
    pub role: String,
    pub relative: String,
    pub najia: String,
    pub spirit: String,
}

impl ReadingLine {
    pub fn change(&self) -> &'static str {
        match self.kind {
            LineType::OldYin => "动爻 · 阴变阳",
            LineType::OldYang => "动爻 · 阳变阴",
            _ => "静爻 · 不变",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReadingReport {
    pub question_id: Uuid,
    pub question_text: String,
    pub metadata: Vec<(String, String)>,
    pub primary: String,
    pub changed: String,
    pub palace: String,
    pub lines_top_down: Vec<ReadingLine>,
    pub changes: String,
    pub verification: String,
    pub transcript_hash: Option<String>,
    pub notice: String,
}

pub fn load(storage: &Storage, question_id: Uuid) -> Result<ReadingReport, String> {
    let detail = storage
        .load_question_detail(question_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "这条记录不存在，请刷新历史记录。".to_owned())?;
    let question = &detail.question;
    let location = &question.location;
    let mut report = ReadingReport {
        question_id,
        question_text: question.text.clone(),
        metadata: vec![
            ("类别".into(), question.category.zh_name().into()),
            (
                "占问时间（北京时间）".into(),
                question.divination_time_bjt.to_rfc3339(),
            ),
            (
                "锁定时间（北京时间）".into(),
                question.locked_at_bjt.to_rfc3339(),
            ),
            (
                "地点".into(),
                format!(
                    "{} {} {} {}",
                    location.province,
                    location.city,
                    location.district.as_deref().unwrap_or(""),
                    location.street.as_deref().unwrap_or("")
                ),
            ),
            (
                "坐标".into(),
                format!(
                    "经度 {:.6}，纬度 {:.6}；{}",
                    location.longitude_deg,
                    location.latitude_deg,
                    crate::location_precision_notice(location)
                ),
            ),
            ("主体".into(), question.subject.clone()),
            ("对象".into(), question.object.clone()),
            ("目标事件".into(), question.target_event.clone()),
            ("所问时段".into(), question.target_time_text.clone()),
        ],
        primary: "尚未完成六爻起法".into(),
        changed: String::new(),
        palace: String::new(),
        lines_top_down: Vec::new(),
        changes: "本记录尚无完整的六次起法材料。".into(),
        verification: "问题已读取；六爻起法待完成。".into(),
        transcript_hash: None,
        notice: divination_core::practical_action_notice_for_classified_question(
            question.category,
            Some(&detail.ancient_question_type),
        )
        .into(),
    };
    if !question.options.is_empty() {
        report
            .metadata
            .push(("选项".into(), question.options.join("；")));
    }
    if let Some(revision) = &detail.revision {
        report
            .metadata
            .push(("情况变化".into(), revision.change_description.clone()));
        report
            .metadata
            .push(("前次记录".into(), revision.prior_question_id.to_string()));
    }
    let Some(artifact) = storage
        .load_liuyao_artifact(question_id)
        .map_err(|error| error.to_string())?
    else {
        if let Some(progress) = storage
            .load_liuyao_progress(question_id)
            .map_err(|error| error.to_string())?
        {
            report.changes = format!(
                "已保存 {}/6 爻，可从永久记录继续完成；尚不生成完整排盘。",
                progress.events.len()
            );
        }
        return Ok(report);
    };
    let cast = artifact.cast;
    let pair = hexagram_transformation(cast.primary_bits, cast.changed_bits)
        .map_err(|error| error.to_string())?;
    let relatives =
        liuyao_six_relatives_audit(cast.primary_bits).map_err(|error| error.to_string())?;
    let spirits = detail
        .astronomy_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.sexagenary_day_bjt_midnight)
        .map(|day| liuyao_six_spirits_audit(cast.primary_bits, day.stem))
        .transpose()
        .map_err(|error| error.to_string())?;
    let positions = ["初爻", "二爻", "三爻", "四爻", "五爻", "上爻"];
    let mut moving = Vec::new();
    for (index, cast_line) in cast.lines.iter().enumerate() {
        let relative = &relatives.lines_bottom_up[index];
        if cast_line.line_type.is_moving() {
            moving.push(positions[index]);
        }
        report.lines_top_down.push(ReadingLine {
            position: positions[index].into(),
            kind: cast_line.line_type,
            role: if relative.najia.is_host {
                "世"
            } else if relative.najia.is_response {
                "应"
            } else {
                "—"
            }
            .into(),
            relative: relative.relationship.zh_name().into(),
            najia: relative.najia.designation_zh(),
            spirit: spirits
                .as_ref()
                .map(|audit| audit.lines_bottom_up[index].spirit.zh_name())
                .unwrap_or("旧记录缺日干")
                .into(),
        });
    }
    report.lines_top_down.reverse();
    report.primary = pair.primary.compact_zh();
    report.changed = pair.changed.compact_zh();
    report.palace = relatives.najia.eight_palace.compact_zh();
    report.changes = if moving.is_empty() {
        "六爻皆静，本卦与变卦结构相同。".into()
    } else {
        format!(
            "{}个动爻：{}。老阴变阳、老阳变阴，其余爻保持原状。",
            moving.len(),
            moving.join("、")
        )
    };
    report.verification = "已从本地保存的六次起法材料重新计算，与保存卦象一致。".into();
    report.transcript_hash = Some(cast.transcript_hash);
    Ok(report)
}

pub fn symbol(yang: bool) -> &'static str {
    if yang {
        "━━━━━━"
    } else {
        "━━  ━━"
    }
}

pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

impl ReadingReport {
    pub fn text(&self) -> String {
        let mut output = format!("九衡 · 六爻排盘记录\n{SCOPE}\n\n{}\n", self.question_text);
        for (label, value) in &self.metadata {
            output.push_str(&format!("{label}：{value}\n"));
        }
        output.push_str(&format!(
            "\n本卦：{}\n变卦：{}\n{}\n{}\n",
            self.primary, self.changed, self.palace, self.changes
        ));
        for line in &self.lines_top_down {
            output.push_str(&format!(
                "{} {} {}({}) {} {} {} {} → {} [{}]\n",
                line.position,
                symbol(line.kind.primary_is_yang()),
                line.kind.zh_name(),
                line.kind.value(),
                line.role,
                line.relative,
                line.najia,
                line.spirit,
                symbol(line.kind.changed_is_yang()),
                line.change()
            ));
        }
        output.push_str(&format!("\n读盘说明：图从上爻向初爻显示，起法顺序由初爻向上。世应、纳甲、六亲、六神是程序现行约定下的结构标签；这里未判断旺衰、用神、吉凶或应期。\n\n核对状态：{}\n现实行动提示：{}\n记录编号：{}\n报告程序版本：{}\n",
            self.verification, self.notice, self.question_id, env!("CARGO_PKG_VERSION")));
        if let Some(hash) = &self.transcript_hash {
            output.push_str(&format!("起法材料校验值：{hash}\n"));
        }
        output
    }

    pub fn html(&self) -> String {
        let meta = self
            .metadata
            .iter()
            .map(|(label, value)| {
                format!(
                    "<div><dt>{}</dt><dd>{}</dd></div>",
                    escape_html(label),
                    escape_html(value)
                )
            })
            .collect::<String>();
        let rows = self.lines_top_down.iter().map(|line| format!(
            "<tr{}><th>{}</th><td class=yao>{}</td><td>{} {}</td><td>{}</td><td>{} {}</td><td>{}</td><td class=yao>{}</td><td>{}</td></tr>",
            if line.kind.is_moving() { " class=moving" } else { "" }, escape_html(&line.position),
            symbol(line.kind.primary_is_yang()), line.kind.zh_name(), line.kind.value(), escape_html(&line.role),
            escape_html(&line.relative), escape_html(&line.najia), escape_html(&line.spirit),
            symbol(line.kind.changed_is_yang()), line.change())).collect::<String>();
        format!(
            r#"<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'"><title>九衡 · 六爻排盘记录</title><style>
body{{margin:0;background:#f5f3ed;color:#283e34;font:16px/1.7 'Microsoft YaHei',sans-serif}}main{{max-width:1040px;margin:32px auto;padding:32px;background:#fffefb;border-top:6px solid #304f40}}h1{{font-size:28px;margin:0}}.tag,footer{{color:#657066;font-size:14px}}.scope{{padding:12px 16px;background:#f2ead8;border-radius:8px}}h2{{font-size:22px;overflow-wrap:anywhere;white-space:pre-wrap}}dl{{display:grid;grid-template-columns:1fr 1fr;gap:10px 24px}}dt{{color:#6c756e;font-size:13px}}dd{{margin:0;overflow-wrap:anywhere;white-space:pre-wrap}}.pair{{display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-top:24px}}.pair div{{padding:18px;background:#edf2ed;border-radius:8px}}.scroll{{overflow-x:auto}}table{{border-collapse:collapse;width:100%;margin:16px 0;white-space:nowrap}}td,th{{text-align:left;padding:10px;border-bottom:1px solid #ddd}}th{{font-weight:500}}thead{{background:#304f40;color:white}}.moving{{background:#faf0df}}.yao{{font-family:monospace;white-space:pre}}pre{{white-space:pre-wrap;overflow-wrap:anywhere;font:14px/1.7 'Microsoft YaHei',sans-serif;color:#586158}}details{{border-top:1px solid #ddd;padding-top:12px}}@media(max-width:650px){{main{{margin:0;padding:20px}}dl,.pair{{grid-template-columns:1fr}}}}@media print{{body{{background:white}}main{{margin:0;padding:0;border:0}}details{{display:none}}tr{{break-inside:avoid}}}}
</style></head><body><main><div class="tag">JIUHENG / 离线记录</div><h1>九衡 · 六爻排盘记录</h1><p class="scope">{scope}</p><h2>{question}</h2><dl>{meta}</dl><div class="pair"><div>本卦<br><strong>{primary}</strong></div><div>变卦<br><strong>{changed}</strong></div></div><p>{palace}</p><p>{changes}</p><div class="scroll"><table><thead><tr><th>爻位</th><th>本爻</th><th>爻值</th><th>世应</th><th>六亲 / 纳甲</th><th>六神</th><th>变后</th><th>变化</th></tr></thead><tbody>{rows}</tbody></table></div><p>图从上爻向初爻显示；起法从初爻向上。老阴、老阳为动爻。世应、纳甲、六亲、六神为程序现行约定下的结构标签，未判断旺衰、用神、吉凶或应期。</p><p>{verification}</p><p>现实行动提示：{notice}</p><details><summary>完整文字记录与校验信息</summary><pre>{text}</pre></details><footer>记录编号 {id} · 报告程序 v{version}<br>可用浏览器打印功能保存为 PDF。本文件离线可读，含本次占问信息。</footer></main></body></html>"#,
            scope = escape_html(SCOPE),
            question = escape_html(&self.question_text),
            primary = escape_html(&self.primary),
            changed = escape_html(&self.changed),
            palace = escape_html(&self.palace),
            changes = escape_html(&self.changes),
            verification = escape_html(&self.verification),
            notice = escape_html(&self.notice),
            text = escape_html(&self.text()),
            id = self.question_id,
            version = env!("CARGO_PKG_VERSION")
        )
    }

    pub fn export_html(&self, directory: &Path) -> Result<PathBuf, std::io::Error> {
        std::fs::create_dir_all(directory)?;
        let path = directory.join(format!(
            "Jiuheng-Reading-{}-{}.html",
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
    use divination_core::{
        build_event_fingerprint, EntropyEvent, KeywordClassifier, LiuyaoTranscript,
        QuestionCategory, QuestionInput,
    };

    pub(crate) fn fixture(storage: &mut Storage, complete: bool) -> Uuid {
        let now = FixedOffset::east_opt(8 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 11, 12, 10, 30, 0)
            .unwrap();
        let question = QuestionInput {
            id: Uuid::new_v4(),
            category: QuestionCategory::Career,
            subcategory: String::new(),
            text: "测试：这次项目上线是否顺利？ <script>alert('test')</script> & 记录".into(),
            subject: "测试主体".into(),
            object: "项目".into(),
            target_event: "上线".into(),
            result_terms: vec![],
            options: vec![],
            location: storage.locations().unwrap()[0].clone(),
            divination_time_bjt: now,
            target_time_text: "下周".into(),
            birth: None,
            locked_at_bjt: now,
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
        if complete {
            let mut transcript = LiuyaoTranscript::new(question.id, [0x42; 32]);
            for sequence in 0..6 {
                transcript
                    .push_event(EntropyEvent {
                        sequence,
                        wall_clock_unix_nanos: 1_788_068_725_000_000_000 + sequence as i64,
                        monotonic_nanos: 10_000 + sequence as u64,
                        press_duration_nanos: 100_000 + sequence as u64,
                        delta_since_previous_nanos: 200_000 + sequence as u64,
                        click_x_micropoints: 100,
                        click_y_micropoints: 200,
                        pointer_path: Vec::new(),
                    })
                    .unwrap();
            }
            storage
                .save_liuyao_artifact(question.id, &transcript, &transcript.finish().unwrap())
                .unwrap();
        }
        question.id
    }

    #[test]
    fn saved_report_survives_database_reopen_and_exports_safely() {
        let root = std::env::temp_dir().join(format!("jiuheng-report-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let database = root.join("test.sqlite3");
        let (id, original) = {
            let mut storage = Storage::open(&database).unwrap();
            let id = fixture(&mut storage, true);
            let report = load(&storage, id).unwrap();
            assert_eq!(report.lines_top_down.len(), 6);
            assert_eq!(report.lines_top_down[0].position, "上爻");
            assert_eq!(report.lines_top_down[5].position, "初爻");
            assert_eq!(
                report
                    .lines_top_down
                    .iter()
                    .filter(|line| line.role == "世")
                    .count(),
                1
            );
            assert_eq!(
                report
                    .lines_top_down
                    .iter()
                    .filter(|line| line.role == "应")
                    .count(),
                1
            );
            assert!(report.transcript_hash.is_some());
            (id, report.text())
        };
        {
            let storage = Storage::open(&database).unwrap();
            let report = load(&storage, id).unwrap();
            assert_eq!(report.text(), original);
            let html = report.html();
            assert!(!html.contains("<script>"));
            assert!(html.contains("&lt;script&gt;"));
            assert!(html.contains("default-src 'none'"));
            assert!(html.contains(SCOPE));
            let first = report.export_html(&root.join("exports")).unwrap();
            let second = report.export_html(&root.join("exports")).unwrap();
            assert_ne!(first, second);
            assert_eq!(std::fs::read_to_string(&first).unwrap(), html);
            assert_eq!(std::fs::read_to_string(&second).unwrap(), html);
            let blocked = root.join("not-a-directory");
            std::fs::write(&blocked, "keep").unwrap();
            assert!(report.export_html(&blocked).is_err());
            assert_eq!(std::fs::read_to_string(blocked).unwrap(), "keep");
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unfinished_and_missing_records_never_claim_verified_casts() {
        let mut storage = Storage::open_in_memory().unwrap();
        let id = fixture(&mut storage, false);
        let report = load(&storage, id).unwrap();
        assert!(report.lines_top_down.is_empty());
        assert!(report.transcript_hash.is_none());
        assert!(!report.verification.contains("一致"));
        assert!(report.text().contains("待完成"));
        assert!(load(&storage, Uuid::new_v4()).is_err());
    }

    #[test]
    fn moving_and_static_line_explanations_have_correct_directions() {
        for (kind, expected) in [
            (LineType::OldYin, "动爻 · 阴变阳"),
            (LineType::OldYang, "动爻 · 阳变阴"),
            (LineType::YoungYin, "静爻 · 不变"),
            (LineType::YoungYang, "静爻 · 不变"),
        ] {
            let line = ReadingLine {
                position: "初爻".into(),
                kind,
                role: String::new(),
                relative: String::new(),
                najia: String::new(),
                spirit: String::new(),
            };
            assert_eq!(line.change(), expected);
        }
        assert_eq!(escape_html("<>&\"'"), "&lt;&gt;&amp;&quot;&#39;");
    }
}
