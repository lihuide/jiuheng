use crate::{GeoLocation, QuestionCategory, QuestionInput};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const PARAMETER_VALIDATOR_VERSION: &str = "jiuheng.question_parameters.cn_bjt.v1";
const BJT_OFFSET_SECONDS: i32 = 8 * 60 * 60;
pub const SUPPORTED_CIVIL_YEAR_MIN: i32 = -999;
pub const SUPPORTED_CIVIL_YEAR_MAX: i32 = 3000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterValidationReport {
    pub validator_version: String,
    pub can_lock: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn validate_question_parameters(question: &QuestionInput) -> ParameterValidationReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if question.text.trim().is_empty() {
        errors.push("问题原文不能为空".to_owned());
    }
    if question.target_event.trim().is_empty() {
        errors.push("目标事件不能为空".to_owned());
    }
    if question.category == QuestionCategory::Weather && question.target_time_text.trim().is_empty()
    {
        errors.push("天气占问必须填写明确的目标时间或时间范围".to_owned());
    }
    validate_location(&question.location, "占问地点", &mut errors);
    if question.divination_time_bjt.offset().local_minus_utc() != BJT_OFFSET_SECONDS {
        errors.push("占问时间必须以北京时间 UTC+8 保存".to_owned());
    }
    if question.locked_at_bjt.offset().local_minus_utc() != BJT_OFFSET_SECONDS {
        errors.push("锁定时间必须以北京时间 UTC+8 保存".to_owned());
    }
    validate_supported_year(question.divination_time_bjt.year(), "占问时间", &mut errors);

    if question.options.len() == 1 {
        errors.push("多选项比较至少需要两个选项".to_owned());
    }
    let normalized_options = question
        .options
        .iter()
        .map(|option| normalize(option))
        .filter(|option| !option.is_empty())
        .collect::<BTreeSet<_>>();
    if normalized_options.len() != question.options.len() {
        errors.push("候选选项不能为空或重复".to_owned());
    }

    match &question.birth {
        Some(birth) => {
            validate_location(&birth.location, "出生地点", &mut errors);
            if birth.date_time_bjt.offset().local_minus_utc() != BJT_OFFSET_SECONDS {
                errors.push("出生时间必须以北京时间 UTC+8 保存".to_owned());
            }
            validate_supported_year(birth.date_time_bjt.year(), "出生时间", &mut errors);
            if birth.date_time_bjt.timestamp() > question.divination_time_bjt.timestamp() {
                errors.push("出生时间不能晚于占问时间".to_owned());
            }
            if question.category != QuestionCategory::LongTermFate {
                warnings.push(
                    "当前不是长期命理类别；出生资料会保存，但不得强行使即时术式适用".to_owned(),
                );
            }
        }
        None if question.category == QuestionCategory::LongTermFate => warnings
            .push("未提供出生参数；四柱八字、紫微斗数和七政四余必须以参数不足退出".to_owned()),
        None => {}
    }

    ParameterValidationReport {
        validator_version: PARAMETER_VALIDATOR_VERSION.to_owned(),
        can_lock: errors.is_empty(),
        errors,
        warnings,
    }
}

fn validate_supported_year(year: i32, label: &str, errors: &mut Vec<String>) {
    if !(SUPPORTED_CIVIL_YEAR_MIN..=SUPPORTED_CIVIL_YEAR_MAX).contains(&year) {
        errors.push(format!(
            "{label}支持范围为公元前 1000 年（天文年 -999）至公元 3000 年"
        ));
    }
}

