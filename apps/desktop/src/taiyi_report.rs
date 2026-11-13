use divination_core::*;
use divination_storage::{Storage, TaiyiCalculation, TaiyiRecord};
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub const SCOPE: &str = "太乙机械候选：按明确参数排出位置、主客算及将位；不提供吉凶或应期。";
pub const LAYOUT: [[u8; 3]; 3] = [[9, 2, 7], [4, 5, 6], [3, 8, 1]];
pub const ORIENTATION: &str = "太乙宫号 · 上南下北、左东右西；中五只显示将位，不替将位寄宫。";
pub const LIMITS: &str = "尚未从实际日期确定太乙年界、积年口径或阴阳遁。保存时间只用于查找记录，不参与排盘。年计与阴阳七十二局分别计算，不混用门盘；完整太乙盘及正式断法尚未完成。";
pub const DISPUTES: &str = "阴遁传本中有 3 处始击、9 处主客算单元格异文；固定开源参考 144 局中 4 局存在单项算数差异。本页保留现有算法候选，未按表格差异改写结果，仍待原页复核。";

#[derive(Debug, Clone)]
pub struct Cell {
    pub number: u8,
    pub items: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct GateBoard {
    pub title: String,
    pub cells: Vec<Cell>,
    pub missing: String,
}
#[derive(Debug, Clone)]
pub struct TaiyiReport {
    pub record: TaiyiRecord,
    pub question: String,
    pub original_time: String,
    pub context: Vec<String>,
    pub cells: Vec<Cell>,
    pub spirits: Vec<(String, String, String)>,
    pub gates: Vec<GateBoard>,
    pub marks: String,
}

pub fn palace_name(number: u8) -> &'static str {
    match number {
        1 => "一宫 · 乾 / 西北",
        2 => "二宫 · 离 / 南",
        3 => "三宫 · 艮 / 东北",
        4 => "四宫 · 震 / 东",
        5 => "五宫 · 中",
        6 => "六宫 · 兑 / 西",
        7 => "七宫 · 坤 / 西南",
        8 => "八宫 · 坎 / 北",
        9 => "九宫 · 巽 / 东南",
        _ => "未知宫",
    }
}

pub fn load(storage: &Storage, question_id: Uuid) -> Result<Vec<TaiyiReport>, String> {
    let detail = storage
        .load_question_detail(question_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "占问记录不存在。".to_owned())?;
    storage
        .load_taiyi_candidates(question_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|record| {
            Ok(from_record(
                record,
                detail.question.text.clone(),
                detail.question.divination_time_bjt.to_rfc3339(),
            ))
        })
        .collect()
}

fn annual(record: &TaiyiRecord) -> Option<&TaiyiAnnualFrame> {
    match &record.calculation {
        TaiyiCalculation::Annual(frame) => Some(frame),
        TaiyiCalculation::Traditional(frame) => Some(&frame.annual_frame),
        TaiyiCalculation::Board(_) => None,
    }
}

