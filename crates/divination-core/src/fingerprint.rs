use crate::{AncientQuestionType, Classification, QuestionCategory, QuestionInput};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const EVENT_FINGERPRINT_VERSION: u16 = 3;
pub const EVENT_MATCH_ALGORITHM_VERSION: &str = "jiuheng.event_match.rule_similarity.v3";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventFingerprint {
    pub version: u16,
    pub digest_hex: String,
    pub canonical_terms: BTreeSet<String>,
    pub category: QuestionCategory,
    pub ancient_type: AncientQuestionType,
    pub subject_key: String,
    pub object_key: String,
    pub target_event_key: String,
    pub country_key: String,
    pub province_key: String,
    pub city_key: String,
    pub district_key: Option<String>,
    pub street_key: Option<String>,
    pub target_time_key: String,
    pub option_keys: BTreeSet<String>,
    pub birth_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMatchReport {
    pub algorithm_version: String,
    pub same_event: bool,
    pub exact_digest: bool,
    pub score_percent: u8,
    pub reasons: Vec<String>,
}

pub fn build_event_fingerprint(
    question: &QuestionInput,
    classification: &Classification,
) -> EventFingerprint {
    let canonical_terms = classification
        .normalized_terms
        .iter()
        .cloned()
        .chain(question.result_terms.iter().map(|term| normalize(term)))
        .filter(|term| !term.is_empty())
        .collect::<BTreeSet<_>>();
    build_event_fingerprint_from_terms(
        question,
        classification.ancient_type.clone(),
        canonical_terms,
    )
}

                                                                              
                                                                        
                                                                             
pub fn build_event_fingerprint_from_terms(
    question: &QuestionInput,
    ancient_type: AncientQuestionType,
    canonical_terms: BTreeSet<String>,
) -> EventFingerprint {
    let canonical_terms = canonical_terms
        .into_iter()
        .map(|term| normalize(&term))
        .filter(|term| !term.is_empty())
        .collect::<BTreeSet<_>>();
    let subject_key = normalize(&question.subject);
    let object_key = normalize(&question.object);
    let target_event_key = normalize(&question.target_event);
    let country_key = normalize(&question.location.country_code);
    let province_key = normalize(&question.location.province);
    let city_key = normalize(&question.location.city);
    let district_key = normalized_optional(question.location.district.as_deref());
    let street_key = normalized_optional(question.location.street.as_deref());
    let target_time_key = normalize_time_scope(&question.target_time_text);
    let option_keys = question
        .options
        .iter()
        .map(|option| normalize(option))
        .filter(|option| !option.is_empty())
        .collect::<BTreeSet<_>>();
    let birth_key = question.birth.as_ref().map(|birth| {
        format!(
            "{}|{}|{}|{}|{}|{}|{}",
            match birth.sex {
                crate::BiologicalSex::Female => "female",
                crate::BiologicalSex::Male => "male",
            },
            birth.date_time_bjt.to_rfc3339(),
            normalize(&birth.location.country_code),
            normalize(&birth.location.province),
            normalize(&birth.location.city),
            normalized_optional(birth.location.district.as_deref()).unwrap_or_default(),
            normalized_optional(birth.location.street.as_deref()).unwrap_or_default(),
        )
    });

    let payload = serde_json::json!({
        "v": EVENT_FINGERPRINT_VERSION,
        "category": question.category.key(),
        "ancient_type": ancient_type,
        "subject": subject_key,
        "object": object_key,
        "target_event": target_event_key,
        "location": {
            "country": country_key,
            "province": province_key,
            "city": city_key,
            "district": district_key,
            "street": street_key,
        },
        "target_time": target_time_key,
        "terms": canonical_terms,
        "options": option_keys,
        "birth": birth_key,
    });
    let encoded = serde_json::to_vec(&payload).expect("serializable event fingerprint payload");

    EventFingerprint {
        version: EVENT_FINGERPRINT_VERSION,
        digest_hex: blake3::hash(&encoded).to_hex().to_string(),
        canonical_terms,
        category: question.category,
        ancient_type,
        subject_key,
        object_key,
        target_event_key,
        country_key,
        province_key,
        city_key,
        district_key,
        street_key,
        target_time_key,
        option_keys,
        birth_key,
    }
}

pub fn compare_event_fingerprints(
    current: &EventFingerprint,
    previous: &EventFingerprint,
) -> EventMatchReport {
    if current.digest_hex == previous.digest_hex {
        return EventMatchReport {
            algorithm_version: EVENT_MATCH_ALGORITHM_VERSION.to_owned(),
            same_event: true,
            exact_digest: true,
            score_percent: 100,
            reasons: vec!["事件指纹完全一致".to_owned()],
        };
    }

    let mut rejection_reasons = Vec::new();
    if current.ancient_type != previous.ancient_type {
        rejection_reasons.push("古代问题类型不同".to_owned());
    }
    if current.category != previous.category
        && current.category != QuestionCategory::Other
        && previous.category != QuestionCategory::Other
    {
        rejection_reasons.push("固定问题类别不同".to_owned());
    }
    if !same_required_location(current, previous) {
        rejection_reasons.push("地点行政范围不同".to_owned());
    }
    if conflicting_nonempty(&current.subject_key, &previous.subject_key) {
        rejection_reasons.push("主体不同".to_owned());
    }
    if conflicting_nonempty(&current.object_key, &previous.object_key) {
        rejection_reasons.push("客体不同".to_owned());
    }
    if conflicting_optional(&current.district_key, &previous.district_key)
        || conflicting_optional(&current.street_key, &previous.street_key)
    {
        rejection_reasons.push("区县或街道不同".to_owned());
    }
    if conflicting_nonempty(&current.target_time_key, &previous.target_time_key) {
        rejection_reasons.push("目标时间范围不同".to_owned());
    }
    if !current.option_keys.is_empty()
        && !previous.option_keys.is_empty()
        && current.option_keys != previous.option_keys
    {
        rejection_reasons.push("候选选项集合不同".to_owned());
    }
    if current.birth_key != previous.birth_key {
        rejection_reasons.push("出生参数不同或完整性不同".to_owned());
    }
    if !rejection_reasons.is_empty() {
        return EventMatchReport {
            algorithm_version: EVENT_MATCH_ALGORITHM_VERSION.to_owned(),
            same_event: false,
            exact_digest: false,
            score_percent: 0,
            reasons: rejection_reasons,
        };
    }

    let event_similarity = dice_similarity(
        &character_bigrams(&current.target_event_key),
        &character_bigrams(&previous.target_event_key),
    );
    let term_similarity = dice_similarity(&current.canonical_terms, &previous.canonical_terms);

    let mut earned = 0.0;
    let mut possible = 0.0;
    let mut reasons = vec!["古代问题类型一致".to_owned(), "地点行政范围一致".to_owned()];

    possible += 10.0;
    if current.category == previous.category {
        earned += 10.0;
        reasons.push("问题类别一致".to_owned());
    } else {
        earned += 7.0;
        reasons.push("“其他”与固定类别映射到同一古代类型".to_owned());
    }

    possible += 20.0;
    earned += 20.0;

    score_identity_component(
        &current.subject_key,
        &previous.subject_key,
        12.0,
        "主体一致",
        &mut earned,
        &mut possible,
        &mut reasons,
    );
    score_identity_component(
        &current.object_key,
        &previous.object_key,
        12.0,
        "客体一致",
        &mut earned,
        &mut possible,
        &mut reasons,
    );

    possible += 30.0;
    earned += 30.0 * event_similarity;
    reasons.push(format!(
        "目标事件文字结构相似度 {}%",
        (event_similarity * 100.0).round() as u8
    ));

    if !current.canonical_terms.is_empty() || !previous.canonical_terms.is_empty() {
        possible += 14.0;
        earned += 14.0 * term_similarity;
        reasons.push(format!(
            "关键词集合相似度 {}%",
            (term_similarity * 100.0).round() as u8
        ));
    }

    score_identity_component(
        &current.target_time_key,
        &previous.target_time_key,
        10.0,
        "目标时间范围一致",
        &mut earned,
        &mut possible,
        &mut reasons,
    );

    if !current.option_keys.is_empty() || !previous.option_keys.is_empty() {
        possible += 5.0;
        if current.option_keys == previous.option_keys {
            earned += 5.0;
            reasons.push("候选选项集合一致".to_owned());
        }
    }
    if current.birth_key.is_some() || previous.birth_key.is_some() {
        possible += 15.0;
        if current.birth_key == previous.birth_key {
            earned += 15.0;
            reasons.push("出生参数一致".to_owned());
        }
    }

    let score_percent = if possible > 0.0 {
        (earned * 100.0 / possible).round().clamp(0.0, 100.0) as u8
    } else {
        0
    };
    let has_identity_anchor = (!current.subject_key.is_empty()
        && current.subject_key == previous.subject_key)
        || (!current.object_key.is_empty() && current.object_key == previous.object_key);
    let same_event = score_percent >= 75
        && (event_similarity >= 0.55
            || (term_similarity >= 0.65 && has_identity_anchor && event_similarity >= 0.30));

    EventMatchReport {
        algorithm_version: EVENT_MATCH_ALGORITHM_VERSION.to_owned(),
        same_event,
        exact_digest: false,
        score_percent,
        reasons,
    }
}

fn score_identity_component(
    current: &str,
    previous: &str,
    weight: f64,
    reason: &str,
    earned: &mut f64,
    possible: &mut f64,
    reasons: &mut Vec<String>,
) {
    if current.is_empty() && previous.is_empty() {
        return;
    }
    *possible += weight;
    if !current.is_empty() && current == previous {
        *earned += weight;
        reasons.push(reason.to_owned());
    }
}

fn same_required_location(current: &EventFingerprint, previous: &EventFingerprint) -> bool {
    current.country_key == previous.country_key
        && current.province_key == previous.province_key
        && current.city_key == previous.city_key
}

fn conflicting_nonempty(current: &str, previous: &str) -> bool {
    !current.is_empty() && !previous.is_empty() && current != previous
}

fn conflicting_optional(current: &Option<String>, previous: &Option<String>) -> bool {
    matches!((current, previous), (Some(current), Some(previous)) if current != previous)
}

fn dice_similarity(left: &BTreeSet<String>, right: &BTreeSet<String>) -> f64 {
    if left.is_empty() && right.is_empty() {
        return 1.0;
    }
    let intersection = left.intersection(right).count() as f64;
    2.0 * intersection / (left.len() + right.len()) as f64
}

fn character_bigrams(value: &str) -> BTreeSet<String> {
    let characters = value.chars().collect::<Vec<_>>();
    if characters.len() < 2 {
        return if value.is_empty() {
            BTreeSet::new()
        } else {
            BTreeSet::from([value.to_owned()])
        };
    }
    characters
        .windows(2)
        .map(|pair| pair.iter().collect::<String>())
        .collect()
}

fn normalized_optional(value: Option<&str>) -> Option<String> {
    value
        .map(normalize)
        .and_then(|value| (!value.is_empty()).then_some(value))
}

fn normalize_time_scope(value: &str) -> String {
    let mut normalized = normalize(value);
    for (source, replacement) in [
        ("以后", "后"),
        ("之后", "后"),
        ("之内", ""),
        ("以内", ""),
        ("未来", ""),
        ("一个", "1个"),
        ("一小时", "1小时"),
        ("二小时", "2小时"),
        ("两小时", "2小时"),
        ("三小时", "3小时"),
        ("七天", "7天"),
        ("三天", "3天"),
        ("一个月", "1个月"),
        ("三个月", "3个月"),
        ("六个月", "6个月"),
        ("一年", "1年"),
    ] {
        normalized = normalized.replace(source, replacement);
    }
    normalized
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
    use crate::{BiologicalSex, BirthParameters, CoordinatePrecision, GeoLocation};
    use chrono::{FixedOffset, TimeZone};
    use uuid::Uuid;

    fn question(text: &str, target_event: &str) -> QuestionInput {
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        let now = zone.with_ymd_and_hms(2026, 8, 30, 12, 45, 0).unwrap();
        QuestionInput {
            id: Uuid::new_v4(),
            category: QuestionCategory::Weather,
            subcategory: "降雨".to_owned(),
            text: text.to_owned(),
            subject: "深圳南山区".to_owned(),
            object: "降雨".to_owned(),
            target_event: target_event.to_owned(),
            result_terms: vec!["雨".to_owned()],
            options: Vec::new(),
            location: GeoLocation {
                country_code: "CN".to_owned(),
                province: "广东省".to_owned(),
                city: "深圳市".to_owned(),
                district: Some("南山区".to_owned()),
                street: None,
                longitude_deg: 113.9304,
                latitude_deg: 22.5333,
                coordinate_precision: CoordinatePrecision::DistrictCenter,
            },
            divination_time_bjt: now,
            target_time_text: "一小时以后".to_owned(),
            birth: None,
            locked_at_bjt: now,
        }
    }

    fn classification() -> Classification {
        Classification {
            ancient_type: AncientQuestionType::WeatherRain,
            matched_terms: vec!["天气".to_owned()],
            normalized_terms: BTreeSet::from(["天气".to_owned(), "雨".to_owned()]),
        }
    }

    #[test]
    fn punctuation_and_time_spelling_produce_the_same_digest() {
        let first = question("一小时后会下雨吗？", "深圳一小时后下雨");
        let mut second = first.clone();
        second.text = "一小时后，会下雨吗".to_owned();
        second.target_time_text = "1小时后".to_owned();
        let first = build_event_fingerprint(&first, &classification());
        let second = build_event_fingerprint(&second, &classification());
        let report = compare_event_fingerprints(&first, &second);
        assert!(report.same_event);
        assert!(report.exact_digest);
        assert_eq!(report.score_percent, 100);
    }

    #[test]
    fn paraphrased_target_event_is_blocked_by_rule_similarity() {
        let first = build_event_fingerprint(
            &question("一小时后会下雨吗？", "南山区一小时后是否下雨"),
            &classification(),
        );
        let second = build_event_fingerprint(
            &question("一小时后南山区有降雨吗？", "南山区一小时后有没有雨"),
            &classification(),
        );
        let report = compare_event_fingerprints(&first, &second);
        assert!(report.same_event, "{report:?}");
        assert!(!report.exact_digest);
        assert!(report.score_percent >= 75);
    }

    #[test]
    fn different_target_time_is_not_the_same_event() {
        let first = build_event_fingerprint(
            &question("一小时后会下雨吗？", "深圳会下雨"),
            &classification(),
        );
        let mut later_question = question("七天后会下雨吗？", "深圳会下雨");
        later_question.target_time_text = "七天后".to_owned();
        let later = build_event_fingerprint(&later_question, &classification());
        let report = compare_event_fingerprints(&first, &later);
        assert!(!report.same_event);
        assert!(report
            .reasons
            .iter()
            .any(|reason| reason.contains("目标时间")));
    }

    #[test]
    fn different_district_is_not_the_same_event() {
        let first = build_event_fingerprint(
            &question("一小时后会下雨吗？", "深圳会下雨"),
            &classification(),
        );
        let mut other_question = question("一小时后会下雨吗？", "深圳会下雨");
        other_question.location.district = Some("福田区".to_owned());
        let other = build_event_fingerprint(&other_question, &classification());
        let report = compare_event_fingerprints(&first, &other);
        assert!(!report.same_event);
        assert!(report.reasons.iter().any(|reason| reason.contains("区县")));
    }

    #[test]
    fn reordered_options_keep_the_same_digest() {
        let mut first_question = question("甲乙方案哪个好？", "选择项目方案");
        first_question.options = vec!["甲方案".to_owned(), "乙方案".to_owned()];
        let mut second_question = first_question.clone();
        second_question.options.reverse();
        let first = build_event_fingerprint(&first_question, &classification());
        let second = build_event_fingerprint(&second_question, &classification());
        assert_eq!(first.digest_hex, second.digest_hex);
    }

    #[test]
    fn different_birth_identity_is_not_the_same_long_term_event() {
        let mut first_question = question("未来长期趋势如何？", "长期趋势");
        first_question.category = QuestionCategory::LongTermFate;
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        first_question.birth = Some(BirthParameters {
            sex: BiologicalSex::Female,
            date_time_bjt: zone.with_ymd_and_hms(1990, 5, 17, 8, 30, 0).unwrap(),
            location: first_question.location.clone(),
        });
        let mut second_question = first_question.clone();
        second_question.birth.as_mut().unwrap().date_time_bjt =
            zone.with_ymd_and_hms(1991, 5, 17, 8, 30, 0).unwrap();
        let natal_classification = Classification {
            ancient_type: AncientQuestionType::NatalLongTerm,
            matched_terms: vec!["长期".to_owned()],
            normalized_terms: BTreeSet::from(["长期".to_owned()]),
        };
        let first = build_event_fingerprint(&first_question, &natal_classification);
        let second = build_event_fingerprint(&second_question, &natal_classification);
        let report = compare_event_fingerprints(&first, &second);
        assert!(!report.same_event);
        assert!(report
            .reasons
            .iter()
            .any(|reason| reason.contains("出生参数")));
    }
}