fn validate_location(location: &GeoLocation, label: &str, errors: &mut Vec<String>) {
    if !location.country_code.trim().eq_ignore_ascii_case("CN") {
        errors.push(format!("{label}第一版仅支持中国大陆离线地点库"));
    }
    if location.province.trim().is_empty() || location.city.trim().is_empty() {
        errors.push(format!("{label}必须包含省级与城市级名称"));
    }
    if !location.longitude_deg.is_finite() || !(-180.0..=180.0).contains(&location.longitude_deg) {
        errors.push(format!("{label}经度必须是 -180° 至 180° 的有限数值"));
    }
    if !location.latitude_deg.is_finite() || !(-90.0..=90.0).contains(&location.latitude_deg) {
        errors.push(format!("{label}纬度必须是 -90° 至 90° 的有限数值"));
    }
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .filter(|ch| !ch.is_whitespace() && !"，。！？、；：,.!?;:()（）[]【】".contains(*ch))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BiologicalSex, BirthParameters, CoordinatePrecision, GeoLocation, QuestionCategory,
    };
    use chrono::{FixedOffset, TimeZone};
    use uuid::Uuid;

    fn location() -> GeoLocation {
        GeoLocation {
            country_code: "CN".to_owned(),
            province: "广东省".to_owned(),
            city: "深圳市".to_owned(),
            district: Some("南山区".to_owned()),
            street: None,
            longitude_deg: 113.9304,
            latitude_deg: 22.5333,
            coordinate_precision: CoordinatePrecision::DistrictCenter,
        }
    }

    fn question(category: QuestionCategory) -> QuestionInput {
        let zone = FixedOffset::east_opt(BJT_OFFSET_SECONDS).unwrap();
        let now = zone.with_ymd_and_hms(2026, 8, 30, 12, 45, 0).unwrap();
        QuestionInput {
            id: Uuid::new_v4(),
            category,
            subcategory: "测试".to_owned(),
            text: "未来如何？".to_owned(),
            subject: "本人".to_owned(),
            object: String::new(),
            target_event: "长期趋势".to_owned(),
            result_terms: vec!["长期".to_owned()],
            options: Vec::new(),
            location: location(),
            divination_time_bjt: now,
            target_time_text: "未来一年".to_owned(),
            birth: None,
            locked_at_bjt: now,
        }
    }

    #[test]
    fn long_term_question_without_birth_is_lockable_but_warned() {
        let report = validate_question_parameters(&question(QuestionCategory::LongTermFate));
        assert!(report.can_lock);
        assert!(report.errors.is_empty());
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("参数不足")));
    }

    #[test]
    fn valid_birth_parameters_pass() {
        let mut question = question(QuestionCategory::LongTermFate);
        let zone = FixedOffset::east_opt(BJT_OFFSET_SECONDS).unwrap();
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Female,
            date_time_bjt: zone.with_ymd_and_hms(1990, 5, 17, 8, 30, 0).unwrap(),
            location: location(),
        });
        let report = validate_question_parameters(&question);
        assert!(report.can_lock, "{report:?}");
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn future_birth_and_invalid_coordinates_are_rejected() {
        let mut question = question(QuestionCategory::LongTermFate);
        let zone = FixedOffset::east_opt(BJT_OFFSET_SECONDS).unwrap();
        let mut invalid_location = location();
        invalid_location.longitude_deg = f64::NAN;
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: zone.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap(),
            location: invalid_location,
        });
        let report = validate_question_parameters(&question);
        assert!(!report.can_lock);
        assert!(report.errors.iter().any(|error| error.contains("晚于占问")));
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("出生地点经度")));
    }

    #[test]
    fn duplicate_options_are_rejected_after_normalization() {
        let mut question = question(QuestionCategory::Competition);
        question.options = vec!["A 方案".to_owned(), "a方案！".to_owned()];
        let report = validate_question_parameters(&question);
        assert!(!report.can_lock);
        assert!(report.errors.iter().any(|error| error.contains("重复")));
    }

    #[test]
    fn weather_without_target_time_cannot_be_locked() {
        let mut question = question(QuestionCategory::Weather);
        question.target_time_text.clear();
        let report = validate_question_parameters(&question);
        assert!(!report.can_lock);
        assert!(report.errors.iter().any(|error| error.contains("目标时间")));
    }

    #[test]
    fn civil_year_range_uses_astronomical_numbering_for_bce() {
        let zone = FixedOffset::east_opt(BJT_OFFSET_SECONDS).unwrap();
        let mut earliest = question(QuestionCategory::Weather);
        earliest.divination_time_bjt = zone
            .with_ymd_and_hms(SUPPORTED_CIVIL_YEAR_MIN, 1, 1, 0, 0, 0)
            .unwrap();
        assert!(validate_question_parameters(&earliest).can_lock);

        earliest.divination_time_bjt = zone
            .with_ymd_and_hms(SUPPORTED_CIVIL_YEAR_MIN - 1, 12, 31, 0, 0, 0)
            .unwrap();
        let report = validate_question_parameters(&earliest);
        assert!(!report.can_lock);
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("公元前 1000")));
    }
}
