                                                                                
                                                                      

use crate::{QuestionCategory, QuestionInput};
use chrono::{DateTime, Duration, FixedOffset, TimeZone};
use serde::{Deserialize, Serialize};

pub const FIRST_RAIN_SCOPE_VERSION: &str = "jiuheng.weather.first_rain_scope.v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherTimeKind {
    Instant,
    Interval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherTargetWindow {
    pub kind: WeatherTimeKind,
    pub start_bjt: DateTime<FixedOffset>,
    pub end_bjt: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirstRainScopeAudit {
    pub algorithm_version: String,
                                                                   
    pub in_research_scope: bool,
    pub target_window: Option<WeatherTargetWindow>,
    pub reasons: Vec<String>,
}

                                                                            
                                                                      
pub fn parse_weather_target_window(
    anchor_bjt: DateTime<FixedOffset>,
    expression: &str,
) -> Option<WeatherTargetWindow> {
    let compact: String = expression
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    let expression = compact.strip_prefix("未来").unwrap_or(&compact);
    if expression == "今日内" || expression == "今天内" {
        let end = anchor_bjt
            .offset()
            .from_local_datetime(&anchor_bjt.date_naive().and_hms_opt(23, 59, 59)?)
            .single()?;
        if end <= anchor_bjt {
            return None;
        }
        return Some(WeatherTargetWindow {
            kind: WeatherTimeKind::Interval,
            start_bjt: anchor_bjt,
            end_bjt: end,
        });
    }
    for (suffix, kind, unit_hours, limit) in [
        ("小时以后", WeatherTimeKind::Instant, 1_i64, 72_i64),
        ("小时后", WeatherTimeKind::Instant, 1, 72),
        ("小时之内", WeatherTimeKind::Interval, 1, 72),
        ("小时内", WeatherTimeKind::Interval, 1, 72),
        ("天以后", WeatherTimeKind::Instant, 24, 7),
        ("天后", WeatherTimeKind::Instant, 24, 7),
        ("天之内", WeatherTimeKind::Interval, 24, 7),
        ("天内", WeatherTimeKind::Interval, 24, 7),
    ] {
        let Some(number) = expression.strip_suffix(suffix) else {
            continue;
        };
        let count = parse_positive_count(number)?;
        if count > limit {
            return None;
        }
        let end = anchor_bjt.checked_add_signed(Duration::hours(count * unit_hours))?;
        return Some(WeatherTargetWindow {
            kind,
            start_bjt: if kind == WeatherTimeKind::Instant {
                end
            } else {
                anchor_bjt
            },
            end_bjt: end,
        });
    }
    None
}

fn parse_positive_count(value: &str) -> Option<i64> {
    if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) {
        return value.parse::<i64>().ok().filter(|count| *count > 0);
    }
    if let Some((tens, units)) = value.split_once('十') {
        let tens = if tens.is_empty() {
            1
        } else {
            chinese_digit(tens)?
        };
        let units = if units.is_empty() {
            0
        } else {
            chinese_digit(units)?
        };
        return Some(tens * 10 + units).filter(|count| *count > 0);
    }
    chinese_digit(value).filter(|count| *count > 0)
}

fn chinese_digit(value: &str) -> Option<i64> {
    Some(match value {
        "一" => 1,
        "二" | "两" => 2,
        "三" => 3,
        "四" => 4,
        "五" => 5,
        "六" => 6,
        "七" => 7,
        "八" => 8,
        "九" => 9,
        _ => return None,
    })
}

fn time_count_char(ch: char) -> bool {
    ch.is_ascii_digit()
        || matches!(
            ch,
            '一' | '二' | '两' | '三' | '四' | '五' | '六' | '七' | '八' | '九' | '十'
        )
}

                                                                         
                                                                             