fn from_record(record: TaiyiRecord, question: String, original_time: String) -> TaiyiReport {
    let mut context = vec![record.request.label(), LIMITS.into()];
    if let TaiyiCalculation::Traditional(candidate) = &record.calculation {
        context.push(format!("明确选择的传本假设：积年 = 1,937,281 + ({} − 724) = {}。年标签采用天文编号，0 = 公元前 1 年；没有将公历日期自动换算为传统年。来源转录仍待影印及独立复核。", candidate.supplied_traditional_year_label, candidate.accumulated_year_count));
    }
    let (palace, generals, tianmu, shiji, marks) = if let Some(frame) = annual(&record) {
        context.push(format!(
            "积年 {}；360 周期第 {}、72 周期第 {}、24 周期第 {}；宫内第 {} 年。",
            frame.accumulated_year_count,
            frame.taiyi_position.position_in_360,
            frame.taiyi_position.position_in_72,
            frame.taiyi_position.position_in_24,
            frame.taiyi_position.year_within_palace
        ));
        context.push(format!(
            "太岁 {}；计神 {}；合神 {}；主算 {}，客算 {}。",
            frame.tai_sui_designation.zh_name(),
            frame.jishen_branch.zh_name(),
            frame.heshin_branch.zh_name(),
            frame.main_count,
            frame.guest_count
        ));
        context.push(format!(
            "年值事门 {} · 门内第 {} 年；720 周期第 {}、240 周期第 {}。年计未选择阴阳遁。",
            frame.annual_duty_gate.zh_name(),
            frame.year_within_duty_gate,
            frame.eight_gate_position_in_720,
            frame.eight_gate_position_in_240
        ));
        context.push(format!(
            "天目十八步第 {}{}；始击十六位第 {}。",
            frame.tianmu_position_in_18,
            if frame.tianmu_is_repeated_stop {
                "（重留）"
            } else {
                ""
            },
            frame.shiji_ring_position
        ));
        (
            frame.taiyi_position.palace,
            [
                frame.main_great_general_palace,
                frame.main_assistant_general_palace,
                frame.guest_great_general_palace,
                frame.guest_assistant_general_palace,
            ],
            frame.tianmu,
            frame.shiji,
            frame.mechanical_relations.compact_marks_zh(),
        )
    } else if let TaiyiCalculation::Board(frame) = &record.calculation {
        context.push(format!(
            "{}第 {} 局；24 周期第 {}，宫内第 {} 步；计神 {}；主算 {}，客算 {}。",
            frame.dun.zh_name(),
            frame.position_in_72,
            frame.position_in_24,
            frame.position_within_taiyi_palace,
            frame.jishen_branch.zh_name(),
            frame.main_count,
            frame.guest_count
        ));
        context.push(format!(
            "天目十八步第 {}{}；始击十六位第 {}。",
            frame.tianmu_position_in_18,
            if frame.tianmu_is_repeated_stop {
                "（重留）"
            } else {
                ""
            },
            frame.shiji_ring_position
        ));
        context.push(DISPUTES.into());
        (
            frame.taiyi_palace,
            [
                frame.main_great_general_palace,
                frame.main_assistant_general_palace,
                frame.guest_great_general_palace,
                frame.guest_assistant_general_palace,
            ],
            frame.tianmu,
            frame.shiji,
            frame.mechanical_relations.compact_marks_zh(),
        )
    } else {
        unreachable!()
    };
    let mut cells: Vec<Cell> = (1..=9)
        .map(|number| Cell {
            number,
            items: Vec::new(),
        })
        .collect();
    cells[usize::from(palace.number() - 1)]
        .items
        .push("太乙".into());
    for (label, general) in ["主大将", "主参将", "客大将", "客参将"]
        .into_iter()
        .zip(generals)
    {
        cells[usize::from(general.number() - 1)]
            .items
            .push(label.into());
    }
    let spirits = TaiyiSixteenSpirit::RECEIVED_RING_ORDER
        .into_iter()
        .zip([
            "子", "丑", "艮", "寅", "卯", "辰", "巽", "巳", "午", "未", "坤", "申", "酉", "戌",
            "乾", "亥",
        ])
        .map(|(spirit, position)| {
            let mut names = Vec::new();
            if spirit == tianmu {
                names.push("天目");
            }
            if spirit == shiji {
                names.push("始击");
            }
            (position.into(), spirit.zh_name().into(), names.join(" / "))
        })
        .collect();
    let mut gates = Vec::new();
    if let Some(boards) = annual(&record).and_then(|f| f.annual_open_gate_boards.as_ref()) {
        for (title, board) in [
            ("开门加太乙", Some(&boards.taiyi_board)),
            ("开门加主大将", boards.main_great_general_board.as_ref()),
            ("开门加客大将", boards.guest_great_general_board.as_ref()),
        ] {
            let cells = board
                .map(|board| {
                    board
                        .placements
                        .iter()
                        .map(|p| Cell {
                            number: p.palace.number(),
                            items: vec![p.gate.zh_name().into()],
                        })
                        .collect()
                })
                .unwrap_or_default();
            gates.push(GateBoard {
                title: title.into(),
                cells,
                missing: if board.is_none() {
                    "该将位在中五，八门无中宫；此门盘停止生成，未擅自寄宫。".into()
                } else {
                    String::new()
                },
            });
        }
    }
    TaiyiReport {
        record,
        question,
        original_time,
        context,
        cells,
        spirits,
        gates,
        marks,
    }
}

impl TaiyiReport {
    pub fn text(&self) -> String {
        let mut out = format!("九衡 · 太乙机械候选\n{SCOPE}\n{}\n原占问时间（不参与太乙换算）：{}\n{}\n\n{ORIENTATION}\n", self.question, self.original_time, self.context.join("\n"));
        for number in LAYOUT.into_iter().flatten() {
            let cell = &self.cells[usize::from(number - 1)];
            out.push_str(&format!(
                "{}：{}\n",
                palace_name(number),
                if cell.items.is_empty() {
                    "—".into()
                } else {
                    cell.items.join("、")
                }
            ));
        }
        out.push_str("\n十六神位置（独立于九宫编号）\n");
        for (position, spirit, marker) in &self.spirits {
            out.push_str(&format!("{position} · {spirit} · {marker}\n"));
        }
        out.push_str(&format!("\n位置标记（不作吉凶解释）：{}\n", self.marks));
        for board in &self.gates {
            out.push_str(&format!(
                "\n年计八门 · {}\n{}\n",
                board.title, board.missing
            ));
            for cell in &board.cells {
                out.push_str(&format!(
                    "{}：{}\n",
                    palace_name(cell.number),
                    cell.items.join("、")
                ));
            }
        }
        out.push_str(&format!(
            "\n已校验保存内容并按原参数复算；候选 {}\n所属占问 {}\n保存于 {} · 程序 v{}\n",
            self.record.id,
            self.record.question_id,
            self.record.created_at,
            env!("CARGO_PKG_VERSION")
        ));
        out
    }