fn question_time_windows(
    anchor_bjt: DateTime<FixedOffset>,
    text: &str,
) -> Option<Vec<WeatherTargetWindow>> {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    let text = compact.as_str();
                                                                           
                                                                          
    if ["明天", "后天", "今晚", "明早", "明晚", "下周", "下个月"]
        .iter()
        .any(|phrase| text.contains(phrase))
    {
        return None;
    }
    let mut windows = Vec::new();
    for today in ["今日内", "今天内"] {
        if text.contains(today) {
            windows.push(parse_weather_target_window(anchor_bjt, today)?);
        }
    }
    let suffixes = [
        "小时以后",
        "小时后",
        "小时之内",
        "小时内",
        "天以后",
        "天后",
        "天之内",
        "天内",
    ];
    let mut previous_was_count = false;
    for (start, ch) in text.char_indices() {
        let is_count = time_count_char(ch);
        if !is_count || previous_was_count {
            previous_was_count = is_count;
            continue;
        }
        previous_was_count = true;
        if matches!(
            text[..start].chars().last(),
            Some('-' | '~' | '～' | '至' | '到')
        ) {
            return None;
        }
        let tail = &text[start..];
        let number_end = tail
            .char_indices()
            .take_while(|(_, character)| time_count_char(*character))
            .last()
            .map(|(index, character)| index + character.len_utf8())
            .unwrap_or(0);
        for suffix in suffixes {
            if tail[number_end..].starts_with(suffix) {
                windows.push(parse_weather_target_window(
                    anchor_bjt,
                    &tail[..number_end + suffix.len()],
                )?);
                break;
            }
        }
    }
    Some(windows)
}

                                                                       
                                               
pub fn assess_first_rain_scope(question: &QuestionInput) -> FirstRainScopeAudit {
    let mut reasons = Vec::new();
    if question.category != QuestionCategory::Weather {
        reasons.push("非天气类别；不进入首条降雨研究流程".to_owned());
    }
    let event = format!("{} {}", question.subcategory, question.target_event);
    if !event.contains('雨') {
        reasons.push("子类或目标事件未明确为降雨".to_owned());
    }
    if !question.text.contains('雨') {
        reasons.push("问题原文未明确询问降雨".to_owned());
    }
    let combined = format!("{} {}", event, question.text);
    if ["雪", "风", "雷", "雹", "气温", "温度", "雨量", "降水量"]
        .iter()
        .any(|term| combined.contains(term))
    {
        reasons.push("问题混入其他天气现象或降雨量，超出首条是否降雨的范围".to_owned());
    }
    if !["是否", "会不会", "有没有", "有无", "会下", "会有", "能否"]
        .iter()
        .any(|term| question.text.contains(term))
    {
        reasons.push("问题未明确为是否降雨的单一判断".to_owned());
    }
    if !question.options.is_empty() {
        reasons.push("首条降雨流程不接受多选项比较".to_owned());
    }
    let target_window =
        parse_weather_target_window(question.divination_time_bjt, &question.target_time_text);
    if target_window.is_none() {
        reasons.push("目标时段须明确写为如‘1小时后’、‘3小时内’或‘今日内’".to_owned());
    } else if !matches!(
        question_time_windows(question.divination_time_bjt, &question.text),
        Some(ref windows) if windows.iter().all(|window| Some(window) == target_window.as_ref())
    ) {
        reasons.push("问题原文与目标时间栏出现不一致的明确时段".to_owned());
    }
    FirstRainScopeAudit {
        algorithm_version: FIRST_RAIN_SCOPE_VERSION.to_owned(),
        in_research_scope: reasons.is_empty(),
        target_window,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoordinatePrecision, GeoLocation};
    use chrono::TimeZone;
    use uuid::Uuid;

    fn question() -> QuestionInput {
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        let now = zone.with_ymd_and_hms(2026, 9, 29, 12, 45, 0).unwrap();
        QuestionInput {
            id: Uuid::nil(),
            category: QuestionCategory::Weather,
            subcategory: "降雨".to_owned(),
            text: "一小时以后这里会不会下雨？".to_owned(),
            subject: "本人".to_owned(),
            object: "降雨".to_owned(),
            target_event: "是否下雨".to_owned(),
            result_terms: vec!["雨".to_owned()],
            options: Vec::new(),
            location: GeoLocation {
                country_code: "CN".to_owned(),
                province: "广东省".to_owned(),
                city: "深圳市".to_owned(),
                district: Some("南山区".to_owned()),
                street: None,
                longitude_deg: 113.93,
                latitude_deg: 22.53,
                coordinate_precision: CoordinatePrecision::DistrictCenter,
            },
            divination_time_bjt: now,
            target_time_text: "1小时后".to_owned(),
            birth: None,
            locked_at_bjt: now,
        }
    }

    #[test]
    fn canonical_hour_point_is_resolved_without_weather_prediction() {
        let audit = assess_first_rain_scope(&question());
        assert!(audit.in_research_scope);
        let window = audit.target_window.unwrap();
        assert_eq!(window.kind, WeatherTimeKind::Instant);
        assert_eq!(window.start_bjt.to_rfc3339(), "2026-09-29T13:45:00+08:00");
        assert_eq!(window.end_bjt, window.start_bjt);
    }

    #[test]
    fn chinese_hour_interval_and_today_are_bounded() {
        let mut input = question();
        input.target_time_text = "未来三小时内".to_owned();
        let window = assess_first_rain_scope(&input).target_window.unwrap();
        assert_eq!(window.kind, WeatherTimeKind::Interval);
        assert_eq!(window.start_bjt.to_rfc3339(), "2026-09-29T12:45:00+08:00");
        assert_eq!(window.end_bjt.to_rfc3339(), "2026-09-29T15:45:00+08:00");
        input.target_time_text = "今日内".to_owned();
        assert_eq!(
            assess_first_rain_scope(&input)
                .target_window
                .unwrap()
                .end_bjt
                .to_rfc3339(),
            "2026-09-29T23:59:59+08:00"
        );
    }

    #[test]
    fn other_weather_and_vague_times_fail_closed() {
        let mut input = question();
        input.text = "明天会不会刮风下雨？".to_owned();
        assert!(!assess_first_rain_scope(&input).in_research_scope);
        input.text = "明天会不会下雨？".to_owned();
        input.target_time_text = "近期".to_owned();
        assert!(!assess_first_rain_scope(&input).in_research_scope);
        input.target_time_text = "0小时后".to_owned();
        assert!(!assess_first_rain_scope(&input).in_research_scope);
        input.target_time_text = "73小时后".to_owned();
        assert!(!assess_first_rain_scope(&input).in_research_scope);
        input.target_time_text = "3小时后".to_owned();
        input.category = QuestionCategory::Career;
        assert!(!assess_first_rain_scope(&input).in_research_scope);
        input.category = QuestionCategory::Weather;
        input.text = "出门会不会方便？".to_owned();
        assert!(!assess_first_rain_scope(&input).in_research_scope);
    }

    #[test]
    fn malformed_chinese_counts_and_comparisons_fail_closed() {
        let anchor = question().divination_time_bjt;
        for expression in [
            "零小时后",
            "十十小时后",
            "三十三天后",
            "1-3小时内",
            "半小时后",
        ] {
            assert!(parse_weather_target_window(anchor, expression).is_none());
        }
        assert_eq!(
            parse_weather_target_window(anchor, "二十四小时后")
                .unwrap()
                .end_bjt
                .to_rfc3339(),
            "2026-09-30T12:45:00+08:00"
        );
    }

    #[test]
    fn original_question_and_target_field_must_not_disagree() {
        let mut input = question();
        input.text = "3小时后这里会不会下雨？".to_owned();
        assert!(!assess_first_rain_scope(&input).in_research_scope);
        input.text = "一小时后这里会不会下雨？".to_owned();
        assert!(assess_first_rain_scope(&input).in_research_scope);
        input.text = "今日内这里会不会下雨？".to_owned();
        assert!(!assess_first_rain_scope(&input).in_research_scope);
        input.text = "73小时后这里会不会下雨？".to_owned();
        assert!(!assess_first_rain_scope(&input).in_research_scope);
        input.text = "明天这里会不会下雨？".to_owned();
        assert!(!assess_first_rain_scope(&input).in_research_scope);
        input.text = "3 小时后这里会不会下雨？".to_owned();
        assert!(!assess_first_rain_scope(&input).in_research_scope);
        input.text = "1-3小时后这里会不会下雨？".to_owned();
        input.target_time_text = "3小时后".to_owned();
        assert!(!assess_first_rain_scope(&input).in_research_scope);
    }

    #[test]
    fn today_interval_must_not_end_before_it_begins() {
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        let end_of_day = zone.with_ymd_and_hms(2026, 9, 29, 23, 59, 59).unwrap();
        assert!(parse_weather_target_window(end_of_day, "今日内").is_none());
    }
}