    pub fn html(&self) -> String {
        use crate::reading_report::escape_html as e;
        let mut out = format!("<!doctype html><html lang=zh-CN><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><meta http-equiv=Content-Security-Policy content=\"default-src 'none'; style-src 'unsafe-inline'\"><title>九衡太乙机械候选</title><style>body{{background:#f3f1eb;color:#243d36;font:16px/1.7 'Microsoft YaHei',sans-serif;margin:0}}main{{max-width:1000px;margin:24px auto;background:#fffefa;padding:30px;border-top:6px solid #345447}}h1{{font-size:28px}}.notice{{background:#eee8d8;padding:14px}}.palaces{{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px}}.palace{{background:#edf2ed;padding:14px;min-height:85px}}.marker{{color:#92502b;font-size:20px}}table{{border-collapse:collapse;width:100%}}td,th{{text-align:left;border:1px solid #d8ded7;padding:8px}}th{{background:#345447;color:white}}.muted,footer{{font-size:14px;color:#687770}}section{{margin:24px 0}}@media(max-width:600px){{main{{margin:0;padding:14px}}.palace{{padding:8px;font-size:13px}}.marker{{font-size:16px}}}}@media print{{body,main{{background:white;margin:0}}section{{break-inside:avoid}}}}</style><main><h1>九衡 · 太乙机械候选</h1><p class=notice>{}</p><h2>{}</h2><p>原占问时间（不参与太乙换算）：{}</p>", e(SCOPE), e(&self.question), e(&self.original_time));
        for value in &self.context {
            out.push_str(&format!("<p>{}</p>", e(value)));
        }
        out.push_str(&format!("<section><h2>太乙与主客将位</h2><p class=muted>{}</p>{}</section><section><h2>十六神定位</h2><p>天目与始击保持十六位定位，未挤入九宫。</p><table><tr><th>位置</th><th>十六神</th><th>标记</th></tr>", e(ORIENTATION), html_grid(&self.cells)));
        for (position, spirit, marker) in &self.spirits {
            out.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td class=marker>{}</td></tr>",
                e(position),
                e(spirit),
                e(marker)
            ));
        }
        out.push_str(&format!(
            "</table></section><p>位置标记（不作吉凶解释）：{}</p>",
            e(&self.marks)
        ));
        for board in &self.gates {
            out.push_str(&format!("<section><h2>年计八门 · {}</h2><p class=muted>三张独立门盘；不表示时计值事门旋转。</p>", e(&board.title)));
            if board.missing.is_empty() {
                out.push_str(&html_grid(&board.cells));
            } else {
                out.push_str(&format!("<p class=notice>{}</p>", e(&board.missing)));
            }
            out.push_str("</section>");
        }
        out.push_str(&format!("<footer><p>已校验保存内容并按原参数复算。</p><p>候选 {}<br>所属占问 {}<br>保存于 {} · 程序 v{}</p></footer></main></html>", self.record.id, self.record.question_id, e(&self.record.created_at), env!("CARGO_PKG_VERSION")));
        out
    }

    pub fn export_html(&self, directory: &Path) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(directory)?;
        let path = directory.join(format!(
            "Jiuheng-Taiyi-{}-{}.html",
            self.record.id,
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

fn html_grid(cells: &[Cell]) -> String {
    use crate::reading_report::escape_html as e;
    let mut out = String::from("<div class=palaces>");
    for number in LAYOUT.into_iter().flatten() {
        let items = cells
            .iter()
            .find(|c| c.number == number)
            .map(|c| c.items.join("、"))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "—".into());
        out.push_str(&format!(
            "<div class=palace><b>{}</b><div class=marker>{}</div></div>",
            e(palace_name(number)),
            e(&items)
        ));
    }
    out.push_str("</div>");
    out
}

#[cfg(test)]
#[path = "taiyi_report_tests.rs"]
mod tests;
